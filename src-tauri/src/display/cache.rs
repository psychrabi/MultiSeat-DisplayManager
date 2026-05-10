use std::sync::Mutex;
use std::path::PathBuf;
use std::fs;

use serde::{Deserialize, Serialize};

#[cfg(windows)]
#[derive(Clone)]
pub struct CachedTopology {
    pub paths: Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO>,
    pub modes: Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO>,
}

#[cfg(windows)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownMonitor {
    pub device_name: String,
    pub device_string: String,
    pub adapter_luid: u64,
    pub target_id: u32,
    pub edid_hash: Option<String>,
    pub position_x: i32,
    pub position_y: i32,
}

#[cfg(windows)]
pub static TOPOLOGY_CACHE: Mutex<Option<CachedTopology>> = Mutex::new(None);

#[cfg(windows)]
pub static DISCONNECTED_TOPOLOGY: Mutex<Option<CachedTopology>> = Mutex::new(None);

#[cfg(windows)]
pub static PENDING_CONFIRMATION: Mutex<Option<CachedTopology>> = Mutex::new(None);

#[cfg(windows)]
static KNOWN_MONITORS: Mutex<Vec<KnownMonitor>> = Mutex::new(Vec::new());

#[cfg(windows)]
fn known_monitors_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
    path.push("AsterDisplayManager");
    path.push("known_monitors.json");
    path
}

#[cfg(windows)]
fn load_known_monitors() -> Vec<KnownMonitor> {
    let path = known_monitors_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

#[cfg(windows)]
fn save_known_monitors(known: &[KnownMonitor]) {
    let path = known_monitors_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(known) {
        let _ = fs::write(&path, json);
    }
}

#[cfg(windows)]
pub fn with_known_monitors<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<KnownMonitor>) -> R,
{
    if let Ok(mut guard) = KNOWN_MONITORS.lock() {
        if guard.is_empty() {
            *guard = load_known_monitors();
        }
        let result = f(&mut *guard);
        save_known_monitors(&guard);
        result
    } else {
        f(&mut Vec::new())
    }
}
