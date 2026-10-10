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
    pub(super) region: Option<CompactRegion>,
}
pub(super) const COMPACT_SIZE: LogicalSize<f64> = LogicalSize::new(320.0, 208.0);

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CompactRegion {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    diameter: i32,
}

#[cfg(any(target_os = "windows", test))]
impl CompactRegion {
    fn from_client(
        size: PhysicalSize<u32>,
        client_origin: PhysicalPosition<i32>,
        window_origin: PhysicalPosition<i32>,
        scale: f64,
    ) -> Option<Self> {
        if size.width == 0 || size.height == 0 || !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let left = client_origin.x.checked_sub(window_origin.x)?;
        let top = client_origin.y.checked_sub(window_origin.y)?;
        Some(Self {
            left,
            top,
            right: left.checked_add(i32::try_from(size.width).ok()?)?,
            bottom: top.checked_add(i32::try_from(size.height).ok()?)?,
            diameter: (28.0 * scale).round() as i32,
        })
    }
}

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
            // Tao's undecorated shadow reserves native client insets. A fixed
            // rounded widget must fill its host without those frame margins.
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
    use windows_sys::Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{ClientToScreen, CreateRoundRectRgn, DeleteObject, SetWindowRgn},
        UI::WindowsAndMessaging::{GetClientRect, GetWindowRect},
    };
    let hwnd = window.hwnd() as windows_sys::Win32::Foundation::HWND;
    if !layout.compact {
        if layout.region.is_some() && unsafe { SetWindowRgn(hwnd, std::ptr::null_mut(), 1) } != 0 {
            layout.region = None;
        }
        return;
    }
    let mut client = RECT::default();
    let mut outer = RECT::default();
    let mut origin = POINT::default();
    if unsafe { GetClientRect(hwnd, &mut client) } == 0
        || unsafe { GetWindowRect(hwnd, &mut outer) } == 0
        || unsafe { ClientToScreen(hwnd, &mut origin) } == 0
    {
        return;
    }
    let Some(bounds) = CompactRegion::from_client(
        PhysicalSize::new(
            (client.right - client.left).max(0) as u32,
            (client.bottom - client.top).max(0) as u32,
        ),
        PhysicalPosition::new(origin.x, origin.y),
        PhysicalPosition::new(outer.left, outer.top),
        window.scale_factor(),
    ) else {
        return;
    };
    if layout.region == Some(bounds) {
        return;
    }
    // SetWindowRgn uses outer-window coordinates, while WebView2 fills the
    // client area. Clip only that client rectangle, including any native inset
    // during a frame/DPI transition, and cache its offset and corner diameter.
    let region = unsafe {
        CreateRoundRectRgn(
            bounds.left,
            bounds.top,
            bounds.right,
            bounds.bottom,
            bounds.diameter,
            bounds.diameter,
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
        layout.region = Some(bounds);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_widget_excludes_native_frame_without_cropping_client_pixels() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let size = COMPACT_SIZE.to_physical(scale);
            // A monitor left of the primary desktop and the native shadow
            // insets shown in the reported Windows mini-window screenshot.
            let outer = PhysicalPosition::new(-1200, -200);
            for inset in [(0, 0), (8, 1), (12, 2)] {
                let client = PhysicalPosition::new(outer.x + inset.0, outer.y + inset.1);
                let bounds = CompactRegion::from_client(size, client, outer, scale).unwrap();
                assert_eq!((bounds.left, bounds.top), inset);
                assert_eq!(bounds.right - bounds.left, size.width as i32);
                assert_eq!(bounds.bottom - bounds.top, size.height as i32);
                assert_eq!(bounds.diameter, (28.0 * scale).round() as i32);
            }
        }
    }

    #[test]
    fn region_cache_changes_when_frame_insets_or_dpi_change_at_the_same_size() {
        let size = PhysicalSize::new(480, 312);
        let outer = PhysicalPosition::new(100, 100);
        let plain = CompactRegion::from_client(size, outer, outer, 1.5).unwrap();
        let inset =
            CompactRegion::from_client(size, PhysicalPosition::new(108, 101), outer, 1.5).unwrap();
        let rescaled = CompactRegion::from_client(size, outer, outer, 2.0).unwrap();
        assert_ne!(plain, inset);
        assert_ne!(plain, rescaled);
        let moved = PhysicalPosition::new(-500, 300);
        assert_eq!(
            plain,
            CompactRegion::from_client(size, moved, moved, 1.5).unwrap()
        );
    }

    #[test]
    fn invalid_client_geometry_does_not_create_a_native_region() {
        let origin = PhysicalPosition::new(0, 0);
        for size in [
            PhysicalSize::new(0, 208),
            PhysicalSize::new(320, 0),
            PhysicalSize::new(u32::MAX, 208),
        ] {
            assert!(CompactRegion::from_client(size, origin, origin, 1.0).is_none());
        }
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                CompactRegion::from_client(COMPACT_SIZE.to_physical(1.0), origin, origin, scale)
                    .is_none()
            );
        }
    }
}

