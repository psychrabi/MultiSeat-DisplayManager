use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub bits_per_pixel: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayDevice {
    pub index: u32,
    pub device_name: String,
    pub device_string: String,
    pub is_primary: bool,
    pub is_active: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub current_mode: Option<DisplayMode>,
    pub available_modes: Vec<DisplayMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
}

fn wide_to_string(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    OsString::from_wide(&wide[..end])
        .to_string_lossy()
        .into_owned()
}

#[cfg(windows)]
mod win_ccd {
    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        SetDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_MODE_INFO,
        DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME, QDC_ALL_PATHS,
        QUERY_DISPLAY_CONFIG_FLAGS, SDC_ALLOW_CHANGES, SDC_APPLY, SDC_NO_OPTIMIZATION,
        SDC_SAVE_TO_DATABASE, SDC_USE_SUPPLIED_DISPLAY_CONFIG,
    };
    use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR};

    pub const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;
    pub const DISPLAYCONFIG_PATH_MODE_IDX_INVALID: u32 = 0xffffffff;

    pub struct Topology {
        pub paths: Vec<DISPLAYCONFIG_PATH_INFO>,
        pub modes: Vec<DISPLAYCONFIG_MODE_INFO>,
    }

    pub fn get_topology(flags: QUERY_DISPLAY_CONFIG_FLAGS) -> Result<Topology, String> {
        let mut path_count = 0u32;
        let mut mode_count = 0u32;

        unsafe {
            let status = GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count);
            if status != WIN32_ERROR(0) {
                return Err(format!("GetDisplayConfigBufferSizes failed: {:?}", status));
            }

            loop {
                let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
                let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

                let status = QueryDisplayConfig(
                    flags,
                    &mut path_count,
                    paths.as_mut_ptr(),
                    &mut mode_count,
                    modes.as_mut_ptr(),
                    None,
                );

                if status == ERROR_INSUFFICIENT_BUFFER {
                    let _ = GetDisplayConfigBufferSizes(flags, &mut path_count, &mut mode_count);
                    continue;
                }

                if status != WIN32_ERROR(0) {
                    return Err(format!("QueryDisplayConfig failed: {:?}", status));
                }

                paths.truncate(path_count as usize);
                modes.truncate(mode_count as usize);

                return Ok(Topology { paths, modes });
            }
        }
    }

    pub fn get_gdi_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
        let mut info = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
        info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        info.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
        info.header.adapterId = path.sourceInfo.adapterId;
        info.header.id = path.sourceInfo.id;

        if path.sourceInfo.id == DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
            return None;
        }

        let status = unsafe { DisplayConfigGetDeviceInfo(&mut info.header) };
        if status == 0 {
            let name = &info.viewGdiDeviceName;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            Some(String::from_utf16_lossy(&name[..len]))
        } else {
            None
        }
    }

    pub fn get_monitor_path(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
        use windows::Win32::Devices::Display::{
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
        };
        let mut info = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
        info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        info.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        info.header.adapterId = path.targetInfo.adapterId;
        info.header.id = path.targetInfo.id;

        let status = unsafe { DisplayConfigGetDeviceInfo(&mut info.header) };
        if status == 0 {
            let name = &info.monitorDevicePath;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            Some(String::from_utf16_lossy(&name[..len]))
        } else {
            None
        }
    }

    pub fn apply_topology(topology: &Topology) -> Result<(), String> {
        let flags =
            SDC_APPLY | SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_SAVE_TO_DATABASE | SDC_ALLOW_CHANGES;

        let result = unsafe {
            SetDisplayConfig(
                Some(&topology.paths),
                Some(&topology.modes),
                flags | SDC_NO_OPTIMIZATION,
            )
        };

        if result == 0 {
            return Ok(());
        }

        let result =
            unsafe { SetDisplayConfig(Some(&topology.paths), Some(&topology.modes), flags) };

        if result == 0 {
            Ok(())
        } else {
            Err(format!("SetDisplayConfig failed: {:?}", result))
        }
    }

    pub fn get_all_paths_flags() -> QUERY_DISPLAY_CONFIG_FLAGS {
        QDC_ALL_PATHS
    }
}

