//! Azure PIM - macOS Menu Bar Application
//!
//! A menu bar application for Azure authentication management.

#![deny(clippy::all)]

mod app;
mod auth;
mod config;
mod error;
mod keychain;
mod menubar;
mod pim;
mod settings;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use app::delegate::AppDelegate;
use auth::callback_server::{self, CallbackResult};
use auth::graph::{GraphClient, UserInfo};
use auth::oauth::{parse_callback_url, OAuth2Client, PkceChallenge};
use config::Config;
use menubar::builder::MenuBar;
use menubar::delegate::{init_action_channel, MenuAction};
use menubar::state::init_app_state;
use menubar::updates;

fn main() {
    // Load .env file (if present) before anything else
    if let Err(e) = dotenvy::dotenv() {
        // .env file is optional - only log if it's not a "file not found" error
        if !e.to_string().contains("not found") {
            eprintln!("Warning: Failed to load .env file: {}", e);
        }
    }

    // Initialize logging
    init_logging();

    info!("Starting Azure PIM v{}", env!("CARGO_PKG_VERSION"));

    // Must run on main thread for AppKit
    let mtm = MainThreadMarker::new().expect("Must run on main thread");

    // Load configuration
    let config = match Config::load() {
        Ok(c) => {
            info!("Configuration loaded successfully");
            c
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            eprintln!("Configuration error: {}", e);
            eprintln!("\nPlease set the following environment variables:");
            eprintln!("  AZURE_CLIENT_ID=<your-azure-ad-client-id>");
            eprintln!("  AZURE_TENANT_ID=<your-tenant-id>");
            std::process::exit(1);
        }
    };

    // Initialize application state
    let _app_state = init_app_state();
    info!("Application state initialized");

    // Initialize Tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Create OAuth client
    let oauth_client = Arc::new(OAuth2Client::new(&config).expect("Failed to create OAuth client"));

    // Create Graph client
    let graph_client = Arc::new(GraphClient::new().expect("Failed to create Graph client"));

    // Create PIM client
    let pim_client = Arc::new(pim::PimClient::new().expect("Failed to create PIM client"));

    // Initialize action channel
    let action_rx = init_action_channel();

    // Get shared NSApplication
    let ns_app = NSApplication::sharedApplication(mtm);

    // Set activation policy (no dock icon)
    ns_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Create and set app delegate
    let delegate = AppDelegate::new(mtm);
    ns_app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    // Initialize menu bar
    MenuBar::init(mtm);
    MenuBar::build_signed_out_menu(mtm);
    info!("Menu bar initialized");

    // Spawn background task handler
    let config_clone = config.clone();
    let oauth_clone = Arc::clone(&oauth_client);
    let graph_clone = Arc::clone(&graph_client);
    let pim_clone = Arc::clone(&pim_client);

    runtime.spawn(async move {
        run_background_tasks(config_clone, oauth_clone, graph_clone, pim_clone, action_rx).await;
    });

    // Try to restore session from Keychain
    let oauth_restore = Arc::clone(&oauth_client);
    let graph_restore = Arc::clone(&graph_client);
    let config_restore = config.clone();

    runtime.spawn(async move {
        if let Err(e) = try_restore_session(oauth_restore, graph_restore, &config_restore).await {
            info!("No existing session to restore: {}", e);
            // Revert UI to signed-out state if restore fails
            updates::update_signed_out();
        }
    });

    info!("Starting application event loop");

    // Run the application event loop (blocks until app quits)
    unsafe {
        ns_app.run();
    }
}

/// Initialize tracing/logging.
fn init_logging() {
    // Create log directory
    if let Err(e) = settings::init_log_directory() {
        eprintln!("Warning: Could not create log directory: {}", e);
    }

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .init();
}

