#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod branding;
#[cfg(target_os = "macos")]
#[cfg_attr(test, allow(dead_code))]
pub mod capture;
#[cfg(target_os = "windows")]
#[path = "capture_windows.rs"]
#[cfg_attr(test, allow(dead_code))]
pub mod capture;
pub mod collector;
pub mod db;
#[cfg(target_os = "macos")]
pub mod handoff;
pub mod history_import;
pub mod hourly;
pub mod input;
pub mod localization;
pub mod platform;
pub mod presentation;
pub mod runtime;
pub mod source;
pub mod storage;
mod storage_schema;
pub mod version;
pub mod windows_input;
