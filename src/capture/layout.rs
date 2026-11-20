//! Text Input Sources is main-thread-only in a GUI process. The tap only reads
//! owned, aligned Unicode layout tables, never a TIS object or an input method.
use crate::shortcuts::{META, SHIFT, us_character_key};
use core_foundation_sys::{
    base::{CFRelease, CFTypeRef},
    data::{CFDataGetBytePtr, CFDataGetLength},
    dictionary::CFDictionaryRef,
    notification_center::*,
    string::CFStringRef,
};
use objc2::MainThreadMarker;
use std::{
    ffi::{c_ulong, c_void},
    ptr,
    sync::{Once, RwLock},
};

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyUnicodeKeyLayoutData: CFStringRef;
    static kTISNotifySelectedKeyboardInputSourceChanged: CFStringRef;
    fn TISCopyCurrentKeyboardLayoutInputSource() -> CFTypeRef;
    fn TISCopyCurrentASCIICapableKeyboardLayoutInputSource() -> CFTypeRef;
    fn TISGetInputSourceProperty(source: CFTypeRef, property: CFStringRef) -> CFTypeRef;
    fn LMGetKbdType() -> u8;
    fn UCKeyTranslate(
        layout: *const u32,
        key: u16,
        action: u16,
        modifiers: u32,
        keyboard: u32,
        options: u32,
        dead_state: *mut u32,
        capacity: c_ulong,
        length: *mut c_ulong,
        output: *mut u16,
    ) -> i32;
}

struct LayoutSnapshot {
    selected: Option<Box<[u32]>>,
    ascii: Option<Box<[u32]>>,
    keyboard_type: u32,
}

static LAYOUT: RwLock<Option<LayoutSnapshot>> = RwLock::new(None);
static OBSERVER: Once = Once::new();

/// Register once for the lifetime of the AppKit event loop, before capture starts.
pub fn initialize_shortcut_layout(main: MainThreadMarker) {
    OBSERVER.call_once(|| unsafe {
        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDistributedCenter(),
            ptr::addr_of!(LAYOUT).cast(),
            changed,
            kTISNotifySelectedKeyboardInputSourceChanged,
            ptr::null(),
            CFNotificationSuspensionBehaviorDeliverImmediately,
        );
    });
    refresh(main);
}

extern "C" fn changed(
    _: CFNotificationCenterRef,
    _: *mut c_void,
    _: CFStringRef,
    _: *const c_void,
    _: CFDictionaryRef,
) {
    if let Some(main) = MainThreadMarker::new() {
        refresh(main);
    } else {
        dispatch2::DispatchQueue::main().exec_async(|| {
            if let Some(main) = MainThreadMarker::new() {
                refresh(main);
            }
        });
    }
}

fn copy_layout(
    _: MainThreadMarker,
    copy_source: unsafe extern "C" fn() -> CFTypeRef,
) -> Option<Box<[u32]>> {
    unsafe {
        let source = copy_source();
        if source.is_null() {
            return None;
        }
        let data: core_foundation_sys::data::CFDataRef =
            TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData).cast();
        let bytes = if data.is_null() {
            0
        } else {
            CFDataGetLength(data)
        };
        let layout = (bytes > 0).then(|| {
            let bytes = bytes as usize;
            // UCKeyboardLayout contains 32-bit offsets. Preserve alignment and copy
            // exactly the CFData length before releasing the source that owns it.
            let mut words = vec![0u32; bytes.div_ceil(4)].into_boxed_slice();
            ptr::copy_nonoverlapping(CFDataGetBytePtr(data), words.as_mut_ptr().cast(), bytes);
            words
        });
        CFRelease(source);
        layout
    }
}

fn refresh(main: MainThreadMarker) {
    let next = LayoutSnapshot {
        selected: copy_layout(main, TISCopyCurrentKeyboardLayoutInputSource),
        ascii: copy_layout(main, TISCopyCurrentASCIICapableKeyboardLayoutInputSource),
        keyboard_type: u32::from(unsafe { LMGetKbdType() }),
    };
    if let Ok(mut layout) = LAYOUT.write() {
        *layout = Some(next);
    }
}

// A French number-row key can produce punctuation on its base layer and a
// digit with Shift. Preserve that digit in shortcuts such as Command-Shift-4.
// Other shifted shortcuts still use the base key, e.g. Command-Shift-=.
fn shortcut_layer(key: u32, modifiers: u8, lookup: impl Fn(u32) -> Option<char>) -> Option<char> {
    let command = u32::from(modifiers & META != 0);
    if modifiers & SHIFT != 0
        && matches!(key, 18..=23 | 25..=26 | 28..=29)
        && let Some(digit) = lookup(command | 2).filter(char::is_ascii_digit)
    {
        return Some(digit);
    }
    lookup(command)
}

pub(super) fn character(key: u32, modifiers: u8) -> Option<char> {
    let snapshot = LAYOUT.read().ok()?;
    let snapshot = snapshot.as_ref()?;
    let translate = |layout: &[u32]| {
        shortcut_layer(key, modifiers, |layer| {
            let mut output = [0u16; 4];
            let mut dead_state = 0u32;
            // Carbon declares UniCharCount as unsigned long, including the out parameter.
            let mut length: c_ulong = 0;
            // keyDown, Carbon modifier byte, kUCKeyTranslateNoDeadKeysMask.
            let status = unsafe {
                UCKeyTranslate(
                    layout.as_ptr(),
                    key as u16,
                    0,
                    layer,
                    snapshot.keyboard_type,
                    1,
                    &mut dead_state,
                    output.len() as c_ulong,
                    &mut length,
                    output.as_mut_ptr(),
                )
            };
            if status != 0 || length != 1 {
                return None;
            }
            char::from_u32(u32::from(output[0])).filter(|&ch| us_character_key(ch).is_some())
        })
    };
    snapshot
        .selected
        .as_deref()
        .and_then(translate)
        .or_else(|| snapshot.ascii.as_deref().and_then(translate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifted_number_row_uses_the_digit_when_the_layout_requires_shift() {
        let french_four = |layer| match layer {
            3 => Some('4'),
            1 => Some('\''),
            _ => panic!(),
        };
        assert_eq!(shortcut_layer(21, META | SHIFT, french_four), Some('4'));
        assert_eq!(shortcut_layer(21, META, french_four), Some('\''));
        // US Shift+4 resolves to the same digit via the base layer.
        assert_eq!(
            shortcut_layer(21, META | SHIFT, |layer| Some(if layer == 3 {
                '$'
            } else {
                '4'
            })),
            Some('4')
        );
        assert_eq!(
            shortcut_layer(24, META | SHIFT, |layer| {
                assert_eq!(layer, 1);
                Some('=')
            }),
            Some('=')
        );
    }
}
