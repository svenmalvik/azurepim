//! Menu item action delegates.
//!
//! Handles menu item clicks and dispatches to the appropriate callbacks.

use objc2::mutability::MainThreadOnly;
use objc2::rc::Retained;
use objc2::{declare_class, msg_send_id, ClassType, DeclaredClass};
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::keychain;
use crate::menubar::state::{get_app_state, MenuCallbacks};

/// Global menu callbacks.
#[allow(dead_code)]
static MENU_CALLBACKS: OnceCell<Arc<MenuCallbacks>> = OnceCell::new();

/// Initialize menu callbacks.
#[allow(dead_code)]
pub fn init_menu_callbacks(callbacks: MenuCallbacks) {
    MENU_CALLBACKS
        .set(Arc::new(callbacks))
        .expect("Menu callbacks already initialized");
}

/// Get menu callbacks.
#[allow(dead_code)]
pub fn get_menu_callbacks() -> Option<&'static Arc<MenuCallbacks>> {
    MENU_CALLBACKS.get()
}

/// Channel for sending menu actions to the Tokio runtime.
static ACTION_SENDER: OnceCell<mpsc::Sender<MenuAction>> = OnceCell::new();

/// Menu action types.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MenuAction {
    SignIn,
    SignOut,
    RefreshToken,
    CopyToken,
    ToggleAutoLaunch(bool),
    ToggleShowExpiry(bool),
    ClearData,
    CancelSignIn,
}

/// Initialize the action channel.
pub fn init_action_channel() -> mpsc::Receiver<MenuAction> {
    let (tx, rx) = mpsc::channel(10);
    ACTION_SENDER
        .set(tx)
        .expect("Action channel already initialized");
    rx
}

/// Send an action to be processed.
fn send_action(action: MenuAction) {
    if let Some(sender) = ACTION_SENDER.get() {
        if let Err(e) = sender.try_send(action) {
            error!("Failed to send menu action: {}", e);
        }
    }
}

// Define the MenuActionTarget class that receives menu item actions
declare_class!(
    pub struct MenuActionTarget;

    unsafe impl ClassType for MenuActionTarget {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "MenuActionTarget";
    }

    impl DeclaredClass for MenuActionTarget {}

    unsafe impl NSObjectProtocol for MenuActionTarget {}

    unsafe impl MenuActionTarget {
        #[method(signIn:)]
        fn sign_in(&self, _sender: &NSObject) {
            info!("Sign In clicked");
            send_action(MenuAction::SignIn);
        }

        #[method(signOut:)]
        fn sign_out(&self, _sender: &NSObject) {
            info!("Sign Out clicked");
            send_action(MenuAction::SignOut);
        }

        #[method(refreshToken:)]
        fn refresh_token(&self, _sender: &NSObject) {
            info!("Refresh Token clicked");
            send_action(MenuAction::RefreshToken);
        }

        #[method(copyToken:)]
        fn copy_token(&self, _sender: &NSObject) {
            info!("Copy Token clicked");
            send_action(MenuAction::CopyToken);
        }

        #[method(toggleAutoLaunch:)]
        fn toggle_auto_launch(&self, _sender: &NSObject) {
            info!("Toggle Auto Launch clicked");
            if let Some(state) = get_app_state() {
                let current = state.get_settings().auto_launch;
                send_action(MenuAction::ToggleAutoLaunch(!current));
            }
        }

        #[method(toggleShowExpiry:)]
        fn toggle_show_expiry(&self, _sender: &NSObject) {
            info!("Toggle Show Expiry clicked");
            if let Some(state) = get_app_state() {
                let current = state.get_settings().show_expiry;
                send_action(MenuAction::ToggleShowExpiry(!current));
            }
        }

        #[method(clearData:)]
        fn clear_data(&self, _sender: &NSObject) {
            info!("Clear Data clicked");
            send_action(MenuAction::ClearData);
        }

        #[method(cancelSignIn:)]
        fn cancel_sign_in(&self, _sender: &NSObject) {
            info!("Cancel Sign In clicked");
            send_action(MenuAction::CancelSignIn);
        }
    }
);

impl MenuActionTarget {
    /// Create a new MenuActionTarget.
    #[allow(dead_code)]
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

/// Copy the access token to the clipboard.
///
/// Schedules automatic clearing of the clipboard after 2 minutes.
pub fn copy_token_to_clipboard(_mtm: MainThreadMarker) {
    match keychain::get_access_token() {
        Ok(token) => {
            unsafe {
                let pasteboard = NSPasteboard::generalPasteboard();
                pasteboard.clearContents();

                let ns_token = NSString::from_str(&token);

                // Use setString:forType: with the string type
                // NSPasteboardTypeString is "public.utf8-plain-text"
                let type_str = NSString::from_str("public.utf8-plain-text");
                pasteboard.setString_forType(&ns_token, &type_str);
            }

            info!("Access token copied to clipboard");

            // Schedule clipboard clear after 2 minutes
            schedule_clipboard_clear();
        }
        Err(e) => {
            error!("Failed to get access token: {}", e);
        }
    }
}

/// Schedule clearing the clipboard after 2 minutes.
fn schedule_clipboard_clear() {
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(120)).await;

        // Clear from main thread
        dispatch::Queue::main().exec_async(|| {
            unsafe {
                let pasteboard = NSPasteboard::generalPasteboard();
                pasteboard.clearContents();
            }
            info!("Clipboard cleared after 2 minutes");
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_action() {
        let action = MenuAction::SignIn;
        assert!(matches!(action, MenuAction::SignIn));
    }
}
