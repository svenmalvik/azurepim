//! PIM data models for Azure Privileged Identity Management.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Represents an Azure subscription-level role the user is eligible for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibleRole {
    /// Unique identifier for this role assignment eligibility.
    /// Format: /subscriptions/{sub}/providers/.../roleEligibilityScheduleInstances/{id}
    pub id: String,

    /// Role definition ID (full path).
    /// Format: /subscriptions/{sub}/providers/Microsoft.Authorization/roleDefinitions/{roleDefId}
    pub role_definition_id: String,

    /// Human-readable role name (e.g., "Contributor", "Owner", "Reader").
    pub role_name: String,

    /// Subscription ID (GUID only, without /subscriptions/ prefix).
    pub subscription_id: String,

    /// Subscription display name.
    pub subscription_name: String,

    /// Full scope path (e.g., "/subscriptions/{id}").
    pub scope: String,

    /// Principal ID (user's Azure AD object ID).
    pub principal_id: String,
}

impl EligibleRole {
    /// Generates display text for menu: "subscription_name - role_name".
    pub fn display_text(&self) -> String {
        format!("{} - {}", self.subscription_name, self.role_name)
    }

    /// Unique key for favorites storage (stable identifier).
    pub fn favorites_key(&self) -> String {
        format!("{}:{}", self.subscription_id, self.role_definition_id)
    }
}

/// Represents a currently active PIM role assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAssignment {
    /// Assignment schedule instance ID.
    pub id: String,

    /// Role definition ID.
    pub role_definition_id: String,

    /// Role name.
    pub role_name: String,

    /// Subscription ID.
    pub subscription_id: String,

    /// Subscription name.
    pub subscription_name: String,

    /// Full scope.
    pub scope: String,

    /// When the activation started.
    pub start_time: DateTime<Utc>,

    /// When the activation expires.
    pub end_time: DateTime<Utc>,

    /// Justification provided during activation.
    pub justification: String,

    /// Assignment request ID (for extend/deactivate operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignment_request_id: Option<String>,
}

impl ActiveAssignment {
    /// Display text with time remaining.
    pub fn display_text_with_time(&self) -> String {
        let remaining = self.time_remaining();
        let minutes = remaining.num_minutes();
        let time_str = if minutes >= 60 {
            format!("{} hr {} min left", minutes / 60, minutes % 60)
        } else if minutes > 0 {
            format!("{} min left", minutes)
        } else {
            "expired".to_string()
        };
        format!(
            "{} - {}    {}",
            self.subscription_name, self.role_name, time_str
        )
    }

    /// Get time remaining until expiry.
    pub fn time_remaining(&self) -> Duration {
        let now = Utc::now();
        if self.end_time > now {
            self.end_time - now
        } else {
            Duration::zero()
        }
    }

    /// Check if expiring soon (within threshold minutes).
    pub fn is_expiring_soon(&self, threshold_minutes: i64) -> bool {
        self.time_remaining().num_minutes() <= threshold_minutes
    }

    /// Check if this assignment has expired.
    pub fn is_expired(&self) -> bool {
        self.end_time <= Utc::now()
    }
}

/// Justification preset for quick activation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JustificationPreset {
    /// Display label in menu.
    pub label: String,

    /// Justification text sent to Azure.
    pub justification: String,

    /// Whether this is a built-in preset (not deletable).
    pub is_builtin: bool,
}

impl JustificationPreset {
    /// Create built-in presets.
    pub fn builtin_presets() -> Vec<Self> {
        vec![
            Self {
                label: "Incident Investigation".to_string(),
                justification: "Incident Investigation".to_string(),
                is_builtin: true,
            },
            Self {
                label: "Debugging".to_string(),
                justification: "Debugging".to_string(),
                is_builtin: true,
            },
            Self {
                label: "Maintenance".to_string(),
                justification: "Maintenance".to_string(),
                is_builtin: true,
            },
        ]
    }
}

