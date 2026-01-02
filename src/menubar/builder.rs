//! Menu bar and menu construction using AppKit.

use crate::menubar::delegate::{send_activate_role, send_toggle_favorite, MenuActionTarget};
use crate::menubar::state::{get_app_state, AuthState, PimState};
use crate::pim::{EligibleRole, JustificationPreset, PimApiStatus};
use objc2::rc::Retained;
use objc2::runtime::Sel;
use objc2::sel;
use objc2_app_kit::{
    NSControlStateValueOff, NSControlStateValueOn, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{MainThreadMarker, NSString};
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tracing::info;

/// Global menu bar instance.
static MENU_BAR: OnceCell<Mutex<MenuBarInner>> = OnceCell::new();

/// Initialize the global menu bar.
pub fn init_menu_bar(mtm: MainThreadMarker) -> &'static Mutex<MenuBarInner> {
    MENU_BAR.get_or_init(|| {
        let inner = MenuBarInner::new(mtm);
        Mutex::new(inner)
    })
}

/// Get the global menu bar.
pub fn get_menu_bar() -> Option<&'static Mutex<MenuBarInner>> {
    MENU_BAR.get()
}

/// Menu bar wrapper that holds the status item and menu.
pub struct MenuBarInner {
    /// Retained to keep the status item alive (never read, but must not be dropped).
    _status_item: Retained<NSStatusItem>,
    pub menu: Retained<NSMenu>,
    pub action_target: Retained<MenuActionTarget>,
}

// SAFETY: MenuBarInner is only accessed from the main thread via MainThreadMarker checks.
unsafe impl Send for MenuBarInner {}
unsafe impl Sync for MenuBarInner {}

impl MenuBarInner {
    /// Create a new menu bar.
    fn new(mtm: MainThreadMarker) -> Self {
        info!("Creating menu bar");

        // Create the action target for menu callbacks
        let action_target = MenuActionTarget::new(mtm);

        unsafe {
            // Get the system status bar
            let status_bar = NSStatusBar::systemStatusBar();

            // Create a status item with variable length
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            // Set the menu bar icon using SF Symbol
            if let Some(button) = status_item.button(mtm) {
                // Use "lock.shield" SF Symbol - represents identity/authentication
                let symbol_name = NSString::from_str("lock.shield");

                if let Some(image) = NSImage::imageWithSymbolName_variableValue(&symbol_name, 1.0) {
                    // Set as template so it adapts to dark/light menu bar
                    image.setTemplate(true);
                    button.setImage(Some(&image));
                } else {
                    // Fallback to text if SF Symbol not available
                    let title = NSString::from_str("🔐");
                    button.setTitle(&title);
                }
            }

            // Create the menu
            let menu = NSMenu::new(mtm);

            // Set the menu on the status item
            status_item.setMenu(Some(&menu));

            Self {
                _status_item: status_item,
                menu,
                action_target,
            }
        }
    }
}

/// Public menu bar API.
pub struct MenuBar;

