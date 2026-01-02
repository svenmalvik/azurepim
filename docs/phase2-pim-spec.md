# Azure PIM Integration - Phase 2 Specification

## 1. Executive Summary

### Problem Statement

Developers at Vipps MobilePay frequently need to activate Azure PIM (Privileged Identity Management) roles for incident investigation, debugging, and routine maintenance. The current Azure Portal experience is frustrating due to:

- **Too many clicks** to navigate to PIM and activate a role
- **Repetitive justification entry** for the same activation reasons
- **No quick access** to frequently used roles across 10+ eligible roles in 6+ subscriptions

### Proposed Solution

Extend the azurepim menu bar application to provide one-click PIM role activation directly from the macOS menu bar. Users can:

1. **View** eligible and active subscription-level roles
2. **Activate** roles with pre-configured justification presets (2 clicks)
3. **Manage favorites** for quick access to frequently used roles
4. **Monitor** active roles with badge count and expiry indicators

### Success Metrics

- Reduce role activation from 8+ clicks (portal) to 2 clicks (menu bar)
- Sub-3-second activation flow for cached roles
- Zero missed role expirations through visual icon indicators

---

## 2. Requirements

### 2.1 Functional Requirements

#### FR-P2.1: View Eligible Roles
- **FR-P2.1.1**: Display all subscription-level Azure PIM roles the user is eligible for
- **FR-P2.1.2**: Show role name and subscription name in format: `Subscription - Role` (e.g., "vipps-prod-001 - Contributor")
- **FR-P2.1.3**: Organize roles with favorites at top, remaining roles below
- **FR-P2.1.4**: Cache eligible roles for 1 hour to minimize API calls
- **FR-P2.1.5**: Provide manual refresh action to update role list on demand

#### FR-P2.2: Activate Roles
- **FR-P2.2.1**: Activate a role with two clicks: (1) click role, (2) select justification preset
- **FR-P2.2.2**: Provide default justification presets: "Incident Investigation", "Debugging", "Maintenance"
- **FR-P2.2.3**: Allow custom justification presets configurable in settings
- **FR-P2.2.4**: Use configurable default duration (default: 1 hour)
- **FR-P2.2.5**: Close menu and show macOS notification upon successful activation
- **FR-P2.2.6**: Show macOS alert dialog on activation failure with error details

#### FR-P2.3: View Active Roles
- **FR-P2.3.1**: Display currently active roles with subscription name, role name, and time remaining
- **FR-P2.3.2**: Show badge count on menu bar icon indicating number of active roles
- **FR-P2.3.3**: Change menu bar icon color when any role is expiring soon (within warning threshold)
- **FR-P2.3.4**: Sync with Azure to detect roles activated via portal on next menu open

#### FR-P2.4: Manage Favorites
- **FR-P2.4.1**: Toggle favorite status via star icon next to each role in menu
- **FR-P2.4.2**: Show favorite roles in menu even when no roles are active
- **FR-P2.4.3**: Persist favorites locally (not shared between users)

#### FR-P2.5: Extend Active Roles (Post-MVP)
- **FR-P2.5.1**: One-click extend for active roles (reuses original justification)
- **FR-P2.5.2**: When clicking an already-active role, show current status with extend option

#### FR-P2.6: Deactivate Roles (Post-MVP)
- **FR-P2.6.1**: Deactivate active roles with confirmation dialog
- **FR-P2.6.2**: Prevent accidental deactivation through required confirmation

### 2.2 Non-Functional Requirements

#### Performance
- Eligible role list loads from cache in <100ms
- Role activation API call completes in <5 seconds (Azure dependent)
- Menu opens instantly using cached data

#### Reliability
- Graceful degradation if PIM API unavailable (show error, auth features still work)
- Automatic retry on transient network failures

#### Security
- Use existing OAuth2 tokens with required PIM scopes
- No local logging of activation history (rely on Azure audit logs)
- Clear error messages guide users to request missing permissions

### 2.3 Scope

#### In Scope (MVP)
- View eligible subscription-level roles
- Activate roles with justification presets
- Favorite roles management
- Active role badge count
- Expiry icon indicator

