//! Count physical Caps Lock presses separately from macOS lock/language state.
//! The HID value filter admits only keyboard usage 0x39, never text or other keys.
use crate::{
    input::{Device, InputEvent},
    runtime::{CaptureStatus, Command},
};
use anyhow::{Result, anyhow};
use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    base::{CFAllocatorRef, CFRelease, TCFType},
    dictionary::{CFDictionary, CFDictionaryRef},
    number::CFNumber,
    runloop::{CFRunLoop, CFRunLoopRef, kCFRunLoopCommonModes},
    string::{CFString, CFStringRef},
};
use crossbeam_channel::Sender;
use std::{
    cell::RefCell,
    collections::HashSet,
    ffi::c_void,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type HidRef = *mut c_void;
type ValueCallback = unsafe extern "C" fn(*mut c_void, i32, *mut c_void, HidRef);
type DeviceCallback = unsafe extern "C" fn(*mut c_void, i32, *mut c_void, HidRef);
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDManagerCreate(allocator: CFAllocatorRef, options: u32) -> HidRef;
    fn IOHIDManagerSetDeviceMatching(manager: HidRef, matching: CFDictionaryRef);
    fn IOHIDManagerSetInputValueMatching(manager: HidRef, matching: CFDictionaryRef);
    fn IOHIDManagerRegisterInputValueCallback(
        manager: HidRef,
        callback: Option<ValueCallback>,
        context: *mut c_void,
    );
    fn IOHIDManagerRegisterDeviceMatchingCallback(
        manager: HidRef,
        callback: Option<DeviceCallback>,
        context: *mut c_void,
    );
    fn IOHIDManagerRegisterDeviceRemovalCallback(
        manager: HidRef,
        callback: Option<DeviceCallback>,
        context: *mut c_void,
    );
    fn IOHIDManagerScheduleWithRunLoop(manager: HidRef, runloop: CFRunLoopRef, mode: CFStringRef);
    fn IOHIDManagerUnscheduleFromRunLoop(manager: HidRef, runloop: CFRunLoopRef, mode: CFStringRef);
    fn IOHIDManagerOpen(manager: HidRef, options: u32) -> i32;
    fn IOHIDManagerClose(manager: HidRef, options: u32) -> i32;
    fn IOHIDDeviceCopyMatchingElements(
        device: HidRef,
        matching: CFDictionaryRef,
        options: u32,
    ) -> CFArrayRef;
    fn IOHIDDeviceGetValue(device: HidRef, element: HidRef, value: *mut HidRef) -> i32;
    fn IOHIDValueGetElement(value: HidRef) -> HidRef;
    fn IOHIDValueGetIntegerValue(value: HidRef) -> isize;
    fn IOHIDElementGetDevice(element: HidRef) -> HidRef;
    fn IOHIDElementGetUsagePage(element: HidRef) -> u32;
    fn IOHIDElementGetUsage(element: HidRef) -> u32;
}

#[derive(Default)]
struct Presses {
    held: HashSet<usize>,
}
impl Presses {
    fn seed(&mut self, device: usize, down: bool) {
        self.held.remove(&device);
        if down {
            self.held.insert(device);
        }
    }
    fn update(&mut self, device: usize, down: bool) -> bool {
        if down {
            self.held.insert(device)
        } else {
            self.held.remove(&device);
            false
        }
    }
}

fn caps_filter() -> CFDictionary<CFString, CFNumber> {
    CFDictionary::from_CFType_pairs(&[
        (CFString::new("UsagePage"), CFNumber::from(7_i32)),
        (CFString::new("Usage"), CFNumber::from(0x39_i32)),
    ])
}
struct Context {
    presses: RefCell<Presses>,
    tx: Sender<Command>,
    paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
}

unsafe extern "C" fn matched(context: *mut c_void, result: i32, _: *mut c_void, device: HidRef) {
    if result != 0 || device.is_null() {
        return;
    }
    let context = unsafe { &*(context as *const Context) };
    // A key already held at attach/startup is not a new press.
    let filter = caps_filter();
    let elements =
        unsafe { IOHIDDeviceCopyMatchingElements(device, filter.as_concrete_TypeRef(), 0) };
    let mut down = false;
    if !elements.is_null() {
        for index in 0..unsafe { CFArrayGetCount(elements) } {
            let element = unsafe { CFArrayGetValueAtIndex(elements, index) } as HidRef;
            let mut value = ptr::null_mut();
            if unsafe { IOHIDDeviceGetValue(device, element, &mut value) } == 0 && !value.is_null()
            {
                down |= unsafe { IOHIDValueGetIntegerValue(value) } != 0;
            }
        }
        unsafe {
            CFRelease(elements.cast());
        }
    }
    context.presses.borrow_mut().seed(device as usize, down);
}
unsafe extern "C" fn removed(context: *mut c_void, _: i32, _: *mut c_void, device: HidRef) {
    let context = unsafe { &*(context as *const Context) };
    context.presses.borrow_mut().seed(device as usize, false);
}
unsafe extern "C" fn input(context: *mut c_void, result: i32, _: *mut c_void, value: HidRef) {
    if result != 0 || value.is_null() {
        return;
    }
    let element = unsafe { IOHIDValueGetElement(value) };
    if element.is_null()
        || unsafe { IOHIDElementGetUsagePage(element) } != 7
        || unsafe { IOHIDElementGetUsage(element) } != 0x39
    {
        return;
    }
    let device = unsafe { IOHIDElementGetDevice(element) };
    if device.is_null() {
        return;
    }
    let context = unsafe { &*(context as *const Context) };
    let pressed = context.presses.borrow_mut().update(
        device as usize,
        unsafe { IOHIDValueGetIntegerValue(value) } != 0,
    );
    if pressed && !context.paused.load(Ordering::Relaxed) && !context.stop.load(Ordering::Relaxed) {
        let event = InputEvent {
            ts_ms: chrono::Local::now().timestamp_millis(),
            device: Device::Keyboard,
            code: 57,
        };
        if context.tx.try_send(Command::Input(event)).is_err() {
            super::stop_with_error(&context.stop, &context.status);
        }
    }
}

