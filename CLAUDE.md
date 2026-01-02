# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**azurepim** is a macOS menu bar application built with Rust that manages Microsoft Azure authentication. It uses objc2 bindings (NOT Swift) for native macOS AppKit integration and implements OAuth2 with PKCE for secure authentication.

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

# Code quality
cargo clippy --all-targets --all-features
cargo fmt
cargo check
```

### Environment Variables
- `AZURE_CLIENT_ID` - Override OAuth client ID from config
- `AZURE_TENANT_ID` - Override tenant ID
- `RUST_LOG` - Override log level (trace, debug, info, warn, error)

## Architecture Overview

### Threading Model - CRITICAL

This application uses a **dual-threaded architecture**:

1. **Main Thread (AppKit Event Loop)**
   - ALL AppKit operations MUST run here
   - Use `MainThreadMarker` to prove thread safety
   - Use `dispatch::Queue::main().exec_async()` to dispatch from background threads

2. **Tokio Runtime (Background Threads)**
   - OAuth2 HTTP requests, Microsoft Graph API calls
   - Token refresh, keychain operations
   - NEVER call AppKit APIs directly from here

### Module Architecture

```
main.rs → Initializes Tokio runtime, AppState, MenuBar, runs NSApplication event loop

app/
  └─ delegate.rs → NSApplicationDelegate, OAuth callback URL handler (azurepim://callback)

menubar/
  ├─ state.rs    → AppState (global via OnceCell), AuthState enum, UserInfo, Settings
  ├─ delegate.rs → MenuActionTarget (Obj-C class via declare_class!), action channel
  ├─ builder.rs  → Menu construction with NSStatusBar/NSMenuItem
  └─ updates.rs  → Dynamic menu updates (dispatch to main thread)

auth/
  ├─ oauth.rs         → OAuth2Client (PKCE, auth URL, token exchange)
  ├─ token_manager.rs → Auto-refresh logic, expiry tracking
  └─ graph.rs         → Microsoft Graph API (user profile, organization)

keychain/
  ├─ mod.rs    → macOS Keychain operations (store/retrieve/delete tokens)
  └─ secure.rs → Zeroize wrappers for sensitive data

settings.rs → Auto-launch config, log directory management
error.rs    → AppError, AuthError, KeychainError, ApiError (using thiserror)
config.rs   → Load config.toml (embedded at compile time)
```

### Communication Flow

Menu actions flow through channels to avoid blocking the main thread:
1. User clicks menu item → MenuActionTarget receives action
2. Action sent via `mpsc::channel` to Tokio runtime
3. Tokio processes async work (OAuth, API calls)
4. UI updates dispatched back to main thread via `dispatch::Queue::main().exec_async()`

### OAuth2 Flow (PKCE with URL Scheme)

1. Generate PKCE `code_verifier` and `code_challenge` (SHA256)
2. Open browser to Azure AD with `code_challenge` and `state` (CSRF)
3. User authenticates, browser redirects to `azurepim://callback?code=...&state=...`
4. macOS delivers URL to `AppDelegate` via URL scheme handler
5. Validate `state`, exchange `code` for tokens with `code_verifier`
6. Store tokens in macOS Keychain
7. Fetch user profile from Microsoft Graph API
8. Update menu bar UI

### Security Patterns

**Keychain Service**: `com.azurepim.desktop`
- Accounts: `azure_access_token`, `azure_refresh_token`, `azure_user_info`, `azure_token_expiry`
- Use `zeroize` crate to clear sensitive data from memory
- Never log tokens (even in debug mode)

**OAuth2 Security**:
- PKCE prevents auth code interception
- State parameter validation prevents CSRF attacks

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
tenant = "organizations"  # or "common" or specific tenant ID
redirect_uri = "azurepim://callback"

[oauth.scopes]
scopes = ["https://graph.microsoft.com/User.Read", "openid", "profile", "email", "offline_access"]
```

## Azure AD Setup

1. Azure Portal → Azure AD → App registrations → New registration
2. Supported account types: "Accounts in any organizational directory" (multi-tenant)
3. Platform: Public client/native (mobile & desktop)
4. Redirect URI: `azurepim://callback`
5. Enable public client flows: Yes
6. API permissions (delegated): `User.Read`, `openid`, `profile`, `email`, `offline_access`

## Common Pitfalls

1. **Calling AppKit from Tokio**: Always dispatch to main thread with `Queue::main().exec_async()`
2. **Forgetting MainThreadMarker**: All AppKit functions require proof of main thread
3. **Logging tokens**: NEVER log access/refresh tokens
4. **Memory management**: All NSObject references use `Retained<T>` (ARC)

## Reference Documentation

- Full specification: `docs/spec.md`
- objc2: https://docs.rs/objc2/
- Apple AppKit: https://developer.apple.com/documentation/appkit
- Microsoft Identity Platform: https://learn.microsoft.com/en-us/entra/identity-platform/