#### In Scope (Post-MVP)
- Extend active roles
- Deactivate active roles
- Multi-role activation (bulk)

#### Out of Scope
- Azure AD directory roles (only Azure resource roles)
- Resource group or resource-level roles
- Management group roles
- Approval workflow handling (all roles assumed self-service)
- Shared team configurations

---

## 3. User Experience

### 3.1 User Persona

**Primary User**: Enterprise developer at Vipps MobilePay
- Has 10+ eligible Azure PIM roles across 6+ subscriptions
- Frequently activates subscription-level roles (Contributor, Reader, Owner)
- Main use cases: Incident investigation, debugging, maintenance
- Values speed and minimal friction over comprehensive features

### 3.2 Menu Layout

#### Signed In State with PIM (No Active Roles)

```
┌────────────────────────────────────────────────────────┐
│  Sven Malvik                                           │ ← Display only (bold)
│  sven.malvik@vipps.no                                  │ ← Display only
│  Vipps MobilePay                                       │ ← Display only (tenant)
│  Expires in 45 min                                     │ ← Token countdown
├────────────────────────────────────────────────────────┤
│  ⭐ PIM Roles                                          │ ← Section header
│  ────────────────────────────────────────────────────  │
│  ★ vipps-prod-001 - Contributor                      ▶ │ ← Favorite (submenu)
│  ★ vipps-staging - Contributor                       ▶ │ ← Favorite (submenu)
│  ☆ vipps-dev-001 - Owner                             ▶ │ ← Non-favorite
│  ☆ vipps-shared - Reader                             ▶ │ ← Non-favorite
│  ... (more roles)                                      │
│  ────────────────────────────────────────────────────  │
│  ↻ Refresh Roles                                       │ ← Manual refresh
├────────────────────────────────────────────────────────┤
│  Copy Access Token                                     │
│  Refresh Token                                         │
│  Sign Out                                              │
├────────────────────────────────────────────────────────┤
│  Settings                                            ▶ │
├────────────────────────────────────────────────────────┤
│  Quit                                          ⌘Q      │
└────────────────────────────────────────────────────────┘
```

#### Role Activation Submenu

```
vipps-prod-001 - Contributor ▶ ┌────────────────────────────────┐
                               │  Incident Investigation        │
                               │  Debugging                     │
                               │  Maintenance                   │
                               │  ───────────────────────────── │
                               │  Custom: "Sprint review"       │ ← User-defined
                               └────────────────────────────────┘
```

#### Signed In State with Active Roles

```
┌────────────────────────────────────────────────────────┐
│  Sven Malvik                                           │
│  sven.malvik@vipps.no                                  │
│  Vipps MobilePay                                       │
│  Expires in 45 min                                     │
├────────────────────────────────────────────────────────┤
│  🔓 Active Roles (2)                                   │ ← Section header
│  ────────────────────────────────────────────────────  │
│  vipps-prod-001 - Contributor          38 min left   ▶ │ ← Active role
│  vipps-staging - Contributor           52 min left   ▶ │ ← Active role
├────────────────────────────────────────────────────────┤
│  ⭐ PIM Roles                                          │
│  ────────────────────────────────────────────────────  │
│  ★ vipps-dev-001 - Owner                             ▶ │ ← Available favorite
│  ☆ vipps-shared - Reader                             ▶ │ ← Available non-fav
│  ────────────────────────────────────────────────────  │
│  ↻ Refresh Roles                                       │
├────────────────────────────────────────────────────────┤
│  ... (rest of menu)                                    │
└────────────────────────────────────────────────────────┘
```

#### Active Role Submenu (Post-MVP)

```
vipps-prod-001 - Contributor ▶ ┌────────────────────────────────┐
                               │  ● Active: 38 min remaining    │ ← Status (disabled)
                               │  ───────────────────────────── │
                               │  Extend (+ 1 hour)             │ ← One-click extend
                               │  Deactivate...                 │ ← With confirmation
                               └────────────────────────────────┘
```

### 3.3 User Flows