/// Try to restore a previous session from the Keychain.
async fn try_restore_session(
    oauth_client: Arc<OAuth2Client>,
    graph_client: Arc<GraphClient>,
    _config: &Config,
) -> Result<()> {
    info!("Attempting to restore previous session");

    // Check for existing refresh token BEFORE updating UI
    let refresh_token = keychain::get_refresh_token().context("No refresh token found")?;

    // Only show authenticating state after confirming we have a token to restore
    updates::update_authenticating();

    // Try to refresh the access token
    let token_response = oauth_client
        .refresh_token(&refresh_token)
        .await
        .context("Failed to refresh token")?;

    // Store new tokens
    keychain::store_access_token(&token_response.access_token)?;
    if let Some(new_refresh) = &token_response.refresh_token {
        keychain::store_refresh_token(new_refresh)?;
    }

    // Calculate expiry
    let expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
    keychain::store_token_expiry(&expires_at.to_rfc3339())?;

    // Fetch user info
    let user_profile = graph_client
        .get_user_profile(&token_response.access_token)
        .await
        .context("Failed to fetch user profile")?;

    let organization = graph_client
        .get_organization(&token_response.access_token)
        .await
        .context("Failed to fetch organization")?;

    let user_info = UserInfo::from_profile_and_org(user_profile, organization);

    // Store user info
    keychain::store_user_info(&user_info.to_json()?)?;

    // Update UI
    updates::update_signed_in(user_info, expires_at);

    info!("Session restored successfully");
    Ok(())
}