#[cfg(all(test, target_os = "windows"))]
mod native_windows_tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoopBuilder},
        platform::{
            run_return::EventLoopExtRunReturn,
            windows::{EventLoopBuilderExtWindows, WindowExtWindows},
        },
        window::WindowBuilder,
    };
    use windows_sys::Win32::{
        Foundation::{HWND, POINT, RECT},
        Graphics::Gdi::{ClientToScreen, ERROR, GetWindowRgnBox},
        UI::WindowsAndMessaging::{GetClientRect, GetWindowRect},
    };
    use wry::{PageLoadEvent, WebContext, WebView, WebViewBuilder, WebViewExtWindows};

    fn assert_webview_fills_client(window: &tao::window::Window, webview: &WebView) {
        let hwnd = window.hwnd() as HWND;
        let mut client = RECT::default();
        let mut origin = POINT::default();
        let mut container = RECT::default();
        let mut container_hwnd = Default::default();
        unsafe {
            assert_ne!(GetClientRect(hwnd, &mut client), 0);
            assert_ne!(ClientToScreen(hwnd, &mut origin), 0);
            webview
                .controller()
                .ParentWindow(&mut container_hwnd)
                .unwrap();
            assert_ne!(GetWindowRect(container_hwnd.0 as HWND, &mut container), 0);
        }
        let bounds = webview.bounds().unwrap();
        let scale = window.scale_factor();
        assert_eq!(
            bounds.position.to_physical::<i32>(scale),
            PhysicalPosition::new(0, 0)
        );
        assert_eq!(
            bounds.size.to_physical::<i32>(scale),
            PhysicalSize::new(client.right, client.bottom)
        );
        assert_eq!((container.left, container.top), (origin.x, origin.y));
        assert_eq!(container.right - container.left, client.right);
        assert_eq!(container.bottom - container.top, client.bottom);
    }

    #[test]
    fn mini_webview_fills_rounded_window_and_full_view_restores_native_frame() {
        let mut builder = EventLoopBuilder::<()>::with_user_event();
        builder.with_any_thread(true);
        let mut event_loop = builder.build();
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
            region: None,
        };
        let hwnd = window.hwnd() as HWND;
        assert!(window.has_undecorated_shadow());

        // Match --mini startup: apply the native frame before WebView2 exists.
        // A bare HWND test cannot catch offsets introduced by its child window.
        set_compact(&window, &mut layout, true, false);
        let profile = tempfile::tempdir().unwrap();
        let mut context = WebContext::new(Some(profile.path().join("webview")));
        let on_load = event_loop.create_proxy();
        let webview = WebViewBuilder::new_with_web_context(&mut context)
            .with_html("<!doctype html><style>body{margin:0;background:#102542}</style>")
            .with_on_page_load_handler(move |event, _| {
                if matches!(event, PageLoadEvent::Finished) {
                    let _ = on_load.send_event(());
                }
            })
            .build(window.as_ref())
            .expect("native WebView geometry test requires the WebView2 Runtime");

        let deadline = Instant::now() + Duration::from_secs(15);
        let mut loaded = false;
        let mut transition = 0;
        event_loop.run_return(|event, _, flow| {
            if matches!(*flow, ControlFlow::ExitWithCode(_)) {
                return;
            }
            if matches!(event, Event::UserEvent(())) {
                loaded = true;
            }
            *flow = if loaded {
                ControlFlow::Poll
            } else {
                ControlFlow::WaitUntil(deadline)
            };
            if !matches!(event, Event::MainEventsCleared) {
                return;
            }
            assert!(Instant::now() < deadline, "WebView geometry test timed out");
            if !loaded {
                return;
            }
            // Run the native message loop between each transition so Wry's
            // WM_SIZE handler has resized both the controller and its HWND.
            assert_webview_fills_client(&window, &webview);
            let mut region = RECT::default();
            if layout.compact {
                assert!(!window.has_undecorated_shadow());
                assert!(!window.is_resizable());
                assert_eq!(
                    window.inner_size(),
                    COMPACT_SIZE.to_physical::<u32>(window.scale_factor())
                );
                let mut outer = RECT::default();
                let mut client = RECT::default();
                let mut origin = POINT::default();
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
            } else {
                assert!(window.has_undecorated_shadow());
                assert!(window.is_resizable());
                assert!(layout.region.is_none());
                unsafe {
                    assert_eq!(GetWindowRgnBox(hwnd, &mut region), ERROR);
                }
            }

            if transition == 4 {
                *flow = ControlFlow::Exit;
            } else {
                let compact = !layout.compact;
                set_compact(&window, &mut layout, compact, false);
                transition += 1;
                *flow = ControlFlow::Poll;
            }
        });
        unsafe {
            webview.controller().Close().unwrap();
        }
    }
}