impl MenuBar {
    /// Initialize the menu bar.
    pub fn init(mtm: MainThreadMarker) -> &'static Mutex<MenuBarInner> {
        init_menu_bar(mtm)
    }

    /// Build the signed-out menu.
    pub fn build_signed_out_menu(mtm: MainThreadMarker) {
        if let Some(menu_bar) = get_menu_bar() {
            let inner = menu_bar.lock().unwrap();
            let menu = &inner.menu;
            let target = Some(&*inner.action_target);

            // Clear existing items
            unsafe {
                menu.removeAllItems();
            }

            // Sign In item
            let sign_in_item =
                create_menu_item(mtm, "Sign In to Azure", Some(sel!(signIn:)), target);
            menu.addItem(&sign_in_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Quit item
            let quit_item = create_menu_item(mtm, "Quit", Some(sel!(terminate:)), None);
            unsafe {
                quit_item.setKeyEquivalent(&NSString::from_str("q"));
            }
            menu.addItem(&quit_item);

            info!("Built signed-out menu");
        }
    }

    /// Build the authenticating menu.
    pub fn build_authenticating_menu(mtm: MainThreadMarker) {
        if let Some(menu_bar) = get_menu_bar() {
            let inner = menu_bar.lock().unwrap();
            let menu = &inner.menu;
            let target = Some(&*inner.action_target);

            // Clear existing items
            unsafe {
                menu.removeAllItems();
            }

            // Status item (disabled)
            let status_item = create_menu_item(mtm, "Signing in...", None, None);
            unsafe {
                status_item.setEnabled(false);
            }
            menu.addItem(&status_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Cancel item
            let cancel_item = create_menu_item(mtm, "Cancel", Some(sel!(cancelSignIn:)), target);
            menu.addItem(&cancel_item);

            // Quit item
            let quit_item = create_menu_item(mtm, "Quit", Some(sel!(terminate:)), None);
            unsafe {
                quit_item.setKeyEquivalent(&NSString::from_str("q"));
            }
            menu.addItem(&quit_item);

            info!("Built authenticating menu");
        }
    }

    /// Build the signed-in menu with user info.
    pub fn build_signed_in_menu(mtm: MainThreadMarker) {
        if let Some(menu_bar) = get_menu_bar() {
            let inner = menu_bar.lock().unwrap();
            let menu = &inner.menu;
            let target = Some(&*inner.action_target);

            // Clear existing items
            unsafe {
                menu.removeAllItems();
            }

            // Get user info from app state
            let app_state = get_app_state();
            let user_info = app_state.as_ref().and_then(|s| s.get_user_info());

            // User name (disabled, bold-like appearance)
            let name = user_info
                .as_ref()
                .map(|u| u.display_name.as_str())
                .unwrap_or("Unknown User");
            let name_item = create_menu_item(mtm, name, None, None);
            unsafe {
                name_item.setEnabled(false);
            }
            menu.addItem(&name_item);

            // Email (disabled)
            let email = user_info
                .as_ref()
                .map(|u| u.email.as_str())
                .unwrap_or("No email");
            let email_item = create_menu_item(mtm, email, None, None);
            unsafe {
                email_item.setEnabled(false);
            }
            menu.addItem(&email_item);

            // Tenant (disabled)
            let tenant = user_info
                .as_ref()
                .map(|u| u.tenant_name.as_str())
                .unwrap_or("Unknown Tenant");
            let tenant_item = create_menu_item(mtm, tenant, None, None);
            unsafe {
                tenant_item.setEnabled(false);
            }
            menu.addItem(&tenant_item);

            // Token expiry (if enabled in settings)
            if let Some(state) = app_state.as_ref() {
                if state.get_settings().show_expiry {
                    if let Some(expiry) = state.get_token_expiry() {
                        let duration = expiry - chrono::Utc::now();
                        let expiry_text = format!(
                            "Expires in {}",
                            crate::auth::token_manager::format_duration(duration)
                        );
                        let expiry_item = create_menu_item(mtm, &expiry_text, None, None);
                        unsafe {
                            expiry_item.setEnabled(false);
                        }
                        menu.addItem(&expiry_item);
                    }
                }
            }

            // PIM Section
            if let Some(state) = app_state.as_ref() {
                let pim_state = state.get_pim_state();
                add_pim_section(mtm, menu, &pim_state, target);
            }

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Copy Access Token
            let copy_item =
                create_menu_item(mtm, "Copy Access Token", Some(sel!(copyToken:)), target);
            menu.addItem(&copy_item);

            // Refresh Token
            let refresh_item =
                create_menu_item(mtm, "Refresh Token", Some(sel!(refreshToken:)), target);
            menu.addItem(&refresh_item);

            // Sign Out
            let sign_out_item = create_menu_item(mtm, "Sign Out", Some(sel!(signOut:)), target);
            menu.addItem(&sign_out_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Settings submenu
            let settings_menu = create_settings_submenu(mtm, target);
            let settings_item = create_menu_item(mtm, "Settings", None, None);
            settings_item.setSubmenu(Some(&settings_menu));
            menu.addItem(&settings_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Quit
            let quit_item = create_menu_item(mtm, "Quit", Some(sel!(terminate:)), None);
            unsafe {
                quit_item.setKeyEquivalent(&NSString::from_str("q"));
            }
            menu.addItem(&quit_item);

            info!("Built signed-in menu");
        }
    }

    /// Build the error menu.
    pub fn build_error_menu(mtm: MainThreadMarker, error_message: &str) {
        if let Some(menu_bar) = get_menu_bar() {
            let inner = menu_bar.lock().unwrap();
            let menu = &inner.menu;
            let target = Some(&*inner.action_target);

            // Clear existing items
            unsafe {
                menu.removeAllItems();
            }

            // Error status (disabled)
            let error_item = create_menu_item(mtm, "Authentication Failed", None, None);
            unsafe {
                error_item.setEnabled(false);
            }
            menu.addItem(&error_item);

            // Error message (disabled)
            let msg_item = create_menu_item(mtm, error_message, None, None);
            unsafe {
                msg_item.setEnabled(false);
            }
            menu.addItem(&msg_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Try Again
            let retry_item = create_menu_item(mtm, "Try Again", Some(sel!(signIn:)), target);
            menu.addItem(&retry_item);

            // Sign Out
            let sign_out_item = create_menu_item(mtm, "Sign Out", Some(sel!(signOut:)), target);
            menu.addItem(&sign_out_item);

            // Separator
            let separator = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator);

            // Quit
            let quit_item = create_menu_item(mtm, "Quit", Some(sel!(terminate:)), None);
            unsafe {
                quit_item.setKeyEquivalent(&NSString::from_str("q"));
            }
            menu.addItem(&quit_item);

            info!("Built error menu");
        }
    }

    /// Rebuild the menu based on current state.
    pub fn rebuild_menu(mtm: MainThreadMarker) {
        if let Some(state) = get_app_state() {
            match state.get_auth_state() {
                AuthState::SignedOut => Self::build_signed_out_menu(mtm),
                AuthState::Authenticating => Self::build_authenticating_menu(mtm),
                AuthState::SignedIn => Self::build_signed_in_menu(mtm),
                AuthState::Error { message } => Self::build_error_menu(mtm, &message),
                AuthState::Offline => Self::build_signed_in_menu(mtm),
            }
        }
    }
}

/// Create a menu item with the given title, action, and optional target.
fn create_menu_item(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<Sel>,
    target: Option<&MenuActionTarget>,
) -> Retained<NSMenuItem> {
    let ns_title = NSString::from_str(title);
    let key_equiv = NSString::from_str("");

    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(mtm.alloc(), &ns_title, action, &key_equiv)
    };

    // Set the target to our MenuActionTarget for custom actions
    // (not for system actions like terminate:)
    if action.is_some() && action != Some(sel!(terminate:)) {
        if let Some(target) = target {
            unsafe {
                item.setTarget(Some(target));
            }
        }
    }

    item
}

/// Create the settings submenu.
fn create_settings_submenu(
    mtm: MainThreadMarker,
    target: Option<&MenuActionTarget>,
) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);

    // Auto-launch toggle
    let auto_launch_item = create_menu_item(
        mtm,
        "Auto-launch at login",
        Some(sel!(toggleAutoLaunch:)),
        target,
    );
    // Set checkmark based on current setting
    if let Some(state) = get_app_state() {
        let settings = state.get_settings();
        unsafe {
            auto_launch_item.setState(if settings.auto_launch {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }
    menu.addItem(&auto_launch_item);

    // Show expiry toggle
    let show_expiry_item = create_menu_item(
        mtm,
        "Show expiry countdown",
        Some(sel!(toggleShowExpiry:)),
        target,
    );
    if let Some(state) = get_app_state() {
        let settings = state.get_settings();
        unsafe {
            show_expiry_item.setState(if settings.show_expiry {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }
    menu.addItem(&show_expiry_item);

    // Separator
    let separator = NSMenuItem::separatorItem(mtm);
    menu.addItem(&separator);

    // Clear all data
    let clear_item = create_menu_item(mtm, "Clear all data...", Some(sel!(clearData:)), target);
    menu.addItem(&clear_item);

    menu
}

// ─────────────────────────────────────────────────────────────────────────────
// PIM Menu Section
// ─────────────────────────────────────────────────────────────────────────────

/// Add the PIM section to the menu.
fn add_pim_section(
    mtm: MainThreadMarker,
    menu: &NSMenu,
    pim_state: &PimState,
    target: Option<&MenuActionTarget>,
) {
    // Separator before PIM section
    let separator = NSMenuItem::separatorItem(mtm);
    menu.addItem(&separator);

    // Active Roles Section (if any)
    if !pim_state.active_assignments.is_empty() {
        let header_text = format!("Active Roles ({})", pim_state.active_assignments.len());
        let header = create_menu_item(mtm, &header_text, None, None);
        unsafe {
            header.setEnabled(false);
        }
        menu.addItem(&header);

        for assignment in &pim_state.active_assignments {
            let item_text = assignment.display_text_with_time();
            let item = create_menu_item(mtm, &item_text, None, None);
            unsafe {
                item.setEnabled(false);
            }
            menu.addItem(&item);
        }

        // Separator after active roles
        let separator = NSMenuItem::separatorItem(mtm);
        menu.addItem(&separator);
    }

    // Handle different API states
    match &pim_state.api_status {
        PimApiStatus::Loading => {
            let loading_item = create_menu_item(mtm, "PIM Roles (loading...)", None, None);
            unsafe {
                loading_item.setEnabled(false);
            }
            menu.addItem(&loading_item);
        }
        PimApiStatus::PermissionDenied { message } => {
            let error_item = create_menu_item(mtm, &format!("PIM: {}", message), None, None);
            unsafe {
                error_item.setEnabled(false);
            }
            menu.addItem(&error_item);
        }
        PimApiStatus::Unavailable { error } => {
            let error_item = create_menu_item(mtm, &format!("PIM: {}", error), None, None);
            unsafe {
                error_item.setEnabled(false);
            }
            menu.addItem(&error_item);
        }
        PimApiStatus::Unknown | PimApiStatus::Available => {
            if pim_state.eligible_roles.is_empty() {
                let empty_item = create_menu_item(mtm, "No eligible PIM roles", None, None);
                unsafe {
                    empty_item.setEnabled(false);
                }
                menu.addItem(&empty_item);
            } else {
                // ★ Favorites section (flat, at top for quick access)
                let favorites = pim_state.favorite_roles();
                if !favorites.is_empty() {
                    let fav_header = create_menu_item(mtm, "★ Favorites", None, None);
                    unsafe {
                        fav_header.setEnabled(false);
                    }
                    menu.addItem(&fav_header);

                    for role in favorites {
                        let role_item = create_role_menu_item(mtm, role, true);
                        menu.addItem(&role_item);
                    }

                    // Separator after favorites
                    let separator = NSMenuItem::separatorItem(mtm);
                    menu.addItem(&separator);
                }

                // Eligible Roles submenu (grouped by subscription)
                let roles_by_sub = pim_state.roles_by_subscription();
                if !roles_by_sub.is_empty() {
                    let eligible_item = create_menu_item(mtm, "Eligible Roles", None, None);
                    let eligible_submenu = create_eligible_roles_submenu(mtm, &roles_by_sub, pim_state);
                    eligible_item.setSubmenu(Some(&eligible_submenu));
                    menu.addItem(&eligible_item);
                }
            }
        }
    }

    // Refresh Roles item
    let refresh_item =
        create_menu_item(mtm, "↻ Refresh Roles", Some(sel!(refreshPimRoles:)), target);
    menu.addItem(&refresh_item);
}

/// Create the "Eligible Roles" submenu with subscriptions as submenus.
fn create_eligible_roles_submenu(
    mtm: MainThreadMarker,
    roles_by_subscription: &[(&str, Vec<&EligibleRole>)],
    pim_state: &PimState,
) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);

    for (subscription_name, roles) in roles_by_subscription {
        if roles.is_empty() {
            continue;
        }

        // Subscription as a submenu item
        let sub_item = create_menu_item(mtm, subscription_name, None, None);
        let sub_menu = NSMenu::new(mtm);

        // Add roles within this subscription
        for role in roles {
            let is_favorite = pim_state.is_favorite(role);
            let role_item = create_role_menu_item_short(mtm, role, is_favorite);
            sub_menu.addItem(&role_item);
        }

        sub_item.setSubmenu(Some(&sub_menu));
        menu.addItem(&sub_item);
    }

    menu
}

/// Create a menu item for a role (full display: "subscription - role").
fn create_role_menu_item(
    mtm: MainThreadMarker,
    role: &EligibleRole,
    is_favorite: bool,
) -> Retained<NSMenuItem> {
    let title = format!("  {} - {}", role.subscription_name, role.role_name);
    let item = create_menu_item(mtm, &title, None, None);

    // Create submenu with justification presets
    let submenu = create_justification_submenu(mtm, role, is_favorite);
    item.setSubmenu(Some(&submenu));

    item
}

/// Create a menu item for a role (short display: just role name, used within subscription submenu).
fn create_role_menu_item_short(
    mtm: MainThreadMarker,
    role: &EligibleRole,
    is_favorite: bool,
) -> Retained<NSMenuItem> {
    let star = if is_favorite { "★ " } else { "" };
    let title = format!("{}{}", star, role.role_name);
    let item = create_menu_item(mtm, &title, None, None);

    // Create submenu with justification presets
    let submenu = create_justification_submenu(mtm, role, is_favorite);
    item.setSubmenu(Some(&submenu));

    item
}

/// Create the justification submenu for a role.
fn create_justification_submenu(
    mtm: MainThreadMarker,
    role: &EligibleRole,
    is_favorite: bool,
) -> Retained<NSMenu> {
    let menu = NSMenu::new(mtm);
    let role_key = role.favorites_key();

    // Add builtin justification presets
    let presets = JustificationPreset::builtin_presets();
    for preset in presets {
        let preset_item = create_preset_menu_item(mtm, &preset, &role_key);
        menu.addItem(&preset_item);
    }

    // Separator
    let separator = NSMenuItem::separatorItem(mtm);
    menu.addItem(&separator);

    // Favorite toggle
    let favorite_text = if is_favorite {
        "Remove from Favorites"
    } else {
        "Add to Favorites"
    };
    let favorite_item = create_favorite_toggle_item(mtm, favorite_text, &role_key);
    menu.addItem(&favorite_item);

    menu
}

/// Create a menu item for a justification preset.
fn create_preset_menu_item(
    mtm: MainThreadMarker,
    preset: &JustificationPreset,
    role_key: &str,
) -> Retained<NSMenuItem> {
    let ns_title = NSString::from_str(&preset.label);
    let key_equiv = NSString::from_str("");

    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(mtm.alloc(), &ns_title, None, &key_equiv)
    };

    // Store role_key and justification for the callback
    let role_key = role_key.to_string();
    let justification = preset.justification.clone();

    // Set up click handler using a block
    unsafe {
        // We need to use objc2's block support to create a callback
        // For now, we'll set up a custom target/action mechanism
        // The actual activation will happen via notification or custom delegate

        // Create a custom action target that captures the role_key and justification
        // Since we can't easily pass data through NSMenuItem actions, we'll use
        // the representedObject pattern or a custom approach

        // For simplicity in the MVP, we'll use a workaround:
        // Store the action data in the menu item's tag or identifier
        // and look it up when the action is triggered

        // Alternative: Use objc2-block to create a proper block callback
        // For now, we'll make this a simple clickable item that triggers activation

        // Set a unique tag based on hash
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            role_key.hash(&mut hasher);
            justification.hash(&mut hasher);
            hasher.finish() as isize
        };
        item.setTag(hash);

        // Store the callback data globally and dispatch when clicked
        // For the MVP, we'll trigger this via a notification mechanism
        store_preset_callback(&role_key, &justification, hash);
    }

    item
}

/// Create a menu item for toggling favorite status.
fn create_favorite_toggle_item(
    mtm: MainThreadMarker,
    title: &str,
    role_key: &str,
) -> Retained<NSMenuItem> {
    let ns_title = NSString::from_str(title);
    let key_equiv = NSString::from_str("");

    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(mtm.alloc(), &ns_title, None, &key_equiv)
    };

    // Store callback data for favorite toggle
    let role_key = role_key.to_string();
    unsafe {
        let hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            role_key.hash(&mut hasher);
            "favorite".hash(&mut hasher);
            hasher.finish() as isize
        };
        item.setTag(hash);
        store_favorite_callback(&role_key, hash);
    }

    item
}

