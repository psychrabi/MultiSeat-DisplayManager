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
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_MIRRORING_DRIVER, DISPLAY_DEVICE_PRIMARY_DEVICE,
        ENUM_CURRENT_SETTINGS,
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
        let is_active = (adapter.StateFlags & DISPLAY_DEVICE_ACTIVE) != 0;
        let is_primary = (adapter.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;
        let is_mirror = (adapter.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;

        // Skip mirroring drivers (virtual displays)
        if is_mirror {
            adapter_index += 1;
            continue;
        }

        let name_wide: Vec<u16> = device_name_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // Enumerate available modes first to verify this is a real display
        let mut available_modes: Vec<DisplayMode> = Vec::new();
        let mut mode_index = 0u32;
        loop {
            let mut devmode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            if !unsafe {
                EnumDisplaySettingsW(
                    windows::core::PCWSTR(name_wide.as_ptr()),
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(mode_index),
                    &mut devmode,
                )
            }
            .as_bool()
            {
                break;
            }

            let mode = DisplayMode {
                width: devmode.dmPelsWidth,
                height: devmode.dmPelsHeight,
                refresh_rate: devmode.dmDisplayFrequency,
                bits_per_pixel: devmode.dmBitsPerPel,
            };

            if mode.bits_per_pixel == 32 || mode.bits_per_pixel == 0 {
                let dup = available_modes.iter().any(|m| {
                    m.width == mode.width
                        && m.height == mode.height
                        && m.refresh_rate == mode.refresh_rate
                });
                if !dup && mode.width > 0 && mode.height > 0 && mode.refresh_rate > 0 {
                    available_modes.push(mode);
                }
            }
            mode_index += 1;
        }

        // If no modes are available, this is likely not a physical display we can manage
        if available_modes.is_empty() {
            adapter_index += 1;
            continue;
        }

        available_modes.sort_by(|a, b| {
            b.width
                .cmp(&a.width)
                .then(b.height.cmp(&a.height))
                .then(b.refresh_rate.cmp(&a.refresh_rate))
        });

        // Try getting current settings first.
        let mut cur_devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let has_real_current = unsafe {
            EnumDisplaySettingsW(
                windows::core::PCWSTR(name_wide.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut cur_devmode,
            )
        }
        .as_bool()
            && cur_devmode.dmPelsWidth > 0;

        let mut has_metadata = has_real_current;

        // Check if we got junk (0,0) for a non-primary monitor (common with ASTER hidden monitors).
        let pos = unsafe { cur_devmode.Anonymous1.Anonymous2.dmPosition };
        let is_junk_zero = !is_primary && pos.x == 0 && pos.y == 0;

        // Fallback to registry settings ONLY for metadata/position if needed.
        if !has_real_current || is_junk_zero {
            let mut reg_devmode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            if unsafe {
                EnumDisplaySettingsW(
                    windows::core::PCWSTR(name_wide.as_ptr()),
                    windows::Win32::Graphics::Gdi::ENUM_REGISTRY_SETTINGS,
                    &mut reg_devmode,
                )
            }
            .as_bool()
                && reg_devmode.dmPelsWidth > 0
            {
                cur_devmode = reg_devmode;
                has_metadata = true;
            }
        }

        let (current_mode, position_x, position_y) = if has_metadata {
            let pos = unsafe { cur_devmode.Anonymous1.Anonymous2.dmPosition };
            (
                Some(DisplayMode {
                    width: cur_devmode.dmPelsWidth,
                    height: cur_devmode.dmPelsHeight,
                    refresh_rate: cur_devmode.dmDisplayFrequency,
                    bits_per_pixel: cur_devmode.dmBitsPerPel,
                }),
                pos.x,
                pos.y,
            )
        } else {
            (None, 0, 0)
        };

        // Active state means it HAS a real current mode in GDI right now.
        let is_actually_active = has_real_current;

        devices.push(DisplayDevice {
            index: adapter_index,
            device_name: device_name_str,
            device_string: device_string_str,
            is_primary,
            is_active: is_actually_active,
            position_x,
            position_y,
            current_mode,
            available_modes,
        });

        adapter_index += 1;
    }

    devices
}

/// Set a monitor as the Windows primary display.
///
/// Algorithm:
///  1. Get current DEVMODE (position + mode) for every active monitor.
///  2. Compute the offset needed to move the target to (0,0).
///  3. Apply that offset to every OTHER monitor with CDS_UPDATEREGISTRY | CDS_NORESET.
///  4. Apply the target monitor with CDS_SET_PRIMARY | CDS_UPDATEREGISTRY | CDS_NORESET.
///  5. Commit all changes with a single ChangeDisplaySettingsExW(NULL, NULL, ..., 0, NULL).
#[cfg(windows)]
pub fn set_primary_display(target_device_name: &str) -> ApplyResult {
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_NORESET,
        CDS_SET_PRIMARY, CDS_UPDATEREGISTRY, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_MIRRORING_DRIVER, DISPLAY_DEVICE_PRIMARY_DEVICE, DISP_CHANGE_SUCCESSFUL,
        DM_BITSPERPEL, DM_DISPLAYFREQUENCY, DM_PELSHEIGHT, DM_PELSWIDTH, DM_POSITION,
        ENUM_CURRENT_SETTINGS, ENUM_REGISTRY_SETTINGS,
    };

    let mut monitors: Vec<(String, DEVMODEW)> = Vec::new();
    let mut adapter_index = 0u32;

    // 1. Collect EVERY physical monitor listed in the registry.
    // For ASTER, we MUST move all monitors (even those on other seats)
    // to prevent coordinate collisions/overlaps which cause the -1 error and resets.
    loop {
        let mut adapter = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        if !unsafe { EnumDisplayDevicesW(None, adapter_index, &mut adapter, 0) }.as_bool() {
            break;
        }

        let is_mirror = (adapter.StateFlags & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;
        if is_mirror {
            adapter_index += 1;
            continue;
        }

        let name = wide_to_string(&adapter.DeviceName);
        let name_wide_vec: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr = windows::core::PCWSTR(name_wide_vec.as_ptr());

        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        // 1. Try ENUM_CURRENT_SETTINGS first to get live positions
        let mut has_settings =
            unsafe { EnumDisplaySettingsW(pcwstr, ENUM_CURRENT_SETTINGS, &mut devmode) }.as_bool();

        // 2. Fallback to Registry if Current returns (0,0) for non-primary (ASTER mask)
        let is_primary_flag = (adapter.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;
        let pos = unsafe { devmode.Anonymous1.Anonymous2.dmPosition };
        if has_settings && !is_primary_flag && pos.x == 0 && pos.y == 0 {
            has_settings = false;
        }

        if !has_settings {
            has_settings =
                unsafe { EnumDisplaySettingsW(pcwstr, ENUM_REGISTRY_SETTINGS, &mut devmode) }
                    .as_bool();
        }

        if has_settings && devmode.dmPelsWidth > 0 && devmode.dmPelsHeight > 0 {
            if devmode.dmBitsPerPel == 0 {
                devmode.dmBitsPerPel = 32;
            }
            devmode.dmFields =
                DM_POSITION | DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY | DM_BITSPERPEL;
            monitors.push((name, devmode));
        }
        adapter_index += 1;
    }

    // 2. Find the target monitor in this global list
    let target_pos = match monitors.iter().find(|(n, _)| n == target_device_name) {
        Some((_, dm)) => unsafe { dm.Anonymous1.Anonymous2.dmPosition },
        None => {
            return ApplyResult {
                success: false,
                message: format!(
                    "Monitor '{}' not found in registry. It may be disabled.",
                    target_device_name
                ),
            }
        }
    };

    // 3. Calculate the global offset to bring target to (0,0)
    let offset_x = -target_pos.x;
    let offset_y = -target_pos.y;

    let mut last_result = DISP_CHANGE_SUCCESSFUL;

    // 4. Update the position of EVERY monitor in the registry
    for (name, devmode) in &mut monitors {
        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let is_target = name == target_device_name;

        unsafe {
            devmode.Anonymous1.Anonymous2.dmPosition.x += offset_x;
            devmode.Anonymous1.Anonymous2.dmPosition.y += offset_y;
        }

        let flags = if is_target {
            CDS_SET_PRIMARY | CDS_UPDATEREGISTRY | CDS_NORESET
        } else {
            CDS_UPDATEREGISTRY | CDS_NORESET
        };

        let result = unsafe {
            ChangeDisplaySettingsExW(
                windows::core::PCWSTR(name_wide.as_ptr()),
                Some(devmode),
                None,
                flags,
                None,
            )
        };

        if result != DISP_CHANGE_SUCCESSFUL {
            last_result = result;
        }
    }

    // 5. Commit all registry changes to live hardware in one operation
    let commit = unsafe {
        ChangeDisplaySettingsExW(
            None,
            None,
            None,
            windows::Win32::Graphics::Gdi::CDS_TYPE(0),
            None,
        )
    };

    if commit == DISP_CHANGE_SUCCESSFUL && last_result == DISP_CHANGE_SUCCESSFUL {
        ApplyResult {
            success: true,
            message: format!(
                "'{}' set as primary; entire desktop shifted.",
                target_device_name
            ),
        }
    } else {
        let code = if commit != DISP_CHANGE_SUCCESSFUL {
            commit.0
        } else {
            last_result.0
        };
        ApplyResult {
            success: false,
            message: format!(
                "Failed to update layout (code: {}). Relative positions may overlap.",
                code
            ),
        }
    }
}

#[cfg(not(windows))]
pub fn set_primary_display(_target_device_name: &str) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported on this platform".to_string(),
    }
}

