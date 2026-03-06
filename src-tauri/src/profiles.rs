use crate::display::DisplayMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayProfile {
    /// Map of device_name -> desired DisplayMode
    pub assignments: HashMap<String, DisplayMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllProfiles {
    /// Map of username -> DisplayProfile
    pub users: HashMap<String, DisplayProfile>,
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
    path.push("AsterDisplayManager");
    path.push("profiles.json");
    path
}

pub fn load_profiles() -> AllProfiles {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AllProfiles::default()
    }
}

pub fn save_profiles(profiles: &AllProfiles) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(profiles).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn current_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Register (or remove) this app from the current user's startup in the registry
#[cfg(windows)]
pub fn set_startup_enabled(enabled: bool, exe_path: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE,
        )
        .map_err(|e| e.to_string())?;

    if enabled {
        run_key
            .set_value("AsterDisplayManager", &format!("\"{}\" --apply-profile", exe_path))
            .map_err(|e| e.to_string())?;
    } else {
        let _ = run_key.delete_value("AsterDisplayManager");
    }
    Ok(())
}

#[cfg(windows)]
pub fn is_startup_enabled() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        let val: Result<String, _> = run_key.get_value("AsterDisplayManager");
        val.is_ok()
    } else {
        false
    }
}

#[cfg(not(windows))]
pub fn set_startup_enabled(_enabled: bool, _exe_path: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn is_startup_enabled() -> bool {
    false
}
