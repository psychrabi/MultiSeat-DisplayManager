use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DisplayId {
    pub adapter_luid: u64,
    pub target_id: u32,
    pub edid_hash: Option<String>,
}

impl DisplayId {
    #[cfg(windows)]
    pub fn from_path(path: &windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO) -> Self {
        let adapter_luid = ((path.targetInfo.adapterId.HighPart as u64) << 32)
            | (path.targetInfo.adapterId.LowPart as u64);
        let target_id = path.targetInfo.id;
        let edid_hash = get_edid_hash(path);
        Self {
            adapter_luid,
            target_id,
            edid_hash,
        }
    }

    pub fn matches_path_by_assignment(&self, other: &DisplayId) -> bool {
        if let (Some(ref my_edid), Some(ref other_edid)) = (&self.edid_hash, &other.edid_hash) {
            return my_edid == other_edid;
        }
        self.target_id == other.target_id
    }
}

#[cfg(windows)]
fn get_edid_hash(
    path: &windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO,
) -> Option<String> {
    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        DISPLAYCONFIG_TARGET_DEVICE_NAME,
    };

    let mut info = DISPLAYCONFIG_TARGET_DEVICE_NAME::default();
    info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
    info.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
    info.header.adapterId = path.targetInfo.adapterId;
    info.header.id = path.targetInfo.id;

    let status = unsafe { DisplayConfigGetDeviceInfo(&mut info.header) };
    if status == 0 {
        let device_path = &info.monitorDevicePath;
        let len = device_path
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(device_path.len());
        Some(String::from_utf16_lossy(&device_path[..len]))
    } else {
        None
    }
}

#[cfg(windows)]
fn normalize_primary_and_positions(
    paths: &mut Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO>,
    modes: &mut Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO>,
) {
    const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;

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
        if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
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
                    if mode.infoType
                        == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
                    {
                        mode.Anonymous.sourceMode.position.x = 0;
                        mode.Anonymous.sourceMode.position.y = 0;
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub bits_per_pixel: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayOrientation {
    Landscape,
    Portrait,
    LandscapeFlipped,
    PortraitFlipped,
}

impl Default for DisplayOrientation {
    fn default() -> Self {
        DisplayOrientation::Landscape
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayDevice {
    pub index: u32,
    pub device_name: String,
    pub device_string: String,
    pub adapter_name: String,
    pub display_id: DisplayId,
    pub is_primary: bool,
    pub is_active: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub current_mode: Option<DisplayMode>,
    pub available_modes: Vec<DisplayMode>,
    pub orientation: DisplayOrientation,
    pub scale_factor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
}

impl ApplyResult {
    pub fn ok(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: msg.into(),
        }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: msg.into(),
        }
    }
}

#[cfg(windows)]
static TOPOLOGY_CACHE: Mutex<Option<CachedTopology>> = Mutex::new(None);

#[cfg(windows)]
#[derive(Clone)]
struct CachedTopology {
    paths: Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO>,
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
}

#[cfg(windows)]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER,
        ENUM_DISPLAY_SETTINGS_MODE,
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

        if path.is_none() && (adapter.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) == 0 {
            adapter_index += 1;
            continue;
        }

        let monitor_name = if let Some(p) = path {
            win_ccd::get_monitor_friendly_name(p).unwrap_or_else(|| device_string_str.clone())
        } else {
            device_string_str.clone()
        };

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

        let mut current_mode = if is_active {
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
            position_x: pos_x,
            position_y: pos_y,
            current_mode,
            available_modes,
            orientation,
            scale_factor: 100,
        });

        adapter_index += 1;
    }

    devices
}

#[cfg(windows)]
pub fn set_display_orientation(device_name: &str, orientation: DisplayOrientation) -> ApplyResult {
    use windows::Win32::Devices::Display::{
        DISPLAYCONFIG_ROTATION_ROTATE180, DISPLAYCONFIG_ROTATION_ROTATE270,
        DISPLAYCONFIG_ROTATION_ROTATE90,
    };

    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let path = match topology
        .paths
        .iter_mut()
        .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name))
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

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult::ok(format!(
            "Orientation set to {:?} for {}.",
            orientation, device_name
        )),
        Err(e) => ApplyResult::err(e),
    }
}

