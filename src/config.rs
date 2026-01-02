//! Configuration loading and management.
//!
//! Loads configuration from embedded config.toml with environment variable overrides.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

/// Embedded configuration file content.
const CONFIG_TOML: &str = include_str!("../config.toml");

/// Root configuration structure.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub app: AppConfig,
    pub oauth: OAuthConfig,
    pub api: ApiConfig,
    pub token: TokenConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub bundle_identifier: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub tenant: String,
    pub redirect_uri: String,
    pub scopes: ScopesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScopesConfig {
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ApiConfig {
    pub graph_base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TokenConfig {
    pub refresh_before_expiry_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LoggingConfig {
    pub level: String,
    pub log_dir: String,
}

impl Config {
    /// Load configuration from embedded config.toml with environment variable overrides.
    pub fn load() -> Result<Self> {
        // Parse embedded config
        let mut config: Config =
            toml::from_str(CONFIG_TOML).context("Failed to parse embedded config.toml")?;

        // Apply environment variable overrides
        if let Ok(client_id) = env::var("AZURE_CLIENT_ID") {
            config.oauth.client_id = client_id;
        }

        if let Ok(tenant) = env::var("AZURE_TENANT_ID") {
            config.oauth.tenant = tenant;
        }

        if let Ok(redirect_uri) = env::var("AZURE_REDIRECT_URI") {
            config.oauth.redirect_uri = redirect_uri;
        }

        if let Ok(log_level) = env::var("RUST_LOG") {
            config.logging.level = log_level;
        }

        // Validate required fields
        config.validate()?;

        Ok(config)
    }

    /// Validate that required configuration is present.
    fn validate(&self) -> Result<()> {
        if self.oauth.client_id.is_empty() || self.oauth.client_id == "YOUR_AZURE_AD_CLIENT_ID" {
            anyhow::bail!(
                "Azure AD client_id not configured. Set AZURE_CLIENT_ID environment variable \
                 or update config.toml"
            );
        }

        if self.oauth.tenant.is_empty() || self.oauth.tenant == "YOUR_TENANT_ID" {
            anyhow::bail!(
                "Azure AD tenant not configured. Set AZURE_TENANT_ID environment variable \
                 or update config.toml"
            );
        }

        Ok(())
    }

    /// Get the authorization URL for Azure AD.
    #[allow(dead_code)]
    pub fn auth_url(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            self.oauth.tenant
        )
    }

    /// Get the token URL for Azure AD.
    #[allow(dead_code)]
    pub fn token_url(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.oauth.tenant
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        // This will fail validation because of placeholder values,
        // but the parsing should work
        let result = toml::from_str::<Config>(CONFIG_TOML);
        assert!(result.is_ok(), "Config parsing failed: {:?}", result.err());
    }

    #[test]
    fn test_urls() {
        let config = Config {
            app: AppConfig {
                name: "test".into(),
                version: "0.1.0".into(),
                bundle_identifier: "test".into(),
            },
            oauth: OAuthConfig {
                client_id: "test-client".into(),
                tenant: "test-tenant".into(),
                redirect_uri: "azurepim://callback".into(),
                scopes: ScopesConfig {
                    scopes: vec!["User.Read".into()],
                },
            },
            api: ApiConfig {
                graph_base_url: "https://graph.microsoft.com/v1.0".into(),
            },
            token: TokenConfig {
                refresh_before_expiry_seconds: 300,
            },
            logging: LoggingConfig {
                level: "info".into(),
                log_dir: "azurepim".into(),
            },
        };

        assert_eq!(
            config.auth_url(),
            "https://login.microsoftonline.com/test-tenant/oauth2/v2.0/authorize"
        );
        assert_eq!(
            config.token_url(),
            "https://login.microsoftonline.com/test-tenant/oauth2/v2.0/token"
        );
    }
}
