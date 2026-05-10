use crate::display::cache;
use crate::display::types::ManagerError;
use crate::display::win32;

#[cfg(windows)]
pub fn has_pending_confirmation() -> bool {
    if let Ok(guard) = cache::PENDING_CONFIRMATION.lock() {
        guard.is_some()
    } else {
        false
    }
}

#[cfg(windows)]
pub fn confirm_layout() -> Result<(), ManagerError> {
    let mut guard = cache::PENDING_CONFIRMATION
        .lock()
        .map_err(|_| ManagerError::Backend("lock poisoned".into()))?;
    *guard = None;
    Ok(())
}

#[cfg(windows)]
pub fn rollback_layout() -> Result<(), ManagerError> {
    use windows::Win32::Devices::Display::{
        SetDisplayConfig, SDC_ALLOW_CHANGES, SDC_APPLY, SDC_NO_OPTIMIZATION, SDC_SAVE_TO_DATABASE,
        SDC_USE_SUPPLIED_DISPLAY_CONFIG,
    };

    let saved = {
        let mut guard = cache::PENDING_CONFIRMATION
            .lock()
            .map_err(|_| ManagerError::Backend("lock poisoned".into()))?;
        guard.take()
    };
    let saved = saved.ok_or(ManagerError::NoPendingConfirmation)?;

    let flags = SDC_APPLY | SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_SAVE_TO_DATABASE | SDC_ALLOW_CHANGES;
    unsafe {
        let status = SetDisplayConfig(Some(&saved.paths), Some(&saved.modes), flags | SDC_NO_OPTIMIZATION);
        if status == 0 {
            return Ok(());
        }
        let status = SetDisplayConfig(Some(&saved.paths), Some(&saved.modes), flags);
        if status == 0 {
            Ok(())
        } else {
            Err(ManagerError::Backend(format!("rollback failed: {status}")))
        }
    }
}

#[cfg(windows)]
pub fn save_rollback_point() {
    if let Ok(topology) = win32::get_topology(win32::get_all_paths_flags()) {
        if let Ok(mut guard) = cache::PENDING_CONFIRMATION.lock() {
            *guard = Some(cache::CachedTopology {
                paths: topology.paths,
                modes: topology.modes,
            });
        }
    }
}

#[cfg(not(windows))]
pub fn save_rollback_point() {}