#[cfg(windows)]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_MIRRORING_DRIVER, ENUM_DISPLAY_SETTINGS_MODE,
    };

    let topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut devices = Vec::new();
    let mut adapter_index = 0u32;

    loop {
        let mut adapter = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };

        if !unsafe { EnumDisplayDevicesW(None, adapter_index, &mut adapter, 0) }.as_bool() {
            break;
        }

        let device_name_str = wide_to_string(&adapter.DeviceName);
        let device_string_str = wide_to_string(&adapter.DeviceString);
        let is_mirror = (adapter.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;

        if is_mirror {
            adapter_index += 1;
            continue;
        }

        let path = topology
            .paths
            .iter()
            .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name_str));

        let is_active = path.map_or(false, |p| {
            (p.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0
        });

        let (pos_x, pos_y, is_primary) = if let Some(p) = path {
            if is_active {
                let mode_idx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx } as usize;
                if let Some(mode) = topology.modes.get(mode_idx) {
                    let pos = unsafe { mode.Anonymous.sourceMode.position };
                    (pos.x, pos.y, pos.x == 0 && pos.y == 0)
                } else {
                    (0, 0, false)
                }
            } else {
                (0, 0, false)
            }
        } else {
            (0, 0, false)
        };

        let mut available_modes = Vec::new();
        let name_wide: Vec<u16> = device_name_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut mode_index = 0u32;
        loop {
            let mut devmode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            if !unsafe {
                EnumDisplaySettingsW(
                    windows::core::PCWSTR(name_wide.as_ptr()),
                    ENUM_DISPLAY_SETTINGS_MODE(mode_index),
                    &mut devmode,
                )
            }
            .as_bool()
            {
                break;
            }

            if devmode.dmBitsPerPel == 32 || devmode.dmBitsPerPel == 0 {
                let mode = DisplayMode {
                    width: devmode.dmPelsWidth,
                    height: devmode.dmPelsHeight,
                    refresh_rate: devmode.dmDisplayFrequency,
                    bits_per_pixel: devmode.dmBitsPerPel,
                };
                if !available_modes.iter().any(|m: &DisplayMode| {
                    m.width == mode.width
                        && m.height == mode.height
                        && m.refresh_rate == mode.refresh_rate
                }) {
                    if mode.width > 0 && mode.height > 0 {
                        available_modes.push(mode);
                    }
                }
            }
            mode_index += 1;
        }

        available_modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh_rate.cmp(&a.refresh_rate))
        });

        let current_mode = if is_active {
            path.and_then(|p| {
                let mode_idx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx } as usize;
                topology.modes.get(mode_idx).map(|m| unsafe {
                    DisplayMode {
                        width: m.Anonymous.sourceMode.width,
                        height: m.Anonymous.sourceMode.height,
                        refresh_rate: (p.targetInfo.refreshRate.Numerator as u64
                            / p.targetInfo.refreshRate.Denominator.max(1) as u64)
                            as u32,
                        bits_per_pixel: 32,
                    }
                })
            })
        } else {
            None
        };

        devices.push(DisplayDevice {
            index: adapter_index,
            device_name: device_name_str,
            device_string: device_string_str,
            is_primary,
            is_active,
            position_x: pos_x,
            position_y: pos_y,
            current_mode,
            available_modes,
        });

        adapter_index += 1;
    }

    devices
}

#[cfg(windows)]
pub fn set_primary_display(target_device_name: &str) -> ApplyResult {
    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: e,
            }
        }
    };

    let target_path_idx = match topology
        .paths
        .iter()
        .position(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == target_device_name))
    {
        Some(idx) => idx,
        None => {
            return ApplyResult {
                success: false,
                message: "Target display not found.".into(),
            }
        }
    };

    let (offset_x, offset_y) = {
        let path = &topology.paths[target_path_idx];
        if (path.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
            return ApplyResult {
                success: false,
                message: "Cannot set inactive display as primary.".into(),
            };
        }
        let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
        let mode = &topology.modes[mode_idx];
        let pos = unsafe { mode.Anonymous.sourceMode.position };
        (-pos.x, -pos.y)
    };

    for mode in &mut topology.modes {
        if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
            unsafe {
                mode.Anonymous.sourceMode.position.x += offset_x;
                mode.Anonymous.sourceMode.position.y += offset_y;
            }
        }
    }

    let target_path = topology.paths.remove(target_path_idx);
    topology.paths.insert(0, target_path);

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult {
            success: true,
            message: format!("'{}' is now primary.", target_device_name),
        },
        Err(e) => ApplyResult {
            success: false,
            message: e,
        },
    }
}

