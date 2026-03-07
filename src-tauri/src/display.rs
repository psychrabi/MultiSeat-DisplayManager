use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::Mutex;

/// Unique identifier for a display using hardware-based identification
/// This is more reliable than device_name which can change on reconnect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DisplayId {
    pub adapter_luid: u64,      // Graphics adapter identifier (can change on driver update)
    pub target_id: u32,         // Monitor target ID on the adapter
    pub edid_hash: Option<String>, // EDID-based fingerprint (most reliable, survives reconnects)
}

impl DisplayId {
    /// Create a DisplayId from CCD path info
    #[cfg(windows)]
    pub fn from_path(path: &windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO) -> Self {
        let adapter_luid = ((path.targetInfo.adapterId.HighPart as u64) << 32)
            | (path.targetInfo.adapterId.LowPart as u64);
        let target_id = path.targetInfo.id;

        // Try to get EDID hash for more reliable identification
        let edid_hash = get_edid_hash(path);

        Self {
            adapter_luid,
            target_id,
            edid_hash,
        }
    }
}

/// Compute a hash of the monitor's EDID for reliable identification
#[cfg(windows)]
fn get_edid_hash(path: &windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO) -> Option<String> {
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
        // Extract the monitor device path which contains PnP ID and serial
        let device_path = &info.monitorDevicePath;
        let len = device_path.iter().position(|&c| c == 0).unwrap_or(device_path.len());
        let path_str = String::from_utf16_lossy(&device_path[..len]);
        
        // Use the device path as a unique identifier (contains PnP ID and serial)
        // Example: \\?\DISPLAY#MSI30B9#5&1d00314d&0&UID4355#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}
        Some(path_str)
    } else {
        None
    }
}

