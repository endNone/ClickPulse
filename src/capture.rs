//! Passive macOS event tap. Reads physical codes only; never reads text or cursor coordinates.
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
use chrono::Local;
use core_foundation::{
    base::TCFType,
    runloop::{CFRunLoop, kCFRunLoopCommonModes, kCFRunLoopDefaultMode},
};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult, EventField,
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
    fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    fn CGEventSourceFlagsState(state: i32) -> u64;
    fn CGEventTapEnable(tap: *const c_void, enable: bool);
}

pub fn request_permission() -> bool {
    unsafe { CGRequestListenEventAccess() }
}

pub fn has_permission() -> bool {
    unsafe { CGPreflightListenEventAccess() }
}

fn caps_lock_state() -> bool {
    unsafe { CGEventSourceFlagsState(0) & CGEventFlags::CGEventFlagAlphaShift.bits() != 0 }
}

// Caps Lock is a toggle. Count both directions from the event itself, rather
// than querying a global key state that can already reflect a later release.
fn caps_lock_changed(previous: &mut bool, flags: CGEventFlags) -> bool {
    let locked = flags.contains(CGEventFlags::CGEventFlagAlphaShift);
    std::mem::replace(previous, locked) != locked
}

fn stop_with_error(stop: &AtomicBool, status: &Mutex<CaptureStatus>) {
    if let Ok(mut state) = status.lock() {
        // Storage may have failed between the callback's stop check and send.
        // Keep that earlier terminal error rather than replacing it with queue failure.
        if !stop.swap(true, Ordering::Relaxed) {
            state.listening = false;
            state.error_code = Some("error.queue");
            state.error = Some("记录队列不可用，监听已停止。请重启应用。".into());
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
                let keys = RefCell::new(KeyTracker::default());
                let caps_lock = RefCell::new(caps_lock_state());
                let tap_ptr = Arc::new(AtomicUsize::new(0));
                let cb_ptr = tap_ptr.clone();
                let cb_status = status.clone();
                let cb_tx = tx.clone();
                let cb_paused = paused.clone();
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
                        let mut keys = keys.borrow_mut();
                        let input = match kind {
                            CGEventType::TapDisabledByTimeout
                            | CGEventType::TapDisabledByUserInput => {
                                keys.reset();
                                *caps_lock.borrow_mut() = caps_lock_state();
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
                                keys.press(code, repeat).then_some((Device::Keyboard, code))
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
                                if code == 57 {
                                    caps_lock_changed(
                                        &mut caps_lock.borrow_mut(),
                                        event.get_flags(),
                                    )
                                    .then_some((Device::Keyboard, code))
                                } else if unsafe { CGEventSourceKeyState(0, code as u16) } {
                                    keys.press(code, false).then_some((Device::Keyboard, code))
                                } else {
                                    keys.release(code);
                                    None
                                }
                            }
                            CGEventType::LeftMouseDown => Some((Device::Mouse, 1)),
                            CGEventType::RightMouseDown => Some((Device::Mouse, 2)),
                            CGEventType::OtherMouseDown => Some((
                                Device::Mouse,
                                event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER)
                                    as u32
                                    + 1,
                            )),
                            _ => None,
                        };
                        if !cb_paused.load(Ordering::Relaxed)
                            && !cb_stop.load(Ordering::Relaxed)
                            && let Some((device, code)) = input
                        {
                            let input = InputEvent {
                                ts_ms: Local::now().timestamp_millis(),
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
                        s.error = Some("无法启动监听，请检查输入监控权限并重启应用".into());
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
                        s.error = Some("无法建立输入监听，请重启应用".into());
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
    fn caps_lock_counts_ten_toggles_from_either_initial_state() {
        for initial in [false, true] {
            let mut previous = initial;
            let mut count = 0;
            for press in 0..10 {
                let locked = if press % 2 == 0 { !initial } else { initial };
                let flags = if locked {
                    CGEventFlags::CGEventFlagAlphaShift
                } else {
                    CGEventFlags::CGEventFlagNull
                };
                count += usize::from(caps_lock_changed(&mut previous, flags));
                // Duplicate/release notifications with unchanged lock state do not count.
                assert!(!caps_lock_changed(&mut previous, flags));
            }
            assert_eq!(count, 10);
        }
    }

    #[test]
    fn caps_lock_ignores_other_modifier_bits_and_resynchronizes_after_a_gap() {
        let mut previous = true;
        assert!(!caps_lock_changed(
            &mut previous,
            CGEventFlags::CGEventFlagAlphaShift | CGEventFlags::CGEventFlagShift
        ));
        // A disabled tap resynchronizes to the current state before accepting input.
        previous = false;
        assert!(!caps_lock_changed(
            &mut previous,
            CGEventFlags::CGEventFlagControl
        ));
        assert!(caps_lock_changed(
            &mut previous,
            CGEventFlags::CGEventFlagAlphaShift
        ));
    }

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
