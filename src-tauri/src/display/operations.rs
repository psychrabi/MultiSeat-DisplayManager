use crate::display::cache::{self, CachedTopology};
use crate::display::types::*;
use crate::display::win32;

#[cfg(windows)]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER,
        ENUM_DISPLAY_SETTINGS_MODE,
    };

    let topology = match win32::get_topology(win32::get_all_paths_flags()) {
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
            .find(|p| win32::get_gdi_name(p).map_or(false, |n| n == device_name_str));

        if path.is_none() && (adapter.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) == 0 {
            adapter_index += 1;
            continue;
        }

        let monitor_name = if let Some(p) = path {
            win32::get_monitor_friendly_name(p).unwrap_or_else(|| device_string_str.clone())
        } else {
            device_string_str.clone()
        };

        let is_active = path.map_or(false, |p| {
            (p.flags & win32::DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0
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

        let _current_mode = if is_active {
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
            path.and_then(|p| {
                let mode_idx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx } as usize;
                topology.modes.get(mode_idx).map(|m| unsafe {
                    DisplayMode {
                        width: m.Anonymous.sourceMode.width,
                        height: m.Anonymous.sourceMode.height,
                        refresh_rate: 60,
                        bits_per_pixel: 32,
                    }
                })
            })
        };

        let mut current_mode = _current_mode;

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
                }) && mode.width > 0
                    && mode.height > 0
                {
                    available_modes.push(mode);
                }
            }
            mode_index += 1;
        }

        if available_modes.is_empty() {
            if let Some(p) = path {
                let path_adapter_id = p.sourceInfo.adapterId;
                let path_id = p.sourceInfo.id;
                for mode in &topology.modes {
                    if mode.infoType.0 == 1
                        && mode.adapterId == path_adapter_id
                        && mode.id == path_id
                    {
                        let m = unsafe { mode.Anonymous.sourceMode };
                        let mode_obj = DisplayMode {
                            width: m.width,
                            height: m.height,
                            refresh_rate: 60,
                            bits_per_pixel: 32,
                        };
                        if !available_modes.contains(&mode_obj) {
                            available_modes.push(mode_obj);
                        }
                    }
                }
            }
        }

        available_modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh_rate.cmp(&a.refresh_rate))
        });

        if current_mode.is_none() && !available_modes.is_empty() {
            current_mode = Some(available_modes[0].clone());
        }

        let orientation = if let Some(p) = path {
            match p.targetInfo.rotation.0 {
                1 => DisplayOrientation::Landscape,
                2 => DisplayOrientation::Portrait,
                3 => DisplayOrientation::LandscapeFlipped,
                4 => DisplayOrientation::PortraitFlipped,
                _ => DisplayOrientation::Landscape,
            }
        } else {
            DisplayOrientation::Landscape
        };

        let display_id = match path {
            Some(p) => DisplayId::from_path(p),
            None => DisplayId {
                adapter_luid: 0,
                target_id: adapter_index,
                edid_hash: None,
            },
        };

        if display_id.edid_hash.is_none()
            || display_id.edid_hash.as_ref().map_or(true, |h| h.is_empty())
        {
            adapter_index += 1;
            continue;
        }

        devices.push(DisplayDevice {
            index: adapter_index,
            device_name: device_name_str,
            device_string: monitor_name,
            adapter_name: device_string_str,
            display_id,
            is_primary,
            is_active,
            not_detected: false,
            position_x: pos_x,
            position_y: pos_y,
            current_mode,
            available_modes,
            orientation,
            scale_factor: 100,
        });

        adapter_index += 1;
    }

    cache::with_known_monitors(|known| {
        let mut to_add = Vec::new();

        for km in known.iter() {
            if !devices.iter().any(|d| d.device_name == km.device_name) {
                to_add.push(DisplayDevice {
                    index: devices.len() as u32 + to_add.len() as u32,
                    device_name: km.device_name.clone(),
                    device_string: km.device_string.clone(),
                    adapter_name: String::new(),
                    display_id: DisplayId {
                        adapter_luid: km.adapter_luid,
                        target_id: km.target_id,
                        edid_hash: km.edid_hash.clone(),
                    },
                    is_primary: false,
                    is_active: false,
                    not_detected: true,
                    position_x: km.position_x,
                    position_y: km.position_y,
                    current_mode: None,
                    available_modes: Vec::new(),
                    orientation: DisplayOrientation::Landscape,
                    scale_factor: 100,
                });
            }
        }

        let known_count = to_add.len();
        devices.extend(to_add);

        let mut new_known: Vec<cache::KnownMonitor> = Vec::new();
        for d in &devices {
            if !known.iter().any(|k| k.device_name == d.device_name) {
                new_known.push(cache::KnownMonitor {
                    device_name: d.device_name.clone(),
                    device_string: d.device_string.clone(),
                    adapter_luid: d.display_id.adapter_luid,
                    target_id: d.display_id.target_id,
                    edid_hash: d.display_id.edid_hash.clone(),
                    position_x: d.position_x,
                    position_y: d.position_y,
                });
            }
        }

        if known_count > 0 || !new_known.is_empty() {
            known.extend(new_known);
        }
    });

    devices
}

