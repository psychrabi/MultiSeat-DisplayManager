use std::ffi::OsString;
use std::fmt;
use std::os::windows::ffi::OsStringExt;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ManagerError {
    Backend(String),
    NoPendingConfirmation,
    Io(std::io::Error),
}

impl fmt::Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(msg) => write!(f, "backend error: {msg}"),
            Self::NoPendingConfirmation => write!(f, "no pending confirmation"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for ManagerError {}

impl From<std::io::Error> for ManagerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type ManagerResult<T> = Result<T, ManagerError>;

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

impl From<ManagerError> for ApplyResult {
    fn from(e: ManagerError) -> Self {
        ApplyResult {
            success: false,
            message: e.to_string(),
        }
    }
}

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub bits_per_pixel: u32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DisplayOrientation {
    #[default]
    Landscape,
    Portrait,
    LandscapeFlipped,
    PortraitFlipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayDevice {
    pub index: u32,
    pub monitor_number: Option<u32>,
    pub device_name: String,
    pub device_string: String,
    pub adapter_name: String,
    pub display_id: DisplayId,
    pub is_primary: bool,
    pub is_active: bool,
    pub not_detected: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub current_mode: Option<DisplayMode>,
    pub available_modes: Vec<DisplayMode>,
    pub orientation: DisplayOrientation,
    pub scale_factor: u32,
}

pub fn wide_to_string(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    OsString::from_wide(&wide[..end])
        .to_string_lossy()
        .into_owned()
}
