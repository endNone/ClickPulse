//! Platform paths and shell integration. Never select a checkout or backup as data.
#[cfg(any(not(target_os = "linux"), test))]
use anyhow::Context;
use anyhow::Result;
#[cfg(any(target_os = "macos", target_os = "linux", test))]
use std::path::Path;
use std::path::PathBuf;

pub fn name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "other"
    }
}

pub fn data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    return windows_data_dir(std::env::var_os("LOCALAPPDATA"));
    #[cfg(target_os = "macos")]
    return Ok(PathBuf::from(
        std::env::var_os("HOME").context("Cannot determine the user home directory")?,
    )
    .join("Library/Application Support/ClickPulse"));
    #[cfg(target_os = "linux")]
    return Ok(linux::xdg_dir(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::var_os("HOME"),
        ".local/share",
    )?
    .join("ClickPulse"));
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .context("User data directory is unavailable")?;
        Ok(base.join("ClickPulse"))
    }
}

#[cfg(any(target_os = "windows", test))]
fn windows_data_dir(local_app_data: Option<std::ffi::OsString>) -> Result<PathBuf> {
    let base = PathBuf::from(local_app_data.context("LOCALAPPDATA is unavailable")?);
    anyhow::ensure!(base.is_absolute(), "LOCALAPPDATA must be an absolute path");
    Ok(base.join("ClickPulse"))
}

pub fn log_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    return Ok(PathBuf::from(
        std::env::var_os("HOME").context("Cannot determine the log directory")?,
    )
    .join("Library/Logs/clickpulse.log"));
    #[cfg(not(target_os = "macos"))]
    Ok(data_dir()?.join("logs/clickpulse.log"))
}

pub fn auto_start_enabled() -> bool {
    #[cfg(target_os = "macos")]
    return crate::handoff::auto_start_enabled();
    #[cfg(target_os = "windows")]
    return crate::platform::windows::auto_start_enabled();
    #[cfg(target_os = "linux")]
    return linux::auto_start_enabled();
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    false
}

#[cfg(any(target_os = "macos", target_os = "linux", test))]
pub(crate) fn same_executable(saved: &Path, current: &Path) -> bool {
    matches!((saved.canonicalize(), current.canonicalize()), (Ok(a), Ok(b)) if a == b)
}

pub fn repair_auto_start() -> Result<bool> {
    #[cfg(target_os = "macos")]
    return crate::handoff::repair_auto_start();
    #[cfg(target_os = "windows")]
    return windows::repair_auto_start();
    #[cfg(target_os = "linux")]
    return linux::repair_auto_start();
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Ok(false)
}

#[derive(Debug)]
pub struct TemporaryAppLocation;
impl std::fmt::Display for TemporaryAppLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Move ClickPulse to Applications before enabling login startup")
    }
}
impl std::error::Error for TemporaryAppLocation {}

pub fn set_auto_start(enabled: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    return crate::handoff::set_auto_start(enabled);
    #[cfg(target_os = "windows")]
    return windows::set_auto_start(enabled);
    #[cfg(target_os = "linux")]
    return linux::set_auto_start(enabled);
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = enabled;
        anyhow::bail!("Desktop startup is unsupported on this platform")
    }
}

pub fn open_project_home() -> Result<()> {
    open_web_page("https://github.com/endNone/ClickPulse")
}

pub fn open_feedback() -> Result<()> {
    open_web_page("https://github.com/endNone/ClickPulse/issues/new/choose")
}

pub(crate) mod launch;
#[cfg(target_os = "macos")]
pub use launch::open_input_monitoring_settings;
pub use launch::{open_web_page, reveal};

#[cfg(target_os = "windows")]
#[path = "platform_windows.rs"]
pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_data_directory_never_resolves_relative_to_the_working_directory() {
        assert!(windows_data_dir(None).is_err());
        for value in ["", ".", "profile/AppData/Local"] {
            assert!(windows_data_dir(Some(value.into())).is_err());
        }
        #[cfg(target_os = "windows")]
        for value in [
            r"C:Users\tester\AppData\Local",
            r"\Users\tester\AppData\Local",
        ] {
            assert!(windows_data_dir(Some(value.into())).is_err());
        }
        let profile = tempfile::tempdir().unwrap();
        let base = profile.path().join("AppData").join("Local");
        assert_eq!(
            windows_data_dir(Some(base.clone().into_os_string())).unwrap(),
            base.join("ClickPulse")
        );
        assert!(!base.exists(), "resolving a path must not create user data");
    }
}

#[cfg(any(target_os = "linux", test))]
#[path = "platform_linux.rs"]
pub mod linux;
