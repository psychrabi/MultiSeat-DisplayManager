// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(missing_docs)]

fn main() {
    // If launched with --apply-profile flag, apply current user's profile and exit silently.
    if std::env::args().any(|a| a == "--apply-profile") {
        display_manager_lib::apply_current_user_profile();
        return;
    }
    display_manager_lib::run();
}