#[cfg(windows)]
pub fn toggle_monitor_state(device_name: &str, enabled: bool) -> ApplyResult {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};

    let target_monitor_id = {
        let name_wide: Vec<u16> = device_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut monitor_info = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        if unsafe { EnumDisplayDevicesW(PCWSTR(name_wide.as_ptr()), 0, &mut monitor_info, 0) }
            .as_bool()
        {
            Some(wide_to_string(&monitor_info.DeviceID))
        } else {
            None
        }
    };

    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: e,
            }
        }
    };

    let path_idx = match topology.paths.iter().position(|p| {
        if let Some(ref tid) = target_monitor_id {
            if let Some(mid) = win_ccd::get_monitor_path(p) {
                if mid == *tid {
                    return true;
                }
            }
        }
        win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name)
    }) {
        Some(idx) => idx,
        None => {
            return ApplyResult {
                success: false,
                message: "Display not found.".into(),
            }
        }
    };

    if enabled {
        topology.paths[path_idx].flags |= win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG;
        // Reset mode indices to tell Windows to find a fresh valid mode for this path
        topology.paths[path_idx].sourceInfo.Anonymous.modeInfoIdx =
            win_ccd::DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
        topology.paths[path_idx].targetInfo.Anonymous.modeInfoIdx =
            win_ccd::DISPLAYCONFIG_PATH_MODE_IDX_INVALID;
    } else {
        let active_count = topology
            .paths
            .iter()
            .filter(|p| (p.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0)
            .count();
        if active_count <= 1
            && (topology.paths[path_idx].flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0
        {
            return ApplyResult {
                success: false,
                message: "Cannot disable the last active display.".into(),
            };
        }
        topology.paths[path_idx].flags &= !win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG;
    }

    let mut primary_exists = false;
    for p in &topology.paths {
        if (p.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0 {
            let mode_idx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx };
            if mode_idx != win_ccd::DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
                if let Some(mode) = topology.modes.get(mode_idx as usize) {
                    let pos = unsafe { mode.Anonymous.sourceMode.position };
                    if pos.x == 0 && pos.y == 0 {
                        primary_exists = true;
                        break;
                    }
                }
            }
        }
    }

    if !primary_exists {
        if let Some(first_active_idx) = topology
            .paths
            .iter()
            .position(|p| (p.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0)
        {
            let ox_oy = unsafe {
                let p = &topology.paths[first_active_idx];
                let mode_idx = p.sourceInfo.Anonymous.modeInfoIdx;
                if mode_idx != win_ccd::DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
                    topology.modes.get(mode_idx as usize).map(|m| {
                        let pos = m.Anonymous.sourceMode.position;
                        (-pos.x, -pos.y)
                    })
                } else {
                    None
                }
            };

            if let Some((ox, oy)) = ox_oy {
                for mode in &mut topology.modes {
                    if mode.infoType
                        == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
                    {
                        unsafe {
                            mode.Anonymous.sourceMode.position.x += ox;
                            mode.Anonymous.sourceMode.position.y += oy;
                        }
                    }
                }
            }
        }
    }

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult {
            success: true,
            message: format!(
                "Monitor {} {}.",
                device_name,
                if enabled { "connected" } else { "disconnected" }
            ),
        },
        Err(e) => ApplyResult {
            success: false,
            message: e,
        },
    }
}

pub fn apply_display_settings(
    device_name: &str,
    width: u32,
    height: u32,
    refresh_rate: u32,
    _persist: bool,
) -> ApplyResult {
    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: e,
            }
        }
    };

    let path = match topology
        .paths
        .iter_mut()
        .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name))
    {
        Some(p) => p,
        None => {
            return ApplyResult {
                success: false,
                message: "Display not found.".into(),
            }
        }
    };

    let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
    let mode = &mut topology.modes[mode_idx];
    mode.Anonymous.sourceMode.width = width;
    mode.Anonymous.sourceMode.height = height;
    path.targetInfo.refreshRate.Numerator = refresh_rate;
    path.targetInfo.refreshRate.Denominator = 1;

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult {
            success: true,
            message: format!("Settings applied to {}.", device_name),
        },
        Err(e) => ApplyResult {
            success: false,
            message: e,
        },
    }
}

#[cfg(not(windows))]
pub fn set_primary_display(_target_device_name: &str) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported.".into(),
    }
}
#[cfg(not(windows))]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    vec![]
}
#[cfg(not(windows))]
pub fn toggle_monitor_state(_device_name: &str, _enabled: bool) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported.".into(),
    }
}
#[cfg(not(windows))]
pub fn apply_display_settings(_: &str, _: u32, _: u32, _: u32, _: bool) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported.".into(),
    }
}
