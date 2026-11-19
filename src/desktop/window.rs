//! Native window geometry, decorations and display-scale transitions.
use std::sync::Arc;
use tao::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Position};

pub(super) struct WindowLayout {
    pub(super) full_size: LogicalSize<f64>,
    pub(super) full_position: Option<Position>,
    pub(super) compact: bool,
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    pub(super) full_maximized: bool,
    #[cfg(target_os = "windows")]
    pub(super) region: Option<CompactRegion>,
}
pub(super) const COMPACT_SIZE: LogicalSize<f64> = LogicalSize::new(320.0, 208.0);

// Custom AppKit titlebar geometry is validated only on Apple Silicon macOS 26.
// Older systems and Intel retain the system-owned titlebar and button layout.
pub(super) fn integrated_titlebar() -> bool {
    #[cfg(target_os = "macos")]
    {
        static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *SUPPORTED.get_or_init(|| {
            let version = objc2_foundation::NSProcessInfo::processInfo().operatingSystemVersion();
            supports_integrated_titlebar(version.majorVersion, cfg!(target_arch = "aarch64"))
        })
    }
    #[cfg(not(target_os = "macos"))]
    false
}

#[cfg(any(target_os = "macos", test))]
fn supports_integrated_titlebar(major: isize, apple_silicon: bool) -> bool {
    major >= 26 && apple_silicon
}

#[cfg(target_os = "macos")]
pub(super) fn align_titlebar(window: &tao::window::Window, zoom: f64, collapsed: bool) {
    use objc2_app_kit::{
        NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
        NSWindow, NSWindowButton,
    };
    use tao::platform::macos::WindowExtMacOS;

    if !integrated_titlebar() || !window.is_decorated() || window.fullscreen().is_some() {
        return;
    }
    // Called on the event-loop thread. Move the native titlebar container,
    // leaving Apple's buttons and their spacing intact. Only the collapsed
    // sidebar joins the 58 CSS px page toolbar; expanded keeps the native row.
    let native = unsafe { &*window.ns_window().cast::<NSWindow>() };
    let Some(close) = native.standardWindowButton(NSWindowButton::CloseButton) else {
        return;
    };
    // The live NSWindow owns this AppKit hierarchy throughout this call.
    let Some(container) = (unsafe { close.superview().and_then(|view| view.superview()) }) else {
        return;
    };
    // The titlebar overlays our own background, independently of macOS's theme.
    // Match only the native controls to white when collapsed and navy otherwise;
    // a dark appearance makes inactive buttons nearly white on the white header.
    let appearance_name = unsafe {
        if collapsed {
            NSAppearanceNameAqua
        } else {
            NSAppearanceNameDarkAqua
        }
    };
    if container
        .appearance()
        .is_none_or(|appearance| &*appearance.name() != appearance_name)
        && let Some(appearance) = NSAppearance::appearanceNamed(appearance_name)
    {
        container.setAppearance(Some(&appearance));
    }
    let button = close.frame();
    let mut frame = container.frame();
    let center = if collapsed { 29.0 * zoom } else { 16.0 };
    let height = center + button.origin.y + button.size.height / 2.0;
    let top = native.frame().size.height - height;
    // AppKit may lay the titlebar out again on a resize. Only write when it
    // actually moved; a redraw on every event would keep the app awake.
    if (frame.size.height - height).abs() > 0.1 || (frame.origin.y - top).abs() > 0.1 {
        frame.size.height = height;
        frame.origin.y = top;
        container.setFrame(frame);
    }
}

pub(super) fn full_window_bounds(
    available: LogicalSize<f64>,
    requested: LogicalSize<f64>,
) -> (LogicalSize<f64>, LogicalSize<f64>) {
    let width = (available.width - 40.0).max(1.0);
    let height = (available.height - 40.0).max(1.0);
    let minimum = LogicalSize::new(900.0_f64.min(width), 600.0_f64.min(height));
    (
        LogicalSize::new(
            requested.width.max(minimum.width).min(width),
            requested.height.max(minimum.height).min(height),
        ),
        minimum,
    )
}

