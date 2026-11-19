//! Desktop launchers are short-lived children, and must always be reaped.
#[cfg(any(target_os = "windows", target_os = "linux"))]
use anyhow::Context;
use anyhow::Result;
use std::{
    path::Path,
    process::{Child, Command, ExitStatus},
    sync::mpsc,
    thread,
};

#[cfg(target_os = "windows")]
pub(crate) fn windows_tool(name: &str) -> Result<std::path::PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::SystemInformation::{
        GetSystemDirectoryW, GetWindowsDirectoryW,
    };
    let mut buffer = vec![0_u16; 32768];
    let length = unsafe {
        if name == "explorer.exe" {
            GetWindowsDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32)
        } else {
            GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32)
        }
    } as usize;
    anyhow::ensure!(
        length > 0 && length < buffer.len(),
        "Windows system directory is unavailable"
    );
    let directory = std::path::PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length]));
    let relative = match name {
        "powershell.exe" => "WindowsPowerShell/v1.0/powershell.exe",
        "rundll32.exe" | "url.dll" | "explorer.exe" => name,
        _ => anyhow::bail!("Unsupported Windows system tool"),
    };
    Ok(directory.join(relative))
}

fn launch(command: &mut Command) -> Result<thread::JoinHandle<std::io::Result<ExitStatus>>> {
    launch_with_status(command, true)
}

fn launch_with_status(
    command: &mut Command,
    report_status: bool,
) -> Result<thread::JoinHandle<std::io::Result<ExitStatus>>> {
    let (tx, rx) = mpsc::channel::<Child>();
    // Start the waiter first: thread creation failure must not orphan a child.
    let waiter = thread::Builder::new()
        .name("clickpulse-launcher".into())
        .spawn(move || {
            let mut child = rx.recv().map_err(std::io::Error::other)?;
            let result = child.wait();
            if let Err(error) = &result {
                eprintln!("Cannot wait for desktop launcher: {error}");
            } else if let Ok(status) = &result
                && report_status
                && !status.success()
            {
                eprintln!("Desktop launcher failed: {status}");
            }
            result
        })?;
    let child = command.spawn()?;
    if let Err(error) = tx.send(child) {
        // The waiter unexpectedly stopped. Retain ownership and reap anyway.
        let mut child = error.0;
        child.wait()?;
        anyhow::bail!("Desktop launcher waiter stopped");
    }
    Ok(waiter)
}

pub fn open_web_page(url: &str) -> Result<()> {
    anyhow::ensure!(
        url.starts_with("https://"),
        "Only HTTPS web pages are supported"
    );
    open_target(url)
}

fn open_target(target: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    launch(Command::new("/usr/bin/open").arg(target))?;
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        launch(
            Command::new(windows_tool("rundll32.exe")?)
                .arg(format!(
                    "{},FileProtocolHandler",
                    windows_tool("url.dll")?.display()
                ))
                .arg(target)
                .creation_flags(0x08000000),
        )?;
    }
    #[cfg(target_os = "linux")]
    launch(Command::new("xdg-open").arg(target))?;
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    anyhow::bail!("Opening external pages is unsupported on this platform");
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_input_monitoring_settings() -> Result<()> {
    open_target("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
}

pub fn reveal(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    launch(Command::new("/usr/bin/open").arg("-R").arg(path))?;
    #[cfg(target_os = "windows")]
    // Explorer may return 1 after successfully delegating to the running shell.
    // Keep spawn/wait errors, but its exit status is not a success contract.
    launch_with_status(
        Command::new(windows_tool("explorer.exe")?).arg(
            super::windows::shell_path(path.parent().context("Data directory is unavailable")?)
                .as_ref(),
        ),
        false,
    )?;
    #[cfg(target_os = "linux")]
    launch(Command::new("xdg-open").arg(path.parent().context("Data directory is unavailable")?))?;
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    anyhow::bail!("Desktop file browser is unsupported on this platform");
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launcher_reaps_child_and_reports_spawn_failure() {
        let mut command = Command::new(if cfg!(windows) { "cmd.exe" } else { "/bin/sh" });
        command.args(if cfg!(windows) {
            ["/C", "exit 7"]
        } else {
            ["-c", "exit 7"]
        });
        assert_eq!(
            launch(&mut command)
                .unwrap()
                .join()
                .unwrap()
                .unwrap()
                .code(),
            Some(7)
        );
        assert!(launch(&mut Command::new("clickpulse-missing-launcher-test")).is_err());
        assert!(open_web_page("file:///tmp/test").is_err());
    }
}