// ─────────────────────────────────────────────────────────────────────────────
// PIM Callback Storage (MVP approach)
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::RwLock;

/// Global storage for preset callbacks (role_key, justification) by tag.
static PRESET_CALLBACKS: OnceCell<RwLock<HashMap<isize, (String, String)>>> = OnceCell::new();

/// Global storage for favorite callbacks (role_key) by tag.
static FAVORITE_CALLBACKS: OnceCell<RwLock<HashMap<isize, String>>> = OnceCell::new();

fn get_preset_callbacks() -> &'static RwLock<HashMap<isize, (String, String)>> {
    PRESET_CALLBACKS.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_favorite_callbacks() -> &'static RwLock<HashMap<isize, String>> {
    FAVORITE_CALLBACKS.get_or_init(|| RwLock::new(HashMap::new()))
}

fn store_preset_callback(role_key: &str, justification: &str, tag: isize) {
    if let Ok(mut callbacks) = get_preset_callbacks().write() {
        callbacks.insert(tag, (role_key.to_string(), justification.to_string()));
    }
}

fn store_favorite_callback(role_key: &str, tag: isize) {
    if let Ok(mut callbacks) = get_favorite_callbacks().write() {
        callbacks.insert(tag, role_key.to_string());
    }
}

/// Look up and execute a preset callback by tag.
#[allow(dead_code)]
pub fn execute_preset_callback(tag: isize) {
    if let Ok(callbacks) = get_preset_callbacks().read() {
        if let Some((role_key, justification)) = callbacks.get(&tag) {
            send_activate_role(role_key.clone(), justification.clone());
        }
    }
}

/// Look up and execute a favorite callback by tag.
#[allow(dead_code)]
pub fn execute_favorite_callback(tag: isize) {
    if let Ok(callbacks) = get_favorite_callbacks().read() {
        if let Some(role_key) = callbacks.get(&tag) {
            send_toggle_favorite(role_key.clone());
        }
    }
}
