//! Native window geometry, decorations and display-scale transitions.
use super::UiEvent;
use std::sync::Arc;
use tao::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Position},
    event_loop::EventLoopWindowTarget,
};

pub(super) struct WindowLayout {
    pub(super) full_size: LogicalSize<f64>,
    pub(super) full_position: Option<Position>,
    pub(super) compact: bool,
    #[cfg(target_os = "windows")]
    pub(super) full_maximized: bool,
    #[cfg(target_os = "windows")]
    pub(super) region_size: Option<PhysicalSize<u32>>,
}
pub(super) const COMPACT_SIZE: LogicalSize<f64> = LogicalSize::new(320.0, 208.0);

pub(super) fn saved_full_position(position: PhysicalPosition<i32>, scale: f64) -> Position {
    #[cfg(target_os = "macos")]
    {
        // Cocoa's desktop coordinates are points. A mini window can move to a
        // different backing scale before we restore the full window position.
        position.to_logical::<f64>(scale).into()
    }
    #[cfg(target_os = "windows")]
    {
        let _ = scale;
        position.into()
    }
}

impl WindowLayout {
    pub(super) fn apply_scale_change(&self, scale_factor: f64, suggested: &mut PhysicalSize<u32>) {
        if self.compact {
            *suggested = COMPACT_SIZE.to_physical(scale_factor);
        }
    }
}

pub(super) fn fix_compact_size(window: &tao::window::Window) {
    window.set_min_inner_size(Some(COMPACT_SIZE));
    window.set_max_inner_size(Some(COMPACT_SIZE));
    window.set_inner_size(COMPACT_SIZE);
}
pub(super) fn present_window(
    _target: &EventLoopWindowTarget<UiEvent>,
    window: &Arc<tao::window::Window>,
    layout: &mut WindowLayout,
    preferences: &mut clickpulse::presentation::Preferences,
    compact: bool,
) {
    // Visibility and focus do not deminiaturize a Cocoa window. Restore it
    // before applying the requested mode, including when reopening from Dock.
    window.set_minimized(false);
    preferences.compact = compact;
    set_compact(window, layout, compact, preferences.pinned);
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::EventLoopWindowTargetExtMacOS;
        _target.show_application();
    }
    window.set_visible(true);
    window.set_focus();
}
pub(super) fn set_compact(
    window: &Arc<tao::window::Window>,
    layout: &mut WindowLayout,
    compact: bool,
    pinned: bool,
) {
    if compact && !layout.compact {
        #[cfg(target_os = "windows")]
        {
            layout.full_maximized = window.is_maximized();
            window.set_maximized(false);
        }
        layout.full_size = window.inner_size().to_logical(window.scale_factor());
        layout.full_position = window
            .outer_position()
            .ok()
            .map(|position| saved_full_position(position, window.scale_factor()));
        window.set_fullscreen(None);
        window.set_resizable(false);
        window.set_decorations(false);
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            // Tao's undecorated shadow keeps native frame insets, including
            // a top line. The region is relative to the outer HWND, whereas
            // WebView2 fills its client area. Remove those insets so their
            // origins match before sizing and clipping the mini window.
            window.set_undecorated_shadow(false);
        }
        fix_compact_size(window);
        if let Some(monitor) = window.current_monitor() {
            let origin = monitor.position();
            let size = monitor.size();
            let scale = monitor.scale_factor();
            window.set_outer_position(PhysicalPosition::new(
                origin.x + (f64::from(size.width) - 348.0 * scale).max(0.0) as i32,
                origin.y + (f64::from(size.height) - 326.0 * scale).max(0.0) as i32,
            ));
        }
    } else if !compact && layout.compact {
        window.set_max_inner_size(None::<LogicalSize<f64>>);
        window.set_resizable(true);
        window.set_decorations(cfg!(target_os = "macos"));
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            window.set_undecorated_shadow(true);
        }
        restore_full_window(window, layout.full_size, layout.full_position);
        #[cfg(target_os = "windows")]
        window.set_maximized(layout.full_maximized);
    }
    layout.compact = compact;
    #[cfg(target_os = "windows")]
    update_windows_region(window, layout);
    window.set_always_on_top(compact && pinned);
}

