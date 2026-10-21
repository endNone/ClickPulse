//! Passive Windows Raw Input listener on a dedicated message-only window.
//! Read scan codes and button transitions only; no text or pointer coordinates.
use crate::{
    input::{Device, InputEvent, input_queue_closed},
    runtime::{CaptureStatus, Command},
    windows_input::{WindowsKeyboard, mouse_buttons},
};
use anyhow::{Result, ensure};
use chrono::Local;
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
};
use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{Input::*, WindowsAndMessaging::*},
};

struct Listener {
    keys: WindowsKeyboard,
    tx: Sender<Command>,
    paused: Arc<AtomicBool>,
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
    fn enqueue(&self, device: Device, code: u32) {
        if !self.paused.load(Ordering::Relaxed)
            && !self.stop.load(Ordering::Relaxed)
            && input_queue_closed(&self.tx.try_send(Command::Input(InputEvent {
                ts_ms: Local::now().timestamp_millis(),
                device,
                code,
            })))
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
            self.fail(
                "error.capture",
                format!("Read raw input failed: {}", std::io::Error::last_os_error()),
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
                if let Some(code) = self.keys.event(
                    raw.header.hDevice as usize,
                    key.MakeCode,
                    key.Flags,
                    key.VKey,
                ) {
                    self.enqueue(Device::Keyboard, code);
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
                    self.enqueue(Device::Mouse, code);
                }
            }
            _ => {}
        }
    }
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
    Ok(thread::Builder::new()
        .name("clickpulse-input".into())
        .spawn(move || {
            LISTENER.with(|slot| {
                *slot.borrow_mut() = Some(Listener {
                    keys: WindowsKeyboard::default(),
                    tx,
                    paused,
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
    struct InputWindow(HWND);
    impl Drop for InputWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    let _window = InputWindow(hwnd);
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
        unsafe {
            MsgWaitForMultipleObjects(0, null(), 0, 100, QS_ALLINPUT);
        }
    }
    // Drop the window on its owning thread before draining the storage queue.
    Ok(())
}
