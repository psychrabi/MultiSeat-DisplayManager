use crate::display::{DisplayId, DisplayMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Display assignment with hardware-based identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayAssignment {
    pub display_id: DisplayId,
    pub mode: DisplayMode,
    pub position_x: i32,
    pub position_y: i32,
    pub is_primary: bool,
    pub orientation: String,      // "landscape", "portrait", etc.
    pub scale_factor: u32,
    #[serde(default)]
    pub monitor_name: Option<String>,  // Friendly monitor name (e.g., "MSI MAG274QRF-QD")
}

/// Per-user profile containing display assignments
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserDisplayProfile {
    /// Map of display_id (as string key) -> assignment
    /// Key format: "adapter_luid_target_id" or EDID hash if available
    pub assignments: HashMap<String, DisplayAssignment>,
    /// Last known good layout for rollback
    #[serde(default)]
    pub last_known_good_layout: Option<Vec<DisplayAssignment>>,
}

/// All user profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllProfiles {
    /// Map of username -> DisplayProfile
    pub users: HashMap<String, UserDisplayProfile>,
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

/// Generate a stable key for a DisplayId for use in HashMap
pub fn display_id_to_key(display_id: &DisplayId) -> String {
    if let Some(ref edid) = display_id.edid_hash {
        // Use EDID hash as key if available (most stable)
        format!("edid_{}", edid.replace('\\', "_").replace('#', "_"))
    } else {
        // Fallback to adapter_luid_target_id
        format!("{}_{}", display_id.adapter_luid, display_id.target_id)
    }
}

/// Parse a display key back to DisplayId components
pub fn parse_display_key(key: &str) -> Option<(u64, u32)> {
    if key.starts_with("edid_") {
        return None; // Can't reconstruct DisplayId from EDID key alone
    }
    let parts: Vec<&str> = key.split('_').collect();
    if parts.len() == 2 {
        if let (Ok(adapter_luid), Ok(target_id)) = (
            parts[0].parse::<u64>(),
            parts[1].parse::<u32>(),
        ) {
            return Some((adapter_luid, target_id));
        }
    }
    None
}

/// Remap profile assignments to match current displays
/// This handles cases where displays have been reconnected or adapter LUID changed
pub fn remap_profile_assignments(
    profile: &UserDisplayProfile,
    current_displays: &[crate::display::DisplayDevice],
) -> HashMap<String, DisplayAssignment> {
    let mut remapped = HashMap::new();
    let mut used_display_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Build a map of EDID hashes to current displays
    let mut current_by_edid: HashMap<String, &crate::display::DisplayDevice> = HashMap::new();
    let mut current_by_target: HashMap<(u64, u32), &crate::display::DisplayDevice> = HashMap::new();
    
    for display in current_displays {
        if let Some(ref edid) = display.display_id.edid_hash {
            current_by_edid.entry(edid.clone()).or_insert(display);
        }
        let key = (display.display_id.adapter_luid, display.display_id.target_id);
        current_by_target.entry(key).or_insert(display);
    }

    for (profile_key, assignment) in &profile.assignments {
        let mut matched_display: Option<&crate::display::DisplayDevice> = None;
        let mut new_key = profile_key.clone();

        // Try to match by EDID hash first (most reliable)
        if profile_key.starts_with("edid_") {
            let edid_part = profile_key.strip_prefix("edid_").unwrap_or(profile_key);
            // Find matching EDID in current displays
            for (edid, display) in &current_by_edid {
                if edid.contains(edid_part) || edid_part.contains(edid) {
                    matched_display = Some(*display);
                    new_key = display_id_to_key(&display.display_id);
                    break;
                }
            }
        } else if let Some((adapter_luid, target_id)) = parse_display_key(profile_key) {
            // Try exact match by adapter_luid + target_id
            if let Some(display) = current_by_target.get(&(adapter_luid, target_id)) {
                matched_display = Some(*display);
            } else {
                // Try to find by target_id alone (adapter may have changed)
                for ((_luid, tid), display) in &current_by_target {
                    if *tid == target_id && !used_display_ids.contains(&display_id_to_key(&display.display_id)) {
                        matched_display = Some(*display);
                        new_key = display_id_to_key(&display.display_id);
                        break;
                    }
                }
            }
        }

        // If we found a matching display, use it
        if let Some(display) = matched_display {
            if !used_display_ids.contains(&new_key) {
                let mut new_assignment = assignment.clone();
                new_assignment.display_id = display.display_id.clone();
                remapped.insert(new_key.clone(), new_assignment);
                used_display_ids.insert(new_key);
            }
        }
        // If no match found, skip this assignment (display not connected)
    }

    remapped
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
