//! macOS Keychain integration for secure token storage.
//!
//! Stores access tokens, refresh tokens, and user info in the macOS Keychain.

pub mod secure;

use crate::error::KeychainError;
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use zeroize::Zeroizing;

/// Keychain service identifier.
const SERVICE: &str = "de.malvik.azurepim.desktop";

/// Account names for different stored items.
const ACCOUNT_ACCESS_TOKEN: &str = "azure_access_token";
const ACCOUNT_REFRESH_TOKEN: &str = "azure_refresh_token";
const ACCOUNT_USER_INFO: &str = "azure_user_info";
const ACCOUNT_TOKEN_EXPIRY: &str = "azure_token_expiry";

/// Store the access token in the Keychain.
pub fn store_access_token(token: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE, ACCOUNT_ACCESS_TOKEN, token.as_bytes())
        .map_err(|e| KeychainError::StoreFailed(e.to_string()))
}

/// Retrieve the access token from the Keychain.
///
/// Returns a `Zeroizing<String>` that will be securely cleared when dropped.
pub fn get_access_token() -> Result<Zeroizing<String>, KeychainError> {
    let bytes = get_generic_password(SERVICE, ACCOUNT_ACCESS_TOKEN).map_err(|e| {
        if is_not_found_error(&e) {
            KeychainError::NotFound
        } else {
            KeychainError::RetrieveFailed(e.to_string())
        }
    })?;

    let token =
        String::from_utf8(bytes).map_err(|e| KeychainError::RetrieveFailed(e.to_string()))?;

    Ok(Zeroizing::new(token))
}

/// Store the refresh token in the Keychain.
pub fn store_refresh_token(token: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE, ACCOUNT_REFRESH_TOKEN, token.as_bytes())
        .map_err(|e| KeychainError::StoreFailed(e.to_string()))
}

/// Retrieve the refresh token from the Keychain.
///
/// Returns a `Zeroizing<String>` that will be securely cleared when dropped.
pub fn get_refresh_token() -> Result<Zeroizing<String>, KeychainError> {
    let bytes = get_generic_password(SERVICE, ACCOUNT_REFRESH_TOKEN).map_err(|e| {
        if is_not_found_error(&e) {
            KeychainError::NotFound
        } else {
            KeychainError::RetrieveFailed(e.to_string())
        }
    })?;

    let token =
        String::from_utf8(bytes).map_err(|e| KeychainError::RetrieveFailed(e.to_string()))?;

    Ok(Zeroizing::new(token))
}

/// Store the token expiry timestamp (ISO 8601 format).
pub fn store_token_expiry(expiry: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE, ACCOUNT_TOKEN_EXPIRY, expiry.as_bytes())
        .map_err(|e| KeychainError::StoreFailed(e.to_string()))
}

/// Retrieve the token expiry timestamp.
#[allow(dead_code)]
pub fn get_token_expiry() -> Result<String, KeychainError> {
    let bytes = get_generic_password(SERVICE, ACCOUNT_TOKEN_EXPIRY).map_err(|e| {
        if is_not_found_error(&e) {
            KeychainError::NotFound
        } else {
            KeychainError::RetrieveFailed(e.to_string())
        }
    })?;

    String::from_utf8(bytes).map_err(|e| KeychainError::RetrieveFailed(e.to_string()))
}

/// Store user info JSON in the Keychain.
pub fn store_user_info(json: &str) -> Result<(), KeychainError> {
    set_generic_password(SERVICE, ACCOUNT_USER_INFO, json.as_bytes())
        .map_err(|e| KeychainError::StoreFailed(e.to_string()))
}

/// Retrieve user info JSON from the Keychain.
#[allow(dead_code)]
pub fn get_user_info() -> Result<String, KeychainError> {
    let bytes = get_generic_password(SERVICE, ACCOUNT_USER_INFO).map_err(|e| {
        if is_not_found_error(&e) {
            KeychainError::NotFound
        } else {
            KeychainError::RetrieveFailed(e.to_string())
        }
    })?;

    String::from_utf8(bytes).map_err(|e| KeychainError::RetrieveFailed(e.to_string()))
}

/// Delete all stored tokens and user info from the Keychain.
///
/// This is used during sign-out to clear all credentials.
pub fn delete_all() -> Result<(), KeychainError> {
    // Delete each item, ignoring "not found" errors
    let results = [
        delete_generic_password(SERVICE, ACCOUNT_ACCESS_TOKEN),
        delete_generic_password(SERVICE, ACCOUNT_REFRESH_TOKEN),
        delete_generic_password(SERVICE, ACCOUNT_USER_INFO),
        delete_generic_password(SERVICE, ACCOUNT_TOKEN_EXPIRY),
    ];

    // Check if any deletion failed (other than "not found")
    for result in results {
        if let Err(e) = result {
            if !is_not_found_error(&e) {
                return Err(KeychainError::DeleteFailed(e.to_string()));
            }
        }
    }

    Ok(())
}

/// Check if any tokens exist in the Keychain.
#[allow(dead_code)]
pub fn has_tokens() -> bool {
    get_access_token().is_ok() || get_refresh_token().is_ok()
}

/// Helper to check if a security framework error is "item not found".
fn is_not_found_error(error: &security_framework::base::Error) -> bool {
    // errSecItemNotFound = -25300
    error.code() == -25300
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Keychain access and may prompt for permission

    #[test]
    #[ignore = "requires keychain access"]
    fn test_store_and_retrieve_token() {
        let test_token = "test_access_token_12345";

        // Store
        store_access_token(test_token).expect("Failed to store token");

        // Retrieve
        let retrieved = get_access_token().expect("Failed to retrieve token");
        assert_eq!(&*retrieved, test_token);

        // Clean up
        delete_all().expect("Failed to delete tokens");
    }

    #[test]
    #[ignore = "requires keychain access"]
    fn test_delete_all() {
        // Store some data
        store_access_token("test_access").expect("Failed to store access token");
        store_refresh_token("test_refresh").expect("Failed to store refresh token");

        // Delete all
        delete_all().expect("Failed to delete all");

        // Verify deleted
        assert!(matches!(get_access_token(), Err(KeychainError::NotFound)));
        assert!(matches!(get_refresh_token(), Err(KeychainError::NotFound)));
    }
}
