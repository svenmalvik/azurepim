//! Error types for the azurepim application.
//!
//! Uses `thiserror` for library-style errors with automatic `Display` and `Error` implementations.

use thiserror::Error;

/// Top-level application error type.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Keychain error: {0}")]
    Keychain(#[from] KeychainError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("PIM error: {0}")]
    Pim(#[from] PimError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Authentication-related errors.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("OAuth2 authorization failed: {0}")]
    OAuthFailed(String),

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("PKCE generation failed")]
    PkceGenerationFailed,

    #[error("State validation failed (possible CSRF attack)")]
    StateValidationFailed,

    #[error("OAuth callback timeout")]
    CallbackTimeout,

    #[error("User cancelled authentication")]
    UserCancelled,
}

/// Keychain storage errors.
#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Failed to store token: {0}")]
    StoreFailed(String),

    #[error("Failed to retrieve token: {0}")]
    RetrieveFailed(String),

    #[error("Failed to delete token: {0}")]
    DeleteFailed(String),

    #[error("Token not found in keychain")]
    NotFound,
}

/// API-related errors.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Graph API request failed: {0}")]
    GraphRequestFailed(String),

    #[error("Failed to parse API response: {0}")]
    ParseFailed(String),

    #[error("Unauthorized (401): Token may be expired")]
    Unauthorized,

    #[error("Forbidden (403): Insufficient permissions")]
    Forbidden,

    #[error("Rate limited (429): Too many requests")]
    RateLimited,
}

/// PIM (Privileged Identity Management) errors.
#[derive(Error, Debug)]
#[allow(dead_code)] // Full PIM integration pending
pub enum PimError {
    #[error("PIM API permission denied: {0}")]
    PermissionDenied(String),

    #[error("Role activation failed: {0}")]
    ActivationFailed(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Role is already active")]
    RoleAlreadyActive,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Unauthorized (token expired)")]
    Unauthorized,

    #[error("Forbidden (insufficient permissions)")]
    Forbidden,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[allow(dead_code)] // Full PIM integration pending
impl PimError {
    /// Returns a user-friendly message for display in the UI.
    pub fn user_message(&self) -> &str {
        match self {
            Self::PermissionDenied(_) => "PIM access required. Request permissions from IT.",
            Self::ActivationFailed(_) => "Failed to activate role. Please try again.",
            Self::RoleNotFound(_) => "Role not found. Try refreshing the role list.",
            Self::RoleAlreadyActive => "This role is already active.",
            Self::Network(_) => "Network error. Check your connection.",
            Self::InvalidResponse(_) => "Unexpected response from Azure. Please try again.",
            Self::Unauthorized => "Session expired. Please sign in again.",
            Self::Forbidden => "Insufficient permissions for this operation.",
            Self::Io(_) => "Failed to save settings.",
        }
    }

    /// Returns true if this error should trigger a sign-out.
    pub fn requires_sign_out(&self) -> bool {
        matches!(self, Self::Unauthorized)
    }
}

impl AppError {
    /// Returns a user-friendly message for display in the UI.
    #[allow(dead_code)]
    pub fn user_message(&self) -> &str {
        match self {
            Self::Auth(AuthError::OAuthFailed(_)) => "Sign-in failed. Please try again.",
            Self::Auth(AuthError::TokenRefreshFailed(_)) => {
                "Session expired. Please sign in again."
            }
            Self::Auth(AuthError::StateValidationFailed) => {
                "Security error. Please try signing in again."
            }
            Self::Auth(AuthError::CallbackTimeout) => "Sign-in timed out. Please try again.",
            Self::Auth(AuthError::UserCancelled) => "Sign-in was cancelled.",
            Self::Keychain(KeychainError::StoreFailed(_)) => "Failed to save credentials securely.",
            Self::Keychain(KeychainError::NotFound) => "No saved session found.",
            Self::Api(ApiError::Unauthorized) => "Authentication expired. Sign in again.",
            Self::Api(ApiError::Forbidden) => "Insufficient permissions for this operation.",
            Self::Api(ApiError::RateLimited) => "Too many requests. Please wait a moment.",
            Self::Pim(e) => e.user_message(),
            Self::Network(_) => "Network error. Check your connection.",
            Self::Config(_) => "Configuration error. Please check settings.",
            _ => "An error occurred. Please try again.",
        }
    }

    /// Returns true if this error should trigger a sign-out.
    #[allow(dead_code)]
    pub fn requires_sign_out(&self) -> bool {
        matches!(
            self,
            Self::Auth(AuthError::TokenRefreshFailed(_))
                | Self::Api(ApiError::Unauthorized)
                | Self::Pim(PimError::Unauthorized)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_messages() {
        let err = AppError::Auth(AuthError::OAuthFailed("test".into()));
        assert_eq!(err.user_message(), "Sign-in failed. Please try again.");

        let err = AppError::Keychain(KeychainError::NotFound);
        assert_eq!(err.user_message(), "No saved session found.");
    }

    #[test]
    fn test_requires_sign_out() {
        let err = AppError::Api(ApiError::Unauthorized);
        assert!(err.requires_sign_out());

        let err = AppError::Api(ApiError::Forbidden);
        assert!(!err.requires_sign_out());
    }
}