#### Flow 1: Activate a Favorite Role

```
1. Click menu bar icon
2. Menu shows favorites at top of PIM section
3. Hover/click favorite role → Submenu appears with presets
4. Click "Incident Investigation" preset
5. Menu closes, "Activating..." spinner briefly visible
6. macOS notification: "✓ Activated Contributor on vipps-prod-001 for 1 hour"
7. Next menu open shows role in "Active Roles" section with badge
```

#### Flow 2: Add a Role to Favorites

```
1. Click menu bar icon
2. Find role in PIM Roles section (☆ unfavorited)
3. Click the ☆ star icon next to role name
4. Star fills (★) and role moves to top of list
5. Favorite persists across app restarts
```

#### Flow 3: Extend an Active Role (Post-MVP)

```
1. Click menu bar icon
2. Click active role in "Active Roles" section
3. Submenu shows current status and "Extend (+ 1 hour)"
4. Click "Extend (+ 1 hour)"
5. Menu closes, notification: "✓ Extended Contributor on vipps-prod-001"
```

#### Flow 4: Handle Missing Permissions

```
1. User signs in successfully
2. App attempts to fetch eligible PIM roles
3. API returns 403 (insufficient permissions)
4. macOS alert dialog appears:
   Title: "PIM Access Required"
   Message: "Additional permissions needed for PIM role management.
             Request the following API permissions from your IT admin:
             • RoleManagement.ReadWrite.Directory
             • Or appropriate Azure PIM API access"
   Button: [OK]
5. PIM section shows "PIM unavailable - permissions required"
6. Auth features (sign in/out, user info) continue working
```

### 3.4 Menu Bar Icon States

| State | Icon | Badge | Description |
|-------|------|-------|-------------|
| Signed out | Gray cloud | None | No authentication |
| Signed in, no active roles | Blue cloud | None | Authenticated, no PIM roles active |
| Signed in, roles active | Blue cloud | Red badge "2" | 2 PIM roles currently active |
| Role expiring soon | Orange/amber cloud | Red badge | Within 5 minutes of expiry |
| Error | Red cloud | None | Authentication or API error |

---

## 4. Technical Architecture

### 4.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                           main.rs                                    │
│  (existing auth flow + new PIM integration)                          │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐    ┌──────────────────┐    ┌──────────────────────┐
│   menubar/    │    │      auth/       │    │        pim/          │
│               │    │                  │    │                      │
│ - builder.rs  │    │ - oauth.rs       │    │ - mod.rs             │
│   (add PIM    │    │ - graph.rs       │    │ - client.rs          │
│    section)   │    │ - token_manager  │    │   (Azure PIM API)    │
│               │    │                  │    │ - models.rs          │
│ - state.rs    │    │                  │    │   (Role, Assignment) │
│   (add PIM    │    │                  │    │ - cache.rs           │
│    state)     │    │                  │    │   (role cache)       │
│               │    │                  │    │ - favorites.rs       │
│ - updates.rs  │    │                  │    │   (local storage)    │
│   (badge,     │    │                  │    │                      │
│    expiry)    │    │                  │    │                      │
└───────────────┘    └──────────────────┘    └──────────────────────┘
                                │                       │
                                │                       │
                                ▼                       ▼
                     ┌──────────────────┐    ┌──────────────────────┐
                     │    keychain/     │    │   Local Storage      │
                     │                  │    │                      │
                     │  - tokens        │    │  - favorites.json    │
                     │  - user info     │    │  - pim_settings.json │
                     └──────────────────┘    └──────────────────────┘
```

### 4.2 New Module: `pim/`

```
src/pim/
├── mod.rs              # Public API, re-exports
├── client.rs           # Azure PIM API client
├── models.rs           # EligibleRole, ActiveAssignment, ActivationRequest
├── cache.rs            # Role cache with TTL (1 hour)
└── favorites.rs        # Favorites storage and management
```

### 4.3 Technology Stack Additions

```toml
# Additional dependencies for Phase 2
[dependencies]
# No new major dependencies - uses existing:
# - reqwest (HTTP client for PIM API)
# - serde (JSON serialization)
# - chrono (time handling)
# - tokio (async runtime)

