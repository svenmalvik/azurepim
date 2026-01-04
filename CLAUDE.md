# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**azurepim** is a macOS menu bar application built with Rust that manages Microsoft Azure authentication and Privileged Identity Management (PIM) role activation. It uses objc2 bindings (NOT Swift) for native macOS AppKit integration and implements OAuth2 with PKCE for secure authentication.

**Key Features:**
- Azure AD authentication via OAuth2/PKCE
- Microsoft Graph API integration for user profile and group membership
- Azure PIM role discovery (including group-based role assignments)
- PIM role activation with justification presets
- Token management with auto-refresh
- Secure token storage in macOS Keychain

**Target Platform**: macOS 11.0+ (Big Sur and later), Intel and Apple Silicon

## Development Commands

```bash
# Build and run
cargo build                    # Development build
cargo run                      # Run application
RUST_LOG=debug cargo run       # Run with debug logging
cargo build --release          # Release build (binary: target/release/azurepim)

# Testing
cargo test                     # Run all tests
cargo test -- --nocapture      # Run with output visible
cargo test auth::tests         # Test specific module
cargo test pim::models::tests  # Test PIM models

# Code quality
cargo clippy --all-targets --all-features
cargo fmt
cargo check
```

### Environment Variables
- `AZURE_CLIENT_ID` - Azure AD application client ID (required)
- `AZURE_TENANT_ID` - Azure AD tenant ID (required)
- `AZURE_REDIRECT_URI` - Override redirect URI (default: `http://localhost:28491/callback`)
- `RUST_LOG` - Log level (trace, debug, info, warn, error)

### Configuration Files
- `config.toml` - Main configuration (embedded at compile time)
- `.env` - Environment variables (loaded at runtime, optional)

## Architecture Overview

### Threading Model - CRITICAL

This application uses a **dual-threaded architecture**:

1. **Main Thread (AppKit Event Loop)**
   - ALL AppKit operations MUST run here
   - Use `MainThreadMarker` to prove thread safety
   - Use `dispatch::Queue::main().exec_async()` to dispatch from background threads

2. **Tokio Runtime (Background Threads)**
   - OAuth2 HTTP requests, Microsoft Graph API calls
   - Azure Management API calls (PIM operations)
   - Token refresh, keychain operations
   - NEVER call AppKit APIs directly from here

### Module Architecture

```
main.rs           Initializes Tokio runtime, AppState, MenuBar, runs NSApplication event loop

app/
  delegate.rs     NSApplicationDelegate implementation

menubar/
  state.rs        AppState (global via OnceCell), AuthState enum, PimState, UserInfo, Settings
  delegate.rs     MenuActionTarget (Obj-C class via declare_class!), action channel
  builder.rs      Menu construction with NSStatusBar/NSMenuItem, PIM role menus
  updates.rs      Dynamic menu updates (dispatch to main thread)

auth/
  oauth.rs            OAuth2Client (PKCE, auth URL, token exchange, management token)
  token_manager.rs    Auto-refresh logic, expiry tracking
  graph.rs            Microsoft Graph API (user profile, organization, group memberships)
  callback_server.rs  Local HTTP server for OAuth callbacks (port 28491)

pim/
  mod.rs          Module exports
  client.rs       PimClient - Azure Management API for PIM operations
  models.rs       EligibleRole, ActiveAssignment, PimSettings, JustificationPreset
  cache.rs        PimCache with TTL for eligible roles
  settings.rs     PIM settings persistence (favorites, presets)

keychain/
  mod.rs          macOS Keychain operations (store/retrieve/delete tokens)
  secure.rs       Zeroize wrappers for sensitive data

settings.rs       Auto-launch config, log directory management
error.rs          AppError, AuthError, KeychainError, ApiError, PimError (using thiserror)
config.rs         Load config.toml (embedded at compile time)
```

### Communication Flow