#[cfg(not(windows))]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    vec![]
}

#[cfg(windows)]
pub fn apply_display_settings(
    device_name: &str,
    width: u32,
    height: u32,
    refresh_rate: u32,
    persist: bool,
) -> ApplyResult {
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsW, CDS_GLOBAL, CDS_UPDATEREGISTRY, DEVMODEW,
        DISP_CHANGE_SUCCESSFUL, DM_BITSPERPEL, DM_DISPLAYFREQUENCY, DM_PELSHEIGHT, DM_PELSWIDTH,
        DM_POSITION, ENUM_CURRENT_SETTINGS,
    };

    let name_wide: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let pcwstr = windows::core::PCWSTR(name_wide.as_ptr());

    // IMPORTANT: Fetch the current DEVMODE first.
    // Changing resolution without explicitly including DM_POSITION and its current values
    // causes Windows to reset the monitor to (0,0).
    let mut devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };

    if !unsafe { EnumDisplaySettingsW(pcwstr, ENUM_CURRENT_SETTINGS, &mut devmode) }.as_bool() {
        return ApplyResult {
            success: false,
            message: format!("Could not get current settings for {}", device_name),
        };
    }

    devmode.dmPelsWidth = width;
    devmode.dmPelsHeight = height;
    devmode.dmDisplayFrequency = refresh_rate;
    if devmode.dmBitsPerPel == 0 {
        devmode.dmBitsPerPel = 32;
    }

    // IMPORTANT: Tell Windows which fields we want to change!
    devmode.dmFields |= windows::Win32::Graphics::Gdi::DM_PELSWIDTH
        | windows::Win32::Graphics::Gdi::DM_PELSHEIGHT
        | windows::Win32::Graphics::Gdi::DM_DISPLAYFREQUENCY
        | windows::Win32::Graphics::Gdi::DM_BITSPERPEL
        | windows::Win32::Graphics::Gdi::DM_POSITION;

    let flags = if persist {
        CDS_UPDATEREGISTRY | CDS_GLOBAL
    } else {
        windows::Win32::Graphics::Gdi::CDS_TYPE(0)
    };

    // Single step apply for single monitor resolution/refresh changes.
    // This is more reliable for immediate feedback than the two-step registry/commit process.
    let res = unsafe { ChangeDisplaySettingsExW(pcwstr, Some(&mut devmode), None, flags, None) };

    if res == DISP_CHANGE_SUCCESSFUL || res == windows::Win32::Graphics::Gdi::DISP_CHANGE_NOTUPDATED
    {
        ApplyResult {
            success: true,
            message: format!(
                "Applied {}x{}@{}Hz to {}",
                width, height, refresh_rate, device_name
            ),
        }
    } else {
        let result_code = res.0;
        let reason = match result_code {
            -1 => "Display change failed (DISP_CHANGE_FAILED)",
            -2 => "Bad flags (DISP_CHANGE_BADFLAGS)",
            -3 => "Bad parameters (DISP_CHANGE_BADPARAM)",
            -4 => "Bad dual view (DISP_CHANGE_BADDUALVIEW)",
            -5 => "Bad mode (DISP_CHANGE_BADMODE)",
            -6 => "Not updated (DISP_CHANGE_NOTUPDATED)",
            -7 => "Restart required (DISP_CHANGE_RESTART)",
            _ => "Unknown error",
        };
        ApplyResult {
            success: false,
            message: format!(
                "Failed to apply settings: {} (code: {})",
                reason, result_code
            ),
        }
    }
}

