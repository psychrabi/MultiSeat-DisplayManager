mod backend;
mod display;
mod profiles;

use backend::DynDisplayBackend;
use display::ApplyResult;
use display::DisplayId;
use profiles::{remap_profile_assignments, AllProfiles, DisplayAssignment, UserDisplayProfile};
use std::collections::HashMap;
use tauri::Manager;

#[tauri::command]
fn get_displays(backend: tauri::State<'_, DynDisplayBackend>) -> Vec<display::DisplayDevice> {
    backend.list_displays().unwrap_or_default()
}

#[tauri::command]
fn apply_settings(
    device_name: String,
    width: u32,
    height: u32,
    refresh_rate: u32,
    persist: bool,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    backend.apply_settings(&device_name, width, height, refresh_rate, persist)
}

#[tauri::command]
fn get_all_profiles() -> AllProfiles {
    profiles::load_profiles()
}

#[tauri::command]
fn save_user_profile(
    username: String,
    assignments: HashMap<String, DisplayAssignment>,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> Result<(), String> {
    fn same_monitor(a: &DisplayId, b: &DisplayId) -> bool {
        match (&a.edid_hash, &b.edid_hash) {
            (Some(ae), Some(be)) => ae == be,
            _ => a.adapter_luid == b.adapter_luid && a.target_id == b.target_id,
        }
    }

    let mut all = profiles::load_profiles();
    let user_profile = all
        .users
        .entry(username.clone())
        .or_insert_with(UserDisplayProfile::default);

    // Remove stale entries whose key differs but refer to the same physical monitor
    let stale: Vec<String> = user_profile
        .assignments
        .keys()
        .filter(|old_key| {
            !assignments.contains_key(*old_key)
                && assignments.values().any(|new_a| {
                    user_profile
                        .assignments
                        .get(*old_key)
                        .map_or(false, |old_a| {
                            same_monitor(&new_a.display_id, &old_a.display_id)
                        })
                })
        })
        .cloned()
        .collect();
    for k in stale {
        user_profile.assignments.remove(&k);
    }

    user_profile.assignments = assignments;

    let current_displays = backend.list_displays().map_err(|e| e.to_string())?;
    let mut current_layout = Vec::new();
    for d in &current_displays {
        if let Some(ref mode) = d.current_mode {
            current_layout.push(DisplayAssignment {
                display_id: d.display_id.clone(),
                mode: mode.clone(),
                position_x: d.position_x,
                position_y: d.position_y,
                is_primary: d.is_primary,
                orientation: format!("{:?}", d.orientation),
                scale_factor: d.scale_factor,
                monitor_name: Some(d.device_string.clone()),
            });
        }
    }
    if !current_layout.is_empty() {
        user_profile.last_known_good_layout = Some(current_layout);
    }
    profiles::save_profiles(&all)
}

#[tauri::command]
fn delete_user_profile(username: String) -> Result<(), String> {
    let mut all = profiles::load_profiles();
    all.users.remove(&username);
    profiles::save_profiles(&all)
}

#[tauri::command]
fn apply_profile_for_user(
    username: String,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> Vec<ApplyResult> {
    let all = profiles::load_profiles();
    let current_displays = backend.list_displays().unwrap_or_default();
    let mut results = Vec::new();

    if let Some(profile) = all.users.get(&username) {
        let remapped = remap_profile_assignments(profile, &current_displays);

        if remapped.is_empty() {
            results.push(ApplyResult::err("No matching displays found for profile"));
            return results;
        }

        for (_key, assignment) in remapped {
            let matching_display = current_displays.iter().find(|d| {
                d.display_id
                    .matches_path_by_assignment(&assignment.display_id)
            });

            if let Some(display) = matching_display {
                let target_orientation = match assignment.orientation.as_str() {
                    "Portrait" | "portrait" => display::DisplayOrientation::Portrait,
                    "LandscapeFlipped" | "landscapeflipped" => {
                        display::DisplayOrientation::LandscapeFlipped
                    }
                    "PortraitFlipped" | "portraitflipped" => {
                        display::DisplayOrientation::PortraitFlipped
                    }
                    _ => display::DisplayOrientation::Landscape,
                };
                if display.orientation != target_orientation {
                    results.push(backend.set_orientation(&display.device_name, target_orientation));
                }

                results.push(backend.apply_settings(
                    &display.device_name,
                    assignment.mode.width,
                    assignment.mode.height,
                    assignment.mode.refresh_rate,
                    true,
                ));

                if assignment.position_x != display.position_x
                    || assignment.position_y != display.position_y
                {
                    results.push(backend.set_position(
                        &display.device_name,
                        assignment.position_x,
                        assignment.position_y,
                    ));
                }
            } else {
                results.push(ApplyResult::err(
                    "Display not found in current configuration",
                ));
            }
        }
    } else {
        results.push(ApplyResult::err(format!(
            "No profile found for user '{}'",
            username
        )));
    }
    results
}

#[tauri::command]
fn get_current_username() -> String {
    profiles::current_username()
}

#[tauri::command]
fn set_primary_display(
    device_name: String,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    backend.set_primary(&device_name)
}

#[tauri::command]
fn set_orientation(
    device_name: String,
    orientation: display::DisplayOrientation,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    backend.set_orientation(&device_name, orientation)
}

#[tauri::command]
fn set_position(
    device_name: String,
    x: i32,
    y: i32,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    backend.set_position(&device_name, x, y)
}

#[tauri::command]
fn set_scale(
    device_name: String,
    scale_percent: u32,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    backend.set_scale(&device_name, scale_percent)
}

#[tauri::command]
fn toggle_monitor_state(
    device_name: String,
    enabled: bool,
    backend: tauri::State<'_, DynDisplayBackend>,
) -> ApplyResult {
    match backend.toggle_monitor(&device_name, enabled) {
        Ok(msg) => ApplyResult::ok(msg),
        Err(e) => ApplyResult::err(e.to_string()),
    }
}

#[tauri::command]
fn save_rollback_point() {
    display::save_rollback_point();
}

#[tauri::command]
fn has_pending_confirmation() -> bool {
    display::has_pending_confirmation()
}

#[tauri::command]
fn confirm_layout() -> Result<(), String> {
    display::confirm_layout().map_err(|e| e.to_string())
}

#[tauri::command]
fn rollback_layout() -> Result<(), String> {
    display::rollback_layout().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_startup_enabled() -> bool {
    profiles::is_startup_enabled()
}

#[tauri::command]
fn set_startup(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| {
            app.path()
                .resource_dir()
                .map(|d| {
                    d.join("../display-manager.exe")
                        .to_string_lossy()
                        .into_owned()
                })
                .unwrap_or_default()
        });
    profiles::set_startup_enabled(enabled, &exe_path)
}

pub fn apply_current_user_profile() {
    let username = profiles::current_username();
    let all = profiles::load_profiles();
    let current_displays = display::enumerate_displays();

    if let Some(profile) = all.users.get(&username) {
        let remapped = remap_profile_assignments(profile, &current_displays);
        for (_key, assignment) in remapped {
            if let Some(display) = current_displays
                .iter()
                .find(|d| d.display_id.target_id == assignment.display_id.target_id)
            {
                let _ = display::apply_display_settings(
                    &display.device_name,
                    assignment.mode.width,
                    assignment.mode.height,
                    assignment.mode.refresh_rate,
                    true,
                );
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend: DynDisplayBackend = Box::new(backend::Win32Backend);

    tauri::Builder::default()
        .manage(backend)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            let main_window = app
                .get_webview_window("main")
                .expect("main window must exist");
            main_window.set_title(&format!("Display Manager v{version}"))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_displays,
            apply_settings,
            set_primary_display,
            set_orientation,
            set_position,
            set_scale,
            toggle_monitor_state,
            save_rollback_point,
            get_all_profiles,
            save_user_profile,
            delete_user_profile,
            apply_profile_for_user,
            get_current_username,
            get_startup_enabled,
            set_startup,
            has_pending_confirmation,
            confirm_layout,
            rollback_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