pub(super) fn available_size(monitor: &tao::monitor::MonitorHandle) -> LogicalSize<f64> {
    work_area(monitor).1
}
fn work_area(monitor: &tao::monitor::MonitorHandle) -> (PhysicalPosition<i32>, LogicalSize<f64>) {
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::MonitorHandleExtMacOS;
        if let Some(screen) = monitor.ns_screen() {
            let screen = unsafe { &*screen.cast::<objc2_app_kit::NSScreen>() };
            let visible = screen.visibleFrame();
            let frame = screen.frame();
            let scale = monitor.scale_factor();
            let origin = monitor.position();
            return (
                PhysicalPosition::new(
                    origin.x + ((visible.origin.x - frame.origin.x) * scale).round() as i32,
                    origin.y
                        + ((frame.origin.y + frame.size.height
                            - visible.origin.y
                            - visible.size.height)
                            * scale)
                            .round() as i32,
                ),
                LogicalSize::new(visible.size.width, visible.size.height),
            );
        }
    }
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::MonitorHandleExtWindows;
        use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if unsafe { GetMonitorInfoW(monitor.hmonitor() as _, &mut info) } != 0 {
            return (
                PhysicalPosition::new(info.rcWork.left, info.rcWork.top),
                PhysicalSize::new(
                    (info.rcWork.right - info.rcWork.left) as u32,
                    (info.rcWork.bottom - info.rcWork.top) as u32,
                )
                .to_logical(monitor.scale_factor()),
            );
        }
    }
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::MonitorExt;
        use tao::platform::unix::MonitorHandleExtUnix;
        let work = monitor.gdk_monitor().workarea();
        let geometry = monitor.gdk_monitor().geometry();
        let origin = monitor.position();
        let scale = monitor.scale_factor();
        (
            PhysicalPosition::new(
                origin.x + ((work.x() - geometry.x()) as f64 * scale).round() as i32,
                origin.y + ((work.y() - geometry.y()) as f64 * scale).round() as i32,
            ),
            LogicalSize::new(f64::from(work.width()), f64::from(work.height())),
        )
    }
    #[cfg(not(target_os = "linux"))]
    (
        monitor.position(),
        monitor.size().to_logical(monitor.scale_factor()),
    )
}

fn clamp_position(
    requested: PhysicalPosition<i32>,
    origin: PhysicalPosition<i32>,
    available: PhysicalSize<u32>,
    size: PhysicalSize<u32>,
    margin: i32,
) -> PhysicalPosition<i32> {
    let x = origin.x.saturating_add(margin);
    let y = origin.y.saturating_add(margin);
    PhysicalPosition::new(
        requested.x.clamp(
            x,
            x.saturating_add(available.width.saturating_sub(size.width) as i32 - 2 * margin)
                .max(x),
        ),
        requested.y.clamp(
            y,
            y.saturating_add(available.height.saturating_sub(size.height) as i32 - 2 * margin)
                .max(y),
        ),
    )
}

