//! Optional renderer; capture, updates and data tasks live in desktop.rs.
use super::UiEvent;
use anyhow::Result;
use clickpulse::{
    localization::Locale,
    presentation::{Preferences, UiScale},
};
use std::{
    cell::Cell,
    sync::Arc,
    time::{Duration, Instant},
};
use tao::{event_loop::EventLoopProxy, window::Window};
use wry::WebViewBuilder;

const RELEASE_AFTER: Duration = Duration::from_secs(5 * 60);

struct RenderedView {
    inner: wry::WebView,
    // Drop the controller before its environment/cache.
    #[cfg(target_os = "windows")]
    _context: wry::WebContext,
    #[cfg(target_os = "windows")]
    desired: std::rc::Rc<Cell<bool>>,
}

#[derive(Default)]
pub(super) struct VisibleWebView {
    zoom: Cell<f64>,
    keyboard_zoom: Cell<f64>,
    window_unit: Cell<f64>,
    inner: Option<RenderedView>,
    active: Cell<bool>,
    ready: Cell<bool>,
    hidden_since: Cell<Option<Instant>>,
    generation: u64,
}
impl VisibleWebView {
    pub fn ensure(
        &mut self,
        window: &Window,
        proxy: Arc<EventLoopProxy<UiEvent>>,
        locale: Locale,
        preferences: &Preferences,
    ) -> Result<bool> {
        if self.inner.is_some() {
            return Ok(false);
        }
        self.generation = self.generation.wrapping_add(1);
        self.inner = Some(create(window, proxy, locale, preferences, self.generation)?);
        self.zoom.set(0.0);
        self.set_zoom(window, preferences);
        self.active.set(true);
        self.ready.set(false);
        self.hidden_since.set(None);
        Ok(true)
    }
    pub fn mark_ready(&self) {
        self.ready.set(true);
        self.keyboard_zoom.set(0.0);
        self.window_unit.set(0.0);
    }
    pub fn set_zoom(&self, window: &Window, preferences: &Preferences) {
        let width = window
            .inner_size()
            .to_logical::<f64>(window.scale_factor())
            .width;
        let zoom = preferences.ui_scale.factor(width, preferences.compact);
        if (self.zoom.get() - zoom).abs() > 0.001
            && let Some(view) = &self.inner
            && view.inner.zoom(zoom).is_ok()
        {
            self.zoom.set(zoom);
        }
        // Window-driven automatic zoom must not resize the physical keyboard.
        // Explicit user scaling still applies. Publish only when changed and
        // ready, including after a renderer recreation or hidden-window resize.
        let keyboard_zoom = keyboard_zoom(preferences.ui_scale, self.zoom.get());
        // The Mac sidebar control shares a row with native traffic lights,
        // whose size and position stay fixed even when the page is enlarged.
        let window_unit = 1.0 / self.zoom.get().max(1.0);
        if self.is_ready()
            && ((self.keyboard_zoom.get() - keyboard_zoom).abs() > 0.0001
                || (self.window_unit.get() - window_unit).abs() > 0.0001)
            && self.evaluate_script(&format!(
                "document.documentElement.style.setProperty('--keyboard-zoom','{keyboard_zoom}');document.documentElement.style.setProperty('--window-unit','{window_unit}px');"
            )).is_ok()
        {
            self.keyboard_zoom.set(keyboard_zoom);
            self.window_unit.set(window_unit);
        }
    }
    pub fn is_ready(&self) -> bool {
        self.is_active() && self.ready.get()
    }
    #[cfg(target_os = "macos")]
    pub fn sync_titlebar(&self, window: &Window, compact: bool, collapsed: bool) {
        if !compact && self.is_active() {
            super::window::align_titlebar(window, self.zoom.get().max(1.0), collapsed);
        }
    }
    pub fn accepts(&self, generation: u64) -> bool {
        self.inner.is_some() && generation == self.generation
    }
    pub fn is_active(&self) -> bool {
        self.inner.is_some() && self.active.get()
    }
    pub fn set_active(&self, active: bool) {
        let active = active && self.inner.is_some();
        if self.active.replace(active) == active {
            return;
        }
        self.hidden_since
            .set(if active { None } else { Some(Instant::now()) });
        if let Some(view) = &self.inner {
            #[cfg(target_os = "windows")]
            view.set_active(active);
            #[cfg(not(target_os = "windows"))]
            let _ = view.inner.set_visible(active);
        }
    }
    pub fn evaluate_script(&self, script: &str) -> wry::Result<()> {
        if self.is_ready() {
            self.inner.as_ref().unwrap().inner.evaluate_script(script)
        } else {
            Ok(())
        }
    }
    pub fn suspend(&self) {
        let _ = self.evaluate_script(
            "window.captureSessionState?.();window.receiveView?.({visible:false});",
        );
        self.set_active(false);
    }
    pub fn release_if_idle(&mut self, now: Instant, protected: bool) -> bool {
        if self.inner.is_some()
            && release_due(self.is_active(), self.hidden_since.get(), now, protected)
        {
            self.dispose();
            true
        } else {
            false
        }
    }
    pub fn dispose(&mut self) {
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        let had_view = self.inner.is_some();
        self.active.set(false);
        self.hidden_since.set(None);
        self.ready.set(false);
        #[cfg(target_os = "windows")]
        if let Some(view) = &self.inner {
            view.desired.set(false);
        }
        #[cfg(target_os = "linux")]
        if let Some(view) = &self.inner {
            disconnect_linux_ipc(&view.inner);
        }
        self.inner.take();
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        if had_view {
            // Return freed GTK/WebKit heap pages once on disposal, never on a
            // refresh timer. glibc keeps live allocations and is thread-safe.
            unsafe {
                libc::malloc_trim(0);
            }
        }
    }
}

