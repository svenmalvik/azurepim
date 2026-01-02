//! PIM settings persistence using local JSON storage.

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use tracing::{debug, error, warn};

use super::models::PimSettings;
use crate::error::PimError;

/// Settings file name.
const SETTINGS_FILE: &str = "pim_settings.json";

/// Get the path to the PIM settings file.
///
/// Returns `~/Library/Application Support/de.malvik.azurepim/pim_settings.json` on macOS.
pub fn get_settings_path() -> Option<PathBuf> {
    ProjectDirs::from("de", "malvik", "azurepim").map(|dirs| dirs.config_dir().join(SETTINGS_FILE))
}

/// Load PIM settings from disk.
///
/// Returns default settings if file doesn't exist or is corrupted.
pub fn load_pim_settings() -> PimSettings {
    let path = match get_settings_path() {
        Some(p) => p,
        None => {
            warn!("Could not determine config directory, using default settings");
            return PimSettings::default();
        }
    };

    if !path.exists() {
        debug!("PIM settings file does not exist, using defaults");
        return PimSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(settings) => {
                debug!("Loaded PIM settings from {:?}", path);
                settings
            }
            Err(e) => {
                error!("Failed to parse PIM settings: {}, using defaults", e);
                PimSettings::default()
            }
        },
        Err(e) => {
            error!("Failed to read PIM settings file: {}, using defaults", e);
            PimSettings::default()
        }
    }
}

/// Save PIM settings to disk.
pub fn save_pim_settings(settings: &PimSettings) -> Result<(), PimError> {
    let path = get_settings_path().ok_or_else(|| {
        PimError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        ))
    })?;

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(PimError::Io)?;
    }

    let content = serde_json::to_string_pretty(settings).map_err(|e| {
        PimError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;

    fs::write(&path, content).map_err(PimError::Io)?;

    debug!("Saved PIM settings to {:?}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_settings_path() {
        // This test just verifies the path is constructed correctly
        let path = get_settings_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.ends_with("pim_settings.json"));
    }

    #[test]
    fn test_load_default_settings() {
        // Loading from non-existent path should return defaults
        let settings = load_pim_settings();
        assert_eq!(settings.default_duration_minutes, 60);
        assert!(settings.favorite_role_keys.is_empty());
    }

    #[test]
    fn test_settings_roundtrip() {
        // Create a temporary directory for testing
        let temp_dir = env::temp_dir().join("azurepim_test");
        let _ = fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test_settings.json");

        // Create test settings
        let settings = PimSettings {
            default_duration_minutes: 120,
            favorite_role_keys: vec!["sub:role".to_string()],
            ..Default::default()
        };

        // Write to temp file
        let content = serde_json::to_string_pretty(&settings).unwrap();
        fs::write(&test_file, content).unwrap();

        // Read back
        let loaded: PimSettings =
            serde_json::from_str(&fs::read_to_string(&test_file).unwrap()).unwrap();

        assert_eq!(loaded.default_duration_minutes, 120);
        assert_eq!(loaded.favorite_role_keys, vec!["sub:role".to_string()]);

        // Cleanup
        let _ = fs::remove_file(&test_file);
    }
}
