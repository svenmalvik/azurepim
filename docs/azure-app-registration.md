# Azure AD App Registration Setup

This guide walks through creating a new Azure AD App Registration for the azurepim application.

## Prerequisites

- An Azure account with permissions to create App Registrations
- Access to [Azure Portal](https://portal.azure.com)

## Step 1: Create the App Registration

1. Sign in to the [Azure Portal](https://portal.azure.com)
2. Search for **"App registrations"** in the top search bar
3. Click **App registrations**
4. Click **+ New registration**

### Registration Details

| Field | Value |
|-------|-------|
| **Name** | `azurepim` (or your preferred name) |
| **Supported account types** | Select **"Accounts in any organizational directory (Any Microsoft Entra ID tenant - Multitenant)"** |
| **Redirect URI** | Leave blank for now (we'll configure this next) |

5. Click **Register**

## Step 2: Configure Platform and Redirect URI

1. After registration, you'll be on the app's **Overview** page
2. In the left sidebar, click **Authentication**
3. Click **+ Add a platform**
4. Select **Mobile and desktop applications**
5. Under **Custom redirect URIs**, enter:
   ```
   azurepim://callback
   ```
6. Click **Configure**

## Step 3: Enable Public Client Flows

Still in the **Authentication** section:

1. Scroll down to **Advanced settings**
2. Find **Allow public client flows**
3. Set it to **Yes**
4. Click **Save** at the top

## Step 4: Configure API Permissions

1. In the left sidebar, click **API permissions**
2. Click **+ Add a permission**
3. Select **Microsoft Graph**
4. Select **Delegated permissions**
5. Search for and select these permissions:
   - `User.Read` - Sign in and read user profile
   - `openid` - Sign users in (under OpenId permissions)
   - `profile` - View users' basic profile
   - `email` - View users' email address
   - `offline_access` - Maintain access to data you have given it access to

6. Click **Add permissions**

### Optional: Grant Admin Consent

If you have admin privileges and want to pre-consent for all users in your organization:

1. Click **Grant admin consent for [Your Organization]**
2. Click **Yes** to confirm

> **Note**: For personal/development use, admin consent is not required. Users will consent individually on first sign-in.

## Step 5: Copy Your Application (Client) ID

1. In the left sidebar, click **Overview**
2. Find **Application (client) ID**
3. Copy this GUID (e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`)

## Step 6: Configure azurepim

Update your `config.toml` with the new Client ID:

```toml
[oauth]
client_id = "YOUR_NEW_CLIENT_ID_HERE"
tenant = "organizations"
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

Or set it via environment variable:

```bash
export AZURE_CLIENT_ID="YOUR_NEW_CLIENT_ID_HERE"
cargo run
```

## Verification Checklist

Before running the application, verify:

- [ ] App Registration created with correct name
- [ ] Supported account types set to Multitenant (or your preference)
- [ ] Platform: Mobile and desktop applications
- [ ] Redirect URI: `azurepim://callback`
- [ ] Public client flows: Enabled (Yes)
- [ ] API Permissions: User.Read, openid, profile, email, offline_access
- [ ] Client ID copied to config.toml

## Troubleshooting

### AADSTS50011: Redirect URI Mismatch

The redirect URI in your request doesn't match what's configured:
- Verify `azurepim://callback` is added exactly as shown (no trailing slash)
- Wait 1-2 minutes after saving changes for Azure to propagate
- Ensure you're using the correct Client ID

### AADSTS700016: Application Not Found

The Client ID doesn't exist in the specified tenant:
- Double-check you copied the correct Application (client) ID
- Verify the app wasn't deleted
- Check you're signing in to the correct tenant

### AADSTS65001: User or Admin Has Not Consented

The user hasn't granted permissions:
- This is normal on first sign-in; accept the consent prompt
- If blocked by organization policy, contact your IT admin

## Reference Links

- [Microsoft Identity Platform Documentation](https://learn.microsoft.com/en-us/entra/identity-platform/)
- [Register an application](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app)
- [Configure redirect URIs](https://learn.microsoft.com/en-us/entra/identity-platform/reply-url)
- [Microsoft Graph permissions reference](https://learn.microsoft.com/en-us/graph/permissions-reference)
