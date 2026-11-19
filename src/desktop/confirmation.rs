//! Destructive or unsafe-location confirmation: Enter, Escape and close cancel.
use clickpulse::localization::Locale;

pub(super) fn confirm(
    window: Option<&tao::window::Window>,
    locale: Locale,
    title: &str,
    description: &str,
) -> bool {
    #[cfg(target_os = "macos")]
    {
        if let Some(window) = window {
            return macos_sheet(window, locale, title, description);
        }
        use core_foundation::{
            base::TCFType,
            string::{CFString, CFStringRef},
        };
        use std::{ffi::c_void, ptr::null};
        #[link(name = "CoreFoundation", kind = "framework")]
        unsafe extern "C" {
            fn CFUserNotificationDisplayAlert(
                timeout: f64,
                flags: usize,
                icon: *const c_void,
                sound: *const c_void,
                localization: *const c_void,
                header: CFStringRef,
                message: CFStringRef,
                default: CFStringRef,
                alternate: CFStringRef,
                other: CFStringRef,
                response: *mut usize,
            ) -> i32;
        }
        let title = CFString::new(title);
        let description = CFString::new(description);
        let cancel = CFString::new(&locale.text("action.cancel"));
        let proceed = CFString::new(&locale.text("action.continue"));
        let mut response = 3;
        let status = unsafe {
            CFUserNotificationDisplayAlert(
                0.0,
                2,
                null(),
                null(),
                null(),
                title.as_concrete_TypeRef(),
                description.as_concrete_TypeRef(),
                cancel.as_concrete_TypeRef(),
                proceed.as_concrete_TypeRef(),
                null(),
                &mut response,
            )
        };
        status == 0 && response & 3 == 1
    }
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowExtWindows;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IDOK, MB_DEFBUTTON2, MB_ICONWARNING, MB_OKCANCEL, MessageBoxW,
        };
        let _ = locale;
        let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        unsafe {
            MessageBoxW(
                window.map_or(std::ptr::null_mut(), |w| w.hwnd() as _),
                wide(description).as_ptr(),
                wide(title).as_ptr(),
                MB_OKCANCEL | MB_DEFBUTTON2 | MB_ICONWARNING,
            ) == IDOK
        }
    }
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        use tao::platform::unix::WindowExtUnix;
        let dialog = gtk::MessageDialog::builder()
            .modal(true)
            .message_type(gtk::MessageType::Warning)
            .text(title)
            .secondary_text(description)
            .build();
        if let Some(window) = window {
            dialog.set_transient_for(Some(window.gtk_window()));
        }
        dialog.add_action_widget(
            &gtk::Button::builder()
                .label(locale.text("action.cancel"))
                .can_default(true)
                .visible(true)
                .build(),
            gtk::ResponseType::Cancel,
        );
        dialog.add_action_widget(
            &gtk::Button::builder()
                .label(locale.text("action.continue"))
                .can_default(true)
                .visible(true)
                .build(),
            gtk::ResponseType::Accept,
        );
        dialog.set_default_response(gtk::ResponseType::Cancel);
        let accepted = dialog.run() == gtk::ResponseType::Accept;
        unsafe {
            dialog.destroy();
        }
        accepted
    }
}

/// Keep confirmations attached to the App when its window already exists.
/// Keep Return/Enter native; handle Escape without replacing the default key equivalent.
#[cfg(target_os = "macos")]
fn macos_sheet(
    window: &tao::window::Window,
    locale: Locale,
    title: &str,
    description: &str,
) -> bool {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{
        NSAlert, NSAlertSecondButtonReturn, NSAlertStyle, NSApplication, NSButtonCell, NSEvent,
        NSEventMask, NSWindow,
    };
    use objc2_foundation::NSString;
    use tao::platform::macos::WindowExtMacOS;
    let Some(main_thread) = MainThreadMarker::new() else {
        return false;
    };
    let alert = NSAlert::new(main_thread);
    alert.setAlertStyle(NSAlertStyle::Warning);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(description));
    let cancel = alert.addButtonWithTitle(&NSString::from_str(&locale.text("action.cancel")));
    let proceed = alert.addButtonWithTitle(&NSString::from_str(&locale.text("action.continue")));
    proceed.setKeyEquivalent(&NSString::from_str(""));
    let Some(cell) = cancel
        .cell()
        .and_then(|cell| cell.downcast::<NSButtonCell>().ok())
    else {
        return false;
    };
    alert.window().setDefaultButtonCell(Some(&cell));
    // A sheet has no window number until it is presented. Read it at delivery,
    // and leave Return/Enter to AppKit (including full keyboard access focus).
    let alert_window = alert.window();
    let cancel_button = cancel.clone();
    let keys = block2::StackBlock::new(move |event: std::ptr::NonNull<NSEvent>| {
        let key = unsafe { event.as_ref() };
        if key.windowNumber() == alert_window.windowNumber() && key.keyCode() == 53 {
            unsafe {
                cancel_button.performClick(None);
            }
            std::ptr::null_mut()
        } else {
            event.as_ptr()
        }
    });
    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &keys)
    };
    let completion = block2::StackBlock::new(move |result| {
        NSApplication::sharedApplication(main_thread).stopModalWithCode(result)
    });
    // The Tao window remains alive for the whole synchronous modal session.
    let parent = unsafe { &*window.ns_window().cast::<NSWindow>() };
    alert.beginSheetModalForWindow_completionHandler(parent, Some(&completion));
    let accepted = NSApplication::sharedApplication(main_thread).runModalForWindow(&alert.window())
        == NSAlertSecondButtonReturn;
    if let Some(monitor) = monitor {
        unsafe {
            NSEvent::removeMonitor(&monitor);
        }
    }
    accepted
}