pub(super) fn fit_full_window(
    window: &tao::window::Window,
    requested: LogicalSize<f64>,
    position: Option<Position>,
) {
    // Restore to the old monitor if it is still attached. If it disappeared,
    // keep the whole window inside the current monitor's usable area.
    let target = position
        .and_then(|position| {
            window.available_monitors().find(|m| {
                let p = position.to_physical::<i32>(m.scale_factor());
                let o = m.position();
                let s = m.size();
                p.x >= o.x
                    && p.y >= o.y
                    && i64::from(p.x) < i64::from(o.x) + i64::from(s.width)
                    && i64::from(p.y) < i64::from(o.y) + i64::from(s.height)
            })
        })
        .or_else(|| window.current_monitor())
        .or_else(|| window.primary_monitor());
    let Some(monitor) = target else {
        window.set_inner_size(requested);
        return;
    };
    let (origin, available) = work_area(&monitor);
    let scale = monitor.scale_factor();
    let (size, minimum) = full_window_bounds(available, requested);
    window.set_min_inner_size(Some(minimum));
    window.set_inner_size(size);
    let requested = position
        .map(|p| p.to_physical::<i32>(scale))
        .or_else(|| window.outer_position().ok())
        .unwrap_or(origin);
    let margin = (20.0 * scale).round() as i32;
    window.set_outer_position(clamp_position(
        requested,
        origin,
        available.to_physical(scale),
        size.to_physical(scale),
        margin,
    ));
}

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
    #[cfg(any(target_os = "windows", target_os = "linux"))]
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
    window: &Arc<tao::window::Window>,
    layout: &mut WindowLayout,
    preferences: &mut clickpulse::presentation::Preferences,
    compact: bool,
    pending: &mut Option<std::time::Instant>,
) {
    // Visibility and focus do not deminiaturize a Cocoa window. Restore it
    // before applying the requested mode, including when reopening from Dock.
    window.set_minimized(false);
    preferences.compact = compact;
    set_compact(window, layout, compact, preferences.pinned);
    *pending = Some(std::time::Instant::now() + std::time::Duration::from_millis(1500));
}
pub(super) fn set_compact(
    window: &Arc<tao::window::Window>,
    layout: &mut WindowLayout,
    compact: bool,
    pinned: bool,
) {
    if compact && !layout.compact {
        let expanded = window.fullscreen().is_some() || window.is_maximized();
        if !expanded {
            layout.full_size = window.inner_size().to_logical(window.scale_factor());
            layout.full_position = window
                .outer_position()
                .ok()
                .map(|position| saved_full_position(position, window.scale_factor()));
        }
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            // GTK also ignores Mini size requests while maximized.
            layout.full_maximized = window.is_maximized();
            window.set_maximized(false);
        }
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
        // Keep GTK undecorated in both modes. Adding/removing its Wayland
        // header during a resize can temporarily subtract the titlebar height
        // from the WebView, clipping Mini until another allocation arrives.
        window.set_decorations(cfg!(target_os = "macos"));
        #[cfg(target_os = "windows")]
        {
            use tao::platform::windows::WindowExtWindows;
            window.set_undecorated_shadow(true);
        }
        restore_full_window(window, layout.full_size, layout.full_position);
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::GtkWindowExt;
            use tao::platform::unix::WindowExtUnix;
            // Tao snapshots GTK's current resizable flag and restores it from
            // a later idle callback after maximizing. Its queued resize request
            // has not run yet, so update GTK on this UI thread before snapshotting.
            window.gtk_window().set_resizable(true);
        }
        #[cfg(any(target_os = "windows", target_os = "linux"))]
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
            window.set_fullsize_content_view(integrated_titlebar());
            window.set_titlebar_transparent(integrated_titlebar());
        }
        fit_full_window(window, size, position);
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
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    apply(window);
}