# Local storage for favorites/settings
directories = "5.0"  # Platform-specific directories (already in common use)
```

### 4.4 Data Model

#### Core Structures

```rust
// src/pim/models.rs

/// Represents an Azure subscription-level role the user is eligible for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibleRole {
    /// Unique identifier for this role assignment eligibility
    pub id: String,

    /// Role definition ID (e.g., Contributor, Owner, Reader)
    pub role_definition_id: String,

    /// Human-readable role name
    pub role_name: String,

    /// Subscription ID where role applies
    pub subscription_id: String,

    /// Subscription display name
    pub subscription_name: String,

    /// Scope (e.g., "/subscriptions/{id}")
    pub scope: String,

    /// Maximum duration allowed for activation (from policy)
    pub max_duration: Option<Duration>,
}

/// Represents a currently active PIM role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAssignment {
    /// Assignment ID
    pub id: String,

    /// Reference to the eligible role
    pub role_definition_id: String,

    /// Role name
    pub role_name: String,

    /// Subscription ID
    pub subscription_id: String,

    /// Subscription name
    pub subscription_name: String,

    /// When the activation started
    pub start_time: DateTime<Utc>,

    /// When the activation expires
    pub end_time: DateTime<Utc>,

    /// Justification provided during activation
    pub justification: String,
}

/// Request to activate a PIM role
#[derive(Debug, Clone, Serialize)]
pub struct ActivationRequest {
    /// The eligible role to activate
    pub eligible_role_id: String,

    /// Justification reason
    pub justification: String,

    /// Requested duration
    pub duration: Duration,
}

/// Justification preset for quick activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustificationPreset {
    /// Display label
    pub label: String,

    /// Justification text sent to Azure
    pub justification: String,

    /// Whether this is a built-in preset (vs user-defined)
    pub is_builtin: bool,
}

/// User's PIM preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PimSettings {
    /// Default activation duration in minutes
    pub default_duration_minutes: u32,  // Default: 60

    /// Whether to show non-favorite eligible roles
    pub show_all_eligible: bool,  // Default: true

    /// Custom justification presets
    pub custom_presets: Vec<JustificationPreset>,

    /// Favorite role IDs
    pub favorites: Vec<String>,
}
```

#### Application State Extensions

```rust
// Extend existing AppState in src/menubar/state.rs

pub struct AppState {
    // Existing fields...
    pub auth_state: Arc<Mutex<AuthState>>,
    pub user_info: Arc<Mutex<Option<UserInfo>>>,

    // New PIM fields
    pub pim_state: Arc<Mutex<PimState>>,
}

pub struct PimState {
    /// Cached eligible roles (refreshed hourly)
    pub eligible_roles: Vec<EligibleRole>,

    /// Currently active role assignments
    pub active_assignments: Vec<ActiveAssignment>,

    /// When eligible roles were last fetched
    pub roles_cached_at: Option<DateTime<Utc>>,

    /// User's PIM settings (including favorites)
    pub settings: PimSettings,

    /// Current PIM API status
    pub api_status: PimApiStatus,
}

