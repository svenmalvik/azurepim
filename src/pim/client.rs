//! Azure PIM API client for role management.
//!
//! Uses the Azure Resource Management API to interact with PIM.

use std::time::Duration as StdDuration;

use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::models::{ActivationRequest, ActiveAssignment, EligibleRole, Subscription};
use crate::error::PimError;

/// Azure Management API base URL.
const MANAGEMENT_BASE_URL: &str = "https://management.azure.com";

/// API version for PIM operations.
const API_VERSION_PIM: &str = "2020-10-01";

/// API version for subscription operations.
const API_VERSION_SUBS: &str = "2022-12-01";

/// API version for role definitions.
const API_VERSION_ROLES: &str = "2022-04-01";

/// HTTP request timeout.
const HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(30);

/// HTTP connection timeout.
const HTTP_CONNECT_TIMEOUT: StdDuration = StdDuration::from_secs(10);

/// Azure PIM API client.
pub struct PimClient {
    http_client: Client,
}

impl PimClient {
    /// Create a new PIM client.
    pub fn new() -> Result<Self, PimError> {
        let http_client = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .build()
            .map_err(PimError::Network)?;

        Ok(Self { http_client })
    }

    /// List all accessible subscriptions.
    pub async fn list_subscriptions(
        &self,
        access_token: &str,
    ) -> Result<Vec<Subscription>, PimError> {
        let url = format!(
            "{}/subscriptions?api-version={}",
            MANAGEMENT_BASE_URL, API_VERSION_SUBS
        );

        debug!("Fetching subscriptions from {}", url);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(PimError::Network)?;

        let status = response.status();
        match status.as_u16() {
            200 => {
                let body: SubscriptionListResponse = response
                    .json()
                    .await
                    .map_err(|e| PimError::InvalidResponse(e.to_string()))?;

                let subscriptions: Vec<Subscription> = body
                    .value
                    .into_iter()
                    .filter(|s| s.state == "Enabled")
                    .map(|s| Subscription {
                        subscription_id: s.subscription_id,
                        display_name: s.display_name,
                        state: s.state,
                    })
                    .collect();

                info!("Found {} enabled subscriptions", subscriptions.len());
                Ok(subscriptions)
            }
            401 => Err(PimError::Unauthorized),
            403 => Err(PimError::Forbidden),
            _ => {
                let body = response.text().await.unwrap_or_default();
                error!("Failed to list subscriptions: HTTP {} - {}", status, body);
                Err(PimError::InvalidResponse(format!("HTTP {}", status)))
            }
        }
    }