Menu actions flow through channels to avoid blocking the main thread:
1. User clicks menu item -> MenuActionTarget receives action
2. Action sent via `mpsc::channel` to Tokio runtime
3. Tokio processes async work (OAuth, API calls, PIM operations)
4. UI updates dispatched back to main thread via `dispatch::Queue::main().exec_async()`

### OAuth2 Flow (PKCE with Localhost Callback)

The application uses a **localhost HTTP callback server** (not URL scheme) for better browser UX:

1. Generate PKCE `code_verifier` and `code_challenge` (SHA256)
2. Start local HTTP server on `localhost:28491`
3. Open browser to Azure AD with `code_challenge` and `state` (CSRF)
4. User authenticates in browser
5. Azure redirects to `http://localhost:28491/callback?code=...&state=...`
6. Callback server receives request, displays success/error page, returns URL to app
7. Validate `state`, exchange `code` for tokens with `code_verifier`
8. Store tokens in macOS Keychain
9. Fetch user profile from Microsoft Graph API
10. Update menu bar UI

### PIM Integration

The application discovers and manages Azure PIM roles:

1. **Role Discovery**: Queries Azure Management API for eligible roles
2. **Group-Based Roles**: Fetches user's group memberships via Graph API, then queries PIM for roles assigned to those groups
3. **Dual Token Strategy**: Uses Graph API token for user/group info, separate Management API token for PIM operations
4. **Role Activation**: Activates roles with justification and configurable duration
5. **Favorites**: Users can mark frequently-used roles as favorites for quick access

**PIM Menu Structure:**
```
Active Roles (N)              # Currently active assignments with time remaining
  Subscription - Role  X hr Y min left

Favorites                    # Quick access to favorite roles
  Subscription - Role >      # Submenu with justification presets
    Incident Investigation
    Debugging
    Maintenance
    ---
    Remove from Favorites

Eligible Roles >             # All eligible roles, grouped by subscription
  Subscription Name >
    Role Name >
      [justification presets]
      Add to Favorites

Refresh Roles
```

### Security Patterns

**Keychain Service**: `com.azurepim.desktop`
- Accounts: `azure_access_token`, `azure_refresh_token`, `azure_user_info`, `azure_token_expiry`
- Use `zeroize` crate to clear sensitive data from memory
- Never log tokens (even in debug mode)

**OAuth2 Security**:
- PKCE prevents auth code interception
- State parameter validation prevents CSRF attacks
- Localhost callback with XSS protection (HTML escaping)

**API Security**:
- Tokens stored in macOS Keychain (encrypted)
- Separate tokens for Graph API and Management API
- Token refresh before expiry

## Working with AppKit (objc2)

Always use `MainThreadMarker` for AppKit operations:

```rust
use objc2_foundation::MainThreadMarker;
use dispatch::Queue;

pub fn update_menu_item(title: &str) {
    if let Some(mtm) = MainThreadMarker::new() {
        update_on_main_thread(title, mtm);
        return;
    }
    // Not on main thread - dispatch
    let title = title.to_string();
    Queue::main().exec_async(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            update_on_main_thread(&title, mtm);
        }
    });
}
```

### Objective-C Delegates

Use `declare_class!` macro to define custom Objective-C classes:

```rust
declare_class!(
    pub struct MenuActionTarget;

    unsafe impl ClassType for MenuActionTarget {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "MenuActionTarget";
    }
    // ... methods with #[method(selector:)] attributes
);
```

## Configuration

Edit `config.toml` or set environment variables:

```toml
[oauth]
client_id = "YOUR_AZURE_AD_CLIENT_ID"
tenant = "YOUR_TENANT_ID"
redirect_uri = "http://localhost:28491/callback"

[oauth.scopes]
# Graph API scopes (for user profile and group membership)
scopes = [
    "https://graph.microsoft.com/User.Read",
    "https://graph.microsoft.com/GroupMember.Read.All",
    "openid",
    "profile",
    "email",
    "offline_access"
]

[api]
graph_base_url = "https://graph.microsoft.com/v1.0"
management_base_url = "https://management.azure.com"
```

