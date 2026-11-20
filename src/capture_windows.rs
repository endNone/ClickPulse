//! Passive Windows Raw Input listener on a dedicated message-only window.
//! Counts use scan codes. Only eligible shortcuts consult the foreground layout.
use crate::{
    input::{Device, InputEvent},
    runtime::{CaptureStatus, Command},
    windows_input::{WindowsKeyboard, mouse_buttons},
};
use anyhow::{Result, ensure};
use crossbeam_channel::Sender;
use std::{
    cell::RefCell,
    ptr::null,
    ptr::null_mut,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::*},
    UI::{
        Accessibility::*,
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, GetKeyboardLayout, HKL, MAPVK_VK_TO_CHAR, MAPVK_VSC_TO_VK_EX,
            MapVirtualKeyExW,
        },
        Input::*,
        WindowsAndMessaging::*,
    },
};

fn foreground_layout() -> Option<HKL> {
    unsafe {
        let window_layout = |window: HWND| {
            if window.is_null() {
                return None;
            }
            let tid = GetWindowThreadProcessId(window, null_mut());
            if tid == 0 {
                return None;
            }
            let layout = GetKeyboardLayout(tid);
            (!layout.is_null()).then_some(layout)
        };
        // The focused control can belong to a different thread than its host window.
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        let focused = GetGUIThreadInfo(0, &mut info) != 0;
        focused
            .then(|| window_layout(info.hwndFocus))
            .flatten()
            .or_else(|| window_layout(GetForegroundWindow()))
    }
}

struct Listener {
    keys: WindowsKeyboard,
    tx: Sender<Command>,
    paused: Arc<AtomicBool>,
    storage_blocked: Arc<AtomicBool>,
    ignore_software_input: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
}
thread_local! { static LISTENER: RefCell<Option<Listener>> = const { RefCell::new(None) }; }

fn stop_with_error(
    stop: &AtomicBool,
    status: &Mutex<CaptureStatus>,
    code: &'static str,
    message: String,
) {
    match status.lock() {
        Ok(mut status) => {
            // Preserve an earlier terminal error, especially a storage failure.
            if !stop.swap(true, Ordering::Relaxed) {
                status.listening = false;
                status.error_code = Some(code);
                status.error = Some(message);
            }
        }
        Err(_) => stop.store(true, Ordering::Relaxed),
    }
}

impl Listener {
    fn fail(&self, code: &'static str, message: String) {
        stop_with_error(&self.stop, &self.status, code, message);
    }
    fn enqueue(&self, device: Device, code: u32, shortcut: Option<crate::shortcuts::Chord>) {
        if !self.storage_blocked.load(Ordering::Relaxed)
            && !self.paused.load(Ordering::Relaxed)
            && !self.stop.load(Ordering::Relaxed)
            && self
                .tx
                .try_send(Command::Input(InputEvent {
                    shortcut,
                    ts_ms: chrono::Utc::now().timestamp_millis(),
                    device,
                    code,
                }))
                .is_err()
        {
            self.fail(
                "error.queue",
                "Input queue unavailable; recording stopped".into(),
            );
        }
    }
    fn input(&mut self, handle: HRAWINPUT) {
        let mut raw = RAWINPUT::default();
        let mut size = std::mem::size_of::<RAWINPUT>() as u32;
        // RAWINPUT is correctly aligned and large enough for keyboard/mouse packets.
        let read = unsafe {
            GetRawInputData(
                handle,
                RID_INPUT,
                &mut raw as *mut _ as *mut _,
                &mut size,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            )
        };
        if read == u32::MAX {
            // A stale/oversized packet is not a terminal device failure.
            // Continue listening; never permanently stop capture for one bad handle.
            eprintln!(
                "Skipped unreadable raw input packet: {}",
                std::io::Error::last_os_error()
            );
            return;
        }
        if read < std::mem::size_of::<RAWINPUTHEADER>() as u32 {
            return;
        }
        match raw.header.dwType {
            RIM_TYPEKEYBOARD => {
                if read
                    < (std::mem::size_of::<RAWINPUTHEADER>() + std::mem::size_of::<RAWKEYBOARD>())
                        as u32
                {
                    return;
                }
                let key = unsafe { raw.data.keyboard };
                if let Some(code) = self.keys.filtered_event(
                    raw.header.hDevice as usize,
                    key.MakeCode,
                    key.Flags,
                    key.VKey,
                    self.ignore_software_input.load(Ordering::Relaxed),
                ) {
                    if self.paused.load(Ordering::Relaxed)
                        || self.storage_blocked.load(Ordering::Relaxed)
                        || self.stop.load(Ordering::Relaxed)
                    {
                        return;
                    }
                    let shortcut = self.keys.shortcut_with_layout(
                        raw.header.hDevice as usize,
                        code,
                        Instant::now(),
                        |vk| unsafe { GetAsyncKeyState(i32::from(vk)) < 0 },
                        |_| {
                            let layout = foreground_layout();
                            crate::windows_input::shortcut_character_with_oem(
                                key.MakeCode,
                                key.Flags,
                                key.VKey,
                                |scan| {
                                    u16::try_from(unsafe {
                                        MapVirtualKeyExW(scan, MAPVK_VSC_TO_VK_EX, layout?)
                                    })
                                    .ok()
                                },
                                |vk| {
                                    Some(unsafe {
                                        MapVirtualKeyExW(u32::from(vk), MAPVK_VK_TO_CHAR, layout?)
                                    })
                                },
                            )
                        },
                    );
                    self.enqueue(Device::Keyboard, code, shortcut);
                }
            }
            RIM_TYPEMOUSE => {
                if read
                    < (std::mem::size_of::<RAWINPUTHEADER>() + std::mem::size_of::<RAWMOUSE>())
                        as u32
                {
                    return;
                }
                let flags = unsafe { raw.data.mouse.Anonymous.Anonymous.usButtonFlags };
                for code in mouse_buttons(flags) {
                    self.enqueue(Device::Mouse, code, None);
                }
            }
            _ => {}
        }
    }
}

