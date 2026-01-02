# Azure Authentication Menu Bar Application - Specification

## Project Overview

A macOS menu bar application built with Rust that provides seamless Microsoft Azure authentication management. The application lives exclusively in the system menu bar (no dock icon) and allows users to sign in/sign out of their Azure account with OAuth2 authentication flow, displaying their authentication status, user information, and tenant details.

### Project Name
`azurepim` (Azure Authentication Manager)

### Target Platform
- macOS 11.0+ (Big Sur and later)
- Intel and Apple Silicon (universal binary)

### Primary Use Case
Azure authentication state management for enterprise developers (specifically Vipps MobilePay) who need quick access to Azure sign-in status and, in Phase 2, Azure PIM role activation without running a full browser session or Azure portal.

### Target Organization
- **Company**: Vipps MobilePay
- **Tenant**: Single Azure AD tenant (centrally managed)
- **App Registration**: Managed by IT team

---

## Technology Stack

### Language & Runtime
- **Rust 2021 Edition** - Systems programming for performance and safety
- **Tokio** (async runtime) - Full async/await support for HTTP operations and OAuth flows
- **Main Thread Pattern** - Critical for macOS AppKit integration

### macOS Native Integration (via objc2)
- **objc2** - Zero-cost Rust bindings to Objective-C/AppKit (NOT Swift)
- **objc2-foundation** - Core Foundation framework bindings
- **objc2-app-kit** - AppKit framework bindings for UI
- **dispatch** - Grand Central Dispatch for main thread operations
- **security-framework** - macOS Keychain for secure token storage

### OAuth2 & Azure Integration
- **oauth2** crate - RFC 6749 OAuth 2.0 implementation
- **reqwest** - HTTP client for Azure AD and Microsoft Graph API calls
- **azure_identity** (optional) - Official Azure SDK for authentication helpers
- **serde_json** - JSON serialization for API responses

### Core Dependencies
```toml
[dependencies]
# macOS Integration
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSData", "NSString", "NSThread", "NSObject", "NSOperation", "NSAttributedString", "NSRange", "NSDictionary", "NSURL"] }
objc2-app-kit = { version = "0.2", features = [
    "NSApplication",
    "NSMenu", "NSMenuItem",
    "NSStatusBar", "NSStatusBarButton", "NSStatusItem",
    "NSWindow", "NSView", "NSTextField", "NSButton",
    "NSImage"
] }
block2 = "0.5"
dispatch = "0.2"
security-framework = "2.9"

# Async & HTTP
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# OAuth2
oauth2 = "4.4"
url = "2.5"

# Azure SDK (optional, for helpers)
azure_identity = { version = "0.20", optional = true }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error Handling & Logging
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
once_cell = "1.19"
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.21"

# Security
zeroize = "1.7"

# Auto-updates (Sparkle framework via Rust bindings)
# Note: Sparkle is a macOS framework, integrate via build script or swift bridge
```

---

## Functional Requirements

### FR-1: Azure Authentication
- **FR-1.1**: User can initiate sign-in via menu bar menu item
- **FR-1.2**: Application launches system default browser for Azure AD OAuth2 authentication
- **FR-1.3**: Application registers custom URL scheme (`azurepim://callback`) for OAuth callback - macOS delivers the callback URL directly to the app via `NSApplicationDelegate.application:openURLs:`
- **FR-1.4**: Application exchanges authorization code for access token and refresh token
- **FR-1.5**: Tokens are stored securely in macOS Keychain
- **FR-1.6**: Application fetches user profile information from Microsoft Graph API
- **FR-1.7**: User can sign out, which clears tokens from Keychain and resets UI state

### FR-2: Menu Bar UI
- **FR-2.1**: Application appears as icon in macOS menu bar (right side, near system icons)
- **FR-2.2**: Icon displays different states:
  - Signed out: Gray/inactive icon
  - Signed in: Blue/active icon (Azure brand color)
  - Authenticating: Animated or alternate icon
  - Error: Red/warning icon
- **FR-2.3**: Clicking icon reveals dropdown menu with:
  - When signed out:
    - "Sign In to Azure" menu item
    - Separator
    - "Quit" menu item
  - When signed in:
    - User name (bold, disabled item for display only)
    - User email (disabled item for display only)
    - Tenant name (disabled item for display only)
    - Separator
    - "Refresh Token" menu item (force token refresh)
    - "Sign Out" menu item
    - Separator
    - "Quit" menu item

### FR-3: Token Management
- **FR-3.1**: Access tokens are automatically refreshed before expiration (default: 1 hour)
- **FR-3.2**: Refresh tokens are used to obtain new access tokens without user interaction
- **FR-3.3**: If refresh fails, user is prompted to sign in again
- **FR-3.4**: Token expiration countdown is optionally displayed in menu (future enhancement)

### FR-4: Data Display
- **FR-4.1**: Display authenticated user's display name (from Microsoft Graph `/me` endpoint)
- **FR-4.2**: Display user's email/UPN (User Principal Name)
- **FR-4.3**: Display Azure AD tenant name and tenant ID
- **FR-4.4**: All displayed information updates immediately upon successful authentication