#[cfg(windows)]
pub fn set_display_position(device_name: &str, x: i32, y: i32) -> ApplyResult {
    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let path = match topology
        .paths
        .iter_mut()
        .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name))
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

    match win_ccd::apply_topology(&topology) {
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

    let topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(format!("Failed to get display topology: {}", e)),
    };

    let path = match topology
        .paths
        .iter()
        .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name))
    {
        Some(p) => p,
        None => return ApplyResult::err("Display not found in topology."),
    };

    let adapter_luid = ((path.targetInfo.adapterId.HighPart as u64) << 32)
        | (path.targetInfo.adapterId.LowPart as u64);
    let target_id = path.targetInfo.id;

    let monitor_key = if let Some(device_path) = win_ccd::get_monitor_path(path) {
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

    notify_dwm_of_dpi_change();
    refresh_user_system_parameters();
    trigger_display_refresh();

    ApplyResult::ok(format!(
        "Scale set to {}% for {}.",
        scale_percent, device_name
    ))
}

#[cfg(windows)]
fn notify_dwm_of_dpi_change() {
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
            0,
            0,
            0,
            0,
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
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER,
                );
            } else {
                break;
            }
        }
    }
}

#[cfg(windows)]
fn refresh_user_system_parameters() {
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

#[cfg(windows)]
fn trigger_display_refresh() {
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

#[cfg(windows)]
pub fn set_primary_display(target_device_name: &str) -> ApplyResult {
    let mut topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => return ApplyResult::err(e),
    };

    let target_path_idx = match topology
        .paths
        .iter()
        .position(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == target_device_name))
    {
        Some(idx) => idx,
        None => return ApplyResult::err("Target display not found."),
    };

    let target_path = &topology.paths[target_path_idx];
    if (target_path.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
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
    normalize_primary_and_positions(&mut topology.paths, &mut topology.modes);

    match win_ccd::apply_topology(&topology) {
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
        let (mut paths, mut modes) = {
            let cache_guard = TOPOLOGY_CACHE.lock();
            match cache_guard {
                Ok(ref cache) if cache.is_some() => {
                    let cached = cache.as_ref().unwrap();
                    (cached.paths.clone(), cached.modes.clone())
                }
                _ => {
                    return ApplyResult::err(
                        "No cached topology. Disconnect first, then reconnect.",
                    )
                }
            }
        };

        let found = paths.iter_mut().any(|path| {
            let gdi_name = win_ccd::get_gdi_name(path);
            if gdi_name.as_ref().map_or(false, |n| n == device_name) {
                path.flags |= DISPLAYCONFIG_PATH_ACTIVE_FLAG;
                let source_mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx };
                if source_mode_idx == 0xFFFFFFFF || (source_mode_idx as usize) >= modes.len() {
                    let path_adapter_id = path.sourceInfo.adapterId;
                    let path_id = path.sourceInfo.id;
                    for (midx, mode) in modes.iter().enumerate() {
                        if mode.infoType.0 == 1
                            && mode.adapterId == path_adapter_id
                            && mode.id == path_id
                        {
                            path.sourceInfo.Anonymous.modeInfoIdx = midx as u32;
                            break;
                        }
                    }
                }
                true
            } else {
                false
            }
        });

        if !found {
            if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
                *cache = None;
            }
            return ApplyResult::err(
                "Display not found in cached topology. Please disconnect first.",
            );
        }

        normalize_primary_and_positions(&mut paths, &mut modes);

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
                if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
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

        if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
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
            let gdi_name = win_ccd::get_gdi_name(path);
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

        normalize_primary_and_positions(&mut paths, &mut modes);

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
pub fn set_primary_display(_target_device_name: &str) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn enumerate_displays() -> Vec<DisplayDevice> {
    vec![]
}
#[cfg(not(windows))]
pub fn toggle_monitor_state(_device_name: &str, _enabled: bool) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
#[cfg(not(windows))]
pub fn apply_display_settings(_: &str, _: u32, _: u32, _: u32, _: bool) -> ApplyResult {
    ApplyResult::err("Not supported.")
}
