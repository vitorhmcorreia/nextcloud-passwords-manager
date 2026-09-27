//! User preferences, stored as plain JSON next to the vault.
//!
//! Nothing here is secret, so it isn't encrypted. That also means it can be
//! read before the vault is unlocked.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Lock the vault after this many minutes without input.
    pub auto_lock_mins: u32,
    /// Wipe a copied secret from the clipboard after this many seconds.
    pub clipboard_clear_secs: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { auto_lock_mins: 5, clipboard_clear_secs: 20 }
    }
}

impl Settings {
    /// Reject values outside what the settings screen offers.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=60).contains(&self.auto_lock_mins) {
            return Err("Auto-lock must be between 1 and 60 minutes".into());
        }
        if !(5..=120).contains(&self.clipboard_clear_secs) {
            return Err("Clipboard clear time must be between 5 and 120 seconds".into());
        }
        Ok(())
    }

    /// Read the file; a missing or broken file just gives the defaults.
    pub fn load(path: &Path) -> Settings {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Settings>(&bytes).ok())
            .filter(|s| s.validate().is_ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// The current settings, shared by the commands.
#[derive(Default)]
pub struct SettingsState(pub Mutex<Settings>);

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json"))
}

#[tauri::command]
pub fn get_settings(state: tauri::State<SettingsState>) -> Settings {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_settings(app: AppHandle, state: tauri::State<SettingsState>, settings: Settings) -> Result<(), String> {
    settings.validate()?;
    settings.save(&settings_path(&app)?)?;
    *state.0.lock().unwrap() = settings;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_bad_file_gives_defaults() {
        let dir = std::env::temp_dir().join(format!("ncpass-settings-{}", std::process::id()));
        let path = dir.join("settings.json");
        assert_eq!(Settings::load(&path), Settings::default());

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "not json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());

        // Out of range values are ignored too.
        std::fs::write(&path, r#"{"autoLockMins": 0, "clipboardClearSecs": 20}"#).unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn round_trip_and_missing_fields() {
        let dir = std::env::temp_dir().join(format!("ncpass-settings-rt-{}", std::process::id()));
        let path = dir.join("settings.json");
        let s = Settings { auto_lock_mins: 15, clipboard_clear_secs: 45 };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path), s);

        // A file from an older version with fewer fields fills in the defaults.
        std::fs::write(&path, r#"{"autoLockMins": 10}"#).unwrap();
        assert_eq!(Settings::load(&path), Settings { auto_lock_mins: 10, clipboard_clear_secs: 20 });
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
