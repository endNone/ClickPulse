use anyhow::{Context, Result, ensure};
use std::{
    ffi::OsStr,
    hash::{Hash, Hasher},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, GetLastError,
        HANDLE, WAIT_OBJECT_0,
    },
    Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME},
    System::{Console::*, Registry::*, Threading::*},
    UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
};

fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "ClickPulse";
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}
fn read_run() -> Result<Option<Vec<u16>>> {
    read_run_at(RUN_KEY)
}
fn read_run_at(path: &str) -> Result<Option<Vec<u16>>> {
    let mut key = null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(path).as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    ensure!(
        result == ERROR_SUCCESS,
        "Read startup settings: Windows error {result}"
    );
    let key = Key(key);
    let name = wide(RUN_VALUE);
    let mut kind = 0;
    let mut bytes = 0;
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            null(),
            &mut kind,
            null_mut(),
            &mut bytes,
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    ensure!(
        result == ERROR_SUCCESS && kind == REG_SZ && bytes <= 32768,
        "Invalid ClickPulse startup value"
    );
    let mut value = vec![0u16; (bytes as usize).div_ceil(2)];
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            null(),
            &mut kind,
            value.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    ensure!(
        result == ERROR_SUCCESS,
        "Read startup command: Windows error {result}"
    );
    Ok(Some(value))
}
fn run_command(exe: &Path) -> Vec<u16> {
    OsStr::new("\"")
        .encode_wide()
        .chain(exe.as_os_str().encode_wide())
        .chain(OsStr::new("\" --background").encode_wide())
        .chain(Some(0))
        .collect()
}
pub fn auto_start_enabled() -> bool {
    std::env::current_exe().is_ok_and(|exe| {
        read_run().is_ok_and(|value| {
            value
                .and_then(|value| startup_executable(&value))
                .is_some_and(|path| crate::windows_installation::same_program(&path, &exe))
        })
    })
}
pub fn set_auto_start(enabled: bool) -> Result<()> {
    let command = if enabled {
        Some(run_command(&startup_target(&std::env::current_exe()?)?))
    } else {
        None
    };
    write_run_at(RUN_KEY, command.as_deref())
}
fn startup_target(executable: &Path) -> Result<PathBuf> {
    Ok(
        crate::windows_installation::winget_installation(executable)?
            .and_then(|installation| installation.link)
            .unwrap_or_else(|| executable.to_path_buf()),
    )
}
fn startup_executable(value: &[u16]) -> Option<PathBuf> {
    let end = value.iter().rposition(|&c| c != 0)? + 1;
    let value = &value[..end];
    if value.first() != Some(&(b'"' as u16)) || value.contains(&0) {
        return None;
    }
    let quote = value[1..].iter().position(|&c| c == b'"' as u16)? + 1;
    if value[quote + 1..] != " --background".encode_utf16().collect::<Vec<_>>() {
        return None;
    }
    let executable = PathBuf::from(std::ffi::OsString::from_wide(&value[1..quote]));
    (executable.is_absolute()
        && executable
            .file_name()?
            .to_str()?
            .eq_ignore_ascii_case("ClickPulse.exe"))
    .then_some(executable)
}
fn repaired_run(value: Option<&[u16]>, executable: &Path, target: &Path) -> Option<Vec<u16>> {
    let value = value?;
    let previous = startup_executable(value)?;
    // Keep absent/disabled settings and other working portable installations.
    // A missing old executable is repaired only for our exact background command.
    if !previous.try_exists().ok()?
        || crate::windows_installation::same_program(&previous, executable)
    {
        let desired = run_command(target);
        if desired != value {
            return Some(desired);
        }
    }
    None
}
pub fn repair_auto_start() -> Result<bool> {
    let value = read_run()?;
    if value.is_none() {
        return Ok(false);
    }
    let executable = std::env::current_exe()?;
    let target = startup_target(&executable)?;
    if let Some(command) = repaired_run(value.as_deref(), &executable, &target) {
        write_run_at(RUN_KEY, Some(&command))?;
        return Ok(true);
    }
    Ok(false)
}
fn write_run_at(path: &str, command: Option<&[u16]>) -> Result<()> {
    let mut key = null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide(path).as_ptr(),
            0,
            null(),
            0,
            KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        )
    };
    ensure!(
        result == ERROR_SUCCESS,
        "Update startup settings: Windows error {result}"
    );
    let key = Key(key);
    let name = wide(RUN_VALUE);
    let result = if let Some(command) = command {
        unsafe {
            RegSetValueExW(
                key.0,
                name.as_ptr(),
                0,
                REG_SZ,
                command.as_ptr().cast(),
                (command.len() * 2) as u32,
            )
        }
    } else {
        unsafe { RegDeleteValueW(key.0, name.as_ptr()) }
    };
    ensure!(
        result == ERROR_SUCCESS || (command.is_none() && result == ERROR_FILE_NOT_FOUND),
        "Update startup command: Windows error {result}"
    );
    Ok(())
}

pub fn preferred_languages() -> Option<Vec<String>> {
    let mut count = 0;
    let mut len = 0;
    if unsafe { GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, null_mut(), &mut len) }
        == 0
        || len == 0
        || len > 4096
    {
        return None;
    }
    let mut buffer = vec![0u16; len as usize];
    if unsafe {
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, buffer.as_mut_ptr(), &mut len)
    } == 0
    {
        return None;
    }
    Some(
        buffer
            .split(|c| *c == 0)
            .filter(|part| !part.is_empty())
            .map(String::from_utf16_lossy)
            .collect(),
    )
}

