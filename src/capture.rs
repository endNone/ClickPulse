//! Physical counts are independent of layout. Only eligible shortcuts consult a layout snapshot.
mod layout;
pub use layout::initialize_shortcut_layout;
mod caps_lock;
use std::{
    cell::RefCell,
    ffi::c_void,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::Result;
use core_foundation::{
    base::TCFType,
    runloop::{CFRunLoop, kCFRunLoopCommonModes, kCFRunLoopDefaultMode},
};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult, EventField,
};
use crossbeam_channel::Sender;

use crate::{
    input::{Device, InputEvent, KeyTracker},
    runtime::{CaptureStatus, Command},
};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
    fn CGEventTapEnable(tap: *const c_void, enable: bool);
}

/// Interpret the modifier flags and physical keycode of a keyboard press.
pub fn keyboard_shortcut(code: u32, flags: u64) -> Option<crate::shortcuts::Chord> {
    let modifiers = crate::shortcuts::mac_flags(flags);
    crate::shortcuts::Chord::with_layout(code, modifiers, "macos", |key| {
        layout::character(key, modifiers)
    })
}

pub fn request_permission() -> bool {
    unsafe { CGRequestListenEventAccess() }
}

pub fn has_permission() -> bool {
    unsafe { CGPreflightListenEventAccess() }
}

