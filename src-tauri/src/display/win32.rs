use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    SetDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
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

pub fn get_monitor_friendly_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
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
        let name = &info.monitorFriendlyDeviceName;
        let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        let friendly_name = String::from_utf16_lossy(&name[..len]);
        if !friendly_name.trim().is_empty() {
            Some(friendly_name)
        } else {
            None
        }
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

pub fn normalize_primary_and_positions(
    paths: &mut Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: &mut Vec<DISPLAYCONFIG_MODE_INFO>,
) {
    let mut primary_found = false;
    for path in paths.iter_mut() {
        if (path.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
            continue;
        }
        let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
        if mode_idx >= modes.len() {
            continue;
        }
        let mode = &modes[mode_idx];
        if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
            let pos = unsafe { mode.Anonymous.sourceMode.position };
            if pos.x == 0 && pos.y == 0 {
                primary_found = true;
                break;
            }
        }
    }

    if !primary_found {
        for path in paths.iter_mut() {
            if (path.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0 {
                let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
                if mode_idx < modes.len() {
                    let mode = &mut modes[mode_idx];
                    if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                        mode.Anonymous.sourceMode.position.x = 0;
                        mode.Anonymous.sourceMode.position.y = 0;
                        break;
                    }
                }
            }
        }
    }
}

// Internal helpers for display operations

pub(crate) fn notify_dwm_of_dpi_change() {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetDesktopWindow, GetWindow, SendNotifyMessageW, SetWindowPos, GET_WINDOW_CMD,
        HWND_BROADCAST, HWND_TOP, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
        SWP_NOZORDER, SWP_SHOWWINDOW, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_FONTCHANGE,
    };

    unsafe {
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_DPICHANGED,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_DISPLAYCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_FONTCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );

        let desktop_hwnd = GetDesktopWindow();
        let _ = SetWindowPos(
            desktop_hwnd,
            HWND_TOP,
            0, 0, 0, 0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW,
        );

        let mut hwnd = desktop_hwnd;
        loop {
            if let Ok(next) = GetWindow(hwnd, GET_WINDOW_CMD(2)) {
                if next.0.is_null() {
                    break;
                }
                hwnd = next;
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    0, 0, 0, 0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER,
                );
            } else {
                break;
            }
        }
    }
}

pub(crate) fn refresh_user_system_parameters() {
    use windows::Win32::UI::WindowsAndMessaging::{
        SendNotifyMessageW, SystemParametersInfoW, HWND_BROADCAST, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, WM_SETTINGCHANGE,
    };

    unsafe {
        const SPI_SETNONCLIENTMETRICS: u32 = 0x0029;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(
                SPI_SETNONCLIENTMETRICS,
            ),
            0,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        const SPI_SETICONTITLELOGFONT: u32 = 0x0022;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(
                SPI_SETICONTITLELOGFONT,
            ),
            0,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        let setting_name: Vec<u16> = "WindowsMetrics\0".encode_utf16().collect();
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(setting_name.as_ptr() as isize),
        );
    }
}

pub(crate) fn trigger_display_refresh() {
    use std::mem;
    use std::ptr::null_mut;
    use windows::Win32::Graphics::Gdi::{ChangeDisplaySettingsExW, DEVMODEW, DEVMODE_FIELD_FLAGS};

    unsafe {
        let mut devmode: DEVMODEW = mem::zeroed();
        devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;

        let enum_result = windows::Win32::Graphics::Gdi::EnumDisplaySettingsW(
            windows::core::PCWSTR(null_mut()),
            windows::Win32::Graphics::Gdi::ENUM_CURRENT_SETTINGS,
            &mut devmode,
        );

        if enum_result.as_bool() {
            let original_freq = devmode.dmDisplayFrequency;
            let mut alt_freq = 0;
            let mut mode_idx = 0u32;
            while alt_freq == 0 && mode_idx < 50 {
                let mut check_devmode: DEVMODEW = mem::zeroed();
                check_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                let result = windows::Win32::Graphics::Gdi::EnumDisplaySettingsW(
                    windows::core::PCWSTR(null_mut()),
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(mode_idx),
                    &mut check_devmode,
                );
                if !result.as_bool() {
                    break;
                }
                if check_devmode.dmPelsWidth == devmode.dmPelsWidth
                    && check_devmode.dmPelsHeight == devmode.dmPelsHeight
                    && check_devmode.dmDisplayFrequency != original_freq
                {
                    alt_freq = check_devmode.dmDisplayFrequency;
                }
                mode_idx += 1;
            }

            let dm_fields = DEVMODE_FIELD_FLAGS(0x00080000 | 0x00100000 | 0x00040000 | 0x00400000);

            if alt_freq != 0 {
                let mut temp_devmode: DEVMODEW = mem::zeroed();
                temp_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                temp_devmode.dmPelsWidth = devmode.dmPelsWidth;
                temp_devmode.dmPelsHeight = devmode.dmPelsHeight;
                temp_devmode.dmBitsPerPel = devmode.dmBitsPerPel;
                temp_devmode.dmDisplayFrequency = alt_freq;
                temp_devmode.dmFields = dm_fields;
                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&temp_devmode),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );
                std::thread::sleep(std::time::Duration::from_millis(50));

                let mut original_devmode: DEVMODEW = mem::zeroed();
                original_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                original_devmode.dmPelsWidth = devmode.dmPelsWidth;
                original_devmode.dmPelsHeight = devmode.dmPelsHeight;
                original_devmode.dmBitsPerPel = devmode.dmBitsPerPel;
                original_devmode.dmDisplayFrequency = original_freq;
                original_devmode.dmFields = dm_fields;
                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&original_devmode),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );
            } else {
                let mut current_devmode: DEVMODEW = mem::zeroed();
                current_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                current_devmode.dmPelsWidth = devmode.dmPelsWidth;
                current_devmode.dmPelsHeight = devmode.dmPelsHeight;
                current_devmode.dmBitsPerPel = devmode.dmBitsPerPel;
                current_devmode.dmDisplayFrequency = devmode.dmDisplayFrequency;
                current_devmode.dmFields = dm_fields;
                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&current_devmode),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );
            }
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
}
