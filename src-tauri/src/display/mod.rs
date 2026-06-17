//! Display management subsystem.
//!
//! Handles monitor enumeration, display configuration (resolution, orientation,
//! position, scale, primary), topology caching, and layout confirmation/rollback.
//! Uses the Win32 Display Config API (`QueryDisplayConfig` / `SetDisplayConfig`)
//! on Windows and provides fallback stubs on other platforms.

mod types;
mod cache;
mod win32;
mod operations;
mod confirmation;

pub use types::*;
pub use operations::*;
pub use confirmation::*;