pub enum PimApiStatus {
    /// Not yet checked
    Unknown,
    /// PIM API accessible
    Available,
    /// PIM API returned permission error
    PermissionDenied { message: String },
    /// PIM API unreachable
    Unavailable { error: String },
}
```

### 4.5 Azure PIM API Integration

#### API Choice: Microsoft Graph API

Using Microsoft Graph API for PIM operations because:
1. Already integrated for user profile (Phase 1)
2. Unified authentication (same token, add scopes)
3. Better documentation and SDK support
4. Consistent with Microsoft's direction

#### Required API Scopes

Add to existing OAuth scopes:
```
RoleEligibilitySchedule.Read.Directory
RoleAssignmentSchedule.ReadWrite.Directory
```

Or use Azure RBAC API if Graph PIM is not available:
```
https://management.azure.com/.default
```

#### Key API Endpoints

**List Eligible Role Assignments** (subscription-level):
```http
GET https://management.azure.com/subscriptions/{subscriptionId}/providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version=2020-10-01&$filter=principalId eq '{userId}'
Authorization: Bearer {access_token}
```

**List Active Role Assignments**:
```http
GET https://management.azure.com/subscriptions/{subscriptionId}/providers/Microsoft.Authorization/roleAssignmentScheduleInstances?api-version=2020-10-01&$filter=principalId eq '{userId}'
Authorization: Bearer {access_token}
```

**Activate Role** (create assignment schedule request):
```http
PUT https://management.azure.com/{scope}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{requestId}?api-version=2020-10-01
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "properties": {
    "principalId": "{userId}",
    "roleDefinitionId": "{roleDefinitionId}",
    "requestType": "SelfActivate",
    "justification": "Incident Investigation",
    "scheduleInfo": {
      "startDateTime": "2024-01-15T10:00:00Z",
      "expiration": {
        "type": "AfterDuration",
        "duration": "PT1H"
      }
    }
  }
}
```

#### PIM Client Implementation

```rust
// src/pim/client.rs

pub struct PimClient {
    http_client: reqwest::Client,
    base_url: String,
}

impl PimClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: "https://management.azure.com".to_string(),
        }
    }

    /// Fetch all eligible roles across all accessible subscriptions
    pub async fn get_eligible_roles(
        &self,
        access_token: &str,
        user_id: &str,
    ) -> Result<Vec<EligibleRole>, PimError> {
        // 1. List subscriptions user has access to
        // 2. For each subscription, fetch eligible role assignments
        // 3. Resolve role definition names
        // 4. Return aggregated list
    }

    /// Fetch currently active role assignments
    pub async fn get_active_assignments(
        &self,
        access_token: &str,
        user_id: &str,
    ) -> Result<Vec<ActiveAssignment>, PimError> {
        // Similar to eligible, but query active assignments
    }

    /// Activate a role
    pub async fn activate_role(
        &self,
        access_token: &str,
        request: ActivationRequest,
    ) -> Result<ActiveAssignment, PimError> {
        // PUT role assignment schedule request
    }

    /// Extend an active role (Post-MVP)
    pub async fn extend_role(
        &self,
        access_token: &str,
        assignment_id: &str,
        additional_duration: Duration,
    ) -> Result<ActiveAssignment, PimError> {
        // ...
    }

    /// Deactivate a role (Post-MVP)
    pub async fn deactivate_role(
        &self,
        access_token: &str,
        assignment_id: &str,
    ) -> Result<(), PimError> {
        // ...
    }
}
```

### 4.6 Caching Strategy

```rust
// src/pim/cache.rs

pub struct PimCache {
    /// Cache TTL (1 hour)
    ttl: Duration,

    /// Cached eligible roles
    eligible_roles: Option<CachedData<Vec<EligibleRole>>>,
}

struct CachedData<T> {
    data: T,
    cached_at: DateTime<Utc>,
}

impl PimCache {
    pub fn new() -> Self {
        Self {
            ttl: Duration::hours(1),
            eligible_roles: None,
        }
    }

    pub fn get_eligible_roles(&self) -> Option<&Vec<EligibleRole>> {
        self.eligible_roles.as_ref().and_then(|cached| {
            if Utc::now() - cached.cached_at < self.ttl {
                Some(&cached.data)
            } else {
                None
            }
        })
    }

    pub fn set_eligible_roles(&mut self, roles: Vec<EligibleRole>) {
        self.eligible_roles = Some(CachedData {
            data: roles,
            cached_at: Utc::now(),
        });
    }

    pub fn invalidate(&mut self) {
        self.eligible_roles = None;
    }
}
```

### 4.7 Local Storage for Favorites/Settings

```rust
// src/pim/favorites.rs

use directories::ProjectDirs;

const SETTINGS_FILE: &str = "pim_settings.json";

pub fn get_settings_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("de", "malvik", "azurepim")
        .expect("Could not determine project directories");
    proj_dirs.config_dir().join(SETTINGS_FILE)
}