/// Normalize primary display assignment and rebalance positions
/// Ensures only one primary (first enabled if none) and rebases positions so primary is at (0,0)
#[cfg(windows)]
fn normalize_primary_and_positions(
    paths: &mut Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO>,
    modes: &mut Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO>,
) {
    const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;
    
    // Step 1: Ensure only one primary among enabled outputs
    let mut primary_found = false;
    for path in paths.iter_mut() {
        if (path.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
            // Disabled paths are not primary
            continue;
        }
        
        let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
        if mode_idx >= modes.len() {
            continue;
        }
        
        let mode = &mut modes[mode_idx];
        if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
            let pos = unsafe { mode.Anonymous.sourceMode.position };
            
            if pos.x == 0 && pos.y == 0 && !primary_found {
                // This is the primary (at origin)
                primary_found = true;
            } else if primary_found {
                // Clear primary flag for subsequent displays
                // (Windows determines primary by position 0,0, not by a flag)
            }
        }
    }
    
    // Step 2: If no primary found, assign first enabled display as primary
    if !primary_found {
        for path in paths.iter_mut() {
            if (path.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0 {
                let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
                if mode_idx < modes.len() {
                    let mode = &mut modes[mode_idx];
                    if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                        mode.Anonymous.sourceMode.position.x = 0;
                        mode.Anonymous.sourceMode.position.y = 0;
                        break;
                    }
                }
            }
        }
    }
    
    // Step 3: Rebase all positions so primary is at (0,0)
    // Find the primary display position
    let mut primary_x = 0i32;
    let mut primary_y = 0i32;
    for path in paths.iter() {
        if (path.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0 {
            let mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx } as usize;
            if mode_idx < modes.len() {
                let mode = &modes[mode_idx];
                if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                    let pos = unsafe { mode.Anonymous.sourceMode.position };
                    if pos.x == 0 && pos.y == 0 {
                        primary_x = pos.x;
                        primary_y = pos.y;
                        break;
                    }
                }
            }
        }
    }
    
    // Apply offset to all positions
    if primary_x != 0 || primary_y != 0 {
        for mode in modes.iter_mut() {
            if mode.infoType == windows::Win32::Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                unsafe {
                    mode.Anonymous.sourceMode.position.x -= primary_x;
                    mode.Anonymous.sourceMode.position.y -= primary_y;
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
    pub device_string: String, // Monitor friendly name
    pub adapter_name: String,  // Graphics adapter name
    pub display_id: DisplayId, // Hardware-based unique identifier
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

/// Cache for storing topology snapshot before disabling displays
/// This allows us to re-enable displays with their original modes
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

    #[allow(dead_code)]
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
            // Return friendly name if not empty
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

        // Skip mirroring drivers
        if is_mirror {
            adapter_index += 1;
            continue;
        }

        let path = topology
            .paths
            .iter()
            .find(|p| win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name_str));

        // Skip if not in CCD topology and not attached to desktop
        // This ensures we include inactive (disabled) monitors from CCD
        if path.is_none() && (adapter.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) == 0 {
            adapter_index += 1;
            continue;
        }

        // Get monitor friendly name from DisplayConfig API
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
        
        // For inactive displays, try to get modes from cached topology
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
            // For inactive displays, get mode from topology if available
            path.and_then(|p| {
                let mode_idx = unsafe { p.sourceInfo.Anonymous.modeInfoIdx } as usize;
                topology.modes.get(mode_idx).map(|m| unsafe {
                    DisplayMode {
                        width: m.Anonymous.sourceMode.width,
                        height: m.Anonymous.sourceMode.height,
                        refresh_rate: 60, // Can't get refresh rate for inactive
                        bits_per_pixel: 32,
                    }
                })
            })
        };

        // Get available modes from GDI (works for active displays)
        // For inactive displays, this may still return cached modes from Windows
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

        // If no modes from GDI (inactive display), use modes from topology
        if available_modes.is_empty() {
            if let Some(p) = path {
                let path_adapter_id = p.sourceInfo.adapterId;
                let path_id = p.sourceInfo.id;
                
                // Find all modes for this source in topology
                for mode in &topology.modes {
                    if mode.infoType.0 == 1 { // SOURCE
                        if mode.adapterId == path_adapter_id && mode.id == path_id {
                            let m = unsafe { mode.Anonymous.sourceMode };
                            let mode_obj = DisplayMode {
                                width: m.width,
                                height: m.height,
                                refresh_rate: 60, // Default for inactive
                                bits_per_pixel: 32,
                            };
                            if !available_modes.contains(&mode_obj) {
                                available_modes.push(mode_obj);
                            }
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

        // If no current mode set but we have available modes, use the first (highest) one
        if current_mode.is_none() && !available_modes.is_empty() {
            current_mode = Some(available_modes[0].clone());
        }

        // Get orientation from topology
        let orientation = if let Some(p) = path {
            let rotation = p.targetInfo.rotation;
            match rotation.0 {
                1 => DisplayOrientation::Landscape,
                2 => DisplayOrientation::Portrait,
                3 => DisplayOrientation::LandscapeFlipped,
                4 => DisplayOrientation::PortraitFlipped,
                _ => DisplayOrientation::Landscape,
            }
        } else {
            DisplayOrientation::Landscape
        };

        // Scale factor is typically 100 (100%), 125, 150, 200, etc.
        // Windows doesn't expose this via CCD, so we default to 100
        let scale_factor = 100u32;

        // Create DisplayId from path (or default if path not found)
        let display_id = match path {
            Some(p) => DisplayId::from_path(p),
            None => DisplayId {
                adapter_luid: 0,
                target_id: adapter_index,
                edid_hash: None,
            },
        };

        // Skip monitors without valid EDID hash (virtual displays, etc.)
        if display_id.edid_hash.is_none() || display_id.edid_hash.as_ref().map_or(true, |h| h.is_empty()) {
            adapter_index += 1;
            continue;
        }

        devices.push(DisplayDevice {
            index: adapter_index,
            device_name: device_name_str.clone(),
            device_string: monitor_name,
            adapter_name: device_string_str, // Graphics adapter name
            display_id,
            is_primary,
            is_active,
            position_x: pos_x,
            position_y: pos_y,
            current_mode,
            available_modes,
            orientation,
            scale_factor,
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

    // Set rotation
    let rotation = match orientation {
        DisplayOrientation::Landscape => {
            windows::Win32::Devices::Display::DISPLAYCONFIG_ROTATION(0)
        }
        DisplayOrientation::Portrait => DISPLAYCONFIG_ROTATION_ROTATE90,
        DisplayOrientation::LandscapeFlipped => DISPLAYCONFIG_ROTATION_ROTATE180,
        DisplayOrientation::PortraitFlipped => DISPLAYCONFIG_ROTATION_ROTATE270,
    };
    path.targetInfo.rotation = rotation;

    // Note: We don't swap width/height here - Windows handles that automatically
    // based on the rotation flag. Manually swapping causes issues when changing back.

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult {
            success: true,
            message: format!("Orientation set to {:?} for {}.", orientation, device_name),
        },
        Err(e) => ApplyResult {
            success: false,
            message: e,
        },
    }
}

#[cfg(windows)]
pub fn set_display_position(device_name: &str, x: i32, y: i32) -> ApplyResult {
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
    if mode_idx >= topology.modes.len() {
        return ApplyResult {
            success: false,
            message: "Invalid mode index.".into(),
        };
    }

    let mode = &mut topology.modes[mode_idx];
    mode.Anonymous.sourceMode.position.x = x;
    mode.Anonymous.sourceMode.position.y = y;

    match win_ccd::apply_topology(&topology) {
        Ok(_) => ApplyResult {
            success: true,
            message: format!("Position set to ({}, {}) for {}.", x, y, device_name),
        },
        Err(e) => ApplyResult {
            success: false,
            message: e,
        },
    }
}

#[cfg(windows)]
pub fn set_display_scale(device_name: &str, scale_percent: u32) -> ApplyResult {
    use winreg::enums::*;
    use winreg::RegKey;
    use windows::Win32::UI::WindowsAndMessaging::{
        SendNotifyMessageW, SystemParametersInfoW, HWND_BROADCAST,
        SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, WM_SETTINGCHANGE,
    };

    // Map scale percent to DPI value and DpiValue
    // Windows DpiValue mapping:
    //   0 = 100% (96 DPI)
    //   1 = 125% (120 DPI)
    //   2 = 150% (144 DPI)
    //   3 = 175% (168 DPI)
    //   4 = 200% (192 DPI)
    //   5 = 250% (240 DPI)
    //   6 = 300% (288 DPI)
    //   7 = 400% (384 DPI)
    //   8 = 500% (480 DPI)
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
            // Custom scale - calculate DpiValue and LogPixels
            // For custom scales, Windows uses DpiValue=2 and stores actual DPI in LogPixels
            let calculated_dpi = (96u32 * scale_percent) / 100;
            (2, calculated_dpi)
        }
    };

    // Get the topology to find the monitor's adapter/target info
    let topology = match win_ccd::get_topology(win_ccd::get_all_paths_flags()) {
        Ok(t) => t,
        Err(e) => {
            return ApplyResult {
                success: false,
                message: format!("Failed to get display topology: {}", e),
            }
        }
    };

    // Find the path for this display
    let path = match topology.paths.iter().find(|p| {
        win_ccd::get_gdi_name(p).map_or(false, |n| n == device_name)
    }) {
        Some(p) => p,
        None => {
            return ApplyResult {
                success: false,
                message: "Display not found in topology.".into(),
            }
        }
    };

    // Build the registry key from monitor's PnP ID and device path
    // Windows uses format: {PnP_ID}_{Device_Path_Hash}
    // Example: MSI30B9PB9H315203840_09_07E9_2B^6ED676955A666D6AC664757DCE563C2A

    // Get adapter LUID for fallback
    let adapter_luid = ((path.targetInfo.adapterId.HighPart as u64) << 32)
        | (path.targetInfo.adapterId.LowPart as u64);
    let target_id = path.targetInfo.id;

    // Get monitor target device name which contains PnP info
    let monitor_key = if let Some(device_path) = win_ccd::get_monitor_path(path) {
        // device_path format: \\?\DISPLAY#MSI30B9#5&1d00314d&0&UID4355#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}
        // Extract the PnP ID part (between DISPLAY# and #)
        if let Some(start) = device_path.find("DISPLAY#") {
            let after_display = &device_path[start + 8..];
            if let Some(end) = after_display.find('#') {
                let pnp_id = &after_display[..end];
                // Replace # with _ to match Windows format
                pnp_id.replace('#', "_")
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
    let dpi_key_path = format!("Control Panel\\Desktop\\PerMonitorSettings\\{}", monitor_key);

    // Set per-monitor DpiValue and LogPixels
    // Note: DpiValue must be a string for Windows to recognize it as a preset scale
    match hkcu.create_subkey(&dpi_key_path) {
        Ok((dpi_key, _)) => {
            // Convert DpiValue to string format that Windows expects
            // "0"=100%, "1"=125%, "2"=150%, "3"=175%, "4"=200%, "5"=250%, "6"=300%, "7"=400%, "8"=500%
            // For custom scales, use "2" with LogPixels set
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
                _ => "2", // Custom
            };
            
            if let Err(e) = dpi_key.set_value("DpiValue", &dpi_value_str) {
                eprintln!("Failed to set DpiValue: {}", e);
            }
            // Store the actual DPI value (LogPixels)
            if let Err(e) = dpi_key.set_value("LogPixels", &log_pixels) {
                eprintln!("Failed to set LogPixels: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to create PerMonitorSettings key: {}", e);
        }
    }

    // Also set global LogPixels for compatibility
    if let Ok(desktop_key) = hkcu.open_subkey_with_flags("Control Panel\\Desktop", KEY_SET_VALUE) {
        let _ = desktop_key.set_value("LogPixels", &log_pixels);
        let _ = desktop_key.set_value("Win8DpiScaling", &1u32);
    }

    // Set AppliedDPI in WindowMetrics
    if let Ok(metrics_key) = hkcu.open_subkey_with_flags(
        "Control Panel\\Desktop\\WindowMetrics",
        KEY_SET_VALUE
    ) {
        let _ = metrics_key.set_value("AppliedDPI", &log_pixels);
    }

    // Broadcast DPI change to all applications
    // Use WM_SETTINGCHANGE with "Environment" parameter
    unsafe {
        let setting_name: Vec<u16> = "Environment\0".encode_utf16().collect();
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(setting_name.as_ptr() as isize),
        );

        // Also try SPI_SETLOGPIXELS (0x009F)
        const SPI_SETLOGPIXELS: u32 = 0x009F;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(SPI_SETLOGPIXELS),
            log_pixels,
            Some(&mut log_pixels as *mut u32 as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
    }

    // Notify DWM and refresh system parameters
    notify_dwm_of_dpi_change();
    refresh_user_system_parameters();

    // Trigger a dummy display mode change to force Windows to re-read DPI settings
    // This is the key to immediate DPI application without logoff
    trigger_display_refresh();

    ApplyResult {
        success: true,
        message: format!(
            "Scale set to {}% for {}. Desktop and Explorer should refresh momentarily.",
            scale_percent, device_name
        ),
    }
}

/// Notify DWM (Desktop Window Manager) about DPI changes
#[cfg(windows)]
fn notify_dwm_of_dpi_change() {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetDesktopWindow, GetWindow, SendNotifyMessageW, SetWindowPos,
        GET_WINDOW_CMD, HWND_BROADCAST, HWND_TOP, SWP_FRAMECHANGED, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
        WM_DPICHANGED, WM_DISPLAYCHANGE, WM_FONTCHANGE,
    };

    unsafe {
        // Broadcast DPI changed message to all top-level windows
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_DPICHANGED,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );

        // Also broadcast display change message
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_DISPLAYCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );

        // Broadcast font change message
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_FONTCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(0),
        );

        // Force refresh on the desktop window (Progman/Shell_DDLLView)
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

        // Also try to find and refresh WorkerW windows (used by DWM)
        let mut hwnd = desktop_hwnd;
        loop {
            if let Ok(next) = GetWindow(hwnd, GET_WINDOW_CMD(2)) { // GW_HWNDNEXT = 2
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

/// Force Windows to reload per-user system parameters including DPI
#[cfg(windows)]
fn refresh_user_system_parameters() {
    use windows::Win32::UI::WindowsAndMessaging::{
        SendNotifyMessageW, SystemParametersInfoW, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, WM_SETTINGCHANGE,
    };

    unsafe {
        // SPI_SETNONCLIENTMETRICS triggers a full system metrics refresh
        const SPI_SETNONCLIENTMETRICS: u32 = 0x0029;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(SPI_SETNONCLIENTMETRICS),
            0,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        // SPI_SETICONTITLELOGFONT - refresh icon title fonts
        const SPI_SETICONTITLELOGFONT: u32 = 0x0022;
        let _ = SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_ACTION(SPI_SETICONTITLELOGFONT),
            0,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        // Broadcast WM_SETTINGCHANGE with "WindowsMetrics"
        let setting_name: Vec<u16> = "WindowsMetrics\0".encode_utf16().collect();
        let _ = SendNotifyMessageW(
            windows::Win32::UI::WindowsAndMessaging::HWND_BROADCAST,
            WM_SETTINGCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(setting_name.as_ptr() as isize),
        );
    }
}

/// Trigger a display refresh to force Windows to re-read DPI settings
/// This performs a brief mode change (same resolution, different refresh rate and back)
/// to trigger Windows to apply the new DPI settings immediately
#[cfg(windows)]
fn trigger_display_refresh() {
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, DEVMODEW, DEVMODE_FIELD_FLAGS,
    };
    use std::mem;
    use std::ptr::null_mut;

    unsafe {
        // Use null device name to target primary display
        // First, get current display settings
        let mut devmode: DEVMODEW = mem::zeroed();
        devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;

        // Enumerate current settings
        let enum_result = windows::Win32::Graphics::Gdi::EnumDisplaySettingsW(
            windows::core::PCWSTR(null_mut()),
            windows::Win32::Graphics::Gdi::ENUM_CURRENT_SETTINGS,
            &mut devmode,
        );

        if enum_result.as_bool() {
            // Store original values
            let original_freq = devmode.dmDisplayFrequency;

            // Try to find an alternate refresh rate
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

            // DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL | DM_DISPLAYFREQUENCY
            let dm_fields = DEVMODE_FIELD_FLAGS(0x00080000 | 0x00100000 | 0x00040000 | 0x00400000);

            // If we found an alternate frequency, do a quick switch
            if alt_freq != 0 {
                let mut temp_devmode2: DEVMODEW = mem::zeroed();
                temp_devmode2.dmSize = mem::size_of::<DEVMODEW>() as u16;
                temp_devmode2.dmPelsWidth = devmode.dmPelsWidth;
                temp_devmode2.dmPelsHeight = devmode.dmPelsHeight;
                temp_devmode2.dmBitsPerPel = devmode.dmBitsPerPel;
                temp_devmode2.dmDisplayFrequency = alt_freq;
                temp_devmode2.dmFields = dm_fields;

                // Switch to alternate refresh rate
                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&temp_devmode2 as *const DEVMODEW),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );

                // Brief pause
                std::thread::sleep(std::time::Duration::from_millis(50));

                // Switch back to original refresh rate
                let mut original_devmode: DEVMODEW = mem::zeroed();
                original_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                original_devmode.dmPelsWidth = devmode.dmPelsWidth;
                original_devmode.dmPelsHeight = devmode.dmPelsHeight;
                original_devmode.dmBitsPerPel = devmode.dmBitsPerPel;
                original_devmode.dmDisplayFrequency = original_freq;
                original_devmode.dmFields = dm_fields;

                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&original_devmode as *const DEVMODEW),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );
            } else {
                // No alternate refresh rate found, just trigger a mode enumeration refresh
                // by calling ChangeDisplaySettingsEx with current settings and CDS_NORESET
                let mut current_devmode: DEVMODEW = mem::zeroed();
                current_devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
                current_devmode.dmPelsWidth = devmode.dmPelsWidth;
                current_devmode.dmPelsHeight = devmode.dmPelsHeight;
                current_devmode.dmBitsPerPel = devmode.dmBitsPerPel;
                current_devmode.dmDisplayFrequency = devmode.dmDisplayFrequency;
                current_devmode.dmFields = dm_fields;

                let _ = ChangeDisplaySettingsExW(
                    windows::core::PCWSTR(null_mut()),
                    Some(&current_devmode as *const DEVMODEW),
                    None,
                    windows::Win32::Graphics::Gdi::CDS_NORESET,
                    None,
                );
            }
        }
    }

    // Small delay to allow Windows to process the changes
    std::thread::sleep(std::time::Duration::from_millis(100));
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

    // Check if target is active
    let target_path = &topology.paths[target_path_idx];
    if (target_path.flags & win_ccd::DISPLAYCONFIG_PATH_ACTIVE_FLAG) == 0 {
        return ApplyResult {
            success: false,
            message: "Cannot set inactive display as primary.".into(),
        };
    }

    // Get target position for rebasing
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

    // Rebase all positions so target becomes (0, 0)
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

    // Move target path to front (Windows uses first active path as primary)
    let target_path = topology.paths.remove(target_path_idx);
    topology.paths.insert(0, target_path);

    // Normalize to ensure consistent state
    normalize_primary_and_positions(&mut topology.paths, &mut topology.modes);

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
    eprintln!(
        "[toggle_monitor_state] {} monitor: {}",
        if enabled { "Enabling" } else { "Disabling" },
        device_name
    );

    if enabled {
        // Monarch approach: Use cached topology which has the disabled display's modes
        eprintln!("[toggle_monitor_state] Step 1: Checking cached topology...");

        let (mut paths, mut modes) = {
            let cache_guard = TOPOLOGY_CACHE.lock();
            match cache_guard {
                Ok(ref cache) if cache.is_some() => {
                    let cached = cache.as_ref().unwrap();
                    eprintln!(
                        "[toggle_monitor_state] Using cached topology: {} paths, {} modes",
                        cached.paths.len(),
                        cached.modes.len()
                    );
                    (cached.paths.clone(), cached.modes.clone())
                }
                _ => {
                    eprintln!("[toggle_monitor_state] No cached topology available");
                    return ApplyResult {
                        success: false,
                        message: "No cached topology. Disconnect first, then reconnect.".into(),
                    };
                }
            }
        };

        use windows::Win32::Devices::Display::SetDisplayConfig;
        use windows::Win32::Devices::Display::{
            SDC_ALLOW_CHANGES, SDC_APPLY, SDC_SAVE_TO_DATABASE, SDC_USE_SUPPLIED_DISPLAY_CONFIG,
        };

        const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;

        eprintln!("[toggle_monitor_state] Step 2: Finding display in topology and enabling...");

        // Find the path for this display and enable it
        let mut found = false;
        let mut found_idx = None;
        for (idx, path) in paths.iter_mut().enumerate() {
            let gdi_name = get_gdi_name_from_path(path);
            eprintln!(
                "[toggle_monitor_state] Path {}: gdi_name={:?}, looking_for={}",
                idx, gdi_name, device_name
            );
            if gdi_name.as_ref().map_or(false, |n| n == device_name) {
                eprintln!(
                    "[toggle_monitor_state] Found matching path at index {}",
                    idx
                );
                // Enable the path
                path.flags |= DISPLAYCONFIG_PATH_ACTIVE_FLAG;

                // The cached topology should have valid mode indices
                let source_mode_idx = unsafe { path.sourceInfo.Anonymous.modeInfoIdx };
                eprintln!(
                    "[toggle_monitor_state] Source mode index: {}",
                    source_mode_idx
                );

                if source_mode_idx == 0xFFFFFFFF || (source_mode_idx as usize) >= modes.len() {
                    eprintln!(
                        "[toggle_monitor_state] Mode index invalid, searching for valid mode..."
                    );
                    // Find a valid source mode for this path
                    let path_adapter_id = path.sourceInfo.adapterId;
                    let path_id = path.sourceInfo.id;

                    for (midx, mode) in modes.iter().enumerate() {
                        if mode.infoType.0 == 1 {
                            // SOURCE
                            if mode.adapterId == path_adapter_id && mode.id == path_id {
                                eprintln!(
                                    "[toggle_monitor_state] Found valid source mode at index {}",
                                    midx
                                );
                                path.sourceInfo.Anonymous.modeInfoIdx = midx as u32;
                                break;
                            }
                        }
                    }
                }

                found = true;
                found_idx = Some(idx);
                break;
            }
        }

        eprintln!(
            "[toggle_monitor_state] Display found: {}, index: {:?}",
            found, found_idx
        );

        if !found {
            eprintln!("[toggle_monitor_state] Display not found in cached topology");
            // Clear the stale cache
            if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
                *cache = None;
            }
            return ApplyResult {
                success: false,
                message: "Display not found in cached topology. Please disconnect first.".into(),
            };
        }

        // Apply the changes using cached topology
        eprintln!(
            "[toggle_monitor_state] Step 3: Applying topology with {} paths and {} modes",
            paths.len(),
            modes.len()
        );
        
        // Normalize primary and positions after enabling
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
            eprintln!("[toggle_monitor_state] SetDisplayConfig status: {}", status);

            if status == 0 {
                // Clear cache after successful enable
                if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
                    *cache = None;
                }
                ApplyResult {
                    success: true,
                    message: format!("Monitor {} connected.", device_name),
                }
            } else {
                ApplyResult {
                    success: false,
                    message: format!("Failed to connect monitor: {}", status),
                }
            }
        }
    } else {
        // For disabling, use CCD API and cache the topology for later re-enable
        use windows::Win32::Devices::Display::SetDisplayConfig;
        use windows::Win32::Devices::Display::{
            QDC_ALL_PATHS, SDC_ALLOW_CHANGES, SDC_APPLY, SDC_SAVE_TO_DATABASE,
            SDC_USE_SUPPLIED_DISPLAY_CONFIG,
        };

        const DISPLAYCONFIG_PATH_ACTIVE_FLAG: u32 = 0x0000_0001;

        // Get current topology and cache it BEFORE disabling
        let mut path_count = 0u32;
        let mut mode_count = 0u32;

        unsafe {
            use windows::Win32::Devices::Display::GetDisplayConfigBufferSizes;
            let status =
                GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count);
            eprintln!(
                "[toggle_monitor_state] Disable: GetDisplayConfigBufferSizes: paths={}, modes={}",
                path_count, mode_count
            );
            if status.0 != 0 {
                return ApplyResult {
                    success: false,
                    message: format!("Failed to get display config: {}", status.0),
                };
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
            use windows::Win32::Devices::Display::QueryDisplayConfig;
            let status = QueryDisplayConfig(
                QDC_ALL_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            );
            eprintln!(
                "[toggle_monitor_state] Disable: QueryDisplayConfig: out_paths={}, out_modes={}, status={}",
                path_count, mode_count, status.0
            );
            if status.0 != 0 {
                return ApplyResult {
                    success: false,
                    message: format!("Failed to query display config: {}", status.0),
                };
            }
        }

        paths.truncate(path_count as usize);
        modes.truncate(mode_count as usize);
        eprintln!(
            "[toggle_monitor_state] Disable: Total paths: {}, modes: {}",
            paths.len(),
            modes.len()
        );

        // Cache the full topology BEFORE making changes (Monarch approach)
        eprintln!("[toggle_monitor_state] Caching topology for later re-enable");
        if let Ok(mut cache) = TOPOLOGY_CACHE.lock() {
            *cache = Some(CachedTopology {
                paths: paths.clone(),
                modes: modes.clone(),
            });
        }

        // Count active displays first
        let total_active_count = paths
            .iter()
            .filter(|p| (p.flags & DISPLAYCONFIG_PATH_ACTIVE_FLAG) != 0)
            .count();
        eprintln!(
            "[toggle_monitor_state] Disable: Active displays: {}",
            total_active_count
        );

        // Find the path for this display and disable it
        let mut found = false;
        for (idx, path) in paths.iter_mut().enumerate() {
            let gdi_name = get_gdi_name_from_path(path);
            eprintln!(
                "[toggle_monitor_state] Disable: Path {}: gdi_name={:?}, looking_for={}",
                idx, gdi_name, device_name
            );
            if gdi_name.as_ref().map_or(false, |n| n == device_name) {
                // Check if this is the last active display
                if total_active_count <= 1 {
                    eprintln!("[toggle_monitor_state] Disable: Cannot disable last active display");
                    return ApplyResult {
                        success: false,
                        message: "Cannot disable the last active display.".into(),
                    };
                }

                eprintln!(
                    "[toggle_monitor_state] Disable: Found matching path at index {}, disabling",
                    idx
                );
                path.flags &= !DISPLAYCONFIG_PATH_ACTIVE_FLAG;
                found = true;
                break;
            }
        }

        eprintln!("[toggle_monitor_state] Disable: Display found: {}", found);

        if !found {
            return ApplyResult {
                success: false,
                message: "Display not found.".into(),
            };
        }

        // Normalize primary and positions after disabling
        normalize_primary_and_positions(&mut paths, &mut modes);

        // Apply the changes
        eprintln!("[toggle_monitor_state] Disable: Applying topology");
        unsafe {
            let status = SetDisplayConfig(
                Some(&paths),
                Some(&modes),
                SDC_APPLY
                    | SDC_USE_SUPPLIED_DISPLAY_CONFIG
                    | SDC_SAVE_TO_DATABASE
                    | SDC_ALLOW_CHANGES,
            );
            eprintln!(
                "[toggle_monitor_state] Disable: SetDisplayConfig status: {}",
                status
            );

            if status == 0 {
                ApplyResult {
                    success: true,
                    message: format!("Monitor {} disconnected.", device_name),
                }
            } else {
                ApplyResult {
                    success: false,
                    message: format!("Failed to disconnect monitor: {}", status),
                }
            }
        }
    }
}

