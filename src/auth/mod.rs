//! Azure AD authentication module.
//!
//! Provides OAuth2 with PKCE authentication, Microsoft Graph API client,
//! and automatic token refresh management.

pub mod callback_server;
pub mod graph;
pub mod oauth;
pub mod token_manager;