**Note**: The Management API scope (`https://management.azure.com/.default`) is acquired via a separate token request because Azure AD doesn't allow multiple resource scopes in a single token.

## Azure AD Setup

### App Registration

1. Azure Portal -> Azure AD -> App registrations -> New registration
2. **Name**: Azure PIM Desktop (or similar)
3. **Supported account types**: "Accounts in any organizational directory" (multi-tenant) or single-tenant
4. **Platform**: Public client/native (mobile & desktop)
5. **Redirect URI**: `http://localhost:28491/callback` (Web platform for localhost)
6. Enable **public client flows**: Yes (Authentication -> Advanced settings)

### API Permissions (Delegated)

| API | Permission | Purpose |
|-----|------------|---------|
| Microsoft Graph | `User.Read` | Read user profile |
| Microsoft Graph | `GroupMember.Read.All` | List user's group memberships (for group-based PIM roles) |
| Microsoft Graph | `openid` | OpenID Connect |
| Microsoft Graph | `profile` | User profile claims |
| Microsoft Graph | `email` | User email |
| Microsoft Graph | `offline_access` | Refresh tokens |
| Azure Service Management | `user_impersonation` | PIM role management |

**Admin Consent**: `GroupMember.Read.All` may require admin consent in some tenants.

### PIM Requirements

For PIM functionality to work, the user must:
1. Have eligible role assignments in Azure PIM (direct or via group membership)
2. Have appropriate Azure RBAC permissions to query PIM APIs
3. Be in a tenant with Azure AD Premium P2 or equivalent licensing

## Error Handling

The codebase uses a structured error hierarchy:

```rust
AppError                    // Top-level application error
  AuthError                // Authentication errors (OAuth, token)
  KeychainError            // Keychain storage errors
  ApiError                 // Microsoft Graph API errors
  PimError                 // PIM-specific errors

// All errors implement:
// - Display (human-readable messages)
// - user_message() -> &str (UI-friendly messages)
// - requires_sign_out() -> bool (whether to trigger sign-out)
```

## Common Pitfalls

1. **Calling AppKit from Tokio**: Always dispatch to main thread with `Queue::main().exec_async()`
2. **Forgetting MainThreadMarker**: All AppKit functions require proof of main thread
3. **Logging tokens**: NEVER log access/refresh tokens
4. **Memory management**: All NSObject references use `Retained<T>` (ARC)
5. **Multiple API tokens**: Graph API and Management API require separate tokens
6. **Group-based roles**: Remember to query PIM for group IDs, not just user ID
7. **Callback server port conflicts**: Port 28491 must be available; existing server cancelled before new sign-in

## Testing

### Unit Tests

```bash
cargo test                           # All tests
cargo test pim::models::tests        # PIM model tests
cargo test auth::graph::tests        # Graph API tests
cargo test error::tests              # Error handling tests
```

### Manual Testing

1. **Sign-in flow**: Click "Sign In" -> Browser opens -> Authenticate -> Success page shown
2. **PIM roles**: Click "Refresh Roles" -> View eligible and active roles
3. **Token refresh**: Click "Refresh Token" -> Token expiry updates
4. **Role activation**: (Future) Select role -> Choose justification -> Activate

## Reference Documentation

- Full specification: `docs/spec.md`
- Phase 2 PIM spec: `docs/phase2-pim-spec.md`
- Azure app registration: `docs/azure-app-registration.md`
- objc2: https://docs.rs/objc2/
- Apple AppKit: https://developer.apple.com/documentation/appkit
- Microsoft Identity Platform: https://learn.microsoft.com/en-us/entra/identity-platform/
- Azure PIM API: https://learn.microsoft.com/en-us/rest/api/authorization/role-eligibility-schedule-instances