/// User's PIM preferences - persisted locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PimSettings {
    /// Default activation duration in minutes (default: 60).
    pub default_duration_minutes: u32,

    /// Warning threshold for expiring roles (minutes before expiry to show warning).
    pub expiry_warning_minutes: u32,

    /// Whether to show non-favorite eligible roles in menu.
    pub show_all_eligible: bool,

    /// Custom justification presets (user-defined).
    pub custom_presets: Vec<JustificationPreset>,

    /// Favorite role keys (subscription_id:role_definition_id format).
    pub favorite_role_keys: Vec<String>,
}

impl Default for PimSettings {
    fn default() -> Self {
        Self {
            default_duration_minutes: 60,
            expiry_warning_minutes: 5,
            show_all_eligible: true,
            custom_presets: vec![],
            favorite_role_keys: vec![],
        }
    }
}

impl PimSettings {
    /// Get all justification presets (builtin + custom).
    pub fn all_presets(&self) -> Vec<JustificationPreset> {
        let mut presets = JustificationPreset::builtin_presets();
        presets.extend(self.custom_presets.clone());
        presets
    }

    /// Check if a role key is in favorites.
    pub fn is_favorite(&self, role_key: &str) -> bool {
        self.favorite_role_keys.contains(&role_key.to_string())
    }

    /// Toggle favorite status for a role key.
    pub fn toggle_favorite(&mut self, role_key: &str) {
        if self.is_favorite(role_key) {
            self.favorite_role_keys.retain(|k| k != role_key);
        } else {
            self.favorite_role_keys.push(role_key.to_string());
        }
    }
}

/// PIM API availability status.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PimApiStatus {
    /// Not yet checked.
    #[default]
    Unknown,
    /// API accessible and working.
    Available,
    /// Permission denied (needs admin consent or role assignment).
    PermissionDenied { message: String },
    /// API unreachable or other error.
    Unavailable { error: String },
    /// Currently loading data.
    Loading,
}

/// Request to activate a PIM role.
#[derive(Debug, Clone)]
pub struct ActivationRequest {
    /// The eligible role to activate.
    pub eligible_role: EligibleRole,

    /// Justification reason.
    pub justification: String,

    /// Requested duration in minutes.
    pub duration_minutes: u32,
}

/// Azure subscription info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID (GUID).
    pub subscription_id: String,

    /// Display name.
    pub display_name: String,

    /// Subscription state (e.g., "Enabled").
    pub state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eligible_role_display_text() {
        let role = EligibleRole {
            id: "test-id".to_string(),
            role_definition_id: "role-def-id".to_string(),
            role_name: "Contributor".to_string(),
            subscription_id: "sub-id".to_string(),
            subscription_name: "vipps-prod-001".to_string(),
            scope: "/subscriptions/sub-id".to_string(),
            principal_id: "principal-id".to_string(),
        };

        assert_eq!(role.display_text(), "vipps-prod-001 - Contributor");
        assert_eq!(role.favorites_key(), "sub-id:role-def-id");
    }

    #[test]
    fn test_active_assignment_time_remaining() {
        let now = Utc::now();
        let assignment = ActiveAssignment {
            id: "test-id".to_string(),
            role_definition_id: "role-def-id".to_string(),
            role_name: "Contributor".to_string(),
            subscription_id: "sub-id".to_string(),
            subscription_name: "vipps-prod-001".to_string(),
            scope: "/subscriptions/sub-id".to_string(),
            start_time: now - Duration::minutes(30),
            end_time: now + Duration::minutes(30),
            justification: "Testing".to_string(),
            assignment_request_id: None,
        };

        assert!(!assignment.is_expired());
        assert!(assignment.time_remaining().num_minutes() > 0);
        assert!(assignment.is_expiring_soon(35));
        assert!(!assignment.is_expiring_soon(25));
    }

    #[test]
    fn test_pim_settings_favorites() {
        let mut settings = PimSettings::default();
        let key = "sub-id:role-def-id";

        assert!(!settings.is_favorite(key));

        settings.toggle_favorite(key);
        assert!(settings.is_favorite(key));

        settings.toggle_favorite(key);
        assert!(!settings.is_favorite(key));
    }

    #[test]
    fn test_justification_presets() {
        let presets = JustificationPreset::builtin_presets();
        assert_eq!(presets.len(), 3);
        assert!(presets.iter().all(|p| p.is_builtin));
    }
}