fn stop_with_error(stop: &AtomicBool, status: &Mutex<CaptureStatus>) {
    if let Ok(mut state) = status.lock() {
        // Storage may have failed between the callback's stop check and send.
        // Keep that earlier terminal error rather than replacing it with queue failure.
        if !stop.swap(true, Ordering::Relaxed) {
            state.listening = false;
            state.error_code = Some("error.queue");
            state.error = Some(
                "The recording queue is unavailable. Capture stopped; restart the app.".into(),
            );
        }
    } else {
        stop.store(true, Ordering::Relaxed);
    }
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
            while !stop.load(Ordering::Relaxed) {
                if !unsafe { CGPreflightListenEventAccess() } {
                    if let Ok(mut s) = status.lock()
                        && !stop.load(Ordering::Relaxed)
                    {
                        s.permission_needed = true;
                        s.listening = false;
                        if s.error_code == Some("error.capture") {
                            s.error = None;
                            s.error_code = None;
                        }
                    }
                    thread::sleep(Duration::from_millis(300));
                    continue;
                }
                // Independent state prevents a generated release from clearing
                // a physically held key, including when the setting changes.
                let keys = RefCell::new([KeyTracker::default(), KeyTracker::default()]);
                let _caps_lock = match caps_lock::CapsLockCapture::start(
                    tx.clone(), paused.clone(), stop.clone(), status.clone(),
                ) {
                    Ok(capture) => capture,
                    Err(error) => {
                        if let Ok(mut state) = status.lock()
                            && !stop.load(Ordering::Relaxed)
                        {
                            state.listening = false;
                            state.permission_needed = !has_permission();
                            state.error_code = Some("error.capture");
                            state.error = Some(error.to_string());
                        }
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                };
                let tap_ptr = Arc::new(AtomicUsize::new(0));
                let cb_ptr = tap_ptr.clone();
                let cb_status = status.clone();
                let cb_tx = tx.clone();
                let cb_paused = paused.clone();
                let cb_storage_blocked = storage_blocked.clone();
                let cb_ignore = ignore_software_input.clone();
                let cb_stop = stop.clone();
                let tap = CGEventTap::new(
                    CGEventTapLocation::Session,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::ListenOnly,
                    vec![
                        CGEventType::KeyDown,
                        CGEventType::KeyUp,
                        CGEventType::FlagsChanged,
                        CGEventType::LeftMouseDown,
                        CGEventType::RightMouseDown,
                        CGEventType::OtherMouseDown,
                    ],
                    move |_, kind, event| {
                        let generated = event.get_integer_value_field(EventField::EVENT_SOURCE_UNIX_PROCESS_ID) != 0;
                        let mut trackers = keys.borrow_mut();
                        if matches!(kind, CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput) {
                            for keys in trackers.iter_mut() { keys.reset(); }
                        }
                        let keys = &mut trackers[usize::from(generated)];
                        let input = match kind {
                            CGEventType::TapDisabledByTimeout
                            | CGEventType::TapDisabledByUserInput => {
                                keys.reset();
                                let ptr = cb_ptr.load(Ordering::Relaxed) as *const c_void;
                                if !ptr.is_null() {
                                    unsafe {
                                        CGEventTapEnable(ptr, true);
                                    }
                                }
                                None
                            }
                            CGEventType::KeyDown => {
                                let code = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                    as u32;
                                let repeat = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT)
                                    != 0;
                                // Caps Lock comes only from its physical HID press stream.
                                (code != 57 && keys.press(code, repeat))
                                    .then_some((Device::Keyboard, code))
                            }
                            CGEventType::KeyUp => {
                                keys.release(
                                    event
                                        .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                        as u32,
                                );
                                None
                            }
                            CGEventType::FlagsChanged => {
                                let code = event
                                    .get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE)
                                    as u32;
                                keys.mac_modifier(code, event.get_flags().bits())
                                    .then_some((Device::Keyboard, code))
                            }
                            CGEventType::LeftMouseDown => Some((Device::Mouse, 1)),
                            CGEventType::RightMouseDown => Some((Device::Mouse, 2)),
                            CGEventType::OtherMouseDown => mouse_button_code(event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER)).map(|code| (Device::Mouse, code)),
                            _ => None,
                        };
                        if !cb_storage_blocked.load(Ordering::Relaxed)
                            && !(generated && cb_ignore.load(Ordering::Relaxed))
                            && !cb_paused.load(Ordering::Relaxed)
                            && !cb_stop.load(Ordering::Relaxed)
                            && let Some((device, code)) = input
                        {
                            let input = InputEvent {
                                shortcut: (device == Device::Keyboard).then(|| keyboard_shortcut(code, event.get_flags().bits())).flatten(),
                                ts_ms: chrono::Utc::now().timestamp_millis(),
                                device,
                                code,
                            };
                            if cb_tx.try_send(Command::Input(input)).is_err() {
                                stop_with_error(&cb_stop, &cb_status);
                            }
                        }
                        CallbackResult::Keep
                    },
                );
                let Ok(tap) = tap else {
                    if let Ok(mut s) = status.lock()
                        && !stop.load(Ordering::Relaxed)
                    {
                        s.listening = false;
                        s.permission_needed = true;
                        s.error_code = Some("error.capture");
                        s.error = Some("Cannot start capture; check Input Monitoring permission and restart the app".into());
                    }
                    thread::sleep(Duration::from_millis(500));
                    continue;
                };
                tap_ptr.store(
                    tap.mach_port().as_concrete_TypeRef() as usize,
                    Ordering::Relaxed,
                );
                let Ok(source) = tap.mach_port().create_runloop_source(0) else {
                    if let Ok(mut s) = status.lock()
                        && !stop.load(Ordering::Relaxed)
                    {
                        s.error_code = Some("error.capture");
                        s.error = Some("Cannot create the input event source; restart the app".into());
                    }
                    break;
                };
                let runloop = CFRunLoop::get_current();
                runloop.add_source(&source, unsafe { kCFRunLoopCommonModes });
                tap.enable();
                if let Ok(mut s) = status.lock()
                    && !stop.load(Ordering::Relaxed)
                {
                    s.permission_needed = false;
                    s.listening = true;
                    s.error = None;
                    s.error_code = None;
                }
                let mut permission_check = Instant::now();
                while !stop.load(Ordering::Relaxed) {
                    CFRunLoop::run_in_mode(
                        unsafe { kCFRunLoopDefaultMode },
                        Duration::from_millis(200),
                        false,
                    );
                    if permission_check.elapsed() >= Duration::from_secs(1) {
                        permission_check = Instant::now();
                        if !has_permission() {
                            unsafe {
                                CGEventTapEnable(
                                    tap.mach_port().as_concrete_TypeRef() as *const c_void,
                                    false,
                                );
                            }
                            if let Ok(mut s) = status.lock()
                                && !stop.load(Ordering::Relaxed)
                            {
                                s.listening = false;
                                s.permission_needed = true;
                            }
                            break;
                        }
                    }
                }
                runloop.remove_source(&source, unsafe { kCFRunLoopCommonModes });
                drop(tap);
            }
            if let Ok(mut s) = status.lock() {
                s.listening = false;
            }
        })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_failure_preserves_an_earlier_storage_error() {
        let stop = AtomicBool::new(true);
        let status = Mutex::new(CaptureStatus {
            error_code: Some("error.storage"),
            error: Some("simulated disk failure".into()),
            ..Default::default()
        });
        stop_with_error(&stop, &status);
        let status = status.lock().unwrap();
        assert_eq!(status.error_code, Some("error.storage"));
        assert_eq!(status.error.as_deref(), Some("simulated disk failure"));
    }

    #[test]
    fn queue_failure_stops_capture_and_reports_the_initial_error() {
        let stop = AtomicBool::new(false);
        let status = Mutex::new(CaptureStatus {
            listening: true,
            ..Default::default()
        });
        stop_with_error(&stop, &status);
        assert!(stop.load(Ordering::Relaxed));
        let status = status.lock().unwrap();
        assert!(!status.listening);
        assert_eq!(status.error_code, Some("error.queue"));
    }
}

fn mouse_button_code(number: i64) -> Option<u32> {
    u32::try_from(number).ok().and_then(|n| n.checked_add(1))
}

#[cfg(test)]
mod button_tests {
    #[test]
    fn native_mouse_button_numbers_are_checked() {
        for (native, expected) in [
            (-1, None),
            (i64::MAX, None),
            (u32::MAX as i64, None),
            (2, Some(3)),
            (4, Some(5)),
        ] {
            assert_eq!(super::mouse_button_code(native), expected);
        }
    }
}
