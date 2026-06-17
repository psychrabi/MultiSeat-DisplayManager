use crate::display::{ApplyResult, DisplayDevice, DisplayOrientation, ManagerResult};

pub type DynDisplayBackend = Box<dyn DisplayBackend + Send + Sync>;

pub trait DisplayBackend {
    fn list_displays(&self) -> ManagerResult<Vec<DisplayDevice>>;
    fn apply_settings(&self, device_name: &str, width: u32, height: u32, refresh_rate: u32, persist: bool) -> ApplyResult;
    fn set_primary(&self, device_name: &str) -> ApplyResult;
    fn set_orientation(&self, device_name: &str, orientation: DisplayOrientation) -> ApplyResult;
    fn set_position(&self, device_name: &str, x: i32, y: i32) -> ApplyResult;
    fn set_scale(&self, device_name: &str, scale_percent: u32) -> ApplyResult;
}

pub struct Win32Backend;

impl DisplayBackend for Win32Backend {
    fn list_displays(&self) -> ManagerResult<Vec<DisplayDevice>> {
        Ok(crate::display::enumerate_displays())
    }

    fn apply_settings(&self, device_name: &str, width: u32, height: u32, refresh_rate: u32, persist: bool) -> ApplyResult {
        crate::display::apply_display_settings(device_name, width, height, refresh_rate, persist)
    }

    fn set_primary(&self, device_name: &str) -> ApplyResult {
        crate::display::set_primary_display(device_name)
    }

    fn set_orientation(&self, device_name: &str, orientation: DisplayOrientation) -> ApplyResult {
        crate::display::set_display_orientation(device_name, orientation)
    }

    fn set_position(&self, device_name: &str, x: i32, y: i32) -> ApplyResult {
        crate::display::set_display_position(device_name, x, y)
    }

    fn set_scale(&self, device_name: &str, scale_percent: u32) -> ApplyResult {
        crate::display::set_display_scale(device_name, scale_percent)
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub struct MockBackend {
    displays: Vec<DisplayDevice>,
}

#[cfg(test)]
#[allow(dead_code)]
impl MockBackend {
    pub fn new(displays: Vec<DisplayDevice>) -> Self {
        Self { displays }
    }
}

#[cfg(test)]
impl DisplayBackend for MockBackend {
    fn list_displays(&self) -> ManagerResult<Vec<DisplayDevice>> {
        Ok(self.displays.clone())
    }

    fn apply_settings(&self, _device_name: &str, _width: u32, _height: u32, _refresh_rate: u32, _persist: bool) -> ApplyResult {
        ApplyResult::ok("mock: settings applied")
    }

    fn set_primary(&self, device_name: &str) -> ApplyResult {
        ApplyResult::ok(format!("mock: {} set as primary", device_name))
    }

    fn set_orientation(&self, device_name: &str, _orientation: DisplayOrientation) -> ApplyResult {
        ApplyResult::ok(format!("mock: orientation set for {}", device_name))
    }

    fn set_position(&self, device_name: &str, x: i32, y: i32) -> ApplyResult {
        ApplyResult::ok(format!("mock: {} moved to ({}, {})", device_name, x, y))
    }

    fn set_scale(&self, device_name: &str, scale_percent: u32) -> ApplyResult {
        ApplyResult::ok(format!("mock: scale set to {}% for {}", scale_percent, device_name))
    }
}
