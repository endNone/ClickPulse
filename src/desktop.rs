use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use chrono::{Days, Local};
use clickpulse::{
    db::{Counts, DayCount, KeyCount, KeyboardStats, Snapshot, WindowCount},
    localization::Locale,
    runtime::{AppSnapshot, CaptureStatus, DatabaseMode, Runtime},
};
mod ipc;
mod refresh;
mod window;
use ipc::Request;
use refresh::SnapshotGate;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget},
    window::WindowBuilder,
};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
#[cfg(target_os = "windows")]
use window::update_windows_region;
use window::{
    COMPACT_SIZE, WindowLayout, fix_compact_size, present_window, saved_full_position, set_compact,
};
use wry::WebViewBuilder;

#[derive(Debug)]
enum UiEvent {
    Ipc(String),
    Menu(MenuEvent),
    Snapshot {
        id: u64,
        result: std::result::Result<String, String>,
    },
    #[cfg(target_os = "windows")]
    Activate,
}

pub struct Options {
    pub source_mode: bool,
    pub owned: bool,
    pub setup_pending: bool,
    pub background: bool,
    pub mini: bool,
}

pub fn run(
    path: &Path,
    legacy: Option<&Path>,
    paused: bool,
    demo: bool,
    options: Options,
) -> Result<()> {
    #[allow(unused_mut)]
    let mut source_mode = options.source_mode;
    #[allow(unused_mut)]
    let mut setup_pending = options.setup_pending;
    #[cfg(target_os = "macos")]
    let original_path = path.to_path_buf();
    #[cfg(target_os = "windows")]
    let mut activation = if demo {
        None
    } else {
        match clickpulse::platform::windows::SingleInstance::begin(path)? {
            Some(instance) => Some(instance),
            None => return Ok(()),
        }
    };
    let (mut preferences, mut preferences_recovery) = if demo {
        (clickpulse::presentation::Preferences::default(), None)
    } else {
        let loaded = clickpulse::presentation::Preferences::load()?;
        (loaded.preferences, loaded.recovered_from)
    };
    let mut locale = preferences.language.resolve();
    if options.mini {
        preferences.compact = true;
    } else if !options.background {
        preferences.compact = false;
    }
    #[allow(unused_mut)]
    let mut event_loop = EventLoopBuilder::<UiEvent>::with_user_event().build();
    #[cfg(target_os = "windows")]
    event_loop.set_device_event_filter(tao::event_loop::DeviceEventFilter::Always);
    #[cfg(target_os = "windows")]
    if wry::webview_version().is_err() {
        anyhow::bail!("{}", locale.text("error.webview_runtime"));
    }
    // Tao also registers raw input on Windows. Build it first, then register
    // our dedicated background listener last so the collector owns delivery.
    let mut runtime = if demo {
        None
    } else if setup_pending {
        Some(Runtime::prepare_handoff(path, paused)?)
    } else if source_mode {
        Some(Runtime::start_source(path, paused)?)
    } else if options.owned {
        Some(Runtime::start_owned(path, paused)?)
    } else {
        Some(Runtime::start(path, legacy, paused)?)
    };

    let builder = WindowBuilder::new()
        .with_title(locale.text("native.full_title"))
        .with_transparent(cfg!(target_os = "macos"))
        .with_inner_size(LogicalSize::new(1160.0, 840.0))
        .with_min_inner_size(LogicalSize::new(900.0, 680.0))
        .with_visible(!options.background && !preferences.compact);
    #[cfg(target_os = "macos")]
    let builder = {
        use tao::platform::macos::WindowBuilderExtMacOS;
        builder
            .with_titlebar_transparent(true)
            .with_title_hidden(true)
            .with_fullsize_content_view(true)
    };
    #[cfg(target_os = "windows")]
    let builder =
        builder
            .with_decorations(false)
            .with_window_icon(Some(tao::window::Icon::from_rgba(
                clickpulse::branding::tray_rgba()?,
                clickpulse::branding::TRAY_SIZE,
                clickpulse::branding::TRAY_SIZE,
            )?));
    let window = Arc::new(builder.build(&event_loop)?);
    let mut layout = WindowLayout {
        full_size: window.inner_size().to_logical(window.scale_factor()),
        full_position: window
            .outer_position()
            .ok()
            .map(|position| saved_full_position(position, window.scale_factor())),
        compact: false,
        #[cfg(target_os = "windows")]
        full_maximized: false,
        #[cfg(target_os = "windows")]
        region: None,
    };
    set_compact(
        &window,
        &mut layout,
        preferences.compact,
        preferences.pinned,
    );
    window.set_title(&locale.text(if preferences.compact {
        "native.mini_title"
    } else {
        "native.full_title"
    }));
    if !options.background {
        window.set_visible(true);
    }
    let proxy = event_loop.create_proxy();
    let html = include_str!("../ui/index.html")
        .replace("__VERSION__", clickpulse::version::DISPLAY)
        .replace("__BRAND_MARK__", &clickpulse::branding::mark_data_url())
        .replace("__LOCALE__", locale.code())
        .replace("__THEME__", preferences.theme.code())
        .replace("__PLATFORM__", clickpulse::platform::name())
        .replace("/*__MESSAGES__*/", include_str!("../ui/locales.json"))
        .replace(
            "/*__STYLE__*/",
            &format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                clickpulse::branding::font_css(),
                include_str!("../ui/style.css"),
                include_str!("../ui/history.css"),
                include_str!("../ui/compact.css"),
                include_str!("../ui/themes.css"),
                include_str!("../ui/dropdown.css")
            ),
        )
        .replace(
            "/*__SCRIPT__*/",
            &format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                include_str!("../ui/i18n.js"),
                include_str!("../ui/themes.js"),
                include_str!("../ui/dropdown.js"),
                include_str!("../ui/history.js"),
                include_str!("../ui/keyboard.js"),
                include_str!("../ui/app.js"),
                include_str!("../ui/presentation.js")
            ),
        );
    #[cfg(target_os = "windows")]
    let mut web_context =
        wry::WebContext::new(Some(clickpulse::platform::data_dir()?.join("webview")));
    #[cfg(target_os = "windows")]
    let builder = WebViewBuilder::new_with_web_context(&mut web_context)
        .with_custom_protocol("clickpulse".into(), move |_, request| {
            use std::borrow::Cow;
            let (content_type, body, status): (&str, Cow<'static, [u8]>, u16) =
                match request.uri().path() {
                    "/index.html" => (
                        "text/html; charset=utf-8",
                        Cow::Owned(html.as_bytes().to_vec()),
                        200,
                    ),
                    "/fonts/NotoSansSC.ttf" => (
                        "font/ttf",
                        Cow::Borrowed(include_bytes!("../assets/fonts/NotoSansSC.ttf")),
                        200,
                    ),
                    "/fonts/NotoSansMono.ttf" => (
                        "font/ttf",
                        Cow::Borrowed(include_bytes!("../assets/fonts/NotoSansMono.ttf")),
                        200,
                    ),
                    "/fonts/Inter.ttf" => (
                        "font/ttf",
                        Cow::Borrowed(include_bytes!("../assets/fonts/Inter.ttf")),
                        200,
                    ),
                    _ => ("text/plain", Cow::Borrowed(b"Not found"), 404),
                };
            wry::http::Response::builder()
                .status(status)
                .header("Content-Type", content_type)
                .body(body)
                .expect("static resource response")
        })
        .with_url("clickpulse://localhost/index.html");
    #[cfg(target_os = "macos")]
    let builder = WebViewBuilder::new().with_html(html);
    let webview = builder
        .with_transparent(cfg!(target_os = "macos"))
        .with_navigation_handler(|url| {
            url == "about:blank"
                || url.starts_with("about:blank#")
                || matches!(
                    url.as_str(),
                    "clickpulse://localhost/index.html"
                        | "http://clickpulse.localhost/index.html"
                        | "https://clickpulse.localhost/index.html"
                )
        })
        .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
        .with_ipc_handler(move |req| {
            let _ = proxy.send_event(UiEvent::Ipc(req.body().clone()));
        })
        .build(window.as_ref())?;
    let menu = Menu::new();
    let show = MenuItem::new(locale.text("native.open"), true, None);
    let mini = MenuItem::new(locale.text("action.mini"), true, None);
    let hide = MenuItem::new(locale.text("native.hide"), true, None);
    let pause = MenuItem::new(pause_label(locale, source_mode, paused), !demo, None);
    let quit = MenuItem::new(locale.text("native.quit"), true, None);
    menu.append_items(&[
        &show,
        &mini,
        &hide,
        &pause,
        &PredefinedMenuItem::separator(),
        &quit,
    ])?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(tray_icon()?)
        .with_icon_as_template(cfg!(target_os = "macos"))
        .with_menu_on_left_click(!cfg!(target_os = "windows"))
        .with_tooltip(locale.text("native.tooltip"))
        .build()?;
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UiEvent::Menu(event));
    }));
    #[cfg(target_os = "windows")]
    {
        let proxy = event_loop.create_proxy();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            if matches!(
                event,
                tray_icon::TrayIconEvent::DoubleClick {
                    button: tray_icon::MouseButton::Left,
                    ..
                }
            ) {
                let _ = proxy.send_event(UiEvent::Activate);
            }
        }));
        if let Some(instance) = &mut activation {
            let proxy = event_loop.create_proxy();
            instance.listen(move || {
                let _ = proxy.send_event(UiEvent::Activate);
            })?;
        }
    }
    let mut next_refresh = Instant::now();
    let mut ready = false;
    let mut demo_paused = false;
    let mut visible = !options.background;
    let snapshot_proxy = event_loop.create_proxy();
    let mut snapshots = SnapshotGate::default();
    let handle_event = move |event: Event<'_, UiEvent>,
                             target: &EventLoopWindowTarget<UiEvent>,
                             flow: &mut ControlFlow| {
        if !matches!(*flow, ControlFlow::ExitWithCode(_)) {
            *flow = ControlFlow::WaitUntil(next_refresh);
        }
        let mut refresh = false;
        let mut view_changed = false;
        match event {
            Event::UserEvent(UiEvent::Snapshot { id, result }) => {
                if let Some(rerun) = snapshots.complete(id) {
                    refresh = rerun;
                    if ready && visible {
                        match result {
                            Ok(json) => {
                                let _ =
                                    webview.evaluate_script(&format!("window.receive({json});"));
                            }
                            Err(error) => {
                                eprintln!("error.stats: {error}");
                                toast(&webview, &locale.text("error.stats"));
                                if let Ok(json) = serde_json::to_string(&locale.text("error.stats"))
                                {
                                    let _ = webview
                                        .evaluate_script(&format!("window.failure({json});"));
                                }
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    },
                ..
            } => {
                layout.apply_scale_change(scale_factor, new_inner_size);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Native decoration changes and display transitions may emit
                // a second resize. Reapply client-area limits after they settle.
                if layout.compact && size != COMPACT_SIZE.to_physical(window.scale_factor()) {
                    fix_compact_size(&window);
                }
                #[cfg(target_os = "windows")]
                {
                    update_windows_region(&window, &mut layout);
                    if ready {
                        let _ = webview.evaluate_script(&format!(
                            "window.receiveView({{maximized:{}}});",
                            window.is_maximized()
                        ));
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                if demo {
                    *flow = ControlFlow::Exit;
                } else {
                    #[cfg(target_os = "windows")]
                    match preferences.close_action {
                        clickpulse::presentation::CloseAction::Ask => {
                            let _ = webview.evaluate_script("window.requestClose();");
                        }
                        clickpulse::presentation::CloseAction::Quit => *flow = ControlFlow::Exit,
                        clickpulse::presentation::CloseAction::Hide => {
                            window.set_visible(false);
                            visible = false;
                            view_changed = true;
                        }
                    }
                    #[cfg(target_os = "macos")]
                    {
                        window.set_visible(false);
                        visible = false;
                        view_changed = true;
                    }
                }
            }
            Event::Reopen { .. } => {
                present_window(target, &window, &mut layout, &mut preferences, false);
                visible = true;
                view_changed = true;
                refresh = true;
            }
            #[cfg(target_os = "windows")]
            Event::UserEvent(UiEvent::Activate) => {
                present_window(target, &window, &mut layout, &mut preferences, false);
                visible = true;
                view_changed = true;
                refresh = true;
            }
            Event::UserEvent(UiEvent::Menu(event)) => {
                if event.id == show.id() {
                    present_window(target, &window, &mut layout, &mut preferences, false);
                    visible = true;
                    view_changed = true;
                    refresh = true;
                } else if event.id == mini.id() {
                    present_window(target, &window, &mut layout, &mut preferences, true);
                    visible = true;
                    view_changed = true;
                    refresh = true;
                } else if event.id == hide.id() {
                    window.set_visible(false);
                    visible = false;
                    view_changed = true;
                } else if event.id == pause.id() {
                    if let Some(rt) = &runtime {
                        rt.set_paused(!rt.is_paused());
                        refresh = true;
                    }
                } else if event.id == quit.id() {
                    *flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(UiEvent::Ipc(body)) => {
                if let Ok(request) = serde_json::from_str::<Request>(&body) {
                    match request {
                        Request::Language { language } => {
                            preferences.language = language;
                            locale = language.resolve();
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Theme { theme } => {
                            preferences.theme = theme;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Ready {} => {
                            ready = true;
                            view_changed = true;
                            refresh = true;
                        }
                        mode @ (Request::Compact {} | Request::Expand {}) => {
                            present_window(
                                target,
                                &window,
                                &mut layout,
                                &mut preferences,
                                matches!(mode, Request::Compact {}),
                            );
                            visible = true;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Drag {} => {
                            window.drag_window().ok();
                        }
                        #[cfg(target_os = "windows")]
                        Request::Minimize {} => window.set_minimized(true),
                        #[cfg(target_os = "windows")]
                        Request::Maximize {} => {
                            if !layout.compact {
                                window.set_maximized(!window.is_maximized());
                                view_changed = true;
                            }
                        }
                        #[cfg(target_os = "windows")]
                        Request::ClosePreference {
                            close_action: action,
                        } => {
                            preferences.close_action = action;
                            view_changed = true;
                        }
                        #[cfg(target_os = "windows")]
                        Request::CloseDecision {
                            close_action: action,
                            remember,
                        } => {
                            if action != clickpulse::presentation::CloseAction::Ask {
                                if remember && !demo {
                                    preferences.close_action = action;
                                    if let Err(error) = preferences.save() {
                                        report_error(
                                            &webview,
                                            locale,
                                            "message.preferences_failed",
                                            &error,
                                        );
                                    }
                                }
                                if action == clickpulse::presentation::CloseAction::Quit {
                                    *flow = ControlFlow::Exit;
                                } else {
                                    window.set_visible(false);
                                    visible = false;
                                    view_changed = true;
                                }
                            }
                        }
                        Request::Pin {} => {
                            preferences.pinned = !preferences.pinned;
                            window.set_always_on_top(preferences.compact && preferences.pinned);
                            view_changed = true;
                        }
                        Request::Hide {} => {
                            window.set_visible(false);
                            visible = false;
                            view_changed = true;
                        }
                        Request::Refresh {} => refresh = true,
                        Request::Pause {} => {
                            if let Some(rt) = &runtime {
                                rt.set_paused(!rt.is_paused());
                            } else {
                                demo_paused = !demo_paused;
                            }
                            refresh = true;
                        }
                        Request::Permission {} =>
                        {
                            #[cfg(target_os = "macos")]
                            if !demo && (!source_mode || setup_pending) {
                                clickpulse::capture::request_permission();
                                let _=std::process::Command::new("open").arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent").spawn();
                            }
                        }
                        #[cfg(target_os = "macos")]
                        Request::Takeover {} => {
                            if setup_pending
                                && let Some(rt) = &runtime
                                && let Ok(s) = rt.snapshot()
                            {
                                if !s.capture.listening || s.capture.permission_needed {
                                    toast(&webview, &locale.text("message.permission"));
                                } else {
                                    if let Some(mut rt) = runtime.take() {
                                        rt.shutdown().ok();
                                    }
                                    match clickpulse::handoff::take_over(&original_path) {
                                        Ok(new_path) => {
                                            match Runtime::start_owned(&new_path, false) {
                                                Ok(rt) => {
                                                    runtime = Some(rt);
                                                    source_mode = false;
                                                    setup_pending = false;
                                                    toast(
                                                        &webview,
                                                        &locale.text("message.migrated"),
                                                    );
                                                }
                                                Err(e) => {
                                                    clickpulse::handoff::rollback(&new_path).ok();
                                                    runtime = Some(
                                                        Runtime::prepare_handoff(
                                                            &original_path,
                                                            false,
                                                        )
                                                        .expect("无法恢复原统计视图"),
                                                    );
                                                    report_error(
                                                        &webview,
                                                        locale,
                                                        "message.migration_failed",
                                                        &e,
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            runtime = Some(
                                                Runtime::prepare_handoff(&original_path, false)
                                                    .expect("无法恢复原统计视图"),
                                            );
                                            report_error(
                                                &webview,
                                                locale,
                                                "message.migration_incomplete",
                                                &e,
                                            );
                                        }
                                    }
                                    refresh = true;
                                }
                            }
                        }
                        Request::Autostart {} => {
                            if runtime.as_ref().is_some_and(Runtime::can_manage_auto_start) {
                                match clickpulse::platform::set_auto_start(
                                    !clickpulse::platform::auto_start_enabled(),
                                ) {
                                    Ok(()) => refresh = true,
                                    Err(e) => report_error(
                                        &webview,
                                        locale,
                                        "message.autostart_failed",
                                        &e,
                                    ),
                                }
                            }
                        }
                        Request::Export {} => {
                            if let Some(rt) = &runtime {
                                let name = format!(
                                    "ClickPulse-{}.csv",
                                    Local::now().format("%Y%m%d-%H%M%S")
                                );
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title(locale.text("native.export_title"))
                                    .set_file_name(name)
                                    .add_filter("CSV", &["csv"])
                                    .save_file()
                                {
                                    match rt.export(path.clone()) {
                                        Ok(()) => toast(
                                            &webview,
                                            &locale
                                                .text("message.exported")
                                                .replace("{path}", &path.display().to_string()),
                                        ),
                                        Err(error) => report_error(
                                            &webview,
                                            locale,
                                            "message.export_failed",
                                            &error,
                                        ),
                                    }
                                }
                            } else {
                                toast(&webview, &locale.text("message.demo_export"));
                            }
                        }
                        Request::Reveal {} => {
                            if let Some(rt) = &runtime
                                && let Ok(snapshot) = rt.snapshot()
                                && let Err(error) =
                                    clickpulse::platform::reveal(Path::new(&snapshot.database_path))
                            {
                                report_error(&webview, locale, "message.reveal_failed", &error);
                            }
                        }
                        Request::Quit {} | Request::Close {} => {
                            #[cfg(target_os = "windows")]
                            match preferences.close_action {
                                clickpulse::presentation::CloseAction::Ask if !demo => {
                                    let _ = webview.evaluate_script("window.requestClose();");
                                }
                                clickpulse::presentation::CloseAction::Hide if !demo => {
                                    window.set_visible(false);
                                    visible = false;
                                    view_changed = true;
                                }
                                _ => *flow = ControlFlow::Exit,
                            }
                            #[cfg(target_os = "macos")]
                            {
                                *flow = ControlFlow::Exit;
                            }
                        }
                    }
                } else {
                    toast(&webview, &locale.text("message.invalid_request"));
                }
            }
            Event::MainEventsCleared if Instant::now() >= next_refresh => {
                refresh = true;
                next_refresh = Instant::now() + Duration::from_secs(2);
            }
            Event::LoopDestroyed => {
                #[cfg(target_os = "windows")]
                drop(activation.take());
                if let Some(mut rt) = runtime.take()
                    && let Err(error) = rt.shutdown()
                {
                    eprintln!("退出保存失败：{error:#}");
                }
                #[cfg(target_os = "windows")]
                {
                    use wry::WebViewExtWindows;
                    // Close the renderer after pending input has been saved.
                    // The controller/environment then drop on normal return.
                    unsafe {
                        webview.controller().Close().ok();
                    }
                }
            }
            _ => {}
        }
        if view_changed {
            if !demo && let Err(e) = preferences.save() {
                report_error(&webview, locale, "message.preferences_failed", &e);
            }
            window.set_title(&locale.text(if preferences.compact {
                "native.mini_title"
            } else {
                "native.full_title"
            }));
            show.set_text(locale.text("native.open"));
            mini.set_text(locale.text("action.mini"));
            hide.set_text(locale.text("native.hide"));
            quit.set_text(locale.text("native.quit"));
            let view = serde_json::json!({"compact":preferences.compact,"pinned":preferences.pinned,"visible":visible,"locale":locale.code(),"language_preference":preferences.language,"theme":preferences.theme,"maximized":window.is_maximized(),"close_action":preferences.close_action});
            if ready {
                let _ = webview.evaluate_script(&format!("window.receiveView({view});"));
                // Launch-at-login can finish loading while hidden. Keep the
                // recovery notice until the user actually opens a window.
                if visible && let Some(path) = preferences_recovery.take() {
                    toast(
                        &webview,
                        &locale
                            .text("message.preferences_recovered")
                            .replace("{path}", &path.display().to_string()),
                    );
                }
            }
        }
        if refresh && ready {
            if let Some(rt) = &runtime
                && let Ok(capture) = rt.capture_status()
            {
                update_status(&tray, &pause, locale, source_mode, rt.is_paused(), &capture);
            }
            if visible {
                if let Some(rt) = &runtime {
                    if let Some(id) = snapshots.begin() {
                        let proxy = snapshot_proxy.clone();
                        match rt.request_snapshot(move |result| {
                            let _ = proxy.send_event(UiEvent::Snapshot {
                                id,
                                result: result.map_err(|error| format!("{error:#}")),
                            });
                        }) {
                            Ok(()) => {}
                            Err(_) => {
                                // The dropped delivery reports failure through the normal snapshot path.
                            }
                        }
                    }
                } else {
                    let snapshot = demo_snapshot(demo_paused);
                    update_status(
                        &tray,
                        &pause,
                        locale,
                        false,
                        snapshot.paused,
                        &snapshot.capture,
                    );
                    if let Ok(json) = serde_json::to_string(&snapshot) {
                        let _ = webview.evaluate_script(&format!("window.receive({json});"));
                    }
                }
            }
        }
    };
    #[cfg(target_os = "windows")]
    {
        // Return normally so WebView2 controllers, tray and native handles are
        // dropped before main exits, after LoopDestroyed flushes the collector.
        use tao::platform::run_return::EventLoopExtRunReturn;
        event_loop.run_return(handle_event);
        Ok(())
    }
    #[cfg(target_os = "macos")]
    event_loop.run(handle_event)
}

fn update_status(
    tray: &tray_icon::TrayIcon,
    pause: &MenuItem,
    locale: Locale,
    source_mode: bool,
    paused: bool,
    capture: &CaptureStatus,
) {
    pause.set_text(pause_label(locale, source_mode, paused));
    let status = if capture.error.is_some() {
        if source_mode {
            "status.read_error"
        } else {
            "status.recording_error"
        }
    } else if paused {
        "status.paused"
    } else if source_mode {
        "status.history"
    } else if capture.listening {
        "status.recording"
    } else {
        "status.permission"
    };
    tray.set_tooltip(Some(format!("ClickPulse · {}", locale.text(status))))
        .ok();
}

fn pause_label(locale: Locale, source_mode: bool, paused: bool) -> String {
    locale.text(if source_mode {
        if paused {
            "action.resume_refresh"
        } else {
            "action.pause_refresh"
        }
    } else if paused {
        "action.resume"
    } else {
        "action.pause"
    })
}
fn report_error(webview: &wry::WebView, locale: Locale, key: &str, error: &anyhow::Error) {
    eprintln!("{key}: {error:#}");
    toast(webview, &locale.text(key));
}

fn toast(webview: &wry::WebView, message: &str) {
    if let Ok(json) = serde_json::to_string(message) {
        let _ = webview.evaluate_script(&format!("window.toast({json});"));
    }
}

fn tray_icon() -> Result<Icon> {
    use clickpulse::branding::{TRAY_SIZE, tray_rgba};
    Ok(Icon::from_rgba(tray_rgba()?, TRAY_SIZE, TRAY_SIZE)?)
}

fn demo_snapshot(paused: bool) -> AppSnapshot {
    let now = Local::now();
    let days: Vec<_> = (0..90)
        .rev()
        .map(|offset| {
            let scale = (((offset * 73 + 41) % 113) as i64 + 10) * 39;
            DayCount {
                day: (now.date_naive() - Days::new(offset)).to_string(),
                counts: Counts {
                    keyboard: scale * 3,
                    mouse: scale,
                },
            }
        })
        .collect();
    let aggregate = |records: &[DayCount]| {
        records.iter().fold(Counts::default(), |mut counts, day| {
            counts.keyboard += day.counts.keyboard;
            counts.mouse += day.counts.mouse;
            counts
        })
    };
    let today = days.last().map(|day| day.counts).unwrap_or_default();
    let week = aggregate(&days[days.len().saturating_sub(7)..]);
    let lifetime = aggregate(&days);
    let history_days = days.clone();
    let key_weights = [
        (49, 30),
        (0, 12),
        (14, 10),
        (36, 7),
        (1, 6),
        (2, 5),
        (12, 4),
        (13, 4),
        (15, 4),
        (17, 4),
        (18, 3),
        (56, 4),
        (55, 5),
        (51, 10),
    ];
    let weight_sum = key_weights.iter().map(|(_, weight)| weight).sum::<i64>();
    let mut remaining = lifetime.keyboard;
    let mut physical: Vec<KeyCount> = key_weights
        .iter()
        .enumerate()
        .map(|(index, &(code, weight))| {
            let count = if index + 1 == key_weights.len() {
                remaining
            } else {
                lifetime.keyboard * weight / weight_sum
            };
            remaining -= count;
            KeyCount { code, count }
        })
        .collect();
    physical.sort_by(|a, b| b.count.cmp(&a.count).then(a.code.cmp(&b.code)));
    let top_keys = physical.iter().take(10).cloned().collect();
    let hour = chrono::Timelike::hour(&now);
    let mut active_hours: Vec<_> = (0..=hour).filter(|hour| (8..19).contains(hour)).collect();
    if active_hours.is_empty() {
        active_hours.push(hour);
    }
    let mut hours = vec![Counts::default(); 24];
    let mut remaining = today;
    for (index, hour) in active_hours.iter().enumerate() {
        let divisor = (active_hours.len() - index) as i64;
        let counts = Counts {
            keyboard: remaining.keyboard / divisor,
            mouse: remaining.mouse / divisor,
        };
        remaining.keyboard -= counts.keyboard;
        remaining.mouse -= counts.mouse;
        hours[*hour as usize] = counts;
    }
    AppSnapshot {
        setup_pending: false,
        auto_start_available: false,
        auto_start: false,
        demo: true,
        stats: Snapshot {
            now: now.to_rfc3339(),
            today,
            week,
            lifetime,
            windows: [60, 600, 3600, 86400]
                .into_iter()
                .map(|seconds| WindowCount {
                    seconds,
                    counts: Counts {
                        keyboard: today.keyboard * seconds / 86400,
                        mouse: today.mouse * seconds / 86400,
                    },
                    approximate: false,
                })
                .collect(),
            days,
            history_days,
            hours,
            top_keys,
            legacy_imported: false,
            keyboard_stats: KeyboardStats {
                physical,
                ..Default::default()
            },
        },
        capture: CaptureStatus {
            listening: true,
            permission_needed: false,
            error: None,
            error_code: None,
        },
        paused,
        database_path: "Demo data · no database".into(),
        database_mode: DatabaseMode::Demo,
        source_mode: false,
    }
}

#[cfg(test)]
mod window_tests {
    use super::*;
    use tao::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn demo_counts_agree_across_metrics_history_hours_and_keyboard() {
        let demo = demo_snapshot(false);
        let add = |days: &[DayCount]| {
            days.iter().fold(Counts::default(), |mut sum, day| {
                sum.keyboard += day.counts.keyboard;
                sum.mouse += day.counts.mouse;
                sum
            })
        };
        assert_eq!(demo.stats.today, demo.stats.days.last().unwrap().counts);
        assert_eq!(
            demo.stats.week,
            add(&demo.stats.days[demo.stats.days.len() - 7..])
        );
        assert_eq!(demo.stats.lifetime, add(&demo.stats.history_days));
        let hours = demo
            .stats
            .hours
            .iter()
            .fold(Counts::default(), |mut sum, counts| {
                sum.keyboard += counts.keyboard;
                sum.mouse += counts.mouse;
                sum
            });
        assert_eq!(hours, demo.stats.today);
        assert_eq!(
            demo.stats
                .keyboard_stats
                .physical
                .iter()
                .map(|key| key.count)
                .sum::<i64>(),
            demo.stats.lifetime.keyboard
        );
        assert_eq!(
            demo.stats
                .top_keys
                .iter()
                .map(|key| key.code)
                .collect::<Vec<_>>(),
            demo.stats
                .keyboard_stats
                .physical
                .iter()
                .take(10)
                .map(|key| key.code)
                .collect::<Vec<_>>()
        );
        assert!(!demo.stats.legacy_imported);
    }

    #[test]
    fn full_window_position_survives_a_different_mini_display_scale() {
        let physical = PhysicalPosition::new(648, 86);
        let saved = saved_full_position(physical, 2.0);
        #[cfg(target_os = "macos")]
        for scale in [1.0, 1.5, 2.0] {
            assert_eq!(
                saved.to_logical::<f64>(scale),
                tao::dpi::LogicalPosition::new(324.0, 43.0)
            );
        }
        #[cfg(target_os = "windows")]
        for scale in [1.0, 1.5, 2.0] {
            assert_eq!(saved.to_physical::<i32>(scale), physical);
        }
    }

    #[test]
    fn mini_keeps_its_logical_size_across_display_round_trips() {
        let layout = WindowLayout {
            full_size: LogicalSize::new(1160.0, 840.0),
            full_position: None,
            compact: true,
            #[cfg(target_os = "windows")]
            full_maximized: false,
            #[cfg(target_os = "windows")]
            region: None,
        };
        for scale in [2.0, 1.0, 1.25, 1.5, 2.0, 1.0, 2.0] {
            // Simulate the oversized suggestions seen after moving displays.
            let mut suggested = LogicalSize::new(420.0, 292.0).to_physical(scale);
            layout.apply_scale_change(scale, &mut suggested);
            assert_eq!(suggested.to_logical::<f64>(scale), COMPACT_SIZE);
        }
    }

    #[test]
    fn full_view_keeps_system_resize_and_restores_in_logical_units() {
        let saved = PhysicalSize::new(2320, 1680).to_logical(2.0);
        let layout = WindowLayout {
            full_size: saved,
            full_position: None,
            compact: false,
            #[cfg(target_os = "windows")]
            full_maximized: false,
            #[cfg(target_os = "windows")]
            region: None,
        };
        let mut suggested = PhysicalSize::new(1000, 720);
        layout.apply_scale_change(1.0, &mut suggested);
        assert_eq!(suggested, PhysicalSize::new(1000, 720));
        assert_eq!(
            layout.full_size.to_physical::<u32>(1.0),
            PhysicalSize::new(1160, 840)
        );
        assert_eq!(
            layout.full_size.to_physical::<u32>(2.0),
            PhysicalSize::new(2320, 1680)
        );
    }
}
