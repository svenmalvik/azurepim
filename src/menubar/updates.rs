//! UI update functions that dispatch to the main thread.
//!
//! These functions ensure all AppKit operations run on the main thread.

use crate::auth::graph::UserInfo;
use crate::menubar::builder::MenuBar;
use crate::menubar::state::{get_app_state, AuthState, Settings};
use chrono::{DateTime, Utc};
use dispatch::Queue;
use objc2_foundation::MainThreadMarker;
use tracing::info;

/// Update the UI to reflect the signed-out state.
pub fn update_signed_out() {
    dispatch_to_main(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_auth_state(AuthState::SignedOut);
                state.set_user_info(None);
                state.set_token_expiry(None);
            }
            MenuBar::update_title(mtm, "Azure");
            MenuBar::build_signed_out_menu(mtm);
            info!("UI updated: signed out");
        }
    });
}

/// Update the UI to reflect the authenticating state.
pub fn update_authenticating() {
    dispatch_to_main(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_auth_state(AuthState::Authenticating);
            }
            MenuBar::update_title(mtm, "Azure...");
            MenuBar::build_authenticating_menu(mtm);
            info!("UI updated: authenticating");
        }
    });
}

/// Update the UI to reflect the signed-in state.
pub fn update_signed_in(user_info: UserInfo, expires_at: DateTime<Utc>) {
    dispatch_to_main(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_auth_state(AuthState::SignedIn);
                state.set_user_info(Some(user_info));
                state.set_token_expiry(Some(expires_at));
            }
            MenuBar::update_title(mtm, "Azure");
            MenuBar::build_signed_in_menu(mtm);
            info!("UI updated: signed in");
        }
    });
}

/// Update the UI to reflect an error state.
pub fn update_error(message: String) {
    dispatch_to_main(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_auth_state(AuthState::Error {
                    message: message.clone(),
                });
            }
            MenuBar::update_title(mtm, "Azure!");
            MenuBar::build_error_menu(mtm, &message);
            info!("UI updated: error - {}", message);
        }
    });
}

/// Update the token expiry time (e.g., after refresh).
pub fn update_token_expiry(expires_at: DateTime<Utc>) {
    dispatch_to_main(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_token_expiry(Some(expires_at));
            }
            // Rebuild menu to update expiry display
            MenuBar::rebuild_menu(mtm);
            info!("Token expiry updated: {}", expires_at);
        }
    });
}

/// Update settings and rebuild menu.
pub fn update_settings(settings: Settings) {
    dispatch_to_main(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(state) = get_app_state() {
                state.set_settings(settings);
            }
            MenuBar::rebuild_menu(mtm);
            info!("Settings updated");
        }
    });
}

/// Rebuild the menu based on current state.
#[allow(dead_code)]
pub fn rebuild_menu() {
    dispatch_to_main(|| {
        if let Some(mtm) = MainThreadMarker::new() {
            MenuBar::rebuild_menu(mtm);
        }
    });
}

/// Helper to dispatch a closure to the main thread.
fn dispatch_to_main<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    if MainThreadMarker::new().is_some() {
        // Already on main thread, execute directly
        f();
    } else {
        // Dispatch to main thread
        Queue::main().exec_async(f);
    }
}

#[cfg(test)]
mod tests {
    // UI update tests would require a running macOS app context
}
