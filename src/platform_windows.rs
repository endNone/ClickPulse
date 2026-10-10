use anyhow::{Context, Result, ensure};
use std::{
    ffi::OsStr,
    hash::{Hash, Hasher},
    os::windows::ffi::OsStrExt,
    path::Path,
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
    let mut key = null_mut();
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(RUN_KEY).as_ptr(),
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
        read_run().is_ok_and(|value| value.is_some_and(|value| value == run_command(&exe)))
    })
}
pub fn set_auto_start(enabled: bool) -> Result<()> {
    let mut key = null_mut();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide(RUN_KEY).as_ptr(),
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
    let result = if enabled {
        let command = run_command(&std::env::current_exe()?);
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
        result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND),
        "Update startup command: Windows error {result}"
    );
    Ok(())
}

pub fn preferred_language() -> Option<String> {
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
    Some(String::from_utf16_lossy(buffer.split(|c| *c == 0).next()?))
}

pub fn attach_parent_console() {
    // Preserve redirected pipes for CLI stats/exports invoked from scripts.
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