pub fn load_settings() -> PimSettings {
    let path = get_settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok();
        content.and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    } else {
        PimSettings::default()
    }
}

pub fn save_settings(settings: &PimSettings) -> Result<(), std::io::Error> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)
}

impl Default for PimSettings {
    fn default() -> Self {
        Self {
            default_duration_minutes: 60,
            show_all_eligible: true,
            custom_presets: vec![],
            favorites: vec![],
        }
    }
}
```

---

## 5. Integration Points

### 5.1 Token Scope Extension

Modify OAuth configuration to request PIM scopes:

```toml
# config.toml
[oauth.scopes]
scopes = [
    # Existing scopes
    "https://graph.microsoft.com/User.Read",
    "openid",
    "profile",
    "email",
    "offline_access",
    # New PIM scopes
    "https://management.azure.com/.default"
]
```

### 5.2 Token Manager Integration

The existing `TokenManager` handles token refresh. PIM operations use the same access token but validate required scopes are present.

### 5.3 Menu Action Channel

Extend the existing action channel to handle PIM actions:

```rust
pub enum MenuAction {
    // Existing
    SignIn,
    SignOut,
    RefreshToken,
    CopyToken,
    Quit,

    // New PIM actions
    ActivateRole { role_id: String, justification: String },
    ExtendRole { assignment_id: String },
    DeactivateRole { assignment_id: String },
    ToggleFavorite { role_id: String },
    RefreshPimRoles,
}
```

---

## 6. Operations

### 6.1 Error Handling

#### PIM-Specific Errors

```rust
// src/pim/error.rs (or add to src/error.rs)

#[derive(Error, Debug)]
pub enum PimError {
    #[error("PIM API permission denied: {0}")]
    PermissionDenied(String),