/// Run background tasks (action handler, OAuth callbacks).
async fn run_background_tasks(
    _config: Config,
    oauth_client: Arc<OAuth2Client>,
    graph_client: Arc<GraphClient>,
    pim_client: Arc<pim::PimClient>,
    mut action_rx: mpsc::Receiver<MenuAction>,
) {
    // Channel to receive callback results from the HTTP server
    let (callback_tx, mut callback_rx) = mpsc::channel::<CallbackResult>(1);

    // Channel to cancel the callback server
    let mut cancel_tx: Option<std::sync::mpsc::Sender<()>> = None;

    // State for in-progress OAuth flow
    let mut pending_pkce: Option<PkceChallenge> = None;
    let mut pending_state: Option<String> = None;

    loop {
        tokio::select! {
            // Handle menu actions
            Some(action) = action_rx.recv() => {
                match action {
                    MenuAction::SignIn => {
                        info!("Starting sign-in flow");

                        // Cancel any existing callback server first
                        if let Some(ctx) = cancel_tx.take() {
                            let _ = ctx.send(());
                            // Brief pause to let the old server release the port
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }

                        updates::update_authenticating();

                        // Generate PKCE
                        let pkce = PkceChallenge::new();
                        let (auth_url, state) = oauth_client.generate_auth_url(&pkce);

                        // Store for callback verification
                        pending_pkce = Some(pkce);
                        pending_state = Some(state.clone());

                        // Create cancellation channel
                        let (ctx, crx) = std::sync::mpsc::channel();
                        cancel_tx = Some(ctx);

                        // Start callback server in a separate thread
                        let tx = callback_tx.clone();
                        std::thread::spawn(move || {
                            let result = callback_server::start_callback_server(crx);
                            let _ = tx.blocking_send(result);
                        });

                        // Open browser
                        if let Err(e) = open::that(auth_url.as_str()) {
                            error!("Failed to open browser: {}", e);
                            // Cancel the server
                            if let Some(ctx) = cancel_tx.take() {
                                let _ = ctx.send(());
                            }
                            updates::update_error("Failed to open browser".to_string());
                        }
                    }
                    MenuAction::SignOut => {
                        info!("Signing out");
                        // Cancel any pending callback server
                        if let Some(ctx) = cancel_tx.take() {
                            let _ = ctx.send(());
                        }
                        pending_pkce = None;
                        pending_state = None;
                        if let Err(e) = keychain::delete_all() {
                            error!("Failed to clear keychain: {}", e);
                        }
                        updates::update_signed_out();
                    }
                    MenuAction::RefreshToken => {
                        info!("Manual token refresh requested");
                        if let Err(e) = refresh_token(&oauth_client).await {
                            error!("Token refresh failed: {}", e);
                            updates::update_error(e.to_string());
                        }
                    }
                    MenuAction::CopyToken => {
                        dispatch::Queue::main().exec_async(|| {
                            if let Some(mtm) = MainThreadMarker::new() {
                                menubar::delegate::copy_token_to_clipboard(mtm);
                            }
                        });
                    }
                    MenuAction::ToggleAutoLaunch(enabled) => {
                        if let Err(e) = settings::set_auto_launch(enabled) {
                            error!("Failed to set auto-launch: {}", e);
                        }
                        let mut settings = menubar::state::get_app_state()
                            .map(|s| s.get_settings())
                            .unwrap_or_default();
                        settings.auto_launch = enabled;
                        updates::update_settings(settings);
                    }
                    MenuAction::ToggleShowExpiry(enabled) => {
                        let mut settings = menubar::state::get_app_state()
                            .map(|s| s.get_settings())
                            .unwrap_or_default();
                        settings.show_expiry = enabled;
                        updates::update_settings(settings);
                    }
                    MenuAction::ClearData => {
                        info!("Clearing all data");
                        if let Err(e) = keychain::delete_all() {
                            error!("Failed to clear keychain: {}", e);
                        }
                        updates::update_signed_out();
                    }
                    MenuAction::CancelSignIn => {
                        info!("Sign-in cancelled");
                        // Cancel the callback server
                        if let Some(ctx) = cancel_tx.take() {
                            let _ = ctx.send(());
                        }
                        pending_pkce = None;
                        pending_state = None;
                        updates::update_signed_out();
                    }

                    // PIM Actions
                    MenuAction::ActivateRole { role_key, justification } => {
                        info!("Activating role {} with justification: {}", role_key, justification);
                        // TODO: Implement role activation with PimClient
                        // For now, just log the action
                    }
                    MenuAction::ToggleFavorite { role_key } => {
                        info!("Toggling favorite for role: {}", role_key);
                        if let Some(state) = menubar::state::get_app_state() {
                            let mut pim_state = state.get_pim_state();
                            pim_state.toggle_favorite(&role_key);
                            state.set_pim_state(pim_state.clone());

                            // Save to disk
                            if let Err(e) = pim::save_pim_settings(&pim_state.settings) {
                                error!("Failed to save PIM settings: {}", e);
                            }

                            updates::rebuild_menu();
                        }
                    }
                    MenuAction::RefreshPimRoles => {
                        info!("Refreshing PIM roles");
                        updates::update_pim_loading();

                        // Get refresh token
                        let refresh_token = match keychain::get_refresh_token() {
                            Ok(token) => token,
                            Err(e) => {
                                error!("Failed to get refresh token for PIM: {}", e);
                                updates::update_pim_error("Sign in required".to_string());
                                continue;
                            }
                        };

                        // Get user info for principal ID
                        let user_id = match menubar::state::get_app_state()
                            .and_then(|s| s.get_user_info())
                            .map(|u| u.user_id.clone())
                        {
                            Some(id) => id,
                            None => {
                                error!("No user info available for PIM");
                                updates::update_pim_error("User info not available".to_string());
                                continue;
                            }
                        };

                        // Get Graph API token to fetch user's groups
                        let graph_token = match oauth_client.refresh_token(&refresh_token).await {
                            Ok(response) => response.access_token,
                            Err(e) => {
                                error!("Failed to get Graph API token: {}", e);
                                updates::update_pim_error("Failed to refresh token".to_string());
                                continue;
                            }
                        };

                        // Fetch user's group memberships
                        let group_ids: Vec<String> = match graph_client.get_user_groups(&graph_token).await {
                            Ok(groups) => {
                                info!("User is member of {} groups", groups.len());
                                groups.into_iter().map(|g| g.id).collect()
                            }
                            Err(e) => {
                                warn!("Failed to fetch user groups: {} - continuing with user ID only", e);
                                vec![]
                            }
                        };

                        // Build list of all principal IDs (user + groups)
                        let mut principal_ids = vec![user_id.clone()];
                        principal_ids.extend(group_ids);
                        info!("Checking PIM roles for {} principal IDs", principal_ids.len());

                        // Get Management API token
                        let mgmt_token = match oauth_client.get_management_token(&refresh_token).await {
                            Ok(response) => response.access_token,
                            Err(e) => {
                                error!("Failed to get Management API token: {}", e);
                                updates::update_pim_permission_denied(
                                    "PIM access not available. Check Azure AD permissions.".to_string()
                                );
                                continue;
                            }
                        };

                        // Fetch eligible roles for user and all groups
                        match pim_client.get_all_eligible_roles(&mgmt_token, &principal_ids).await {
                            Ok(roles) => {
                                info!("Found {} eligible PIM roles", roles.len());
                                updates::update_pim_eligible_roles(roles);
                            }
                            Err(e) => {
                                error!("Failed to fetch PIM roles: {}", e);
                                updates::update_pim_error(format!("Failed to fetch roles: {}", e));
                            }
                        }

                        // Also fetch active assignments for user and all groups
                        match pim_client.get_active_assignments(&mgmt_token, &principal_ids).await {
                            Ok(assignments) => {
                                info!("Found {} active PIM assignments", assignments.len());
                                updates::update_pim_active_assignments(assignments);
                            }
                            Err(e) => {
                                error!("Failed to fetch active assignments: {}", e);
                                // Don't update error - roles may still be available
                            }
                        }
                    }
                }
            }

            // Handle OAuth callbacks from the HTTP server
            Some(callback_result) = callback_rx.recv() => {
                cancel_tx = None; // Server is done

                match callback_result {
                    CallbackResult::Success(url_string) => {
                        info!("Received OAuth callback from server");

                        let result = handle_oauth_callback(
                            &url_string,
                            pending_pkce.take(),
                            pending_state.take(),
                            &oauth_client,
                            &graph_client,
                        ).await;

                        match result {
                            Ok((user_info, expires_at)) => {
                                updates::update_signed_in(user_info, expires_at);
                            }
                            Err(e) => {
                                error!("OAuth callback error: {}", e);
                                updates::update_error(e.to_string());
                            }
                        }
                    }
                    CallbackResult::Cancelled => {
                        info!("OAuth callback server was cancelled");
                        pending_pkce = None;
                        pending_state = None;
                        // Don't update UI - already handled by CancelSignIn
                    }
                    CallbackResult::Error(e) => {
                        error!("Callback server error: {}", e);
                        pending_pkce = None;
                        pending_state = None;
                        updates::update_error(format!("Authentication error: {}", e));
                    }
                }
            }
        }
    }
}