// Helper function to get GDI device name from a DISPLAYCONFIG_PATH_INFO
#[cfg(windows)]
fn get_gdi_name_from_path(
    path: &windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO,
) -> Option<String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Devices::Display::{
        DisplayConfigGetDeviceInfo, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    };

    const DISPLAYCONFIG_PATH_MODE_IDX_INVALID: u32 = 0xffffffff;

    let mut info = DISPLAYCONFIG_SOURCE_DEVICE_NAME::default();
    info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
    info.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
    info.header.adapterId = path.sourceInfo.adapterId;
    info.header.id = path.sourceInfo.id;

    if path.sourceInfo.id == DISPLAYCONFIG_PATH_MODE_IDX_INVALID {
        return None;
    }

    unsafe {
        let status = DisplayConfigGetDeviceInfo(&mut info.header);
        if status == 0 {
            let name = &info.viewGdiDeviceName;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            Some(
                OsString::from_wide(&name[..len])
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
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
    eprintln!(
        "[apply_display_settings] Applying {}x{} @ {}Hz to {}",
        width, height, refresh_rate, device_name
    );

    // Use GDI ChangeDisplaySettingsExW for resolution/refresh rate changes
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsW, CDS_TEST, CDS_UPDATEREGISTRY, DEVMODEW,
        ENUM_DISPLAY_SETTINGS_MODE,
    };

    let gdi_name_wide: Vec<u16> = device_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // Search for the exact mode in the display's available modes
    let mut found_devmode: Option<DEVMODEW> = None;
    let mut mode_index = 0u32;

    // First, get current settings to match BPP
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

    eprintln!("[apply_display_settings] Current BPP: {}", current_bpp);

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
            break; // No more modes
        }

        // Check if this mode matches (including BPP)
        if devmode.dmPelsWidth == width
            && devmode.dmPelsHeight == height
            && devmode.dmDisplayFrequency == refresh_rate
            && (devmode.dmBitsPerPel == 0 || devmode.dmBitsPerPel == current_bpp)
        {
            eprintln!(
                "[apply_display_settings] Found matching mode at index {}: {}x{} @ {}Hz, BPP={}",
                mode_index,
                devmode.dmPelsWidth,
                devmode.dmPelsHeight,
                devmode.dmDisplayFrequency,
                devmode.dmBitsPerPel
            );
            found_devmode = Some(devmode);
            break;
        }

        mode_index += 1;

        // Safety limit
        if mode_index > 300 {
            eprintln!("[apply_display_settings] Mode search limit reached");
            break;
        }
    }

    // If exact match not found, try to build a devmode with proper fields
    let devmode = if let Some(dm) = found_devmode {
        dm
    } else {
        eprintln!("[apply_display_settings] Exact mode not found, trying to build devmode...");
        // Get current settings as base
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
            return ApplyResult {
                success: false,
                message: "Failed to get display settings.".into(),
            };
        }

        devmode.dmPelsWidth = width;
        devmode.dmPelsHeight = height;
        devmode.dmDisplayFrequency = refresh_rate;
        // Keep the current BPP
        devmode.dmBitsPerPel = current_bpp;
        devmode.dmFields = windows::Win32::Graphics::Gdi::DEVMODE_FIELD_FLAGS(
            0x40000 | 0x80000 | 0x400000 | 0x40, // DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY | DM_BITSPERPEL
        );
        devmode
    };

    eprintln!(
        "[apply_display_settings] Testing mode {}x{} @ {}Hz, BPP={}",
        devmode.dmPelsWidth, devmode.dmPelsHeight, devmode.dmDisplayFrequency, devmode.dmBitsPerPel
    );

    // Test the mode first
    let test_result = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(gdi_name_wide.as_ptr()),
            Some(&devmode),
            None,
            CDS_TEST,
            None,
        )
    };

    eprintln!("[apply_display_settings] Test result: {}", test_result.0);

    if test_result.0 != 0 {
        let msg = match test_result.0 {
            -1 => "Display not supported".to_string(),
            -2 => format!(
                "Bad mode - {}x{} @ {}Hz is not supported by this display",
                devmode.dmPelsWidth, devmode.dmPelsHeight, devmode.dmDisplayFrequency
            ),
            -3 => "Bad flags".to_string(),
            -4 => "Bad registry".to_string(),
            _ => format!("Unknown error code {}", test_result.0),
        };
        return ApplyResult {
            success: false,
            message: msg,
        };
    }

    // Apply the settings
    eprintln!("[apply_display_settings] Applying mode...");
    let apply_result = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(gdi_name_wide.as_ptr()),
            Some(&devmode),
            None,
            CDS_UPDATEREGISTRY,
            None,
        )
    };

    eprintln!("[apply_display_settings] Apply result: {}", apply_result.0);

    match apply_result.0 {
        0 => ApplyResult {
            success: true,
            message: format!("Settings applied to {}.", device_name),
        },
        1 => ApplyResult {
            success: true,
            message: format!("Settings applied to {} (restart required).", device_name),
        },
        -1 => ApplyResult {
            success: false,
            message: "Display not supported.".into(),
        },
        -2 => ApplyResult {
            success: false,
            message: "Bad mode.".into(),
        },
        -3 => ApplyResult {
            success: false,
            message: "Bad flags.".into(),
        },
        -4 => ApplyResult {
            success: false,
            message: "Bad registry.".into(),
        },
        code => ApplyResult {
            success: false,
            message: format!("Failed with code: {}", code),
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