### FR-5: Security
- **FR-5.1**: All tokens stored in macOS Keychain (not plain text files)
- **FR-5.2**: OAuth2 flow uses PKCE (Proof Key for Code Exchange) for additional security
- **FR-5.3**: Sensitive data (tokens, credentials) never logged to console or files
- **FR-5.4**: Tokens are zeroized from memory when no longer needed
- **FR-5.5**: Local OAuth callback server only accepts requests from localhost

### FR-6: Error Handling
- **FR-6.1**: Network errors display user-friendly error messages in menu
- **FR-6.2**: Authentication errors (invalid credentials, MFA required) show appropriate messages
- **FR-6.3**: Token refresh failures trigger re-authentication flow
- **FR-6.4**: Application doesn't crash on errors - degrades gracefully

---

## Azure AD OAuth2 Configuration

### Application Registration
The application requires an Azure AD App Registration with the following configuration:

#### App Registration Settings
- **Name**: Azure Auth Menu Bar (or user's choice)
- **Supported account types**:
  - "Accounts in this organizational directory only" (single tenant) - **Recommended for Vipps**
  - OR "Accounts in any organizational directory" (multi-tenant)
- **Redirect URI**:
  - Type: Public client/native (mobile & desktop)
  - URI: `azurepim://callback` (custom URL scheme - recommended)
  - Alternative: `http://localhost:8080/callback` (fallback if URL scheme not supported)

#### API Permissions
Required Microsoft Graph permissions (delegated):
- `User.Read` - Read user profile (display name, email, UPN)
- `openid` - OpenID Connect authentication
- `profile` - Basic profile information
- `email` - Email address
- `offline_access` - Refresh tokens

#### Authentication Settings
- **Allow public client flows**: Yes (for PKCE)
- **Enable mobile and desktop flows**: Yes

### OAuth2 Endpoints (Microsoft Identity Platform v2.0)
```
Authorization Endpoint: https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize
Token Endpoint: https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token
Microsoft Graph API: https://graph.microsoft.com/v1.0/
```

Where `{tenant}` can be:
- `common` - Multi-tenant and personal Microsoft accounts
- `organizations` - Multi-tenant (work/school accounts only)
- `consumers` - Personal Microsoft accounts only
- `{tenant-id}` - Specific tenant ID (single tenant)

### OAuth2 Scopes
```
https://graph.microsoft.com/User.Read
openid
profile
email
offline_access
```

---

## Authentication Flow Details

### Browser-Based OAuth2 Flow with PKCE

```
┌─────────────┐                                      ┌──────────────┐
│  Menu Bar   │                                      │  Azure AD    │
│     App     │                                      │   (Browser)  │
└──────┬──────┘                                      └──────┬───────┘
       │                                                    │
       │ 1. User clicks "Sign In to Azure"                 │
       │                                                    │
       │ 2. Generate PKCE code_verifier & code_challenge   │
       │                                                    │
       │ 3. Start local HTTP server (localhost:8080)       │
       │                                                    │
       │ 4. Build authorization URL                        │
       │    - client_id                                     │
       │    - redirect_uri                                  │
       │    - response_type=code                            │
       │    - scope                                         │
       │    - code_challenge                                │
       │    - code_challenge_method=S256                    │
       │    - state (CSRF protection)                       │
       │                                                    │
       │ 5. Open browser with auth URL ────────────────────>│
       │                                                    │
       │                                    6. User signs in│
       │                                    (username/pwd,  │
       │                                     MFA, etc.)     │
       │                                                    │
       │ 7. Browser redirects to                            │
       │<──────── localhost:8080/callback?code=xxx&state=xxx│
       │                                                    │
       │ 8. Local server receives callback                  │
       │    - Validate state (CSRF check)                   │
       │    - Extract authorization code                    │
       │                                                    │
       │ 9. Exchange code for tokens ──────────────────────>│
       │    POST /oauth2/v2.0/token                         │
       │    - code                                          │
       │    - code_verifier (PKCE)                          │
       │    - client_id                                     │
       │    - redirect_uri                                  │
       │    - grant_type=authorization_code                 │
       │                                                    │
       │<───────────────────────────── 10. Token response   │
       │    {                                               │
       │      "access_token": "...",                        │
       │      "refresh_token": "...",                       │
       │      "expires_in": 3600,                           │
       │      "token_type": "Bearer"                        │
       │    }                                               │
       │                                                    │
       │ 11. Store tokens in macOS Keychain                 │
       │                                                    │
       │ 12. Fetch user info from Microsoft Graph           │
       │     GET https://graph.microsoft.com/v1.0/me        │
       │     Authorization: Bearer {access_token}           │
       │                                                    │
       │ 13. Update menu bar UI with user info              │
       │                                                    │
       │ 14. Shut down local HTTP server                    │
       │                                                    │
       │ 15. Schedule token refresh before expiry           │
       │                                                    │
```

### Token Refresh Flow

```
┌─────────────┐                                      ┌──────────────┐
│  Menu Bar   │                                      │  Azure AD    │
│     App     │                                      │              │
└──────┬──────┘                                      └──────┬───────┘
       │                                                    │
       │ 1. Token expiration approaching (5 min before)    │
       │    OR user clicks "Refresh Token"                 │
       │                                                    │
       │ 2. Retrieve refresh_token from Keychain           │
       │                                                    │
       │ 3. POST /oauth2/v2.0/token ───────────────────────>│
       │    - refresh_token                                 │
       │    - client_id                                     │
       │    - grant_type=refresh_token                      │
       │    - scope                                         │
       │                                                    │
       │<───────────────────────────── 4. New tokens        │
       │    {                                               │
       │      "access_token": "...",                        │
       │      "refresh_token": "...",  (new refresh token)  │
       │      "expires_in": 3600                            │
       │    }                                               │
       │                                                    │
       │ 5. Update tokens in Keychain                       │
       │                                                    │
       │ 6. Schedule next refresh                           │
       │                                                    │
```

---

## User Interface & User Experience

### Menu Bar Icon States

#### Icon Asset Requirements
- **Format**: Template image (monochrome PDF or PNG with alpha)
- **Size**: 22x22 points (@1x), 44x44 points (@2x)
- **Style**: Line icon, works in both light and dark mode

#### Icon States
1. **Signed Out** (default)
   - Gray monochrome icon (cloud with slash or lock icon)
   - Tooltip: "Azure Auth - Not signed in"

2. **Signed In**
   - Tinted icon (system uses accent color, typically blue)
   - Tooltip: "Azure Auth - Signed in as {user_email}"

3. **Authenticating**
   - Slightly animated or different icon
   - Tooltip: "Azure Auth - Signing in..."

4. **Error**
   - Different icon or color (system may tint red on errors if configured)
   - Tooltip: "Azure Auth - Error: {brief_message}"

### Menu Layout

#### Signed Out State
```
┌────────────────────────────────┐
│  Sign In to Azure              │ <- Action item
├────────────────────────────────┤
│  Quit                  ⌘Q      │
└────────────────────────────────┘
```

#### Signed In State
```
┌────────────────────────────────┐
│  Sven Malvik                   │ <- Display only (bold)
│  sven.malvik@vipps.no          │ <- Display only
│  Vipps MobilePay               │ <- Display only (tenant)
│  Expires in 45 min             │ <- Token countdown (if enabled)
├────────────────────────────────┤
│  Copy Access Token             │ <- Copies token (auto-clears 2 min)
│  Refresh Token                 │ <- Force refresh
│  Sign Out                      │
├────────────────────────────────┤
│  Settings                    ▶ │ <- Submenu (see below)
├────────────────────────────────┤
│  Quit                  ⌘Q      │
└────────────────────────────────┘

Settings Submenu:
┌────────────────────────────────┐
│  ✓ Auto-launch at login        │ <- Toggle
│  ✓ Show expiry countdown       │ <- Toggle
│  ─────────────────────────     │
│  Clear all data...             │ <- Reset action
└────────────────────────────────┘
```

#### Authenticating State
```
┌────────────────────────────────┐
│  Signing in...                 │ <- Display only
├────────────────────────────────┤
│  Cancel                        │ <- Cancels auth flow
│  Quit                  ⌘Q      │
└────────────────────────────────┘
```

#### Error State
```
┌────────────────────────────────┐
│  ⚠️ Authentication Failed       │ <- Display only (red text)
│  Network error occurred        │ <- Error message
├────────────────────────────────┤
│  Try Again                     │ <- Retry auth
│  Sign Out                      │
├────────────────────────────────┤
│  Quit                  ⌘Q      │
└────────────────────────────────┘
```

### Menu Item Behaviors
- **Disabled items** (display-only): User name, email, tenant name, status messages
- **Enabled items**: Action items (Sign In, Sign Out, Refresh, Quit)
- **Keyboard shortcuts**:
  - `⌘Q` - Quit application
  - `⌘R` - Refresh token (when signed in)
  - `⌘I` - Sign In (when signed out)

---

## Data Storage & Security

### macOS Keychain Integration

#### Keychain Service Configuration
```rust
const SERVICE_NAME: &str = "de.malvik.azurepim.desktop";
const ACCOUNT_ACCESS_TOKEN: &str = "azure_access_token";
const ACCOUNT_REFRESH_TOKEN: &str = "azure_refresh_token";
const ACCOUNT_USER_INFO: &str = "azure_user_info";
```

#### Stored Data Items

1. **Access Token**
   - Service: `de.malvik.azurepim.desktop`
   - Account: `azure_access_token`
   - Value: JWT access token (as bytes)
   - Security: Keychain encrypted, requires user authentication to access

2. **Refresh Token**
   - Service: `de.malvik.azurepim.desktop`
   - Account: `azure_refresh_token`
   - Value: Opaque refresh token string (as bytes)
   - Security: Keychain encrypted, zeroized in memory after use

3. **User Info Cache** (optional)
   - Service: `de.malvik.azurepim.desktop`
   - Account: `azure_user_info`
   - Value: JSON-serialized user profile
   ```json
   {
     "display_name": "Sven Malvik",
     "email": "sven.malvik@vipps.no",
     "user_principal_name": "sven.malvik@vipps.no",
     "tenant_id": "12345678-1234-1234-1234-123456789012",
     "tenant_name": "Vipps MobilePay",
     "token_expires_at": "2026-01-02T15:30:00Z"
   }
   ```

#### Token Lifecycle
```
┌─────────────────────────────────────────────────────┐
│  Token Acquisition                                  │
│  ├─ Exchange auth code for tokens                   │
│  ├─ Store access_token in Keychain                  │
│  ├─ Store refresh_token in Keychain                 │
│  └─ Zeroize tokens from memory                      │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│  Token Usage                                        │
│  ├─ Retrieve access_token from Keychain             │
│  ├─ Use for API calls                               │
│  └─ Zeroize from memory after use                   │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│  Token Refresh (before expiry)                      │
│  ├─ Retrieve refresh_token from Keychain            │
│  ├─ Exchange for new tokens                         │
│  ├─ Update Keychain with new tokens                 │
│  └─ Zeroize old tokens from memory                  │
└─────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────┐
│  Sign Out                                           │
│  ├─ Delete all tokens from Keychain                 │
│  ├─ Zeroize all sensitive data from memory          │
│  └─ Reset application state                         │
└─────────────────────────────────────────────────────┘
```

### Security Best Practices
1. **No plain text storage**: Never store tokens in files, preferences, or user defaults
2. **Memory safety**: Use `zeroize` crate to clear sensitive data from memory
3. **Minimal logging**: Never log tokens, even in debug mode
4. **HTTPS only**: All network requests to Azure AD and Graph API use TLS
5. **PKCE**: Use OAuth2 PKCE extension to prevent authorization code interception
6. **State parameter**: Validate OAuth state parameter to prevent CSRF attacks
7. **Localhost binding**: OAuth callback server binds only to `127.0.0.1`, not `0.0.0.0`

---

## Application Architecture

### High-Level Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  ├─ Initialize logging (tracing)                            │
│  ├─ Load configuration                                      │
│  ├─ Initialize AppState                                     │
│  ├─ Setup callbacks (sign in, sign out, refresh, quit)     │
│  ├─ Initialize MenuBar                                      │
│  └─ Run event loop (app.run())                              │
└─────────────────────────────────────────────────────────────┘
               │
               ├──────────────────────────────────────┐
               │                                      │
               ▼                                      ▼
┌─────────────────────────────┐      ┌─────────────────────────────┐
│    menubar/                 │      │    auth/                    │
│  ├─ mod.rs                  │      │  ├─ mod.rs                  │
│  ├─ delegate.rs             │      │  ├─ oauth.rs                │
│  ├─ builder.rs              │      │  │   ├─ OAuth2 client       │
│  ├─ state.rs                │      │  │   ├─ PKCE generation     │
│  │   ├─ AppState            │      │  │   ├─ Authorization URL   │
│  │   ├─ MenuCallbacks       │      │  │   └─ Token exchange      │
│  │   └─ UserInfo            │      │  ├─ callback_server.rs      │
│  └─ updates.rs              │      │  │   └─ Local HTTP listener │
│      └─ Dynamic UI updates  │      │  ├─ token_manager.rs        │
└─────────────────────────────┘      │  │   ├─ Token refresh       │
               │                     │  │   └─ Expiry tracking     │
               │                     │  └─ graph.rs                │
               │                     │      └─ MS Graph API calls  │
               │                     └─────────────────────────────┘
               │                                      │
               ▼                                      ▼
┌─────────────────────────────┐      ┌─────────────────────────────┐
│    keychain/                │      │    error.rs                 │
│  ├─ mod.rs                  │      │  └─ Custom error types      │
│  ├─ store_token()           │      │      ├─ AuthError           │
│  ├─ retrieve_token()        │      │      ├─ KeychainError       │
│  └─ delete_token()          │      │      └─ ApiError            │
└─────────────────────────────┘      └─────────────────────────────┘
```

### Core Components

#### 1. AppState
```rust
pub struct AppState {
    pub auth_state: Arc<Mutex<AuthState>>,
    pub user_info: Arc<Mutex<Option<UserInfo>>>,
}

pub enum AuthState {
    SignedOut,
    Authenticating,
    SignedIn { expires_at: DateTime<Utc> },
    Error { message: String },
}

pub struct UserInfo {
    pub display_name: String,
    pub email: String,
    pub user_principal_name: String,
    pub tenant_id: String,
    pub tenant_name: Option<String>,
}
```

#### 2. MenuCallbacks
```rust
pub struct MenuCallbacks {
    pub on_sign_in: Arc<dyn Fn() + Send + Sync>,
    pub on_sign_out: Arc<dyn Fn() + Send + Sync>,
    pub on_refresh_token: Arc<dyn Fn() + Send + Sync>,
    pub on_copy_token: Arc<dyn Fn() + Send + Sync>,
    pub on_quit: Arc<dyn Fn() + Send + Sync>,
}
```

#### 3. OAuth2Client
```rust
pub struct OAuth2Client {
    client_id: String,
    tenant: String,
    redirect_uri: String,
    scopes: Vec<String>,
}

impl OAuth2Client {
    pub fn new(client_id: String, tenant: String) -> Self;
    pub fn generate_pkce() -> (String, String); // (verifier, challenge)
    pub fn build_auth_url(&self, code_challenge: &str, state: &str) -> Result<Url>;
    pub async fn exchange_code(&self, code: &str, verifier: &str) -> Result<TokenResponse>;
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse>;
}
```

#### 4. CallbackServer
```rust
pub struct CallbackServer {
    port: u16,
    expected_state: String,
}

impl CallbackServer {
    pub fn new(port: u16, state: String) -> Self;
    pub async fn listen_for_callback(&self) -> Result<String>; // Returns auth code
    pub fn shutdown(self);
}
```

#### 5. TokenManager
```rust
pub struct TokenManager {
    oauth_client: Arc<OAuth2Client>,
    refresh_handle: Mutex<Option<JoinHandle<()>>>,
}

impl TokenManager {
    pub fn new(oauth_client: Arc<OAuth2Client>) -> Self;
    pub async fn get_access_token(&self) -> Result<String>;
    pub async fn refresh_access_token(&self) -> Result<()>;
    pub fn start_auto_refresh(&self, expires_in: u64);
    pub fn stop_auto_refresh(&self);
}
```

#### 6. GraphClient
```rust
pub struct GraphClient {
    base_url: String,
}

impl GraphClient {
    pub fn new() -> Self;
    pub async fn get_user_profile(&self, access_token: &str) -> Result<UserProfile>;
    pub async fn get_organization(&self, access_token: &str) -> Result<Organization>;
}

pub struct UserProfile {
    pub display_name: String,
    pub mail: Option<String>,
    pub user_principal_name: String,
}

pub struct Organization {
    pub id: String,
    pub display_name: String,
}
```

### Threading Model

```
┌──────────────────────────────────────────────────────────────┐
│  Main Thread (AppKit Event Loop)                             │
│  ├─ MenuBar UI rendering                                     │
│  ├─ NSApplication.run()                                      │
│  ├─ Menu item click handlers                                 │
│  └─ All AppKit API calls                                     │
└──────────────────────────────────────────────────────────────┘
                         │
                         │ Callbacks dispatch to...
                         ▼
┌──────────────────────────────────────────────────────────────┐
│  Tokio Runtime (Background Threads)                          │
│  ├─ OAuth2 HTTP requests                                     │
│  ├─ Callback server (localhost:8080)                         │
│  ├─ Microsoft Graph API calls                                │
│  ├─ Token refresh timer                                      │
│  └─ Keychain operations (sync, but called from async)        │
└──────────────────────────────────────────────────────────────┘
                         │
                         │ UI updates dispatch back to...
                         ▼
┌──────────────────────────────────────────────────────────────┐
│  Main Thread (via dispatch::Queue::main())                   │
│  └─ MenuBar UI updates (icons, menu items, state)            │
└──────────────────────────────────────────────────────────────┘
```

**Critical Rule**: All AppKit operations MUST use `MainThreadMarker` and run on the main thread.

---

## Error Handling Strategy

### Error Types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Keychain error: {0}")]
    Keychain(#[from] KeychainError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("OAuth2 authorization failed: {0}")]
    OAuthFailed(String),

    #[error("Invalid authorization code")]
    InvalidAuthCode,

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("PKCE generation failed")]
    PkceGenerationFailed,

    #[error("State validation failed (possible CSRF attack)")]
    StateValidationFailed,

    #[error("Callback server error: {0}")]
    CallbackServerError(String),
}

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Failed to store token: {0}")]
    StoreFailed(String),

    #[error("Failed to retrieve token: {0}")]
    RetrieveFailed(String),

    #[error("Failed to delete token: {0}")]
    DeleteFailed(String),

    #[error("Token not found in keychain")]
    TokenNotFound,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Graph API request failed: {0}")]
    GraphRequestFailed(String),

    #[error("Failed to parse API response: {0}")]
    ParseFailed(String),

    #[error("Unauthorized (401): Token may be expired")]
    Unauthorized,

    #[error("Forbidden (403): Insufficient permissions")]
    Forbidden,
}
```

### User-Facing Error Messages

Map technical errors to user-friendly messages:

| Internal Error | User Message | Action |
|---------------|--------------|--------|
| `AuthError::OAuthFailed` | "Sign-in failed. Please try again." | Show in menu, offer "Try Again" |
| `AuthError::TokenRefreshFailed` | "Session expired. Please sign in again." | Auto sign-out, show sign-in option |
| `KeychainError::StoreFailed` | "Failed to save credentials securely." | Log details, show error |
| `ApiError::GraphRequestFailed` | "Failed to load user information." | Keep signed in state, retry |
| `Network error` | "Network error. Check your connection." | Show in menu, offer retry |
| `ApiError::Unauthorized` | "Authentication expired. Sign in again." | Auto sign-out |

---

## Configuration Management

### Application Configuration
```toml
# config.toml (embedded at compile time)
[app]
name = "Azure Auth"
version = "0.1.0"
bundle_identifier = "de.malvik.azurepim.desktop"

[oauth]
client_id = "YOUR_AZURE_AD_CLIENT_ID"  # From Azure App Registration (managed by IT)
tenant = "YOUR_VIPPS_TENANT_ID"        # Single tenant for Vipps MobilePay
redirect_uri = "azurepim://callback"   # Custom URL scheme (recommended)
# Alternative: redirect_uri = "http://localhost:8080/callback"
# callback_port = 8080  # Only needed if using localhost redirect

[oauth.scopes]
scopes = [
    "https://graph.microsoft.com/User.Read",
    "openid",
    "profile",
    "email",
    "offline_access"
]

[api]
graph_base_url = "https://graph.microsoft.com/v1.0"

[token]
refresh_before_expiry_seconds = 300  # Refresh 5 minutes before expiry

[logging]
level = "info"  # trace, debug, info, warn, error
```

### Runtime Configuration Override
Allow environment variables to override config:
- `AZURE_CLIENT_ID` - Override OAuth client ID
- `AZURE_TENANT_ID` - Override tenant
- `AZURE_REDIRECT_PORT` - Override callback port
- `RUST_LOG` - Override log level

---

## Project Structure

```
azurepim/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── config.toml                 # Application configuration
│
├── docs/
│   ├── spec.md                # This specification
│   └── rust-macos-menubar-architecture.md  # Reference architecture
│
├── assets/
│   ├── icon-signed-out.pdf    # Menu bar icon (signed out)
│   ├── icon-signed-in.pdf     # Menu bar icon (signed in)
│   └── icon-error.pdf         # Menu bar icon (error state)
│
└── src/
    ├── main.rs                # Entry point, initialization
    │
    ├── menubar/              # Menu bar UI implementation
    │   ├── mod.rs           # MenuBar struct, init, run
    │   ├── delegate.rs      # Objective-C delegate class
    │   ├── builder.rs       # Menu construction
    │   ├── state.rs         # AppState, MenuCallbacks, UserInfo
    │   └── updates.rs       # Dynamic menu updates (icons, items)
    │
    ├── auth/                # Azure authentication
    │   ├── mod.rs          # Public API, re-exports
    │   ├── oauth.rs        # OAuth2Client implementation
    │   ├── callback_server.rs  # Local HTTP server for OAuth callback
    │   ├── token_manager.rs    # Token refresh and lifecycle
    │   └── graph.rs        # Microsoft Graph API client
    │
    ├── keychain/           # macOS Keychain integration
    │   ├── mod.rs          # Keychain operations
    │   └── secure.rs       # Zeroize wrappers for sensitive data
    │
    ├── config.rs           # Configuration loading
    ├── error.rs            # Error types (AppError, AuthError, etc.)
    └── utils.rs            # Utility functions
```

---

## Microsoft Graph API Integration

### User Profile Endpoint
```http
GET https://graph.microsoft.com/v1.0/me
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "@odata.context": "https://graph.microsoft.com/v1.0/$metadata#users/$entity",
  "id": "12345678-1234-1234-1234-123456789012",
  "displayName": "Sven Malvik",
  "givenName": "Sven",
  "surname": "Malvik",
  "mail": "sven.malvik@vipps.no",
  "userPrincipalName": "sven.malvik@vipps.no",
  "jobTitle": "Software Engineer",
  "officeLocation": "Oslo"
}
```

### Organization Endpoint
```http
GET https://graph.microsoft.com/v1.0/organization
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "@odata.context": "https://graph.microsoft.com/v1.0/$metadata#organization",
  "value": [
    {
      "id": "12345678-1234-1234-1234-123456789012",
      "displayName": "Contoso Corporation",
      "verifiedDomains": [
        {
          "capabilities": "Email, OfficeCommunicationsOnline",
          "isDefault": true,
          "isInitial": false,
          "name": "contoso.com",
          "type": "Managed"
        }
      ]
    }
  ]
}
```

### Error Responses
```json
{
  "error": {
    "code": "InvalidAuthenticationToken",
    "message": "Access token has expired or is not yet valid.",
    "innerError": {
      "request-id": "...",
      "date": "2026-01-02T10:30:00"
    }
  }
}
```

---

## Development Workflow

### Initial Setup
1. **Azure AD App Registration**:
   - Create app registration in Azure Portal
   - Configure redirect URI: `http://localhost:8080/callback`
   - Add API permissions: `User.Read`, `openid`, `profile`, `email`, `offline_access`
   - Copy Application (client) ID

2. **Update Configuration**:
   - Edit `config.toml` with your client ID
   - Choose tenant (`common` or specific tenant ID)

3. **Build and Run**:
   ```bash
   cargo build
   cargo run
   ```

### Development Build
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with specific Azure tenant
AZURE_TENANT_ID=12345678-... cargo run

# Run with custom callback port
AZURE_REDIRECT_PORT=9090 cargo run
```

### Release Build
```bash
cargo build --release

# Binary location:
# target/release/azurepim
```

### Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test auth::tests

# Integration tests
cargo test --test integration
```

---

## Success Criteria

### Minimum Viable Product (MVP)
- ✅ Application appears in macOS menu bar
- ✅ User can click "Sign In to Azure" and complete OAuth2 flow in browser
- ✅ Access token and refresh token stored securely in Keychain
- ✅ User profile information fetched from Microsoft Graph API
- ✅ Menu displays user name, email, and tenant name when signed in
- ✅ User can sign out (clears Keychain)
- ✅ Application doesn't crash on network errors or auth failures
- ✅ Tokens automatically refresh before expiration
- ✅ Application quits cleanly

### Quality Benchmarks
- **Security**: All tokens in Keychain, PKCE enabled, no plaintext secrets
- **Performance**: Sign-in completes within 5 seconds (network permitting)
- **Reliability**: Handles network failures gracefully, auto-recovers from token expiry
- **UX**: Clear status indication, helpful error messages
- **Code Quality**: No compiler warnings, passes `cargo clippy`, formatted with `cargo fmt`

---

## Future Enhancements (Out of Scope for MVP)

### Phase 2: Enhanced Features
- **Multiple Account Support**: Switch between multiple Azure accounts
- **Token Expiration Countdown**: Show time remaining in menu ("Expires in 45m")
- **Conditional Access Status**: Indicate if MFA or compliance checks are required
- **Custom Scopes**: Allow user to configure additional API permissions
- **Tenant Switcher**: Switch between multiple tenants for multi-tenant users

### Phase 3: Advanced Features
- **PIM Integration**: Activate/deactivate Azure PIM roles
- **Role Management**: View eligible and active Azure AD roles
- **Notifications**: macOS notifications for token expiry, auth failures
- **Command-Line Interface**: CLI for scripting and automation (`azurepim signin`, `azurepim token`)
- **Browser Extension**: Integrate with browser for seamless SSO

### Phase 4: Power User Features
- **Global Hotkeys**: Keyboard shortcuts to trigger sign-in/out
- **Clipboard Integration**: Copy access token to clipboard with expiry warning
- **Audit Log**: Local log of authentication events
- **Token Inspector**: Decode and display JWT token claims
- **Azure Resource Browser**: List and manage Azure resources from menu bar

---

## Security Considerations

### Threat Model

#### Assets to Protect
1. **Access Tokens**: Short-lived (1 hour), grants access to Microsoft Graph API
2. **Refresh Tokens**: Long-lived (90 days), can obtain new access tokens
3. **User Information**: Display name, email, tenant ID (PII)

#### Threats

| Threat | Mitigation |
|--------|-----------|
| **Token Theft from Disk** | Store tokens in macOS Keychain (encrypted, requires auth) |
| **Token Theft from Memory** | Use `zeroize` crate to clear sensitive data from memory |
| **Authorization Code Interception** | Use PKCE (Proof Key for Code Exchange) |
| **CSRF on OAuth Callback** | Validate `state` parameter matches expected value |
| **Man-in-the-Middle (MITM)** | Use HTTPS for all Azure AD/Graph API requests |
| **Malicious Callback Server** | Bind callback server to `127.0.0.1` only (not `0.0.0.0`) |
| **Token Logging** | Never log tokens, even in debug mode |
| **Keychain Access by Other Apps** | Service name `de.malvik.azurepim.desktop` prevents conflicts |

#### Out of Scope
- Protection against malicious browser extensions (user responsibility)
- Protection against compromised macOS Keychain (requires system-level compromise)
- Protection against shoulder surfing (physical security)

### Compliance Notes
- **GDPR**: No user data sent to third parties (only Microsoft)
- **Data Retention**: Tokens cleared on sign-out, no persistent logs
- **Audit**: Application logs authentication events (no PII)

---

## Interview Clarifications & Decisions

*The following decisions were made through a structured interview process on 2026-01-02.*

### Target User & Context
- **Primary User**: Enterprise developer at Vipps MobilePay accessing corporate Azure
- **Organization**: Single tenant (Vipps MobilePay), centrally managed Azure AD app registration by IT
- **Use Case**: Quick access to Azure authentication state, with PIM role activation as Phase 2

### Resolved Configuration Questions

| Question | Decision | Rationale |
|----------|----------|-----------|
| **Client ID Distribution** | Centrally managed by IT | Vipps IT team registers and manages the Azure AD app |
| **Application Name** | `azurepim` binary, keep PIM in name | PIM is Phase 2 priority, not just an afterthought |
| **Tenant Scope** | Single tenant hardcoded in config | Vipps-only initially, no tenant switching needed |
| **Token Expiry Display** | Show countdown in menu | Helpful for developers to know session status |

### Authentication & Session Decisions

| Feature | Decision | Notes |
|---------|----------|-------|
| **OAuth Redirect** | Custom URL scheme `azurepim://callback` | Avoids port conflicts, cleaner than localhost |
| **Auth Browser** | Default system browser | Enables SSO with existing Azure sessions |
| **MFA Handling** | Browser handles it | MFA is mandatory at Vipps, Azure AD manages flow |
| **Auto Sign-in** | Yes, with status indicator | Restore session on launch, show "Signing in..." |
| **Session Expiry Notification** | Menu bar icon change only | Subtle, non-intrusive notification |
| **Quit Behavior** | Stay signed in (preserve tokens) | Relaunch continues session seamlessly |

### UI/UX Decisions

| Feature | Decision | Notes |
|---------|----------|-------|
| **Menu Bar Icons** | Azure logo/brand colors | Blue when signed in, gray when not |
| **Tooltip on Hover** | No tooltip | All info visible in menu dropdown |
| **Keyboard Shortcuts** | Only ⌘Q for quit | Menu access is sufficient |
| **Settings Location** | Submenu in main menu | Accessible but not cluttered |
| **Offline Indicator** | Show in menu bar icon | Visual indicator when network unavailable |

### Settings Menu Options (MVP)
1. **Auto-launch at login** - Toggle to enable/disable
2. **Show expiry in menu bar** - Toggle countdown display
3. **Clear all data / Reset** - Wipe keychain and start fresh

### Security Decisions

| Feature | Decision | Notes |
|---------|----------|-------|
| **Copy Token Clipboard** | Auto-clear after 2 minutes | Security best practice |
| **Conditional Access Errors** | Show specific CA failure messages | Parse Azure AD error codes for helpful messages |

### Distribution & Updates

| Feature | Decision | Notes |
|---------|----------|-------|
| **Distribution** | Apple signed + notarized | No Gatekeeper warnings, professional distribution |
| **Auto-updates** | Sparkle framework | Industry standard macOS auto-updater |
| **Logs Location** | `~/Library/Logs/azurepim/` | Standard macOS location, Console.app accessible |

### Phase 2: PIM Integration Decisions

| Feature | Decision | Notes |
|---------|----------|-------|
| **PIM Scope** | Azure resource roles (not Azure AD roles) | Primary use case for Vipps developers |
| **PIM Actions** | Activate, view active, extend/deactivate | Core role management functionality |
| **Justification Input** | Quick preset reasons + custom option | "Debugging", "Deployment", "Investigation", etc. |
| **Default Duration** | Short sessions (1-2 hours) | User can adjust in settings |
| **Touch ID for PIM** | No - Azure MFA is sufficient | Avoid unnecessary friction |
| **Role Expiry Notification** | macOS notification 5 min before | Push notification for awareness |

### Project Timeline
- **Timeline**: No deadline, quality focus
- **Approach**: Build MVP right, release when ready

---

## Appendix

### A. OAuth2 PKCE Implementation

**PKCE (Proof Key for Code Exchange)** prevents authorization code interception attacks.

#### PKCE Flow
1. Generate random `code_verifier` (43-128 characters, URL-safe)
2. Create `code_challenge` = BASE64URL(SHA256(code_verifier))
3. Send `code_challenge` and `code_challenge_method=S256` in authorization URL
4. Azure AD stores the `code_challenge`
5. When exchanging code for token, send `code_verifier`
6. Azure AD verifies: SHA256(code_verifier) == stored code_challenge

#### Rust Implementation
```rust
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;

pub fn generate_pkce() -> (String, String) {
    // Generate code_verifier (64 random bytes = 86 base64 chars)
    let mut rng = rand::thread_rng();
    let verifier_bytes: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

    // Generate code_challenge = BASE64URL(SHA256(verifier))
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let challenge_hash = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(challenge_hash);

    (code_verifier, code_challenge)
}
```

### B. Token Storage Schema

Keychain items use generic password storage:

| Field | Value |
|-------|-------|
| **Service** | `de.malvik.azurepim.desktop` |
| **Account** | `azure_access_token` / `azure_refresh_token` / `azure_user_info` |
| **Password (data)** | Token or JSON as bytes |
| **Access Control** | Current user only |
| **Encryption** | macOS Keychain encryption (AES-256) |

### C. Example OAuth2 Authorization URL

```
https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize?
  client_id=12345678-1234-1234-1234-123456789012
  &response_type=code
  &redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback
  &response_mode=query
  &scope=https%3A%2F%2Fgraph.microsoft.com%2FUser.Read%20openid%20profile%20email%20offline_access
  &state=abc123randomstate
  &code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
  &code_challenge_method=S256
```

### D. Example Token Exchange Request

```http
POST https://login.microsoftonline.com/organizations/oauth2/v2.0/token
Content-Type: application/x-www-form-urlencoded

client_id=12345678-1234-1234-1234-123456789012
&grant_type=authorization_code
&code=OAAABAAAAiL...
&redirect_uri=http://localhost:8080/callback
&code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
```

### E. Logging Configuration

```rust
// main.rs
use tracing_subscriber::EnvFilter;

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .init();
}
```

**Log Levels**:
- `ERROR`: Authentication failures, critical errors
- `WARN`: Token refresh failures, network timeouts
- `INFO`: Sign-in/sign-out events, token refresh success
- `DEBUG`: OAuth flow steps, API requests
- `TRACE`: Detailed request/response bodies (never include tokens!)

---

## Document Version

- **Version**: 1.1
- **Date**: 2026-01-02
- **Author**: Claude Code (AI Assistant)
- **Reviewed by**: Sven Malvik
- **Status**: Ready for Implementation

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.1 | 2026-01-02 | Added interview clarifications, updated for Vipps MobilePay context, custom URL scheme, Settings submenu, Sparkle updates, bundle ID change |
| 1.0 | 2026-01-02 | Initial specification based on user requirements and reference architecture |

---

**End of Specification**