pub fn attach_parent_console() {
    // Preserve redirected pipes for CLI statistics invoked from scripts.
    let handles = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE]
        .map(|kind| (kind, unsafe { GetStdHandle(kind) }));
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } != 0 {
        for (kind, handle) in handles {
            if !handle.is_null() && handle as isize != -1 {
                unsafe {
                    SetStdHandle(kind, handle);
                }
            }
        }
    }
}
pub fn show_startup_error(error: &anyhow::Error) {
    let locale = crate::presentation::Preferences::locale_for_error();
    let message = format!("{}\n\n{error:#}", locale.text("error.startup"));
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(message).as_ptr(),
            wide("ClickPulse").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// Relaunching the executable signals the existing window rather than starting
/// another sensor. The signal survives startup while the WebView is loading.
pub struct SingleInstance {
    handle: HANDLE,
    running_marker: HANDLE,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl SingleInstance {
    pub fn begin(path: &Path) -> Result<Option<Self>> {
        let setup = unsafe {
            OpenMutexW(
                SYNCHRONIZATION_SYNCHRONIZE,
                0,
                wide("Local\\ClickPulse.Setup").as_ptr(),
            )
        };
        if !setup.is_null() {
            unsafe {
                CloseHandle(setup);
            }
            anyhow::bail!(
                "{}",
                crate::presentation::Preferences::locale_for_error().text("error.install_running")
            );
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent)?;
        let identity = parent
            .canonicalize()?
            .join(path.file_name().context("Invalid data path")?);
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        identity.to_string_lossy().to_lowercase().hash(&mut hash);
        let name = wide(format!("Local\\ClickPulse.Open.{:016x}", hash.finish()));
        let handle = unsafe { CreateEventW(null(), 0, 0, name.as_ptr()) };
        ensure!(
            !handle.is_null(),
            "Create App activation event: {}",
            std::io::Error::last_os_error()
        );
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe {
                SetEvent(handle);
                CloseHandle(handle);
            }
            return Ok(None);
        }
        let running_marker =
            unsafe { CreateMutexW(null(), 0, wide("Local\\ClickPulse.Running").as_ptr()) };
        if running_marker.is_null() {
            unsafe {
                CloseHandle(handle);
            }
            anyhow::bail!("Create running marker: {}", std::io::Error::last_os_error());
        }
        Ok(Some(Self {
            handle,
            running_marker,
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
        }))
    }
    pub fn listen(&mut self, activate: impl Fn() + Send + 'static) -> Result<()> {
        let handle = self.handle as usize;
        let stop = self.stop.clone();
        self.worker = Some(
            thread::Builder::new()
                .name("clickpulse-activate".into())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        if unsafe { WaitForSingleObject(handle as HANDLE, 200) } == WAIT_OBJECT_0
                            && !stop.load(Ordering::Relaxed)
                        {
                            activate();
                        }
                    }
                })?,
        );
        Ok(())
    }
}
impl Drop for SingleInstance {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        unsafe {
            SetEvent(self.handle);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        unsafe {
            CloseHandle(self.handle);
            CloseHandle(self.running_marker);
        }
    }
}

#[cfg(test)]
mod startup_tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_startup_paths_are_repaired_without_enabling_or_replacing_other_apps() {
        let temp = tempfile::tempdir().unwrap();
        let exe = temp.path().join("new folder/ClickPulse.exe");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"new").unwrap();
        let old = temp.path().join("old folder/ClickPulse.exe");
        let stable = temp.path().join("Links/clickpulse.exe");
        assert!(repaired_run(None, &exe, &stable).is_none());
        assert_eq!(
            repaired_run(Some(&run_command(&old)), &exe, &stable),
            Some(run_command(&stable))
        );
        assert_eq!(
            repaired_run(Some(&run_command(&exe)), &exe, &stable),
            Some(run_command(&stable))
        );
        fs::create_dir_all(old.parent().unwrap()).unwrap();
        fs::write(&old, b"other copy").unwrap();
        assert!(repaired_run(Some(&run_command(&old)), &exe, &stable).is_none());
        assert!(
            repaired_run(
                Some(&run_command(&temp.path().join("another.exe"))),
                &exe,
                &stable
            )
            .is_none()
        );
        let custom = wide(format!("\"{}\" --demo", exe.display()));
        assert!(repaired_run(Some(&custom), &exe, &stable).is_none());
        assert_eq!(startup_executable(&run_command(&exe)), Some(exe.clone()));
        assert!(repaired_run(Some(&run_command(&exe)), &exe, &exe).is_none());
    }

    #[test]
    fn startup_registry_round_trip_uses_a_private_key_and_retains_disabled_state() {
        struct TestKey(String);
        impl Drop for TestKey {
            fn drop(&mut self) {
                unsafe {
                    RegDeleteTreeW(HKEY_CURRENT_USER, wide(&self.0).as_ptr());
                }
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let key = TestKey(format!(
            "Software\\ClickPulse\\Tests\\Startup-{}-{unique}",
            std::process::id()
        ));
        assert!(read_run_at(&key.0).unwrap().is_none());
        let temp = tempfile::tempdir().unwrap();
        let old = temp.path().join("0.3.0/ClickPulse.exe");
        let exe = temp.path().join("0.3.1/ClickPulse.exe");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"app").unwrap();
        write_run_at(&key.0, Some(&run_command(&old))).unwrap();
        let stored = read_run_at(&key.0).unwrap();
        let repaired = repaired_run(stored.as_deref(), &exe, &exe).unwrap();
        write_run_at(&key.0, Some(&repaired)).unwrap();
        assert_eq!(read_run_at(&key.0).unwrap(), Some(run_command(&exe)));
        write_run_at(&key.0, None).unwrap();
        assert!(read_run_at(&key.0).unwrap().is_none());
        assert!(repaired_run(None, &exe, &exe).is_none());
    }
}
