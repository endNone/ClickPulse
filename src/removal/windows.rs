use super::*;
use crate::{localization::Locale, presentation::Preferences};
use std::{
    process::Command,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::*, System::Threading::*, UI::WindowsAndMessaging::*};
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn message(locale: Locale, key: &str, details: &str, flags: MESSAGEBOX_STYLE) -> i32 {
    let text = format!("{}\n\n{details}", locale.text(key));
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(&text).as_ptr(),
            wide(&locale.text("removal.title")).as_ptr(),
            flags,
        )
    }
}
struct MutexGuard(HANDLE);
impl Drop for MutexGuard {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

/// Run from a separate copy so Windows can release the installed executable.
/// Invoked only after a successful normal save/quit, or explicit CLI removal.
pub fn launch() -> Result<()> {
    let original = std::env::current_exe()?.canonicalize()?;
    let directory = tempfile::Builder::new()
        .prefix("ClickPulse-removal-")
        .tempdir()?;
    let helper = directory.path().join("ClickPulse.exe");
    fs::copy(&original, &helper)?;
    let loader = original
        .parent()
        .context("Missing application folder")?
        .join("WebView2Loader.dll");
    if loader.is_file() {
        fs::copy(loader, directory.path().join("WebView2Loader.dll"))?;
    }
    let mut command = Command::new(&helper);
    command.current_dir(std::env::temp_dir());
    command.arg("uninstall").arg("--from").arg(&original);
    command.spawn()?;
    // A running Windows executable cannot be removed. The system temp folder
    // holds only the helper executable/loader, never user history or settings.
    let _ = directory.keep();
    Ok(())
}

pub fn run(original: &Path) -> Result<()> {
    let _ = std::env::set_current_dir(std::env::temp_dir());
    use sha2::{Digest, Sha256};
    let helper = std::env::current_exe()?.canonicalize()?;
    let temporary = std::env::temp_dir().canonicalize()?;
    let helper_root = helper.parent().context("Missing helper folder")?;
    ensure!(
        helper_root.parent() == Some(temporary.as_path())
            && helper_root
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("ClickPulse-removal-")),
        "Removal must run from its isolated helper"
    );
    let _cleanup = HelperCleanup(helper_root.to_owned());
    let original = original.canonicalize()?;
    ensure!(
        original
            .file_name()
            .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("ClickPulse.exe")),
        "Invalid application executable"
    );
    ensure!(
        Sha256::digest(fs::read(&original)?) == Sha256::digest(fs::read(std::env::current_exe()?)?),
        "Removal helper does not match application"
    );
    let locale = Preferences::locale_for_error();
    let setup = unsafe {
        CreateMutexW(
            std::ptr::null(),
            0,
            wide("Local\\ClickPulse.Setup").as_ptr(),
        )
    };
    ensure!(!setup.is_null(), "Unable to start removal");
    let already = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _setup = MutexGuard(setup);
    ensure!(!already, "Another installation or removal is running");
    let manager = crate::updater::package_manager(&original)?;
    let started = Instant::now();
    loop {
        let running = unsafe {
            OpenMutexW(
                SYNCHRONIZATION_SYNCHRONIZE,
                0,
                wide("Local\\ClickPulse.Running").as_ptr(),
            )
        };
        if running.is_null() {
            break;
        }
        unsafe {
            CloseHandle(running);
        }
        if started.elapsed() > Duration::from_secs(10) {
            message(locale, "removal.running", "", MB_OK | MB_ICONINFORMATION);
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let raw_profile = crate::platform::data_dir()?;
    if raw_profile.exists() {
        plain(&raw_profile)?;
    }
    let profile = if raw_profile.exists() {
        raw_profile.canonicalize()?
    } else {
        raw_profile
            .parent()
            .context("Missing data parent")?
            .canonicalize()?
            .join("ClickPulse")
    };
    let root = original.parent().context("Missing application folder")?;
    ensure!(
        !root.starts_with(&profile) && !profile.starts_with(root),
        "Program and data folders overlap; remove manually"
    );
    let profile_lock = if profile.exists() {
        plain(&profile)?;
        let path = profile.join("profile.lock");
        if path.exists() {
            plain(&path)?;
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        crate::storage::lock_profile_for_helper(&file, Duration::from_secs(5))
            .context("ClickPulse data is still in use")?;
        Some(file)
    } else {
        None
    };
    // Never follow storage.json to remove data outside the owned profile.
    let custom = fs::read(profile.join("storage.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v["database"].as_str().map(PathBuf::from))
        .map(|path| path.canonicalize().unwrap_or(path))
        .filter(|path| !path.starts_with(&profile));
    let mut details = format!(
        "{}\n\n{}",
        readable_windows_path(root),
        readable_windows_path(&profile)
    );
    if let Some(path) = &custom {
        details.push_str(&format!(
            "\n\n{}\n{}",
            locale.text("removal.custom"),
            readable_windows_path(path)
        ));
    }
    if manager.is_some() {
        details.push_str(&format!("\n\n{}", locale.text("removal.managed")));
    }
    // Yes = keep (default), No = delete, Cancel/Escape = no change.
    let choice = message(
        locale,
        "removal.choose",
        &details,
        MB_YESNOCANCEL | MB_DEFBUTTON1 | MB_ICONQUESTION,
    );
    if choice != IDYES && choice != IDNO {
        return Ok(());
    }
    let clear = choice == IDNO;
    if clear
        && message(
            locale,
            "removal.confirm",
            &readable_windows_path(&profile),
            MB_OKCANCEL | MB_DEFBUTTON2 | MB_ICONWARNING,
        ) != IDOK
    {
        return Ok(());
    }
    let data = profile_cleanup_files(&profile, clear)
        .with_context(|| format!("Program files retained at {}", readable_windows_path(root)))?;
    let programs = if manager.is_none() {
        program_files(root)?
    } else {
        Vec::new()
    };
    data_before_program(
        || {
            // Keep the lock through cache cleanup. The setup mutex prevents new App
            // starts while WebView2 is releasing its last files.
            let lock = profile.join("profile.lock");
            let data: Vec<_> = data
                .into_iter()
                .filter(|p| p != &profile && p != &lock)
                .collect();
            delete_profile_files(&profile, &data).with_context(|| {
                format!("Program files retained at {}", readable_windows_path(root))
            })?;
            drop(profile_lock);
            if lock.try_exists()? {
                delete_one(&lock)
                    .with_context(|| format!("Data remains at {}", readable_windows_path(&lock)))?;
            }
            if clear && profile.try_exists()? {
                delete_one(&profile).with_context(|| {
                    format!("Data remains at {}", readable_windows_path(&profile))
                })?;
            }
            crate::platform::windows::remove_matching_startup(&original)?;
            // Applies to portable, Scoop and WinGet copies alike.
            crate::platform::windows::remove_notification_identity()?;
            Ok(())
        },
        || {
            if manager.is_none() {
                delete_files(&programs)?;
                let _ = fs::remove_dir(root.join("Licenses"));
                let _ = fs::remove_dir(root); // Unrelated files remain.
            }
            Ok(())
        },
    )
    .with_context(|| {
        let remaining = [root, profile.as_path()]
            .into_iter()
            .filter(|path| path.try_exists().unwrap_or(true))
            .map(|path| readable_windows_path(path).to_string())
            .collect::<Vec<_>>();
        format!("Remaining folders:\n{}", remaining.join("\n"))
    })?;
    let command = match manager {
        Some("winget") => "winget uninstall --id endNone.ClickPulse --exact",
        Some("scoop") => "scoop uninstall clickpulse",
        None => "",
        _ => anyhow::bail!("Unsupported package manager"),
    };
    let mut detail = if clear {
        String::new()
    } else {
        readable_windows_path(&profile).to_string()
    };
    if let Some(path) = custom {
        detail.push_str(&format!(
            "\n\n{}\n{}",
            locale.text("removal.custom"),
            readable_windows_path(&path)
        ));
    }
    if !command.is_empty() {
        detail.push_str(&format!(
            "\n\n{}\n{command}",
            locale.text("removal.command")
        ));
    }
    message(
        locale,
        if clear {
            "removal.cleared"
        } else {
            "removal.kept"
        },
        &detail,
        MB_OK | MB_ICONINFORMATION,
    );
    Ok(())
}

/// Remove only this task's two helper files after Windows releases the executable.
struct HelperCleanup(PathBuf);
impl Drop for HelperCleanup {
    fn drop(&mut self) {
        use std::os::windows::process::CommandExt;
        // Avoid PowerShell provider/Join-Path wildcard parsing of canonical \\?\ paths.
        let script = r#"
$p=$env:CLICKPULSE_REMOVAL_TEMP
for($i=0;$i -lt 150;$i++) {
    try {
        [System.IO.File]::Delete([System.IO.Path]::Combine($p, 'ClickPulse.exe'))
        [System.IO.File]::Delete([System.IO.Path]::Combine($p, 'WebView2Loader.dll'))
        [System.IO.Directory]::Delete($p)
        break
    } catch { Start-Sleep -Milliseconds 200 }
}
"#;
        let Ok(powershell) = crate::platform::launch::windows_tool("powershell.exe") else {
            return;
        };
        let _ = Command::new(powershell)
            .current_dir(std::env::temp_dir())
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("CLICKPULSE_REMOVAL_TEMP", readable_windows_path(&self.0))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
}