#[cfg(not(windows))]
pub fn apply_display_settings(
    _device_name: &str,
    _width: u32,
    _height: u32,
    _refresh_rate: u32,
    _persist: bool,
) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported on this platform".to_string(),
    }
}

#[cfg(windows)]
pub fn toggle_monitor_state(device_name: &str, enabled: bool) -> ApplyResult {
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsW, CDS_GLOBAL, CDS_UPDATEREGISTRY, DEVMODEW,
        DISP_CHANGE_SUCCESSFUL, DM_PELSHEIGHT, DM_PELSWIDTH, DM_POSITION, ENUM_REGISTRY_SETTINGS,
    };

    let name_wide_vec: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let pcwstr = windows::core::PCWSTR(name_wide_vec.as_ptr());

    let mut devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };

    if enabled {
        // 1. Try Registry settings (user's preferred mode)
        let has_reg =
            unsafe { EnumDisplaySettingsW(pcwstr, ENUM_REGISTRY_SETTINGS, &mut devmode) }.as_bool();

        if !has_reg || devmode.dmPelsWidth == 0 {
            // 2. Fallback: Find native/preferred mode (index 0)
            if !unsafe {
                EnumDisplaySettingsW(
                    pcwstr,
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(0),
                    &mut devmode,
                )
            }
            .as_bool()
            {
                return ApplyResult {
                    success: false,
                    message: format!("Could not find any valid mode to enable {}", device_name),
                };
            }
        }
    } else {
        // SAFETY: Don't disconnect the primary!
        let mut adapter = windows::Win32::Graphics::Gdi::DISPLAY_DEVICEW {
            cb: std::mem::size_of::<windows::Win32::Graphics::Gdi::DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        let mut adapter_index = 0u32;
        let mut is_primary = false;
        loop {
            if !unsafe {
                windows::Win32::Graphics::Gdi::EnumDisplayDevicesW(
                    None,
                    adapter_index,
                    &mut adapter,
                    0,
                )
            }
            .as_bool()
            {
                break;
            }
            if wide_to_string(&adapter.DeviceName) == device_name {
                is_primary = (adapter.StateFlags
                    & windows::Win32::Graphics::Gdi::DISPLAY_DEVICE_PRIMARY_DEVICE)
                    != 0;
                break;
            }
            adapter_index += 1;
        }

        if is_primary {
            return ApplyResult {
                success: false,
                message: "Cannot disconnect the primary display.".to_string(),
            };
        }

        devmode.dmPelsWidth = 0;
        devmode.dmPelsHeight = 0;
        unsafe {
            devmode.Anonymous1.Anonymous2.dmPosition.x = 0;
            devmode.Anonymous1.Anonymous2.dmPosition.y = 0;
        }
    }

    devmode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_POSITION;

    // Use a single direct call for immediate hardware effect
    let flags = CDS_UPDATEREGISTRY | CDS_GLOBAL;
    let res = unsafe { ChangeDisplaySettingsExW(pcwstr, Some(&mut devmode), None, flags, None) };

    if res == DISP_CHANGE_SUCCESSFUL {
        ApplyResult {
            success: true,
            message: format!(
                "Monitor {} {}",
                device_name,
                if enabled { "connected" } else { "disconnected" }
            ),
        }
    } else {
        ApplyResult {
            success: false,
            message: format!("Failed to toggle monitor {} (code: {})", device_name, res.0),
        }
    }
}

#[cfg(not(windows))]
pub fn toggle_monitor_state(_device_name: &str, _enabled: bool) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "Not supported on this platform".to_string(),
    }
}