#[cfg(windows)]
pub fn set_display_orientation(device_name: &str, orientation: DisplayOrientation) -> ApplyResult {
    use windows::Win32::Devices::Display::{
        DISPLAYCONFIG_ROTATION_ROTATE180, DISPLAYCONFIG_ROTATION_ROTATE270,
        DISPLAYCONFIG_ROTATION_ROTATE90,
    };

    let mut topology = match win32::get_topology(win32::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let path = match topology
        .paths
        .iter_mut()
        .find(|p| win32::get_gdi_name(p).map_or(false, |n| n == device_name))
    {
        Some(p) => p,
        None => return ApplyResult::err("Display not found."),
    };

    path.targetInfo.rotation = match orientation {
        DisplayOrientation::Landscape => {
            windows::Win32::Devices::Display::DISPLAYCONFIG_ROTATION(0)
        }
        DisplayOrientation::Portrait => DISPLAYCONFIG_ROTATION_ROTATE90,
        DisplayOrientation::LandscapeFlipped => DISPLAYCONFIG_ROTATION_ROTATE180,
        DisplayOrientation::PortraitFlipped => DISPLAYCONFIG_ROTATION_ROTATE270,
    };

    match win32::apply_topology(&topology) {
        Ok(_) => ApplyResult::ok(format!(
            "Orientation set to {:?} for {}.",
            orientation, device_name
        )),
        Err(e) => ApplyResult::err(e),
    }
}

#[cfg(windows)]
pub fn set_display_position(device_name: &str, x: i32, y: i32) -> ApplyResult {
    let mut topology = match win32::get_topology(win32::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let path = match topology
        .paths
        .iter_mut()
        .find(|p| win32::get_gdi_name(p).map_or(false, |n| n == device_name))
    {
        Some(p) => p,
        None => return ApplyResult::err("Display not found."),
    };

    let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
    if mode_idx >= topology.modes.len() {
        return ApplyResult::err("Invalid mode index.");
    }

    let mode = &mut topology.modes[mode_idx];
    mode.Anonymous.sourceMode.position.x = x;
    mode.Anonymous.sourceMode.position.y = y;

    match win32::apply_topology(&topology) {
        Ok(_) => ApplyResult::ok(format!(
            "Position set to ({}, {}) for {}.",
            x, y, device_name
        )),
        Err(e) => ApplyResult::err(e),
    }
}

#[cfg(windows)]
pub fn set_display_scale(device_name: &str, scale_percent: u32) -> ApplyResult {
    use windows::Win32::UI::WindowsAndMessaging::{
        SendNotifyMessageW, SystemParametersInfoW, HWND_BROADCAST, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, WM_SETTINGCHANGE,
    };
    use winreg::enums::*;
    use winreg::RegKey;

    let (dpi_value, mut log_pixels) = match scale_percent {
        100 => (0u32, 96u32),
        125 => (1, 120),
        150 => (2, 144),
        175 => (3, 168),
        200 => (4, 192),
        250 => (5, 240),
        300 => (6, 288),
        400 => (7, 384),
        500 => (8, 480),
        _ => {
            let calculated_dpi = (96u32 * scale_percent) / 100;
            (2, calculated_dpi)
        }
    };

    let topology = match win32::get_topology(win32::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(format!("Failed to get display topology: {}", e)),
    };

    let path = match topology
        .paths
        .iter()
        .find(|p| win32::get_gdi_name(p).map_or(false, |n| n == device_name))
    {
        Some(p) => p,
        None => return ApplyResult::err("Display not found in topology."),
    };

    let adapter_luid = ((path.targetInfo.adapterId.HighPart as u64) << 32)
        | (path.targetInfo.adapterId.LowPart as u64);
    let target_id = path.targetInfo.id;

    let monitor_key = if let Some(device_path) = win32::get_monitor_path(path) {
        if let Some(start) = device_path.find("DISPLAY#") {
            let after_display = &device_path[start + 8..];
            if let Some(end) = after_display.find('#') {
                after_display[..end].replace('#', "_")
            } else {
                format!("{}_{}", adapter_luid, target_id)
            }
        } else {
            format!("{}_{}", adapter_luid, target_id)
        }
    } else {
        format!("{}_{}", adapter_luid, target_id)
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let dpi_key_path = format!(
        "Control Panel\\Desktop\\PerMonitorSettings\\{}",
        monitor_key
    );

    if let Ok((dpi_key, _)) = hkcu.create_subkey(&dpi_key_path) {
        let dpi_value_str = match dpi_value {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            _ => "2",
        };
        let _ = dpi_key.set_value("DpiValue", &dpi_value_str);
        let _ = dpi_key.set_value("LogPixels", &log_pixels);
    }

    if let Ok(desktop_key) = hkcu.open_subkey_with_flags("Control Panel\\Desktop", KEY_SET_VALUE) {
        let _ = desktop_key.set_value("LogPixels", &log_pixels);
        let _ = desktop_key.set_value("Win8DpiScaling", &1u32);
    }

    if let Ok(metrics_key) =
        hkcu.open_subkey_with_flags("Control Panel\\Desktop\\WindowMetrics", KEY_SET_VALUE)
    {
        let _ = metrics_key.set_value("AppliedDPI", &log_pixels);
    }

    unsafe {
        let setting_name: Vec<u16> = "Environment\0".encode_utf16().collect();
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(setting_name.as_ptr() as isize),
        );

        const SPI_SETLOGPIXELS: u32 = 0x009F;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(
                SPI_SETLOGPIXELS,
            ),
            log_pixels,
            Some(&mut log_pixels as *mut u32 as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
    }

    win32::notify_dwm_of_dpi_change();
    win32::refresh_user_system_parameters();
    win32::trigger_display_refresh();

    ApplyResult::ok(format!(
        "Scale set to {}% for {}.",
        scale_percent, device_name
    ))
}

#[cfg(windows)]
pub fn set_primary_display(target_device_name: &str) -> ApplyResult {
    let mut topology = match win32::get_topology(win32::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let target_path_idx = match topology
        .paths
        .iter()
        .position(|p| win32::get_gdi_name(p).map_or(false, |n| n == target_device_name))
    {
        Some(idx) => idx,
        None => return ApplyResult::err("Target display not found."),
    };

    let target_path = &topology.paths[target_path_idx];
    if (target_path.flags & win32::DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
        return ApplyResult::err("Cannot set inactive display as primary.");
    }

    let mode_idx = unsafe { target_path.sourceInfo.Anonymous.modeInfoIdx } as usize;
    let target_pos = if mode_idx < topology.modes.len() {
        let mode = &topology.modes[mode_idx];
        if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
            unsafe { mode.Anonymous.sourceMode.position }
        } else {
            windows::Win32::Foundation::POINTL { x: 0, y: 0 }
        }
    } else {
        windows::Win32::Foundation::POINTL { x: 0, y: 0 }
    };

    let offset_x = -target_pos.x;
    let offset_y = -target_pos.y;

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
    win32::normalize_primary_and_positions(&mut topology.paths, &mut topology.modes);

    match win32::apply_topology(&topology) {
        Ok(_) => ApplyResult::ok(format!("'{}' is now primary.", target_device_name)),
        Err(e) => ApplyResult::err(e),
    }
}

#[cfg(windows)]
pub fn toggle_monitor_state(device_name: &str, enabled: bool) -> ApplyResult {
    use windows::Win32::Devices::Display::{
        GetDisplayConfigBufferSizes, QueryDisplayConfig, SetDisplayConfig, QDC_ALL_PATHS,
        SDC_ALLOW_CHANGES, SDC_APPLY, SDC_SAVE_TO_DATABASE, SDC_USE_SUPPLIED_DISPLAY_CONFIG,
    };

    const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;

    if enabled {
        fn query_current_topology() -> Result<
            (
                Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO>,
                Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO>,
            ),
            String,
        > {
            let mut path_count = 0u32;
            let mut mode_count = 0u32;
            unsafe {
                if GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count)
                    != windows::Win32::Foundation::WIN32_ERROR(0)
                {
                    return Err("Failed to query display config buffer sizes.".into());
                }
            }
            let mut paths = vec![Default::default(); path_count as usize];
            let mut modes = vec![Default::default(); mode_count as usize];
            unsafe {
                let status = QueryDisplayConfig(
                    QDC_ALL_PATHS,
                    &mut path_count,
                    paths.as_mut_ptr(),
                    &mut mode_count,
                    modes.as_mut_ptr(),
                    None,
                );
                if status != windows::Win32::Foundation::WIN32_ERROR(0) {
                    return Err("Failed to query display config.".into());
                }
            }
            paths.truncate(path_count as usize);
            modes.truncate(mode_count as usize);
            Ok((paths, modes))
        }

        let (mut paths, mut modes) = {
            let cache_guard = cache::TOPOLOGY_CACHE.lock();
            match cache_guard {
                Ok(ref cache) if cache.is_some() => {
                    let cached = cache.as_ref().unwrap();
                    (cached.paths.clone(), cached.modes.clone())
                }
                _ => match query_current_topology() {
                    Ok(result) => result,
                    Err(e) => return ApplyResult::err(e),
                },
            }
        };

        // Save pre-enable state for confirmation/rollback
        if let Ok(mut guard) = cache::PENDING_CONFIRMATION.lock() {
            *guard = Some(CachedTopology {
                paths: paths.clone(),
                modes: modes.clone(),
            });
        }

        let ref_resolution = paths.iter()
            .filter(|p| (p.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0)
            .filter_map(|p| {
                let midx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx } as usize;
                modes.get(midx).and_then(|m| {
                    if m.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                        Some(unsafe { (m.Anonymous.sourceMode.width, m.Anonymous.sourceMode.height) })
                    } else {
                        None
                    }
                })
            })
            .next()
            .unwrap_or((1920, 1080));

        let found = paths.iter_mut().any(|path| {
            let gdi_name = win32::get_gdi_name(path);
            if gdi_name.as_ref().map_or(false, |n| n == device_name) {
                path.flags |= DISPLAYCONFIG_PATH_ACTIVE_FLAG;
                let source_mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx };
                if source_mode_idx == 0xFFFFFFFF || (source_mode_idx as usize) >= modes.len() {
                    let path_adapter_id = path.sourceInfo.adapterId;
                    let path_id = path.sourceInfo.id;
                    let mut assigned = false;
                    for (midx, mode) in modes.iter().enumerate() {
                        if mode.infoType.0 == 1
                            && mode.adapterId == path_adapter_id
                            && mode.id == path_id
                        {
                            path.sourceInfo.Anonymous.modeInfoIdx = midx as u32;
                            assigned = true;
                            break;
                        }
                    }
                    if !assigned {
                        if let Ok(saved) = cache::DISCONNECTED_TOPOLOGY.lock() {
                            if let Some(ref saved_topology) = *saved {
                                if let Some(saved_path) = saved_topology.paths.iter().find(|sp| {
                                    sp.sourceInfo.adapterId.LowPart == path_adapter_id.LowPart
                                        && sp.sourceInfo.adapterId.HighPart == path_adapter_id.HighPart
                                        && sp.sourceInfo.id == path_id
                                }) {
                                    let saved_midx = unsafe { saved_path.sourceInfo.Anonymous.modeInfoIdx } as usize;
                                    if saved_midx < saved_topology.modes.len() {
                                        let saved_mode = &saved_topology.modes[saved_midx];
                                        if saved_mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                                            modes.push(*saved_mode);
                                            path.sourceInfo.Anonymous.modeInfoIdx = (modes.len() - 1) as u32;
                                            assigned = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !assigned {
                        let mut new_mode = windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO::default();
                        new_mode.infoType = windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE;
                        new_mode.adapterId = path_adapter_id;
                        new_mode.id = path_id;
                        new_mode.Anonymous.sourceMode.width = ref_resolution.0;
                        new_mode.Anonymous.sourceMode.height = ref_resolution.1;
                        new_mode.Anonymous.sourceMode.pixelFormat = windows::Win32::Devices::Display::DISPLAYCONFIG_PIXELFORMAT_32BPP;
                        new_mode.Anonymous.sourceMode.position = windows::Win32::Foundation::POINTL { x: 0, y: 0 };
                        modes.push(new_mode);
                        path.sourceInfo.Anonymous.modeInfoIdx = (modes.len() - 1) as u32;
                    }
                }
                true
            } else {
                false
            }
        });

        if !found {
            if let Ok(mut cache) = cache::TOPOLOGY_CACHE.lock() {
                *cache = None;
            }
            if let Ok(mut guard) = cache::PENDING_CONFIRMATION.lock() {
                *guard = None;
            }
            return ApplyResult::err("Display not found in display topology.");
        }

        win32::normalize_primary_and_positions(&mut paths, &mut modes);

        unsafe {
            let status = SetDisplayConfig(
                Some(&paths),
                Some(&modes),
                SDC_APPLY
                    | SDC_USE_SUPPLIED_DISPLAY_CONFIG
                    | SDC_SAVE_TO_DATABASE
                    | SDC_ALLOW_CHANGES,
            );
            if status == 0 {
                if let Ok(mut cache) = cache::TOPOLOGY_CACHE.lock() {
                    *cache = None;
                }
                ApplyResult::ok(format!("Monitor {} connected.", device_name))
            } else {
                ApplyResult::err(format!("Failed to connect monitor: {}", status))
            }
        }
    } else {
        let mut path_count = 0u32;
        let mut mode_count = 0u32;

        unsafe {
            let status =
                GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count);
            if status.0 != 0 {
                return ApplyResult::err(format!("Failed to get display config: {}", status.0));
            }
        }

        let mut paths = vec![
            windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO::default();
            path_count as usize
        ];
        let mut modes = vec![
            windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO::default();
            mode_count as usize
        ];

        unsafe {
            let status = QueryDisplayConfig(
                QDC_ALL_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            );
            if status.0 != 0 {
                return ApplyResult::err(format!("Failed to query display config: {}", status.0));
            }
        }

        paths.truncate(path_count as usize);
        modes.truncate(mode_count as usize);

        // Save pre-disconnect topology for later reconnection
        if let Ok(mut saved) = cache::DISCONNECTED_TOPOLOGY.lock() {
            *saved = Some(CachedTopology {
                paths: paths.clone(),
                modes: modes.clone(),
            });
        }

        // Save pre-disable state for confirmation/rollback
        if let Ok(mut guard) = cache::PENDING_CONFIRMATION.lock() {
            *guard = Some(CachedTopology {
                paths: paths.clone(),
                modes: modes.clone(),
            });
        }

        if let Ok(mut cache) = cache::TOPOLOGY_CACHE.lock() {
            *cache = Some(CachedTopology {
                paths: paths.clone(),
                modes: modes.clone(),
            });
        }

        let total_active_count = paths
            .iter()
            .filter(|p| (p.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0)
            .count();

        let found = paths.iter_mut().any(|path| {
            let gdi_name = win32::get_gdi_name(path);
            if gdi_name.as_ref().map_or(false, |n| n == device_name) {
                if total_active_count <= 1 {
                    return true;
                }
                path.flags &= !DISPLAYCONFIG_PATH_ACTIVE_FLAG;
                true
            } else {
                false
            }
        });

        if !found {
            return ApplyResult::err("Display not found.");
        }

        if total_active_count <= 1 {
            return ApplyResult::err("Cannot disable the last active display.");
        }

        win32::normalize_primary_and_positions(&mut paths, &mut modes);

        unsafe {
            let status = SetDisplayConfig(
                Some(&paths),
                Some(&modes),
                SDC_APPLY
                    | SDC_USE_SUPPLIED_DISPLAY_CONFIG
                    | SDC_SAVE_TO_DATABASE
                    | SDC_ALLOW_CHANGES,
            );
            if status == 0 {
                ApplyResult::ok(format!("Monitor {} disconnected.", device_name))
            } else {
                ApplyResult::err(format!("Failed to disconnect monitor: {}", status))
            }
        }
    }
}

#[cfg(windows)]
pub fn apply_display_settings(
    device_name: &str,
    width: u32,
    height: u32,
    refresh_rate: u32,
    _persist: bool,
) -> ApplyResult {
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsW, CDS_TEST, CDS_UPDATEREGISTRY, DEVMODEW,
        ENUM_DISPLAY_SETTINGS_MODE,
    };

    let gdi_name_wide: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut found_devmode: Option<DEVMODEW> = None;
    let mut mode_index = 0u32;

    let mut current_devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    let _ = unsafe {
        EnumDisplaySettingsW(
            windows::core::PCWSTR(gdi_name_wide.as_ptr()),
            ENUM_DISPLAY_SETTINGS_MODE(0),
            &mut current_devmode,
        )
    };
    let current_bpp = if current_devmode.dmBitsPerPel > 0 {
        current_devmode.dmBitsPerPel
    } else {
        32
    };

    loop {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let result = unsafe {
            EnumDisplaySettingsW(
                windows::core::PCWSTR(gdi_name_wide.as_ptr()),
                ENUM_DISPLAY_SETTINGS_MODE(mode_index),
                &mut devmode,
            )
        };
        if !result.as_bool() {
            break;
        }

        if devmode.dmPelsWidth == width
            && devmode.dmPelsHeight == height
            && devmode.dmDisplayFrequency == refresh_rate
            && (devmode.dmBitsPerPel == 0 || devmode.dmBitsPerPel == current_bpp)
        {
            found_devmode = Some(devmode);
            break;
        }
        mode_index += 1;
        if mode_index > 300 {
            break;
        }
    }

    let devmode = if let Some(dm) = found_devmode {
        dm
    } else {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let result = unsafe {
            EnumDisplaySettingsW(
                windows::core::PCWSTR(gdi_name_wide.as_ptr()),
                ENUM_DISPLAY_SETTINGS_MODE(0),
                &mut devmode,
            )
        };
        if !result.as_bool() {
            return ApplyResult::err("Failed to get display settings.");
        }

        devmode.dmPelsWidth = width;
        devmode.dmPelsHeight = height;
        devmode.dmDisplayFrequency = refresh_rate;
        devmode.dmBitsPerPel = current_bpp;
        devmode.dmFields =
            windows::Win32::Graphics::Gdi::DEVMODE_FIELD_FLAGS(0x40000 | 0x80000 | 0x400000 | 0x40);
        devmode
    };

    let test_result = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(gdi_name_wide.as_ptr()),
            Some(&devmode),
            None,
            CDS_TEST,
            None,
        )
    };

    if test_result.0 != 0 {
        return ApplyResult::err(match test_result.0 {
            -1 => "Display not supported".into(),
            -2 => format!(
                "Bad mode - {}x{} @ {}Hz is not supported by this display",
                devmode.dmPelsWidth, devmode.dmPelsHeight, devmode.dmDisplayFrequency
            ),
            -3 => "Bad flags".into(),
            -4 => "Bad registry".into(),
            code => format!("Unknown error code {}", code),
        });
    }

    let apply_result = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(gdi_name_wide.as_ptr()),
            Some(&devmode),
            None,
            CDS_UPDATEREGISTRY,
            None,
        )
    };

    match apply_result.0 {
        0 => ApplyResult::ok(format!("Settings applied to {}.", device_name)),
        1 => ApplyResult::ok(format!(
            "Settings applied to {} (restart required).",
            device_name
        )),
        -1 => ApplyResult::err("Display not supported."),
        -2 => ApplyResult::err("Bad mode."),
        -3 => ApplyResult::err("Bad flags."),
        -4 => ApplyResult::err("Bad registry."),
        code => ApplyResult::err(format!("Failed with code: {}", code)),
    }
}

#[cfg(not(windows))]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    vec![]
}
#[cfg(not(windows))]
pub fn set_display_orientation(_device_name: &str, _orientation: DisplayOrientation) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn set_display_position(_device_name: &str, _x: i32, _y: i32) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn set_display_scale(_device_name: &str, _scale_percent: u32) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn set_primary_display(_target_device_name: &str) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn toggle_monitor_state(_device_name: &str, _enabled: bool) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn apply_display_settings(_: &str, _: u32, _: u32, _: u32, _: bool) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
