//! Microsoft Graph API client for fetching user profile and organization info.

use crate::error::ApiError;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base URL for Microsoft Graph API.
const GRAPH_BASE_URL: &str = "https://graph.microsoft.com/v1.0";

/// HTTP request timeout.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
/// HTTP connection timeout.
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Microsoft Graph API client.
pub struct GraphClient {
    http_client: reqwest::Client,
}

impl GraphClient {
    /// Create a new Graph client.
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { http_client })
    }

    /// Fetch the current user's profile.
    pub async fn get_user_profile(&self, access_token: &str) -> Result<UserProfile, ApiError> {
        let url = format!("{}/me", GRAPH_BASE_URL);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ApiError::GraphRequestFailed(e.to_string()))?;

        match response.status().as_u16() {
            200 => {
                let profile: UserProfile = response
                    .json()
                    .await
                    .map_err(|e| ApiError::ParseFailed(e.to_string()))?;
                Ok(profile)
            }
            401 => Err(ApiError::Unauthorized),
            403 => Err(ApiError::Forbidden),
            429 => Err(ApiError::RateLimited),
            // Don't expose raw API error details - just log status code
            status => Err(ApiError::GraphRequestFailed(format!("HTTP {}", status))),
        }
    }

    /// Fetch the user's organization info.
    pub async fn get_organization(&self, access_token: &str) -> Result<Organization, ApiError> {
        let url = format!("{}/organization", GRAPH_BASE_URL);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ApiError::GraphRequestFailed(e.to_string()))?;

        match response.status().as_u16() {
            200 => {
                let org_response: OrganizationResponse = response
                    .json()
                    .await
                    .map_err(|e| ApiError::ParseFailed(e.to_string()))?;

                org_response
                    .value
                    .into_iter()
                    .next()
                    .ok_or_else(|| ApiError::ParseFailed("No organization found".to_string()))
            }
            401 => Err(ApiError::Unauthorized),
            403 => Err(ApiError::Forbidden),
            429 => Err(ApiError::RateLimited),
            // Don't expose raw API error details - just log status code
            status => Err(ApiError::GraphRequestFailed(format!("HTTP {}", status))),
        }
    }

    /// Fetch the current user's group memberships (security groups and Microsoft 365 groups).
    /// Returns a list of group IDs that the user is a member of.
    pub async fn get_user_groups(&self, access_token: &str) -> Result<Vec<GroupMembership>, ApiError> {
        let mut all_groups = Vec::new();
        let mut next_link: Option<String> = None;
        let initial_url = format!(
            "{}/me/memberOf?$select=id,displayName&$filter=isof('microsoft.graph.group')",
            GRAPH_BASE_URL
        );

        loop {
            let url = next_link.as_ref().unwrap_or(&initial_url);

            let response = self
                .http_client
                .get(url)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| ApiError::GraphRequestFailed(e.to_string()))?;

            match response.status().as_u16() {
                200 => {
                    let groups_response: GroupMembershipResponse = response
                        .json()
                        .await
                        .map_err(|e| ApiError::ParseFailed(e.to_string()))?;

                    // Filter to only include groups (not roles or other directory objects)
                    for item in groups_response.value {
                        if item.odata_type == Some("#microsoft.graph.group".to_string()) {
                            all_groups.push(GroupMembership {
                                id: item.id,
                                display_name: item.display_name,
                            });
                        }
                    }

                    // Check for pagination
                    if let Some(link) = groups_response.odata_next_link {
                        next_link = Some(link);
                    } else {
                        break;
                    }
                }
                401 => return Err(ApiError::Unauthorized),
                403 => return Err(ApiError::Forbidden),
                429 => return Err(ApiError::RateLimited),
                status => return Err(ApiError::GraphRequestFailed(format!("HTTP {}", status))),
            }
        }

        Ok(all_groups)
    }
}

impl Default for GraphClient {
    fn default() -> Self {
        Self::new().expect("Failed to create GraphClient")
    }
}

