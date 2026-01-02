//! Application state management for the menu bar.

use crate::auth::graph::UserInfo;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

/// Global application state.
pub static APP_STATE: OnceCell<Arc<AppState>> = OnceCell::new();

/// Initialize the global application state.
pub fn init_app_state() -> Arc<AppState> {
    let state = Arc::new(AppState::new());
    APP_STATE
        .set(Arc::clone(&state))
        .expect("App state already initialized");
    state
}

/// Get the global application state.
pub fn get_app_state() -> Option<Arc<AppState>> {
    APP_STATE.get().cloned()
}

/// Central application state.
#[derive(Debug)]
pub struct AppState {
    /// Current authentication state.
    pub auth_state: Mutex<AuthState>,
    /// Cached user information.
    pub user_info: Mutex<Option<UserInfo>>,
    /// Token expiry time.
    pub token_expiry: Mutex<Option<DateTime<Utc>>>,
    /// Settings.
    pub settings: Mutex<Settings>,
}

impl AppState {
    /// Create new application state.
    pub fn new() -> Self {
        Self {
            auth_state: Mutex::new(AuthState::SignedOut),
            user_info: Mutex::new(None),
            token_expiry: Mutex::new(None),
            settings: Mutex::new(Settings::default()),
        }
    }

    /// Get the current authentication state.
    pub fn get_auth_state(&self) -> AuthState {
        self.auth_state.lock().unwrap().clone()
    }

    /// Set the authentication state.
    pub fn set_auth_state(&self, state: AuthState) {
        *self.auth_state.lock().unwrap() = state;
    }

    /// Get the cached user info.
    pub fn get_user_info(&self) -> Option<UserInfo> {
        self.user_info.lock().unwrap().clone()
    }

    /// Set the user info.
    pub fn set_user_info(&self, info: Option<UserInfo>) {
        *self.user_info.lock().unwrap() = info;
    }

    /// Get the token expiry time.
    pub fn get_token_expiry(&self) -> Option<DateTime<Utc>> {
        *self.token_expiry.lock().unwrap()
    }

    /// Set the token expiry time.
    pub fn set_token_expiry(&self, expiry: Option<DateTime<Utc>>) {
        *self.token_expiry.lock().unwrap() = expiry;
    }

    /// Get the settings.
    pub fn get_settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    /// Update settings.
    pub fn set_settings(&self, settings: Settings) {
        *self.settings.lock().unwrap() = settings;
    }

    /// Clear all state (for sign-out).
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.set_auth_state(AuthState::SignedOut);
        self.set_user_info(None);
        self.set_token_expiry(None);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication state enum.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AuthState {
    /// Not signed in.
    SignedOut,
    /// Currently authenticating (browser open).
    Authenticating,
    /// Successfully signed in.
    SignedIn,
    /// Error occurred.
    Error { message: String },
    /// Offline (no network).
    Offline,
}

impl AuthState {
    /// Check if signed in.
    #[allow(dead_code)]
    pub fn is_signed_in(&self) -> bool {
        matches!(self, AuthState::SignedIn)
    }

    /// Check if authenticating.
    #[allow(dead_code)]
    pub fn is_authenticating(&self) -> bool {
        matches!(self, AuthState::Authenticating)
    }

    /// Get error message if in error state.
    #[allow(dead_code)]
    pub fn error_message(&self) -> Option<&str> {
        match self {
            AuthState::Error { message } => Some(message),
            _ => None,
        }
    }
}

/// Application settings.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Auto-launch at login.
    pub auto_launch: bool,
    /// Show token expiry countdown in menu.
    pub show_expiry: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_launch: true,
            show_expiry: true,
        }
    }
}

/// Callbacks for menu actions.
///
/// These are called from the menu item handlers and typically
/// dispatch work to the Tokio runtime.
#[allow(dead_code)]
pub struct MenuCallbacks {
    pub on_sign_in: Box<dyn Fn() + Send + Sync>,
    pub on_sign_out: Box<dyn Fn() + Send + Sync>,
    pub on_refresh_token: Box<dyn Fn() + Send + Sync>,
    pub on_copy_token: Box<dyn Fn() + Send + Sync>,
    pub on_toggle_auto_launch: Box<dyn Fn(bool) + Send + Sync>,
    pub on_toggle_show_expiry: Box<dyn Fn(bool) + Send + Sync>,
    pub on_clear_data: Box<dyn Fn() + Send + Sync>,
    pub on_quit: Box<dyn Fn() + Send + Sync>,
}

impl MenuCallbacks {
    /// Create a new MenuCallbacks with no-op handlers.
    pub fn new() -> Self {
        Self {
            on_sign_in: Box::new(|| {}),
            on_sign_out: Box::new(|| {}),
            on_refresh_token: Box::new(|| {}),
            on_copy_token: Box::new(|| {}),
            on_toggle_auto_launch: Box::new(|_| {}),
            on_toggle_show_expiry: Box::new(|_| {}),
            on_clear_data: Box::new(|| {}),
            on_quit: Box::new(|| {}),
        }
    }
}

impl Default for MenuCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MenuCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuCallbacks").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state() {
        let state = AuthState::SignedOut;
        assert!(!state.is_signed_in());

        let state = AuthState::SignedIn;
        assert!(state.is_signed_in());

        let state = AuthState::Error {
            message: "test".into(),
        };
        assert_eq!(state.error_message(), Some("test"));
    }

    #[test]
    fn test_app_state() {
        let app_state = AppState::new();

        assert!(!app_state.get_auth_state().is_signed_in());

        app_state.set_auth_state(AuthState::SignedIn);
        assert!(app_state.get_auth_state().is_signed_in());

        app_state.clear();
        assert!(!app_state.get_auth_state().is_signed_in());
    }
}