#[cfg(target_os = "windows")]
pub(super) fn update_windows_region(window: &tao::window::Window, layout: &mut WindowLayout) {
    use tao::platform::windows::WindowExtWindows;
    use windows_sys::Win32::Graphics::Gdi::{CreateRoundRectRgn, DeleteObject, SetWindowRgn};
    let hwnd = window.hwnd() as windows_sys::Win32::Foundation::HWND;
    if !layout.compact {
        if layout.region_size.take().is_some() {
            unsafe {
                SetWindowRgn(hwnd, std::ptr::null_mut(), 1);
            }
        }
        return;
    }
    let size = window.inner_size();
    if size.width == 0 || size.height == 0 || layout.region_size == Some(size) {
        return;
    }
    let diameter = (28.0 * window.scale_factor()).round() as i32;
    // Windows WebView2 is opaque: clip the entire HWND instead of exposing
    // a white host surface through CSS-only transparent rounded corners.
    let region = unsafe {
        CreateRoundRectRgn(
            0,
            0,
            size.width as i32 + 1,
            size.height as i32 + 1,
            diameter,
            diameter,
        )
    };
    if region.is_null() {
        return;
    }
    if unsafe { SetWindowRgn(hwnd, region, 1) } == 0 {
        // SetWindowRgn takes ownership only on success.
        unsafe {
            DeleteObject(region);
        }
    } else {
        layout.region_size = Some(size);
    }
}

pub(super) fn restore_full_window(
    window: &Arc<tao::window::Window>,
    size: LogicalSize<f64>,
    position: Option<Position>,
) {
    let apply = move |window: &tao::window::Window| {
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::WindowExtMacOS;
            window.set_fullsize_content_view(true);
            window.set_titlebar_transparent(true);
        }
        window.set_min_inner_size(Some(LogicalSize::new(900.0, 680.0)));
        window.set_inner_size(size);
        if let Some(position) = position {
            window.set_outer_position(position);
        }
    };
    #[cfg(target_os = "macos")]
    {
        // Tao queues decoration changes and drops FullSizeContentView. Restore
        // the integrated titlebar on that same main queue after those changes.
        let window = Arc::clone(window);
        dispatch2::DispatchQueue::main().exec_async(move || {
            if window.is_decorated() {
                apply(&window);
            }
        });
    }
    #[cfg(target_os = "windows")]
    apply(window);
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use tao::{
        event_loop::EventLoopBuilder,
        platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows},
        window::WindowBuilder,
    };
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, RECT},
        Graphics::Gdi::{ClientToScreen, ERROR, GetWindowRgnBox},
        UI::WindowsAndMessaging::{GetClientRect, GetWindowRect},
    };

    #[test]
    fn mini_client_fills_rounded_window_and_full_view_restores_native_frame() {
        let mut builder = EventLoopBuilder::<UiEvent>::with_user_event();
        builder.with_any_thread(true);
        let event_loop = builder.build();
        event_loop.set_device_event_filter(tao::event_loop::DeviceEventFilter::Always);
        let window = Arc::new(
            WindowBuilder::new()
                .with_visible(false)
                .with_decorations(false)
                .with_inner_size(LogicalSize::new(960.0, 720.0))
                .build(&event_loop)
                .unwrap(),
        );
        let mut layout = WindowLayout {
            full_size: window.inner_size().to_logical(window.scale_factor()),
            full_position: None,
            compact: false,
            full_maximized: false,
            region_size: None,
        };
        let hwnd = window.hwnd() as HWND;
        assert!(window.has_undecorated_shadow());

        // Reentering mini mode also exercises invalidation of its old region.
        for _ in 0..2 {
            set_compact(&window, &mut layout, true, false);
            assert!(!window.has_undecorated_shadow());
            assert!(!window.is_resizable());
            assert_eq!(
                window.inner_size(),
                COMPACT_SIZE.to_physical::<u32>(window.scale_factor())
            );
            let mut outer = RECT::default();
            let mut client = RECT::default();
            let mut origin = POINT::default();
            let mut region = RECT::default();
            unsafe {
                assert_ne!(GetWindowRect(hwnd, &mut outer), 0);
                assert_ne!(GetClientRect(hwnd, &mut client), 0);
                assert_ne!(ClientToScreen(hwnd, &mut origin), 0);
                assert_ne!(GetWindowRgnBox(hwnd, &mut region), ERROR);
            }
            assert_eq!((origin.x, origin.y), (outer.left, outer.top));
            assert_eq!(client.right, outer.right - outer.left);
            assert_eq!(client.bottom, outer.bottom - outer.top);
            assert_eq!((region.left, region.top), (0, 0));

            set_compact(&window, &mut layout, false, false);
            assert!(window.has_undecorated_shadow());
            assert!(window.is_resizable());
            assert!(layout.region_size.is_none());
            unsafe {
                assert_eq!(GetWindowRgnBox(hwnd, &mut region), ERROR);
            }
        }
    }
}