// Delivered on the listener's message-loop thread. Covers secure-desktop UAC
// and foreground transitions into/out of elevated windows, where UIPI can
// swallow Raw Input releases without switching desktops. Packet-order chord
// tracking remains unchanged between these discontinuities.
unsafe extern "system" fn input_context_changed(
    _: HWINEVENTHOOK,
    event: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    LISTENER.with(|state| {
        if let Some(listener) = state.borrow_mut().as_mut() {
            if event == EVENT_SYSTEM_FOREGROUND {
                listener
                    .keys
                    .reconcile_foreground(|vk| unsafe { GetAsyncKeyState(i32::from(vk)) < 0 });
            } else {
                listener.keys.reset();
            }
        }
    });
}

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_INPUT {
        LISTENER.with(|state| {
            if let Some(listener) = state.borrow_mut().as_mut() {
                listener.input(lparam as HRAWINPUT);
            }
        });
    } else if message == WM_WTSSESSION_CHANGE
        && matches!(
            wparam as u32,
            WTS_SESSION_LOCK
                | WTS_SESSION_UNLOCK
                | WTS_CONSOLE_CONNECT
                | WTS_REMOTE_CONNECT
                | WTS_CONSOLE_DISCONNECT
                | WTS_REMOTE_DISCONNECT
        )
    {
        LISTENER.with(|state| {
            if let Some(listener) = state.borrow_mut().as_mut() {
                listener.keys.reset();
            }
        });
    } else if message == WM_INPUT_DEVICE_CHANGE && wparam == GIDC_REMOVAL as usize {
        LISTENER.with(|state| {
            if let Some(listener) = state.borrow_mut().as_mut() {
                listener.keys.remove_device(lparam as usize);
            }
        });
    }
    // Required cleanup for foreground raw input; never suppress other apps' input.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

