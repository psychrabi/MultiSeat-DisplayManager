mod display;
mod profiles;

use display::{ApplyResult, DisplayDevice, DisplayMode};
use profiles::{AllProfiles, DisplayProfile};
use std::collections::HashMap;
use tauri::Manager;

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
fn get_displays() -> Vec<DisplayDevice> {
    display::enumerate_displays()
}

#[tauri::command]
fn apply_settings(
    device_name: String,
    width: u32,
    height: u32,
    refresh_rate: u32,
    persist: bool,
) -> ApplyResult {
    display::apply_display_settings(&device_name, width, height, refresh_rate, persist)
}

#[tauri::command]
fn get_all_profiles() -> AllProfiles {
    profiles::load_profiles()
}

#[tauri::command]
fn save_user_profile(
    username: String,
    assignments: HashMap<String, DisplayMode>,
) -> Result<(), String> {
    let mut all = profiles::load_profiles();
    all.users.insert(username, DisplayProfile { assignments });
    profiles::save_profiles(&all)
}

#[tauri::command]
fn delete_user_profile(username: String) -> Result<(), String> {
    let mut all = profiles::load_profiles();
    all.users.remove(&username);
    profiles::save_profiles(&all)
}

#[tauri::command]
fn apply_profile_for_user(username: String) -> Vec<ApplyResult> {
    let all = profiles::load_profiles();
    let mut results = Vec::new();
    if let Some(profile) = all.users.get(&username) {
        for (device_name, mode) in &profile.assignments {
            let r = display::apply_display_settings(
                device_name,
                mode.width,
                mode.height,
                mode.refresh_rate,
                true,
            );
            results.push(r);
        }
    } else {
        results.push(ApplyResult {
            success: false,
            message: format!("No profile found for user '{}'", username),
        });
    }
    results
}

#[tauri::command]
fn get_current_username() -> String {
    profiles::current_username()
}

#[tauri::command]
fn set_primary_display(device_name: String) -> ApplyResult {
    display::set_primary_display(&device_name)
}

#[tauri::command]
fn toggle_monitor_state(device_name: String, enabled: bool) -> ApplyResult {
    display::toggle_monitor_state(&device_name, enabled)
}

#[tauri::command]
fn get_startup_enabled() -> bool {
    profiles::is_startup_enabled()
}

#[tauri::command]
fn set_startup(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let exe = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("../aster-display-manager.exe")
        .to_string_lossy()
        .into_owned();

    // Fallback: use current exe path
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(exe);

    profiles::set_startup_enabled(enabled, &exe_path)
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Called by main.rs when launched with --apply-profile.
/// Applies the current Windows user's saved display profile silently.
pub fn apply_current_user_profile() {
    let username = profiles::current_username();
    let all = profiles::load_profiles();
    if let Some(profile) = all.users.get(&username) {
        for (device_name, mode) in &profile.assignments {
            display::apply_display_settings(
                device_name,
                mode.width,
                mode.height,
                mode.refresh_rate,
                true,
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_displays,
            apply_settings,
            set_primary_display,
            get_all_profiles,
            save_user_profile,
            delete_user_profile,
            apply_profile_for_user,
            get_current_username,
            get_startup_enabled,
            set_startup,
            toggle_monitor_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
