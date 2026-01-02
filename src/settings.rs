//! Application settings management including auto-launch at login.

use anyhow::Result;
use tracing::{info, warn};

/// Check if the app is set to auto-launch at login.
#[allow(dead_code)]
pub fn is_auto_launch_enabled() -> bool {
    // For now, return a default value
    // Full implementation would use SMAppService or LaunchServices
    // This requires more complex integration with macOS APIs
    warn!("Auto-launch check not fully implemented");
    false
}

/// Enable or disable auto-launch at login.
pub fn set_auto_launch(enabled: bool) -> Result<()> {
    // Full implementation would use SMAppService (macOS 13+) or
    // LaunchServices/LoginItems for older macOS versions
    //
    // For SMAppService:
    // ```
    // use objc2_service_management::SMAppService;
    // let service = SMAppService::mainApp();
    // if enabled {
    //     service.registerAndReturnError()?;
    // } else {
    //     service.unregisterAndReturnError()?;
    // }
    // ```
    //
    // For now, we'll log the intent and rely on manual configuration

    if enabled {
        info!("Auto-launch enabled (manual configuration required)");
    } else {
        info!("Auto-launch disabled (manual configuration required)");
    }

    // Print instructions for the user
    if enabled {
        info!("To enable auto-launch, add the app to System Settings > General > Login Items");
    }

    Ok(())
}

/// Get the path to the log directory.
pub fn log_directory() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    home.join("Library/Logs/azurepim")
}

/// Initialize the log directory.
pub fn init_log_directory() -> Result<()> {
    let log_dir = log_directory();
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_directory() {
        let path = log_directory();
        assert!(path.to_string_lossy().contains("azurepim"));
    }
}
