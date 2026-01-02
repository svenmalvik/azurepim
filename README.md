# azurepim

A macOS menu bar application for Microsoft Azure authentication management, built with Rust and native AppKit integration via objc2.

## Features

- **Menu Bar Integration** - Lives exclusively in the macOS menu bar (no dock icon)
- **OAuth2 with PKCE** - Secure authentication using Proof Key for Code Exchange
- **Automatic Token Refresh** - Tokens refresh automatically before expiration
- **Secure Storage** - Tokens stored in macOS Keychain, never in plain text
- **Microsoft Graph API** - Fetches user profile and organization info
- **Native macOS Experience** - Built with objc2/AppKit, supports Intel and Apple Silicon

## Requirements

- macOS 11.0+ (Big Sur or later)
- Rust 1.70+
- Azure AD App Registration (see [Azure AD Setup](#azure-ad-setup))

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/azurepim.git
cd azurepim

# Build release binary
cargo build --release

# Run the application
./target/release/azurepim
```

### Development

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check code quality
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Configuration

Copy `config.toml` and update with your Azure AD app details:

```toml
[oauth]
client_id = "YOUR_AZURE_AD_CLIENT_ID"
tenant = "YOUR_TENANT_ID"  # or "organizations" for multi-tenant
redirect_uri = "azurepim://callback"

[oauth.scopes]
scopes = [
    "https://graph.microsoft.com/User.Read",
    "openid",
    "profile",
    "email",
    "offline_access"
]
```

### Environment Variables

Override configuration at runtime:

| Variable | Description |
|----------|-------------|
| `AZURE_CLIENT_ID` | Override OAuth client ID |
| `AZURE_TENANT_ID` | Override tenant ID |
| `RUST_LOG` | Set log level (trace, debug, info, warn, error) |

## Azure AD Setup

1. Go to **Azure Portal** > **Azure Active Directory** > **App registrations**
2. Click **New registration**
3. Configure:
   - **Name**: Azure PIM (or your preferred name)
   - **Supported account types**: Choose based on your needs
   - **Redirect URI**: Select "Public client/native" and enter `azurepim://callback`
4. Under **Authentication**:
   - Enable **Allow public client flows**: Yes
5. Under **API permissions**, add (delegated):
   - `User.Read`
   - `openid`
   - `profile`
   - `email`
   - `offline_access`
6. Copy the **Application (client) ID** to your `config.toml`

## Usage

Once running, the app appears in your menu bar:

**Signed Out:**
- Click the menu bar icon
- Select "Sign In to Azure"
- Complete authentication in your browser
- The app receives the callback and stores tokens securely

**Signed In:**
- View your display name, email, and tenant
- **Refresh Token** - Force a token refresh
- **Sign Out** - Clear all stored tokens

**Settings:**
- Auto-launch at login
- Show token expiry countdown
- Clear all data

## Architecture

```
src/
├── main.rs           # Entry point, Tokio runtime, AppKit event loop
├── app/              # NSApplicationDelegate, OAuth callback handler
├── menubar/          # Menu bar UI (state, delegate, builder, updates)
├── auth/             # OAuth2, token management, Microsoft Graph API
├── keychain/         # macOS Keychain integration
├── config.rs         # Configuration loading
├── error.rs          # Error types
└── settings.rs       # Auto-launch, log directory
```

**Threading Model:**
- Main thread handles all AppKit UI operations
- Tokio runtime handles async HTTP requests and token management
- Communication via channels with dispatch to main thread for UI updates

## Security

- **PKCE** - Prevents authorization code interception
- **State Validation** - Protects against CSRF attacks
- **Keychain Storage** - Tokens encrypted by macOS
- **Memory Safety** - Sensitive data zeroized after use
- **No Token Logging** - Tokens never written to logs

## Logs

Application logs are stored in `~/Library/Logs/azurepim/`

View logs:
```bash
# Follow logs in real-time
tail -f ~/Library/Logs/azurepim/*.log

# Or use Console.app
open -a Console ~/Library/Logs/azurepim/
```

## License

MIT

## Author

Sven Malvik