// A frameless desktop app has no native keyboard menu. Leave all other system
// commands (Alt+F4, Alt+Tab, resize and move) with the original window procedure.
#[cfg(any(target_os = "windows", test))]
fn keyboard_menu_command(message: u32, command: usize, parameter: isize) -> bool {
    message == 0x0112 && command & 0xfff0 == 0xf100 && parameter == 0
}
#[cfg(target_os = "windows")]
pub(super) fn disable_keyboard_menu(window: &tao::window::Window) -> anyhow::Result<()> {
    use tao::platform::windows::WindowExtWindows;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    unsafe extern "system" fn callback(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        id: usize,
        _: usize,
    ) -> LRESULT {
        if keyboard_menu_command(message, wparam, lparam) {
            return 0;
        }
        if message == 0x0082 {
            unsafe {
                RemoveWindowSubclass(hwnd, Some(callback), id);
            }
        }
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }
    anyhow::ensure!(
        unsafe { SetWindowSubclass(window.hwnd() as _, Some(callback), 0x43504d, 0) } != 0,
        "Unable to configure desktop keyboard handling"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn unverified_mac_titlebars_keep_system_layout() {
        for major in [10, 11, 12, 13, 14, 15, 25] {
            assert!(!super::supports_integrated_titlebar(major, true));
        }
        for major in [12, 15, 26, 27] {
            assert!(!super::supports_integrated_titlebar(major, false));
        }
        assert!(super::supports_integrated_titlebar(26, true));
    }

    use super::*;
    #[test]
    fn saved_offscreen_position_is_clamped_to_work_area() {
        assert_eq!(
            clamp_position(
                PhysicalPosition::new(1800, 900),
                PhysicalPosition::new(0, 30),
                PhysicalSize::new(1280, 672),
                PhysicalSize::new(1160, 632),
                20
            ),
            PhysicalPosition::new(100, 50)
        );
        assert_eq!(
            clamp_position(
                PhysicalPosition::new(-5000, -900),
                PhysicalPosition::new(-1920, 0),
                PhysicalSize::new(1920, 1040),
                PhysicalSize::new(1160, 840),
                20
            ),
            PhysicalPosition::new(-1900, 20)
        );
    }
    #[test]
    fn keyboard_menu_does_not_swallow_close_or_mouse_system_menu() {
        assert!(keyboard_menu_command(0x112, 0xf100, 0));
        assert!(!keyboard_menu_command(0x112, 0xf060, 0));
        assert!(!keyboard_menu_command(0x112, 0xf100, 32));
        assert!(!keyboard_menu_command(0x104, 0x12, 0));
    }
    #[test]
    fn full_window_fits_small_work_areas_and_expansion() {
        let (size, min) = full_window_bounds(
            LogicalSize::new(1280.0, 672.0),
            LogicalSize::new(1160.0, 840.0),
        );
        assert_eq!(size, LogicalSize::new(1160.0, 632.0));
        assert_eq!(min, LogicalSize::new(900.0, 600.0));
        let (size, min) = full_window_bounds(
            LogicalSize::new(800.0, 580.0),
            LogicalSize::new(1920.0, 1080.0),
        );
        assert_eq!(size, LogicalSize::new(760.0, 540.0));
        assert_eq!(min, size);
    }

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

#[cfg(all(test, target_os = "linux"))]
mod native_linux_tests {
    use super::*;
    use crate::desktop::view_state::renderer_active;
    use clickpulse::presentation::Preferences;
    use std::time::{Duration, Instant};
    use tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoopBuilder},
        platform::{
            run_return::EventLoopExtRunReturn,
            unix::{EventLoopBuilderExtUnix, WindowExtUnix},
        },
        window::WindowBuilder,
    };
    use wry::{PageLoadEvent, WebViewBuilder, WebViewBuilderExtUnix};

    #[test]
    #[ignore = "requires isolated Xvfb with a window manager; never the user's desktop"]
    fn minimized_hidden_or_released_renderer_can_reopen_full_and_mini() {
        let mut builder = EventLoopBuilder::<()>::with_user_event();
        builder.with_any_thread(true);
        let mut event_loop = builder.build();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Isolated lifecycle test")
                .with_inner_size(LogicalSize::new(960., 720.))
                .build(&event_loop)
                .unwrap(),
        );
        let mut layout = WindowLayout {
            full_size: LogicalSize::new(960., 720.),
            full_position: None,
            compact: false,
            full_maximized: false,
        };
        let mut preferences = Preferences::default();
        let proxy = event_loop.create_proxy();
        let create_view = || {
            let proxy = proxy.clone();
            WebViewBuilder::new()
                .with_html("<!doctype html><p>Isolated test only</p>")
                .with_on_page_load_handler(move |event, _| {
                    if matches!(event, PageLoadEvent::Finished) {
                        let _ = proxy.send_event(());
                    }
                })
                .build_gtk(window.default_vbox().unwrap())
                .unwrap()
        };
        let mut view = Some(create_view());
        let mut loaded = false;
        let mut pending = None;
        let mut phase = 0;
        let mut round = 0;
        let mut changed = Instant::now();
        let deadline = changed + Duration::from_secs(30);
        event_loop.run_return(|event, _, flow| {
            if matches!(*flow, ControlFlow::ExitWithCode(_)) {
                return;
            }
            if matches!(event, Event::UserEvent(())) {
                loaded = true;
            }
            *flow = ControlFlow::Poll;
            if !matches!(event, Event::MainEventsCleared) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "GTK restoration timed out: round {round}, phase {phase}"
            );
            match phase {
                0 if loaded && changed.elapsed() > Duration::from_millis(200) => {
                    window.set_minimized(true);
                    phase = 1;
                }
                1 if window.is_minimized() => {
                    if round % 2 == 0 {
                        window.set_visible(false);
                    } else {
                        view.take();
                        loaded = false;
                    }
                    present_window(
                        &window,
                        &mut layout,
                        &mut preferences,
                        round >= 2,
                        &mut pending,
                    );
                    assert!(renderer_active(
                        true,
                        window.is_visible(),
                        window.is_minimized(),
                        pending.is_some()
                    ));
                    if view.is_none() {
                        view = Some(create_view());
                    }
                    phase = 2;
                }
                2 if loaded => {
                    window.set_visible(true);
                    window.set_minimized(false);
                    pending = None;
                    changed = Instant::now();
                    phase = 3;
                }
                3 if window.is_visible()
                    && !window.is_minimized()
                    && changed.elapsed() > Duration::from_millis(200) =>
                {
                    assert!(renderer_active(true, true, false, pending.is_some()));
                    round += 1;
                    phase = 0;
                    changed = Instant::now();
                    if round == 4 {
                        *flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            }
        });
        assert_eq!(round, 4);
    }
}