fn keyboard_zoom(scale: UiScale, applied_zoom: f64) -> f64 {
    if scale == UiScale::Auto && applied_zoom > 0.0 {
        1.0 / applied_zoom
    } else {
        1.0
    }
}
// Wry 0.55's GTK IPC signal captures a strong WebView clone. The view owns
// its user-content manager, so destroying the widget alone leaves a cycle and
// keeps its WebContext/network process alive. Disconnect only this owned IPC
// signal before dropping the view; other applications/contexts are untouched.
#[cfg(target_os = "linux")]
fn disconnect_linux_ipc(view: &wry::WebView) {
    use gtk::glib::{ObjectExt, gobject_ffi, object::ObjectType, translate::IntoGlib};
    use wry::WebViewExtUnix;
    let native = view.webview();
    let manager: gtk::glib::Object = native.property("user-content-manager");
    // Both objects stay strongly held on the GTK event-loop thread throughout
    // disconnection. Find + disconnect works on Ubuntu 22.04's GLib too; the
    // ID-only disconnect_matched shortcut requires GLib 2.78.
    unsafe {
        let signal = gobject_ffi::g_signal_lookup(
            c"script-message-received".as_ptr(),
            manager.type_().into_glib(),
        );
        if signal == 0 {
            return;
        }
        loop {
            let handler = gobject_ffi::g_signal_handler_find(
                manager.as_ptr(),
                gobject_ffi::G_SIGNAL_MATCH_ID,
                signal,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if handler == 0 {
                break;
            }
            gobject_ffi::g_signal_handler_disconnect(manager.as_ptr(), handler);
        }
    }
}

fn release_due(active: bool, hidden: Option<Instant>, now: Instant, protected: bool) -> bool {
    !active
        && !protected
        && hidden.is_some_and(|since| now.saturating_duration_since(since) >= RELEASE_AFTER)
}

#[cfg(target_os = "windows")]
impl RenderedView {
    fn set_active(&self, active: bool) {
        use webview2_com::{
            Microsoft::Web::WebView2::Win32::ICoreWebView2_3, TrySuspendCompletedHandler,
        };
        use windows_core::Interface;
        use wry::{MemoryUsageLevel, WebViewExtWindows};
        self.desired.set(active);
        let core = self.inner.webview().cast::<ICoreWebView2_3>();
        if active {
            if let Ok(core) = core {
                unsafe {
                    let _ = core.Resume();
                }
            }
            let _ = self.inner.set_memory_usage_level(MemoryUsageLevel::Normal);
            let _ = self.inner.set_visible(true);
        } else {
            let _ = self.inner.set_visible(false);
            let _ = self.inner.set_memory_usage_level(MemoryUsageLevel::Low);
            if let Ok(core) = core {
                let desired = std::rc::Rc::downgrade(&self.desired);
                let resume = core.clone();
                let callback = TrySuspendCompletedHandler::create(Box::new(move |_, _| {
                    // A show can race the asynchronous suspension. A discarded
                    // view has no desired flag and must never be resumed.
                    if desired.upgrade().is_some_and(|flag| flag.get()) {
                        unsafe {
                            resume.Resume()?;
                        }
                    }
                    Ok(())
                }));
                unsafe {
                    let _ = core.TrySuspend(&callback);
                }
            }
        }
    }
}

fn create(
    window: &Window,
    proxy: Arc<EventLoopProxy<UiEvent>>,
    locale: Locale,
    preferences: &Preferences,
    generation: u64,
) -> Result<RenderedView> {
    // Avoid reusing a previous build's document; preferences are embedded too.
    let document_path = format!(
        "/index.html?build={}&view={generation}",
        clickpulse::version::INTERNAL_BUILD
    );
    let theme = preferences.theme.code();
    let integrated_titlebar = super::window::integrated_titlebar();
    #[cfg(target_os = "windows")]
    let mut web_context =
        wry::WebContext::new(Some(clickpulse::platform::data_dir()?.join("webview")));
    #[cfg(target_os = "windows")]
    let builder = WebViewBuilder::new_with_web_context(&mut web_context);
    #[cfg(target_os = "windows")]
    let builder = {
        use wry::WebViewBuilderExtWindows;
        builder
            .with_browser_accelerator_keys(false)
            .with_default_context_menus(false)
    };
    #[cfg(not(target_os = "windows"))]
    let builder = WebViewBuilder::new();
    let builder = builder
        .with_custom_protocol("clickpulse".into(), move |_, request| {
            use std::borrow::Cow;
            let (content_type, body, status): (&str, Cow<'static, [u8]>, u16) =
                match request.uri().path() {
                    "/index.html" => {
                        let html = include_str!("../../ui/index.html")
                            .replace(
                                "__MAC_TITLEBAR__",
                                if integrated_titlebar {
                                    "integrated"
                                } else {
                                    "system"
                                },
                            )
                            .replace("__VERSION__", clickpulse::version::PUBLIC)
                            .replace("__BRAND_MARK__", "images/brand-mark.png")
                            .replace("__LOCALE__", locale.code())
                            .replace("__THEME__", theme)
                            .replace("__PLATFORM__", clickpulse::platform::name())
                            .replace("/*__MESSAGES__*/", include_str!("../../ui/locales.json"))
                            .replace(
                                "/*__STYLE__*/",
                                &format!(
                                    "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                                    clickpulse::branding::font_css(),
                                    include_str!("../../ui/style.css"),
                                    include_str!("../../ui/history.css"),
                                    include_str!("../../ui/compact.css"),
                                    include_str!("../../ui/themes.css"),
                                    include_str!("../../ui/dropdown.css"),
                                    include_str!("../../ui/sport.css"),
                                    include_str!("../../ui/insights.css")
                                ),
                            )
                            .replace(
                                "/*__SCRIPT__*/",
                                &format!(
                                    "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                                    include_str!("../../ui/i18n.js"),
                                    include_str!("../../ui/themes.js"),
                                    include_str!("../../ui/dropdown.js"),
                                    include_str!("../../ui/history.js"),
                                    include_str!("../../ui/keyboard.js"),
                                    include_str!("../../ui/sport.js"),
                                    include_str!("../../ui/shortcuts-catalog.js"),
                                    include_str!("../../ui/date-picker.js"),
                                    include_str!("../../ui/insights.js"),
                                    include_str!("../../ui/app.js"),
                                    include_str!("../../ui/presentation.js")
                                ),
                            );

                        (
                            "text/html; charset=utf-8",
                            Cow::Owned(html.into_bytes()),
                            200,
                        )
                    }
                    "/images/brand-mark.png" => (
                        "image/png",
                        Cow::Borrowed(clickpulse::branding::ui_mark_png()),
                        200,
                    ),
                    "/fonts/Inter.woff2" => (
                        "font/woff2",
                        Cow::Borrowed(include_bytes!("../../assets/fonts/Inter.woff2")),
                        200,
                    ),
                    "/fonts/NotoSansMono.woff2" => (
                        "font/woff2",
                        Cow::Borrowed(include_bytes!("../../assets/fonts/NotoSansMono.woff2")),
                        200,
                    ),
                    _ => ("text/plain", Cow::Borrowed(b"Not found"), 404),
                };
            wry::http::Response::builder()
                .status(status)
                .header("Content-Type", content_type)
                .header("Cache-Control", "no-store")
                .body(body)
                .expect("static resource response")
        })
        .with_url(format!("clickpulse://localhost{document_path}"));
    let builder = builder
        .with_transparent(cfg!(any(target_os = "macos", target_os = "linux")))
        .with_navigation_handler(move |url| {
            url == "about:blank"
                || url.starts_with("about:blank#")
                || [
                    "clickpulse://localhost",
                    "http://clickpulse.localhost",
                    "https://clickpulse.localhost",
                ]
                .iter()
                .any(|origin| url == format!("{origin}{document_path}"))
        })
        .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
        .with_ipc_handler(move |req| {
            let _ = proxy.send_event(UiEvent::Ipc {
                generation,
                body: req.body().clone(),
            });
        });
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(window)?;
    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        builder.build_gtk(window.default_vbox().expect("Tao GTK container"))?
    };
    Ok(RenderedView {
        inner: webview,
        #[cfg(target_os = "windows")]
        _context: web_context,
        #[cfg(target_os = "windows")]
        desired: std::rc::Rc::new(Cell::new(true)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_size_is_stable_during_auto_resize_and_respects_manual_scale() {
        for width in [900.0, 1160.0, 1334.0, 1600.0, 1920.0, 2560.0] {
            for compact in [false, true] {
                let zoom = UiScale::Auto.factor(width, compact);
                assert!((44.0 * zoom * keyboard_zoom(UiScale::Auto, zoom) - 44.0).abs() < 0.001);
            }
            for scale in [
                UiScale::Percent100,
                UiScale::Percent115,
                UiScale::Percent130,
                UiScale::Percent150,
            ] {
                let zoom = scale.factor(width, false);
                assert_eq!(keyboard_zoom(scale, zoom), 1.0);
            }
        }
        assert_eq!(keyboard_zoom(UiScale::Auto, 0.0), 1.0);
    }

    #[test]
    fn hidden_renderer_expires_only_when_idle_and_invisible() {
        let start = Instant::now();
        assert!(!release_due(false, None, start + RELEASE_AFTER, false));
        assert!(!release_due(
            false,
            Some(start),
            start + RELEASE_AFTER - Duration::from_secs(1),
            false
        ));
        assert!(release_due(
            false,
            Some(start),
            start + RELEASE_AFTER,
            false
        ));
        assert!(!release_due(
            true,
            Some(start),
            start + RELEASE_AFTER * 2,
            false
        ));
        assert!(!release_due(
            false,
            Some(start),
            start + RELEASE_AFTER * 2,
            true
        ));
        assert!(release_due(
            false,
            Some(start),
            start + RELEASE_AFTER * 2,
            false
        ));
    }
}