/// Handle an OAuth callback URL.
async fn handle_oauth_callback(
    url_string: &str,
    pkce: Option<PkceChallenge>,
    expected_state: Option<String>,
    oauth_client: &OAuth2Client,
    graph_client: &GraphClient,
) -> Result<(UserInfo, chrono::DateTime<Utc>)> {
    // Parse the callback URL
    let (code, state) = parse_callback_url(url_string)?;

    // Verify state
    if expected_state.as_ref() != Some(&state) {
        anyhow::bail!("State mismatch - possible CSRF attack");
    }

    // Get PKCE verifier
    let pkce = pkce.ok_or_else(|| anyhow::anyhow!("No pending PKCE challenge"))?;

    // Exchange code for tokens
    let token_response = oauth_client
        .exchange_code(&code, &pkce.verifier)
        .await
        .context("Failed to exchange authorization code")?;

    // Store tokens
    keychain::store_access_token(&token_response.access_token)?;
    if let Some(refresh_token) = &token_response.refresh_token {
        keychain::store_refresh_token(refresh_token)?;
    }

    // Calculate expiry
    let expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
    keychain::store_token_expiry(&expires_at.to_rfc3339())?;

    // Fetch user info
    let user_profile = graph_client
        .get_user_profile(&token_response.access_token)
        .await
        .context("Failed to fetch user profile")?;

    let organization = graph_client
        .get_organization(&token_response.access_token)
        .await
        .context("Failed to fetch organization")?;

    let user_info = UserInfo::from_profile_and_org(user_profile, organization);

    // Store user info
    keychain::store_user_info(&user_info.to_json()?)?;

    info!("Sign-in successful: {}", user_info.display_name);

    Ok((user_info, expires_at))
}

/// Refresh the access token.
async fn refresh_token(oauth_client: &OAuth2Client) -> Result<()> {
    let refresh_token = keychain::get_refresh_token()?;

    let token_response = oauth_client
        .refresh_token(&refresh_token)
        .await
        .context("Token refresh failed")?;

    // Store new tokens
    keychain::store_access_token(&token_response.access_token)?;
    if let Some(new_refresh) = &token_response.refresh_token {
        keychain::store_refresh_token(new_refresh)?;
    }

    // Calculate and store expiry
    let expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);
    keychain::store_token_expiry(&expires_at.to_rfc3339())?;

    // Update UI
    updates::update_token_expiry(expires_at);

    info!("Token refreshed, expires at {}", expires_at);
    Ok(())
}