pub(crate) fn spawn(
    tx: Sender<Command>,
    paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
) -> Result<JoinHandle<()>> {
    let storage_blocked = status
        .lock()
        .map_err(|_| anyhow::anyhow!("Capture status is unavailable"))?
        .storage_blocked
        .clone();
    let ignore_software_input = status
        .lock()
        .map_err(|_| anyhow::anyhow!("Capture status is unavailable"))?
        .ignore_software_input
        .clone();
    Ok(thread::Builder::new()
        .name("clickpulse-input".into())
        .spawn(move || {
            LISTENER.with(|slot| {
                *slot.borrow_mut() = Some(Listener {
                    keys: WindowsKeyboard::default(),
                    tx,
                    paused,
                    storage_blocked,
                    ignore_software_input,
                    stop: stop.clone(),
                    status: status.clone(),
                })
            });
            let result = listen(&stop, &status);
            if let Err(error) = result {
                stop_with_error(
                    &stop,
                    &status,
                    "error.capture",
                    format!("Windows input capture failed: {error:#}"),
                );
            }
            if let Ok(mut state) = status.lock() {
                state.listening = false;
            }
            LISTENER.with(|slot| *slot.borrow_mut() = None);
        })?)
}

fn listen(stop: &AtomicBool, status: &Mutex<CaptureStatus>) -> Result<()> {
    let class: Vec<u16> = "ClickPulse.RawInput"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let module = unsafe { GetModuleHandleW(null()) };
    let wc = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: module,
        lpszClassName: class.as_ptr(),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&wc) } == 0 {
        ensure!(
            unsafe { GetLastError() } == ERROR_CLASS_ALREADY_EXISTS,
            "Register input window: {}",
            std::io::Error::last_os_error()
        );
    }
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            module,
            null(),
        )
    };
    ensure!(
        !hwnd.is_null(),
        "Create input window: {}",
        std::io::Error::last_os_error()
    );
    struct InputWindow {
        hwnd: HWND,
        session_notifications: bool,
        context_hooks: [HWINEVENTHOOK; 2],
    }
    impl Drop for InputWindow {
        fn drop(&mut self) {
            unsafe {
                if self.session_notifications {
                    WTSUnRegisterSessionNotification(self.hwnd);
                }
                for hook in self.context_hooks {
                    if !hook.is_null() {
                        UnhookWinEvent(hook);
                    }
                }
                DestroyWindow(self.hwnd);
            }
        }
    }
    let session_notifications =
        unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) } != 0;
    if !session_notifications {
        // The service may not be ready at login; the desktop-switch hook also
        // resets swallowed releases.
        eprintln!(
            "Register session notifications failed: {}",
            std::io::Error::last_os_error()
        );
    }
    let mut input_window = InputWindow {
        hwnd,
        session_notifications,
        context_hooks: [null_mut(); 2],
    };
    for (hook, event) in input_window
        .context_hooks
        .iter_mut()
        .zip([EVENT_SYSTEM_DESKTOPSWITCH, EVENT_SYSTEM_FOREGROUND])
    {
        *hook = unsafe {
            SetWinEventHook(
                event,
                event,
                null_mut(),
                Some(input_context_changed),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.is_null() {
            // Optional defense: restricted systems must still count input.
            // Without this hook, lock/secure-desktop transitions can affect
            // shortcut counts until session notifications or stale-state
            // reconciliation clear a missing modifier release.
            // This API does not promise a meaningful GetLastError value.
            eprintln!(
                "Input context notification unavailable (event {event}); continuing Raw Input capture"
            );
        }
    }
    let devices = [2, 6].map(|usage| RAWINPUTDEVICE {
        usUsagePage: 1,
        usUsage: usage,
        dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
        hwndTarget: hwnd,
    });
    ensure!(
        unsafe {
            RegisterRawInputDevices(
                devices.as_ptr(),
                devices.len() as u32,
                std::mem::size_of::<RAWINPUTDEVICE>() as u32,
            )
        } != 0,
        "Register keyboard/mouse: {}",
        std::io::Error::last_os_error()
    );
    if let Ok(mut state) = status.lock()
        && !stop.load(Ordering::Relaxed)
    {
        state.listening = true;
        state.permission_needed = false;
        state.error = None;
        state.error_code = None;
    }
    let mut message = MSG::default();
    while !stop.load(Ordering::Relaxed) {
        while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            if message.message == WM_QUIT {
                return Ok(());
            }
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            if stop.load(Ordering::Relaxed) {
                break;
            }
        }
        if !stop.load(Ordering::Relaxed) {
            unsafe {
                MsgWaitForMultipleObjects(0, null(), 0, 100, QS_ALLINPUT);
            }
        }
    }
    // Drop the window on its owning thread before draining the storage queue.
    Ok(())
}