    #[error("Role activation failed: {0}")]
    ActivationFailed(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Role already active")]
    RoleAlreadyActive,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Maximum duration exceeded")]
    DurationExceeded,
}
```

#### User-Facing Error Messages

| Internal Error | User Message | Action |
|---------------|--------------|--------|
| `PermissionDenied` | "PIM access required. Request permissions from IT." | Show instructions |
| `ActivationFailed` | "Failed to activate role. Try again." | Retry button |
| `RoleAlreadyActive` | "Role is already active." | Show extend option |
| `Network` | "Network error. Check connection." | Retry button |
| `DurationExceeded` | "Requested duration exceeds policy limit." | Use max duration |

### 6.2 Logging

```rust
// PIM operations logged at INFO level (no sensitive data)
tracing::info!(
    role_name = %role.role_name,
    subscription = %role.subscription_name,
    duration_minutes = %duration.num_minutes(),
    "Activating PIM role"
);

tracing::info!(
    role_name = %role.role_name,
    subscription = %role.subscription_name,
    expires_in_minutes = %time_remaining.num_minutes(),
    "PIM role activated successfully"
);

// Never log justification text (may contain sensitive info)
```

### 6.3 Notifications

Use `mac-notification-sys` or native `NSUserNotification` for activation feedback:

```rust
// src/notifications.rs

pub fn notify_activation_success(role: &str, subscription: &str, duration_minutes: u32) {
    // macOS notification
    // Title: "PIM Role Activated"
    // Body: "Contributor on vipps-prod-001 for 1 hour"
}

pub fn notify_activation_failed(role: &str, error: &str) {
    // macOS notification
    // Title: "PIM Activation Failed"
    // Body: "Could not activate Contributor: {error}"
}
```

---

## 7. Implementation Plan

### 7.1 MVP Phase (View + Activate)

#### Step 1: PIM Module Foundation
- Create `src/pim/` module structure
- Implement `PimClient` with Azure Management API calls
- Add `EligibleRole` and `ActiveAssignment` models
- Unit tests with mock responses

#### Step 2: Cache and Storage
- Implement role cache with 1-hour TTL
- Local settings storage (favorites, preferences)
- Default justification presets

#### Step 3: Menu Integration
- Add PIM section to menu builder
- Display eligible roles (favorites first)
- Star icon toggle for favorites
- Submenu with justification presets

#### Step 4: Activation Flow
- Role activation API integration
- "Activating..." state handling
- Success/failure notifications
- Error dialogs for failures

#### Step 5: Active Role Display
- Show active roles section
- Badge count on menu bar icon
- Expiry time remaining display
- Icon color change for expiring roles

#### Step 6: Polish
- Manual refresh action
- Settings UI for duration/presets
- Missing permissions guidance
- Testing with real Azure PIM

### 7.2 Post-MVP Phase (Extend + Deactivate)

#### Step 7: Extend Flow
- One-click extend action
- Reuse original justification
- Update UI after extension

#### Step 8: Deactivate Flow
- Deactivate with confirmation dialog
- Remove from active list
- Update badge count

#### Step 9: Bulk Operations (Nice-to-Have)
- Multi-select for activation
- Bulk deactivation

### 7.3 Dependencies

- **Phase 1 Complete**: OAuth flow, token management, menu bar infrastructure
- **Azure Permissions**: User must have PIM eligible roles and API access
- **Token Scopes**: Updated OAuth scopes for Azure Management API

### 7.4 Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Azure PIM API changes | High | Pin API version, monitor deprecations |
| Complex subscription enumeration | Medium | Cache aggressively, async loading |
| Permission errors | Medium | Clear error messages, graceful degradation |
| Slow API responses | Medium | Close menu, notify on completion |
| Rate limiting | Low | Respect retry headers, cache extensively |

---

## 8. Tradeoffs & Decisions

### 8.1 Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **API** | Azure Management API | More direct for PIM, Graph PIM limited |
| **Cache TTL** | 1 hour | Balance freshness vs API calls |
| **Scope** | Subscription-level only | Covers primary use case, simpler UI |
| **Storage** | Local JSON file | Simple, no sync needed |
| **Polling** | None (manual refresh) | Reduces complexity and API usage |

### 8.2 Alternatives Considered

**Microsoft Graph PIM API vs Azure Management API**
- Graph PIM: Better for Azure AD roles, but subscription-level support is limited
- Azure Management: Direct support for Azure resource role PIM operations
- **Decision**: Azure Management API for subscription-level roles

**Real-time sync vs manual refresh**
- Real-time: Better UX but complex, higher API usage
- Manual: Simpler, user controls when to refresh
- **Decision**: Manual refresh with 1-hour cache, sync on menu open

### 8.3 Technical Debt

Acceptable in MVP:
- Sequential subscription enumeration (could be parallelized later)
- No offline mode (requires network for all operations)
- Simple favorites storage (could use Keychain for consistency)

---

## 9. Success Criteria

### 9.1 MVP Launch Criteria

- [ ] Can view all eligible subscription-level roles
- [ ] Can activate a role in 2 clicks (role → preset)
- [ ] Favorites persist across restarts
- [ ] Active roles displayed with time remaining
- [ ] Badge count shows on menu bar icon
- [ ] Icon changes color when role expiring soon
- [ ] Errors show helpful messages with guidance
- [ ] No crashes when PIM API unavailable
- [ ] Works with real Vipps Azure PIM roles

### 9.2 Performance Benchmarks

- Menu opens in <100ms with cached data
- Role activation completes in <5 seconds
- Eligible role fetch completes in <10 seconds (cold)

### 9.3 Quality Checklist

- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` applied
- [ ] `cargo test` passes
- [ ] No tokens logged (manual review)
- [ ] Error messages reviewed for clarity

---

## 10. Open Questions

1. **Exact Azure Management API endpoint versions** - Need to verify current stable versions
2. **Multi-subscription performance** - How many subscriptions typical at Vipps?
3. **Role definition name resolution** - May need additional API call to get friendly names
4. **Notification system** - Use native NSUserNotification or third-party crate?

---

## Document Version

- **Version**: 1.0
- **Date**: 2026-01-02
- **Author**: Claude Code (AI Assistant)
- **Interviewee**: Sven Malvik
- **Status**: Ready for Review

---

**End of Phase 2 Specification**