/// User profile from Microsoft Graph /me endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    /// Unique identifier for the user.
    pub id: String,

    /// User's display name.
    pub display_name: Option<String>,

    /// User's given (first) name.
    pub given_name: Option<String>,

    /// User's surname (last name).
    pub surname: Option<String>,

    /// User's email address.
    pub mail: Option<String>,

    /// User Principal Name (typically email-like format).
    pub user_principal_name: Option<String>,

    /// User's job title.
    pub job_title: Option<String>,

    /// User's office location.
    pub office_location: Option<String>,
}

impl UserProfile {
    /// Get the best available display name.
    pub fn display_name_or_upn(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| self.user_principal_name.clone())
            .unwrap_or_else(|| "Unknown User".to_string())
    }

    /// Get the best available email.
    pub fn email(&self) -> String {
        self.mail
            .clone()
            .or_else(|| self.user_principal_name.clone())
            .unwrap_or_else(|| "No email".to_string())
    }
}

/// Organization response wrapper.
#[derive(Debug, Deserialize)]
struct OrganizationResponse {
    value: Vec<Organization>,
}

/// Organization info from Microsoft Graph /organization endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    /// Tenant ID.
    pub id: String,

    /// Organization display name.
    pub display_name: Option<String>,

    /// Verified domains.
    #[serde(default)]
    pub verified_domains: Vec<VerifiedDomain>,
}

impl Organization {
    /// Get the organization name or tenant ID.
    pub fn name_or_id(&self) -> String {
        self.display_name.clone().unwrap_or_else(|| self.id.clone())
    }
}

/// Verified domain info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedDomain {
    pub name: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(rename = "isInitial")]
    pub is_initial: Option<bool>,
}

/// Group membership response from /me/memberOf.
#[derive(Debug, Deserialize)]
struct GroupMembershipResponse {
    value: Vec<DirectoryObject>,
    #[serde(rename = "@odata.nextLink")]
    odata_next_link: Option<String>,
}

/// Directory object from memberOf response.
#[derive(Debug, Deserialize)]
struct DirectoryObject {
    id: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "@odata.type")]
    odata_type: Option<String>,
}

/// Group membership info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMembership {
    /// Group's unique object ID.
    pub id: String,
    /// Group's display name.
    pub display_name: Option<String>,
}

/// Combined user info for UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User's unique object ID (principal ID for PIM).
    pub user_id: String,
    pub display_name: String,
    pub email: String,
    pub tenant_id: String,
    pub tenant_name: String,
}

impl UserInfo {
    /// Create UserInfo from profile and organization.
    pub fn from_profile_and_org(profile: UserProfile, org: Organization) -> Self {
        Self {
            user_id: profile.id.clone(),
            display_name: profile.display_name_or_upn(),
            email: profile.email(),
            tenant_id: org.id.clone(),
            tenant_name: org.name_or_id(),
        }
    }

    /// Serialize to JSON for storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON.
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_display_name() {
        let profile = UserProfile {
            id: "123".into(),
            display_name: Some("John Doe".into()),
            given_name: None,
            surname: None,
            mail: Some("john@example.com".into()),
            user_principal_name: Some("john@example.com".into()),
            job_title: None,
            office_location: None,
        };

        assert_eq!(profile.display_name_or_upn(), "John Doe");
        assert_eq!(profile.email(), "john@example.com");
    }

    #[test]
    fn test_user_profile_fallback() {
        let profile = UserProfile {
            id: "123".into(),
            display_name: None,
            given_name: None,
            surname: None,
            mail: None,
            user_principal_name: Some("user@tenant.com".into()),
            job_title: None,
            office_location: None,
        };

        assert_eq!(profile.display_name_or_upn(), "user@tenant.com");
        assert_eq!(profile.email(), "user@tenant.com");
    }

    #[test]
    fn test_user_info_serialization() {
        let info = UserInfo {
            user_id: "user-object-id".into(),
            display_name: "Test User".into(),
            email: "test@example.com".into(),
            tenant_id: "abc-123".into(),
            tenant_name: "Test Org".into(),
        };

        let json = info.to_json().unwrap();
        let restored = UserInfo::from_json(&json).unwrap();

        assert_eq!(restored.user_id, info.user_id);
        assert_eq!(restored.display_name, info.display_name);
        assert_eq!(restored.email, info.email);
    }
}