    /// Get eligible roles for a single subscription.
    async fn get_eligible_roles_for_subscription(
        &self,
        access_token: &str,
        subscription_id: &str,
        principal_id: &str,
    ) -> Result<Vec<EligibleRole>, PimError> {
        let url = format!(
            "{}/subscriptions/{}/providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version={}&$filter=principalId eq '{}'",
            MANAGEMENT_BASE_URL, subscription_id, API_VERSION_PIM, principal_id
        );

        debug!(
            "Fetching eligible roles for subscription {}",
            subscription_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(PimError::Network)?;

        let status = response.status();
        match status.as_u16() {
            200 => {
                let body: RoleEligibilityListResponse = response
                    .json()
                    .await
                    .map_err(|e| PimError::InvalidResponse(e.to_string()))?;

                // Need to resolve role names from role definition IDs
                let mut roles = Vec::new();
                for item in body.value {
                    let role_name = self
                        .get_role_name(access_token, &item.properties.role_definition_id)
                        .await
                        .unwrap_or_else(|_| "Unknown Role".to_string());

                    roles.push(EligibleRole {
                        id: item.id,
                        role_definition_id: item.properties.role_definition_id,
                        role_name,
                        subscription_id: subscription_id.to_string(),
                        subscription_name: String::new(), // Will be filled by caller
                        scope: item.properties.scope,
                        principal_id: item.properties.principal_id,
                    });
                }

                Ok(roles)
            }
            401 => Err(PimError::Unauthorized),
            403 => {
                // User may not have PIM access to this subscription
                debug!(
                    "No PIM access to subscription {}, skipping",
                    subscription_id
                );
                Ok(vec![])
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                warn!(
                    "Failed to get eligible roles for {}: HTTP {} - {}",
                    subscription_id, status, body
                );
                Ok(vec![])
            }
        }
    }

    /// Get role definition name from role definition ID.
    async fn get_role_name(
        &self,
        access_token: &str,
        role_definition_id: &str,
    ) -> Result<String, PimError> {
        let url = format!(
            "{}{}?api-version={}",
            MANAGEMENT_BASE_URL, role_definition_id, API_VERSION_ROLES
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(PimError::Network)?;

        if response.status().is_success() {
            let body: RoleDefinitionResponse = response
                .json()
                .await
                .map_err(|e| PimError::InvalidResponse(e.to_string()))?;
            Ok(body.properties.role_name)
        } else {
            Err(PimError::InvalidResponse(format!(
                "Failed to get role definition: {}",
                role_definition_id
            )))
        }
    }

    /// Get all eligible roles across all subscriptions.
    ///
    /// `principal_ids` should include the user's object ID plus all group IDs
    /// the user is a member of, to find roles assigned via group membership.
    pub async fn get_all_eligible_roles(
        &self,
        access_token: &str,
        principal_ids: &[String],
    ) -> Result<Vec<EligibleRole>, PimError> {
        if principal_ids.is_empty() {
            return Err(PimError::InvalidResponse("No principal IDs provided".to_string()));
        }

        info!("Checking eligible roles for {} principal IDs (user + groups)", principal_ids.len());

        let subscriptions = self.list_subscriptions(access_token).await?;
        let total_subs = subscriptions.len();

        let mut all_roles = Vec::new();
        let mut seen_role_ids = std::collections::HashSet::new();

        for (idx, sub) in subscriptions.iter().enumerate() {
            // Log progress every 10 subscriptions
            if idx % 10 == 0 {
                info!(
                    "Checking subscription {}/{}: {}",
                    idx + 1,
                    total_subs,
                    sub.display_name
                );
            }

            // Query for each principal ID (user + groups)
            for principal_id in principal_ids {
                match self
                    .get_eligible_roles_for_subscription(
                        access_token,
                        &sub.subscription_id,
                        principal_id,
                    )
                    .await
                {
                    Ok(mut roles) => {
                        // Fill in subscription names and deduplicate
                        for role in &mut roles {
                            role.subscription_name = sub.display_name.clone();
                            // Deduplicate by role ID (same role might appear for multiple groups)
                            if seen_role_ids.insert(role.id.clone()) {
                                all_roles.push(role.clone());
                            }
                        }
                    }
                    Err(PimError::Unauthorized) => return Err(PimError::Unauthorized),
                    Err(e) => {
                        warn!(
                            "Error fetching roles for subscription {} (principal {}): {}",
                            sub.display_name, principal_id, e
                        );
                        // Continue with other subscriptions/principals
                    }
                }
            }
        }

        info!("Found {} total eligible roles (deduplicated)", all_roles.len());
        Ok(all_roles)
    }

    /// Get active role assignments for all subscriptions.
    ///
    /// `principal_ids` should include the user's object ID plus all group IDs
    /// the user is a member of, to find assignments via group membership.
    pub async fn get_active_assignments(
        &self,
        access_token: &str,
        principal_ids: &[String],
    ) -> Result<Vec<ActiveAssignment>, PimError> {
        if principal_ids.is_empty() {
            return Err(PimError::InvalidResponse("No principal IDs provided".to_string()));
        }

        let subscriptions = self.list_subscriptions(access_token).await?;

        let mut all_assignments = Vec::new();
        let mut seen_assignment_ids = std::collections::HashSet::new();

        for sub in &subscriptions {
            for principal_id in principal_ids {
                match self
                    .get_active_assignments_for_subscription(
                        access_token,
                        &sub.subscription_id,
                        principal_id,
                    )
                    .await
                {
                    Ok(mut assignments) => {
                        // Fill in subscription names and deduplicate
                        for assignment in &mut assignments {
                            assignment.subscription_name = sub.display_name.clone();
                            if seen_assignment_ids.insert(assignment.id.clone()) {
                                all_assignments.push(assignment.clone());
                            }
                        }
                    }
                    Err(PimError::Unauthorized) => return Err(PimError::Unauthorized),
                    Err(e) => {
                        warn!(
                            "Error fetching active assignments for subscription {} (principal {}): {}",
                            sub.display_name, principal_id, e
                        );
                    }
                }
            }
        }

        info!("Found {} active assignments (deduplicated)", all_assignments.len());
        Ok(all_assignments)
    }

    /// Get active assignments for a single subscription.
    async fn get_active_assignments_for_subscription(
        &self,
        access_token: &str,
        subscription_id: &str,
        principal_id: &str,
    ) -> Result<Vec<ActiveAssignment>, PimError> {
        let url = format!(
            "{}/subscriptions/{}/providers/Microsoft.Authorization/roleAssignmentScheduleInstances?api-version={}&$filter=principalId eq '{}'",
            MANAGEMENT_BASE_URL, subscription_id, API_VERSION_PIM, principal_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(PimError::Network)?;

        let status = response.status();
        match status.as_u16() {
            200 => {
                let body: RoleAssignmentListResponse = response
                    .json()
                    .await
                    .map_err(|e| PimError::InvalidResponse(e.to_string()))?;

                let mut assignments = Vec::new();
                for item in body.value {
                    // Only include assignments that are PIM-activated (have start/end times)
                    if let (Some(start), Some(end)) = (
                        item.properties.start_date_time,
                        item.properties.end_date_time,
                    ) {
                        let role_name = self
                            .get_role_name(access_token, &item.properties.role_definition_id)
                            .await
                            .unwrap_or_else(|_| "Unknown Role".to_string());

                        assignments.push(ActiveAssignment {
                            id: item.id,
                            role_definition_id: item.properties.role_definition_id,
                            role_name,
                            subscription_id: subscription_id.to_string(),
                            subscription_name: String::new(),
                            scope: item.properties.scope,
                            start_time: start,
                            end_time: end,
                            justification: String::new(), // Not available in list response
                            assignment_request_id: item.properties.role_assignment_schedule_id,
                        });
                    }
                }

                Ok(assignments)
            }
            401 => Err(PimError::Unauthorized),
            403 => Ok(vec![]),
            _ => Ok(vec![]),
        }
    }

    /// Activate a PIM role.
    pub async fn activate_role(
        &self,
        access_token: &str,
        request: ActivationRequest,
    ) -> Result<ActiveAssignment, PimError> {
        let request_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}{}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{}?api-version={}",
            MANAGEMENT_BASE_URL,
            request.eligible_role.scope,
            request_id,
            API_VERSION_PIM
        );

        let start_time = Utc::now();
        let duration = format!("PT{}M", request.duration_minutes);

        let body = ActivationRequestBody {
            properties: ActivationProperties {
                principal_id: request.eligible_role.principal_id.clone(),
                role_definition_id: request.eligible_role.role_definition_id.clone(),
                request_type: "SelfActivate".to_string(),
                justification: request.justification.clone(),
                linked_role_eligibility_schedule_id: Some(request.eligible_role.id.clone()),
                schedule_info: ScheduleInfo {
                    start_date_time: start_time.to_rfc3339(),
                    expiration: Expiration {
                        expiration_type: "AfterDuration".to_string(),
                        duration,
                    },
                },
            },
        };

        info!(
            "Activating role {} on {} for {} minutes",
            request.eligible_role.role_name,
            request.eligible_role.subscription_name,
            request.duration_minutes
        );

        let response = self
            .http_client
            .put(&url)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(PimError::Network)?;

        let status = response.status();
        match status.as_u16() {
            200 | 201 => {
                let response_body: ActivationResponseBody = response
                    .json()
                    .await
                    .map_err(|e| PimError::InvalidResponse(e.to_string()))?;

                let end_time =
                    start_time + chrono::Duration::minutes(request.duration_minutes as i64);

                info!(
                    "Successfully activated role {} until {}",
                    request.eligible_role.role_name, end_time
                );

                Ok(ActiveAssignment {
                    id: response_body.id,
                    role_definition_id: request.eligible_role.role_definition_id,
                    role_name: request.eligible_role.role_name,
                    subscription_id: request.eligible_role.subscription_id,
                    subscription_name: request.eligible_role.subscription_name,
                    scope: request.eligible_role.scope,
                    start_time,
                    end_time,
                    justification: request.justification,
                    assignment_request_id: Some(request_id),
                })
            }
            400 => {
                let body = response.text().await.unwrap_or_default();
                error!("Bad request for role activation: {}", body);
                Err(PimError::ActivationFailed("Bad request".to_string()))
            }
            401 => Err(PimError::Unauthorized),
            403 => Err(PimError::Forbidden),
            409 => {
                warn!("Role is already active");
                Err(PimError::RoleAlreadyActive)
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                error!("Role activation failed: HTTP {} - {}", status, body);
                Err(PimError::ActivationFailed(format!("HTTP {}", status)))
            }
        }
    }
}

// --- API Response Types ---

#[derive(Debug, Deserialize)]
struct SubscriptionListResponse {
    value: Vec<SubscriptionItem>,
}

#[derive(Debug, Deserialize)]
struct SubscriptionItem {
    #[serde(rename = "subscriptionId")]
    subscription_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct RoleEligibilityListResponse {
    value: Vec<RoleEligibilityItem>,
}

#[derive(Debug, Deserialize)]
struct RoleEligibilityItem {
    id: String,
    properties: RoleEligibilityProperties,
}

#[derive(Debug, Deserialize)]
struct RoleEligibilityProperties {
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: String,
    #[serde(rename = "principalId")]
    principal_id: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct RoleDefinitionResponse {
    properties: RoleDefinitionProperties,
}

#[derive(Debug, Deserialize)]
struct RoleDefinitionProperties {
    #[serde(rename = "roleName")]
    role_name: String,
}

#[derive(Debug, Deserialize)]
struct RoleAssignmentListResponse {
    value: Vec<RoleAssignmentItem>,
}

#[derive(Debug, Deserialize)]
struct RoleAssignmentItem {
    id: String,
    properties: RoleAssignmentProperties,
}

#[derive(Debug, Deserialize)]
struct RoleAssignmentProperties {
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: String,
    scope: String,
    #[serde(rename = "startDateTime")]
    start_date_time: Option<chrono::DateTime<Utc>>,
    #[serde(rename = "endDateTime")]
    end_date_time: Option<chrono::DateTime<Utc>>,
    #[serde(rename = "roleAssignmentScheduleId")]
    role_assignment_schedule_id: Option<String>,
}

// --- Request Body Types ---

#[derive(Debug, Serialize)]
struct ActivationRequestBody {
    properties: ActivationProperties,
}

#[derive(Debug, Serialize)]
struct ActivationProperties {
    #[serde(rename = "principalId")]
    principal_id: String,
    #[serde(rename = "roleDefinitionId")]
    role_definition_id: String,
    #[serde(rename = "requestType")]
    request_type: String,
    justification: String,
    #[serde(rename = "linkedRoleEligibilityScheduleId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    linked_role_eligibility_schedule_id: Option<String>,
    #[serde(rename = "scheduleInfo")]
    schedule_info: ScheduleInfo,
}

#[derive(Debug, Serialize)]
struct ScheduleInfo {
    #[serde(rename = "startDateTime")]
    start_date_time: String,
    expiration: Expiration,
}

#[derive(Debug, Serialize)]
struct Expiration {
    #[serde(rename = "type")]
    expiration_type: String,
    duration: String,
}

#[derive(Debug, Deserialize)]
struct ActivationResponseBody {
    id: String,
}
