//! Secure data wrappers that are zeroized on drop.
//!
//! These types ensure sensitive data like tokens are cleared from memory
//! when they're no longer needed.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure string wrapper that zeroizes its contents on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
#[allow(dead_code)]
pub struct SecureString(String);

#[allow(dead_code)]
impl SecureString {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Token response from Azure AD, with secure handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TokenData {
    /// The access token (will be stored in Keychain).
    #[serde(skip_serializing)]
    pub access_token: String,

    /// The refresh token (will be stored in Keychain).
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,

    /// Token type (usually "Bearer").
    pub token_type: String,

    /// Seconds until the access token expires.
    pub expires_in: u64,

    /// Scopes granted by the token.
    #[serde(default)]
    pub scope: String,
}

impl Zeroize for TokenData {
    fn zeroize(&mut self) {
        self.access_token.zeroize();
        if let Some(ref mut rt) = self.refresh_token {
            rt.zeroize();
        }
    }
}

impl Drop for TokenData {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_debug() {
        let secret = SecureString::new("super_secret_token".to_string());
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "[REDACTED]");
        assert!(!debug_output.contains("super_secret"));
    }

    #[test]
    fn test_secure_string_access() {
        let secret = SecureString::new("my_token".to_string());
        assert_eq!(secret.as_str(), "my_token");
    }
}