pub(super) struct CapsLockCapture {
    manager: HidRef,
    runloop: CFRunLoop,
    // Stable address retained until callbacks have been unregistered and unscheduled.
    _context: Box<Context>,
}
impl CapsLockCapture {
    pub(super) fn start(
        tx: Sender<Command>,
        paused: Arc<AtomicBool>,
        stop: Arc<AtomicBool>,
        status: Arc<Mutex<CaptureStatus>>,
    ) -> Result<Self> {
        let manager = unsafe { IOHIDManagerCreate(ptr::null(), 0) };
        if manager.is_null() {
            return Err(anyhow!("Cannot create physical Caps Lock capture"));
        }
        let mut capture = Self {
            manager,
            runloop: CFRunLoop::get_current(),
            _context: Box::new(Context {
                presses: RefCell::default(),
                tx,
                paused,
                stop,
                status,
            }),
        };
        let context = (&mut *capture._context as *mut Context).cast();
        let keyboards = CFDictionary::from_CFType_pairs(&[
            (CFString::new("DeviceUsagePage"), CFNumber::from(1_i32)),
            (CFString::new("DeviceUsage"), CFNumber::from(6_i32)),
        ]);
        let filter = caps_filter();
        unsafe {
            IOHIDManagerSetDeviceMatching(manager, keyboards.as_concrete_TypeRef());
            IOHIDManagerSetInputValueMatching(manager, filter.as_concrete_TypeRef());
            IOHIDManagerRegisterDeviceMatchingCallback(manager, Some(matched), context);
            IOHIDManagerRegisterDeviceRemovalCallback(manager, Some(removed), context);
            IOHIDManagerRegisterInputValueCallback(manager, Some(input), context);
            IOHIDManagerScheduleWithRunLoop(
                manager,
                capture.runloop.as_concrete_TypeRef(),
                kCFRunLoopCommonModes,
            );
        }
        // Zero means shared, passive access. Never seize a keyboard or change its settings.
        let result = unsafe { IOHIDManagerOpen(manager, 0) };
        if result != 0 {
            return Err(anyhow!(
                "Cannot open physical Caps Lock capture ({result:#x})"
            ));
        }
        Ok(capture)
    }
}
impl Drop for CapsLockCapture {
    fn drop(&mut self) {
        unsafe {
            IOHIDManagerRegisterInputValueCallback(self.manager, None, ptr::null_mut());
            IOHIDManagerRegisterDeviceMatchingCallback(self.manager, None, ptr::null_mut());
            IOHIDManagerRegisterDeviceRemovalCallback(self.manager, None, ptr::null_mut());
            IOHIDManagerUnscheduleFromRunLoop(
                self.manager,
                self.runloop.as_concrete_TypeRef(),
                kCFRunLoopCommonModes,
            );
            IOHIDManagerClose(self.manager, 0);
            CFRelease(self.manager.cast());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ten_physical_presses_ignore_repeated_reports_and_releases() {
        let mut tracker = Presses::default();
        let mut count = 0;
        for _ in 0..10 {
            count += usize::from(tracker.update(1, true));
            assert!(!tracker.update(1, true));
            assert!(!tracker.update(1, false));
            assert!(!tracker.update(1, false));
        }
        assert_eq!(count, 10);
    }
    #[test]
    fn startup_held_keys_disconnect_and_separate_keyboards_do_not_share_state() {
        let mut tracker = Presses::default();
        tracker.seed(1, true);
        assert!(!tracker.update(1, true));
        assert!(tracker.update(2, true));
        assert!(!tracker.update(1, false));
        assert!(tracker.update(1, true));
        tracker.seed(1, false); // device removal
        assert!(tracker.update(1, true));
        assert!(!tracker.update(2, true));
    }
    #[test]
    fn paused_press_does_not_count_as_new_when_resuming_while_held() {
        let mut tracker = Presses::default();
        assert!(tracker.update(1, true)); // state advances even while delivery is paused
        assert!(!tracker.update(1, true));
        assert!(!tracker.update(1, false));
        assert!(tracker.update(1, true));
    }
}
