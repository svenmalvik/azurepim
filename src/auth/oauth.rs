//! OAuth2 client with PKCE support for Azure AD authentication.

use crate::config::Config;
use crate::error::AuthError;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

/// HTTP request timeout.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
/// HTTP connection timeout.
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// PKCE code verifier and challenge pair.
#[derive(Debug)]
pub struct PkceChallenge {
    /// The code verifier (stored locally, sent in token exchange).
    pub verifier: String,
    /// The code challenge (SHA256 hash of verifier, sent in auth request).
    pub challenge: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge pair.
    pub fn new() -> Self {
        // Generate 32 random bytes for the verifier
        let mut rng = rand::thread_rng();
        let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

        // Create challenge = BASE64URL(SHA256(verifier))
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        Self {
            verifier,
            challenge,
        }
    }
}

impl Default for PkceChallenge {
    fn default() -> Self {
        Self::new()
    }
}

/// OAuth2 client for Azure AD authentication.
pub struct OAuth2Client {
    client_id: String,
    tenant: String,
    redirect_uri: String,
    scopes: Vec<String>,
    http_client: reqwest::Client,
}

impl OAuth2Client {
    /// Create a new OAuth2 client from configuration.
    pub fn new(config: &Config) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client_id: config.oauth.client_id.clone(),
            tenant: config.oauth.tenant.clone(),
            redirect_uri: config.oauth.redirect_uri.clone(),
            scopes: config.oauth.scopes.scopes.clone(),
            http_client,
        })
    }

    /// Generate the authorization URL for browser-based sign-in.
    ///
    /// Returns the URL and a CSRF state token that must be verified in the callback.
    pub fn generate_auth_url(&self, pkce: &PkceChallenge) -> (Url, String) {
        // Generate random state for CSRF protection
        let mut rng = rand::thread_rng();
        let state_bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        let state = URL_SAFE_NO_PAD.encode(&state_bytes);

        let auth_endpoint = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            self.tenant
        );

        let mut url = Url::parse(&auth_endpoint).expect("Invalid auth endpoint");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_mode", "query")
            .append_pair("scope", &self.scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256");

        (url, state)
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<TokenResponse, AuthError> {
        let token_endpoint = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant
        );

        let params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("code_verifier", pkce_verifier),
            ("scope", &self.scopes.join(" ")),
        ];

        let response = self
            .http_client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenExchangeFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            // Log error details for debugging (doesn't expose to user)
            let error_body = response.text().await.unwrap_or_default();
            tracing::error!("Token exchange failed: HTTP {} - {}", status, error_body);
            return Err(AuthError::TokenExchangeFailed(format!(
                "HTTP {}",
                status.as_u16()
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenExchangeFailed(e.to_string()))?;

        Ok(token_response)
    }

    /// Refresh an access token using a refresh token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AuthError> {
        let token_endpoint = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant
        );

        let params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", &self.scopes.join(" ")),
        ];

        let response = self
            .http_client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            // Log error details for debugging (doesn't expose to user)
            let error_body = response.text().await.unwrap_or_default();
            tracing::error!("Token refresh failed: HTTP {} - {}", status, error_body);
            return Err(AuthError::TokenRefreshFailed(format!(
                "HTTP {}",
                status.as_u16()
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        Ok(token_response)
    }

    /// Get an access token for Azure Management API using a refresh token.
    ///
    /// Azure AD requires separate tokens for different resources (Graph vs Management API).
    /// This uses the refresh token to acquire a token specifically for Azure Management API.
    pub async fn get_management_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, AuthError> {
        let token_endpoint = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant
        );

        // Request token for Azure Management API resource
        let scope = "https://management.azure.com/.default offline_access";

        let params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", scope),
        ];

        tracing::debug!("Requesting Management API token");

        let response = self
            .http_client
            .post(&token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            tracing::error!(
                "Management API token request failed: HTTP {} - {}",
                status,
                error_body
            );
            return Err(AuthError::TokenRefreshFailed(format!(
                "Management API token: HTTP {}",
                status.as_u16()
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        tracing::info!("Successfully acquired Management API token");
        Ok(token_response)
    }
}

/// Token response from Azure AD.
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: String,
}

/// Parse OAuth callback URL to extract code and state.
pub fn parse_callback_url(url_string: &str) -> Result<(String, String), AuthError> {
    let url = Url::parse(url_string).map_err(|_| AuthError::InvalidAuthCode)?;

    let params: HashMap<_, _> = url.query_pairs().collect();

    // Check for error response
    if let Some(error) = params.get("error") {
        let description = params
            .get("error_description")
            .map(|s| s.to_string())
            .unwrap_or_else(|| error.to_string());
        return Err(AuthError::OAuthFailed(description));
    }

    let code = params
        .get("code")
        .ok_or(AuthError::InvalidAuthCode)?
        .to_string();

    let state = params
        .get("state")
        .ok_or(AuthError::StateValidationFailed)?
        .to_string();

    Ok((code, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceChallenge::new();

        // Verifier should be base64url encoded (43 chars for 32 bytes)
        assert!(!pkce.verifier.is_empty());
        assert!(!pkce.challenge.is_empty());

        // Challenge should be different from verifier
        assert_ne!(pkce.verifier, pkce.challenge);
    }

    #[test]
    fn test_parse_callback_success() {
        let url = "http://localhost:28491/callback?code=abc123&state=xyz789";
        let (code, state) = parse_callback_url(url).unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_callback_error() {
        let url = "http://localhost:28491/callback?error=access_denied&error_description=User%20cancelled";
        let result = parse_callback_url(url);
        assert!(matches!(result, Err(AuthError::OAuthFailed(_))));
    }

    #[test]
    fn test_parse_callback_missing_code() {
        let url = "http://localhost:28491/callback?state=xyz789";
        let result = parse_callback_url(url);
        assert!(matches!(result, Err(AuthError::InvalidAuthCode)));
    }
}
