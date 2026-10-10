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
