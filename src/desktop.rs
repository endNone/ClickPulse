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
mod confirmation;
mod data_settings;
#[cfg(target_os = "macos")]
mod edit_menu;
mod ipc;
mod refresh;
mod session;
mod share;
mod storage_recovery;
mod storage_status;
mod updates;
mod view_state;
mod webview;
mod window;
use ipc::Request;
use refresh::SnapshotGate;
#[cfg(target_os = "macos")]
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS, EventLoopWindowTargetExtMacOS};
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
use webview::VisibleWebView;
#[cfg(target_os = "windows")]
use window::update_windows_region;
use window::{
    COMPACT_SIZE, WindowLayout, fix_compact_size, present_window, saved_full_position, set_compact,
};

#[derive(Debug)]
enum UiEvent {
    Update(updates::Event),
    Ipc {
        generation: u64,
        body: String,
    },
    Menu(MenuEvent),
    Snapshot {
        id: u64,
        compact: bool,
        shortcuts: bool,
        share_day: Option<chrono::NaiveDate>,
        result: std::result::Result<String, String>,
    },
    HistoryHours {
        generation: u64,
        id: u32,
        result: std::result::Result<String, String>,
    },
    SportSample {
        id: u64,
        result: std::result::Result<String, String>,
    },
    DataFinished {
        id: u64,
        result: std::result::Result<DataOutcome, String>,
    },
    #[cfg(target_os = "windows")]
    Activate,
}

#[derive(Debug)]
enum DataOutcome {
    Imported(clickpulse::history_import::ImportReport),
    Relocated(clickpulse::runtime::Relocation),
}

enum DataNotice {
    Imported(clickpulse::history_import::ImportReport),
    Failed(String),
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
        let identity = if options.owned {
            clickpulse::platform::data_dir()?.join("profile")
        } else {
            path.to_owned()
        };
        match clickpulse::platform::windows::SingleInstance::begin(&identity)? {
            Some(instance) => Some(instance),
            None => return Ok(()),
        }
    };
    let mut profile_lock = if options.owned && !demo {
        let profile = clickpulse::platform::data_dir()?;
        std::fs::create_dir_all(&profile)?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(profile.join("profile.lock"))?;
        lock.try_lock().map_err(|_| {
            anyhow::anyhow!(
                "ClickPulse is already running. Open its existing menu-bar or tray window."
            )
        })?;
        Some(lock)
    } else {
        None
    };
    let (mut preferences, mut preferences_recovery) = if demo {
        (clickpulse::presentation::Preferences::default(), None)
    } else {
        let loaded = clickpulse::presentation::Preferences::load()?;
        (loaded.preferences, loaded.recovered_from)
    };
    let mut persisted_preferences = view_state::PreferencePersistence::new(&preferences);
    let mut published_view = view_state::Published::default();
    let mut published_paused = view_state::Published::default();
    let mut timezones_pending = true;
    let mut locale = preferences.language.resolve();
    if options.mini {
        preferences.compact = true;
    } else if !options.background {
        preferences.compact = false;
    }
    #[allow(unused_mut)]
    let mut event_builder = EventLoopBuilder::<UiEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    if profile_lock.is_some() {
        clickpulse::handoff::recover_pending()?;
    }
    #[cfg(target_os = "linux")]
    if profile_lock.is_some() {
        use tao::platform::unix::EventLoopBuilderExtUnix;
        event_builder.with_app_id("app.clickpulse.desktop");
    }
    #[allow(unused_mut)]
    let mut event_loop = event_builder.build();
    #[cfg(target_os = "macos")]
    if options.background {
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }
    #[cfg(target_os = "windows")]
    event_loop.set_device_event_filter(tao::event_loop::DeviceEventFilter::Always);
    #[cfg(target_os = "windows")]
    if wry::webview_version().is_err() {
        anyhow::bail!("{}", locale.text("error.webview_runtime"));
    }
    let owned_path;
    let path = if options.owned && !demo && !setup_pending && !source_mode {
        let Some(recovered) = storage_recovery::resolve(locale)? else {
            return Ok(());
        };
        owned_path = recovered;
        owned_path.as_path()
    } else {
        path
    };
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

    if let Some(runtime) = &runtime {
        runtime.set_ignore_software_input(preferences.ignore_software_input);
    }

    let available = event_loop
        .primary_monitor()
        .as_ref()
        .map(window::available_size)
        .unwrap_or(LogicalSize::new(1200.0, 880.0));
    let (initial_size, minimum_size) =
        window::full_window_bounds(available, LogicalSize::new(1160.0, 840.0));
    let builder = WindowBuilder::new()
        .with_title(locale.text("native.full_title"))
        .with_transparent(cfg!(any(target_os = "macos", target_os = "linux")))
        .with_inner_size(initial_size)
        .with_min_inner_size(minimum_size)
        .with_visible(false);
    #[cfg(target_os = "macos")]
    let builder = {
        use tao::platform::macos::WindowBuilderExtMacOS;
        if window::integrated_titlebar() {
            builder
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
        } else {
            builder
        }
    };
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let builder = builder.with_decorations(false);
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let builder = builder.with_window_icon(Some(tao::window::Icon::from_rgba(
        clickpulse::branding::tray_rgba()?,
        clickpulse::branding::TRAY_SIZE,
        clickpulse::branding::TRAY_SIZE,
    )?));
    let window = Arc::new(builder.build(&event_loop)?);
    window::fit_full_window(&window, initial_size, None);
    #[cfg(target_os = "windows")]
    window::disable_keyboard_menu(&window)?;
    let mut layout = WindowLayout {
        full_size: window.inner_size().to_logical(window.scale_factor()),
        full_position: window
            .outer_position()
            .ok()
            .map(|position| saved_full_position(position, window.scale_factor())),
        compact: false,
        #[cfg(any(target_os = "windows", target_os = "linux"))]
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
    // Tao's macOS proxy clone registers a source and wakes the run loop.
    // Share one proxy so idle checks and snapshot replies cannot wake themselves.
    let ui_proxy = Arc::new(event_loop.create_proxy());
    let mut webview = VisibleWebView::default();
    #[cfg(target_os = "macos")]
    let mut fullscreen_sent = None;
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let mut maximized_sent = None;
    let mut tray_status_sent = None;
    if !options.background {
        webview.ensure(&window, ui_proxy.clone(), locale, &preferences)?;
    }
    let menu = Menu::new();
    let show = MenuItem::new(locale.text("native.open"), true, None);
    let mini = MenuItem::new(locale.text("action.mini"), true, None);
    let hide = MenuItem::new(locale.text("native.hide"), true, None);
    let pause = MenuItem::new(pause_label(locale, source_mode, paused), !demo, None);
    let quit_accelerator = {
        #[cfg(target_os = "macos")]
        {
            use tray_icon::menu::accelerator::{Accelerator, Code, Modifiers};
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyQ))
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    };
    let quit = MenuItem::new(locale.text("native.quit"), true, quit_accelerator);
    #[cfg(target_os = "macos")]
    let edit_menu = edit_menu::EditMenu::new(locale, &quit)?;
    menu.append_items(&[
        &show,
        &mini,
        &hide,
        &pause,
        &PredefinedMenuItem::separator(),
        &quit,
    ])?;
    let mut storage_alert = storage_status::StorageAlert::default();
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(tray_icon()?)
        .with_icon_as_template(cfg!(target_os = "macos"))
        .with_menu_on_left_click(!cfg!(target_os = "windows"))
        .with_tooltip(locale.text("native.tooltip"))
        .build()?;
    let proxy = ui_proxy.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UiEvent::Menu(event));
    }));
    #[cfg(target_os = "windows")]
    {
        let proxy = ui_proxy.clone();
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
            let proxy = ui_proxy.clone();
            instance.listen(move || {
                let _ = proxy.send_event(UiEvent::Activate);
            })?;
        }
    }
    let mut next_refresh = Instant::now();
    let mut ready = false;
    let mut restore_pending = false;
    let mut session_state = session::SessionState::default();
    let mut demo_paused = false;
    let mut visible = !options.background;
    #[cfg(target_os = "macos")]
    let mut dock_visible = visible;
    let mut close_requested = false;
    let mut pending_presentation = visible.then(|| Instant::now() + Duration::from_millis(1500));
    let snapshot_proxy = ui_proxy;
    let mut snapshots = SnapshotGate::default();
    let mut sport_samples = SnapshotGate::default();
    let mut last_sport_sample: Option<Instant> = None;
    let mut data_job: Option<u64> = None;
    let mut next_data_job = 0u64;
    let mut quit_after_data = false;
    let mut data_notice: Option<DataNotice> = None;
    let mut updates =
        updates::Controller::new(options.owned && !demo && !source_mode && !setup_pending);
    if options.owned
        && !demo
        && !source_mode
        && !setup_pending
        && let Err(error) = clickpulse::platform::repair_auto_start()
    {
        clickpulse::logging::error(
            clickpulse::platform::log_path().ok().as_deref(),
            &format!("Repair startup entry: {error:#}"),
        );
    }
    #[cfg(windows)]
    let mut uninstall_after_save = false;
    let handle_event = move |event: Event<'_, UiEvent>,
                             _target: &EventLoopWindowTarget<UiEvent>,
                             flow: &mut ControlFlow| {
        let _ = &profile_lock;
        if !matches!(*flow, ControlFlow::ExitWithCode(_)) {
            *flow = ControlFlow::WaitUntil(next_refresh);
        }
        let previous_compact = preferences.compact;
        let previously_active = webview.is_active();
        webview.set_active(view_state::renderer_active(
            visible,
            window.is_visible(),
            window.is_minimized(),
            pending_presentation.is_some(),
        ));
        let mut refresh = !previously_active && webview.is_active();
        let mut view_changed = refresh;
        let mut window_close_requested = false;
        match event {
            Event::UserEvent(UiEvent::Update(event)) => {
                if updates.receive(event) {
                    *flow = ControlFlow::Exit;
                }
                view_changed |= updates.sync_notifications(&mut preferences.update_notifications);
                if ready {
                    updates.publish(&webview);
                }
            }
            Event::UserEvent(UiEvent::DataFinished { id, result }) => {
                if let Some(current) = data_job
                    && current == id
                {
                    data_job = None;
                    snapshots.invalidate();
                    sport_samples.invalidate();
                    let _ = webview.evaluate_script(
                        "window.receiveDataBusy?.(false);window.receiveSport?.({available:false});",
                    );
                    match result {
                        Ok(DataOutcome::Imported(report)) if !quit_after_data => {
                            data_notice = Some(DataNotice::Imported(report))
                        }
                        Ok(DataOutcome::Relocated(relocation)) => {
                            if let Some(rt) = &mut runtime {
                                rt.finish_relocation(relocation);
                            }
                        }
                        Ok(_) => {}
                        Err(error) => {
                            if quit_after_data && (!visible || preferences.compact) {
                                eprintln!("Data operation failed before quit: {error}");
                            } else {
                                data_notice = Some(DataNotice::Failed(error));
                            }
                        }
                    }
                    refresh = true;
                    if quit_after_data {
                        *flow = ControlFlow::Exit;
                    }
                }
            }
            Event::UserEvent(UiEvent::HistoryHours {
                generation,
                id,
                result,
            }) => {
                if webview.accepts(generation) {
                    let rows = result.unwrap_or_else(|error| {
                        eprintln!("error.stats: {error}");
                        "null".to_owned()
                    });
                    let _ = webview
                        .evaluate_script(&format!("window.receiveHistoryHours({id},{rows});"));
                }
            }
            Event::UserEvent(UiEvent::SportSample { id, result }) => {
                if sport_samples.complete(id).is_some()
                    && ready
                    && visible
                    && preferences.compact
                    && preferences.sport_enabled
                    && preferences.sport_visible
                {
                    let script = match result {
                        Ok(json) => format!("window.receiveSport?.({json});"),
                        Err(_) => "window.receiveSport?.({available:false});".to_owned(),
                    };
                    let _ = webview.evaluate_script(&script);
                }
            }
            Event::UserEvent(UiEvent::Snapshot {
                id,
                compact,
                shortcuts,
                share_day,
                result,
            }) => {
                if let Some(rerun) = snapshots.complete(id) {
                    refresh = rerun;
                    if compact != preferences.compact
                        || shortcuts != (!preferences.compact && session_state.wants_shortcuts())
                        || share_day
                            != (!preferences.compact)
                                .then(|| session_state.share_day(preferences.timezone))
                                .flatten()
                    {
                        refresh = true;
                    } else if ready && webview.is_active() {
                        match result {
                            Ok(json) => {
                                let _ = webview.evaluate_script(&format!(
                                    "window.{}({json});",
                                    if compact { "receiveMini" } else { "receive" }
                                ));
                            }
                            Err(_)
                                if runtime
                                    .as_ref()
                                    .and_then(|rt| rt.capture_status().ok())
                                    .is_some_and(|capture| capture.storage_error.is_some()) => {}
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
                if !layout.compact
                    && window.fullscreen().is_none()
                    && !window.is_maximized()
                    && size.width > 0
                    && size.height > 0
                {
                    layout.full_size = size.to_logical(window.scale_factor());
                }
                // Native decoration changes and display transitions may emit
                // a second resize. Reapply client-area limits after they settle.
                if layout.compact && size != COMPACT_SIZE.to_physical(window.scale_factor()) {
                    fix_compact_size(&window);
                }
                #[cfg(target_os = "windows")]
                update_windows_region(&window, &mut layout);
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                if ready && webview.is_ready() {
                    let maximized = window.is_maximized();
                    if maximized_sent != Some(maximized)
                        && webview
                            .evaluate_script(&format!(
                                "window.receiveView({{maximized:{maximized}}});"
                            ))
                            .is_ok()
                    {
                        maximized_sent = Some(maximized);
                    }
                }
                #[cfg(target_os = "macos")]
                if ready {
                    // Native fullscreen emits a resize on entry and exit. Only
                    // publish transitions, not every pixel of a window drag.
                    let fullscreen = window.fullscreen().is_some();
                    if fullscreen_sent != Some(fullscreen)
                        && webview.is_ready()
                        && webview
                            .evaluate_script(&format!(
                                "window.receiveView({{fullscreen:{fullscreen}}});"
                            ))
                            .is_ok()
                    {
                        fullscreen_sent = Some(fullscreen);
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_close_requested = true,
            Event::Reopen { .. } => {
                present_window(
                    &window,
                    &mut layout,
                    &mut preferences,
                    false,
                    &mut pending_presentation,
                );
                visible = true;
                view_changed = true;
                refresh = true;
            }
            #[cfg(target_os = "windows")]
            Event::UserEvent(UiEvent::Activate) => {
                present_window(
                    &window,
                    &mut layout,
                    &mut preferences,
                    false,
                    &mut pending_presentation,
                );
                visible = true;
                view_changed = true;
                refresh = true;
            }
            Event::UserEvent(UiEvent::Menu(event)) => {
                if event.id == show.id() {
                    present_window(
                        &window,
                        &mut layout,
                        &mut preferences,
                        false,
                        &mut pending_presentation,
                    );
                    visible = true;
                    view_changed = true;
                    refresh = true;
                } else if event.id == mini.id() {
                    present_window(
                        &window,
                        &mut layout,
                        &mut preferences,
                        true,
                        &mut pending_presentation,
                    );
                    visible = true;
                    view_changed = true;
                    refresh = true;
                } else if event.id == hide.id() {
                    webview.suspend();
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
            Event::UserEvent(UiEvent::Ipc { generation, body }) => {
                if !webview.accepts(generation) {
                    return;
                }
                if let Ok(request) = serde_json::from_str::<Request>(&body) {
                    match request {
                        Request::SessionState { state } => {
                            if state.valid() {
                                if state.wants_shortcuts() != session_state.wants_shortcuts()
                                    || state.share_day(preferences.timezone)
                                        != session_state.share_day(preferences.timezone)
                                {
                                    snapshots.invalidate();
                                    refresh = true;
                                }
                                session_state = state;
                            }
                        }
                        Request::Language { language } => {
                            preferences.language = language;
                            locale = language.resolve();
                            #[cfg(target_os = "macos")]
                            edit_menu.set_locale(locale);
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Timezone { timezone } => {
                            preferences.timezone = timezone;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::TodayPeriod { period } => {
                            preferences.today_period = period;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Theme { theme } => {
                            preferences.theme = theme;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::KeyboardLayout { layout } => {
                            preferences.keyboard_layout = layout;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::UiScale { scale } => {
                            preferences.ui_scale = scale;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::CheckUpdates {} => {
                            let now = chrono::Utc::now().timestamp();
                            // Demo, read-only and independent sessions share the
                            // preferences file but must not move the installed
                            // app's daily schedule.
                            if updates.check(
                                snapshot_proxy.clone(),
                                now,
                                updates::CheckSource::Manual,
                            ) && updates.automatic_allowed()
                            {
                                preferences.last_update_check = Some(now);
                                view_changed = true;
                            }
                            updates.publish(&webview);
                        }
                        Request::AutomaticUpdates { enabled } => {
                            if updates.automatic_allowed() {
                                preferences.auto_check_updates = enabled;
                            }
                            // Always echo the stored value so a rejected toggle
                            // returns to its actual state.
                            view_changed = true;
                        }
                        Request::InstallUpdate {} => {
                            if data_job.is_some() {
                                toast(&webview, &locale.text("storage.processing"));
                            } else if let Err(error) = updates.install(snapshot_proxy.clone()) {
                                report_error(&webview, locale, "update.failed", &error);
                            }
                            updates.publish(&webview);
                        }
                        Request::UpdateRelease {} => {
                            if let Err(error) = updates.open_release() {
                                report_error(&webview, locale, "update.failed", &error);
                            }
                        }
                        Request::ShortcutReference { source } => {
                            if let Err(error) = share::open_reference(&source) {
                                eprintln!("Cannot open shortcut reference: {error}");
                                toast(&webview, &locale.text("message.invalid_request"));
                            }
                        }
                        Request::ProjectHome {} => {
                            if let Err(error) = clickpulse::platform::open_project_home() {
                                eprintln!("Cannot open project home: {error}");
                                toast(&webview, &locale.text("message.project_home_failed"));
                            }
                        }
                        Request::SendFeedback {} => {
                            if let Err(error) = clickpulse::platform::open_feedback() {
                                eprintln!("Cannot open feedback page: {error}");
                                toast(&webview, &locale.text("message.feedback_failed"));
                            }
                        }
                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        Request::CopyShareCard { png, day } => match share::copy(&png, &day) {
                            Ok(()) => toast(&webview, &locale.text("share.copied")),
                            Err(error) => {
                                eprintln!("Card copy failed: {error}");
                                toast(&webview, &locale.text("share.copy_failed"));
                            }
                        },
                        Request::SaveShareCard { png, day } => {
                            match share::save(&window, &png, &day, &locale.text("share.save")) {
                                Ok(true) => toast(&webview, &locale.text("share.saved")),
                                Ok(false) => {}
                                Err(error) => {
                                    eprintln!("Card export failed: {error}");
                                    toast(&webview, &locale.text("share.failed"));
                                }
                            }
                        }
                        Request::Ready {} => {
                            ready = true;
                            webview.mark_ready();
                            timezones_pending = true;
                            published_view.reset();
                            published_paused.reset();
                            storage_alert.reset_view();
                            restore_pending = true;
                            updates.publish(&webview);
                            view_changed = true;
                            refresh = true;
                        }
                        mode @ (Request::Compact {} | Request::Expand {}) => {
                            present_window(
                                &window,
                                &mut layout,
                                &mut preferences,
                                matches!(mode, Request::Compact {}),
                                &mut pending_presentation,
                            );
                            visible = true;
                            view_changed = true;
                            refresh = true;
                        }
                        Request::Drag {} => {
                            window.drag_window().ok();
                        }
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        Request::Minimize {} => {
                            pending_presentation = None;
                            webview.suspend();
                            window.set_minimized(true);
                        }
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        Request::Maximize {} => {
                            if !layout.compact {
                                window.set_maximized(!window.is_maximized());
                                view_changed = true;
                            }
                        }
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        Request::ClosePreference {
                            close_action: action,
                        } => {
                            preferences.close_action = action;
                            view_changed = true;
                        }
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        Request::CloseDecision {
                            close_action: action,
                            remember,
                        } => {
                            if action != clickpulse::presentation::CloseAction::Ask {
                                if remember && !demo {
                                    preferences.close_action = action;
                                    if let Err(error) = persisted_preferences.save_if_changed(
                                        &preferences,
                                        clickpulse::presentation::Preferences::save,
                                    ) {
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
                                    webview.suspend();
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
                            webview.suspend();
                            window.set_visible(false);
                            visible = false;
                            view_changed = true;
                        }
                        Request::Refresh {} => refresh = true,
                        Request::IgnoreSoftwareInput { enabled } => {
                            preferences.ignore_software_input = enabled;
                            if let Some(runtime) = &runtime {
                                runtime.set_ignore_software_input(enabled);
                            }
                            view_changed = true;
                        }
                        Request::Sport { enabled } => {
                            preferences.sport_enabled = enabled && preferences.sport_visible;
                            view_changed = true;
                        }
                        Request::SportVisible { visible } => {
                            preferences.sport_visible = visible;
                            if !visible {
                                preferences.sport_enabled = false;
                            }
                            view_changed = true;
                        }
                        Request::HistoryHours {
                            id,
                            start,
                            end,
                            timezone,
                        } => {
                            if let Some(rt) = &runtime {
                                let proxy = snapshot_proxy.clone();
                                rt.request_history_hours(start, end, timezone, move |result| {
                                    let result =
                                        result.and_then(|hours| Ok(serde_json::to_string(&hours)?));
                                    let _ = proxy.send_event(UiEvent::HistoryHours {
                                        generation,
                                        id,
                                        result: result.map_err(|error| format!("{error:#}")),
                                    });
                                });
                            }
                        }
                        Request::SportSample {} => {
                            if ready
                                && data_job.is_none()
                                && visible
                                && preferences.compact
                                && preferences.sport_enabled
                                && preferences.sport_visible
                                && last_sport_sample
                                    .is_none_or(|last| last.elapsed() >= Duration::from_millis(180))
                            {
                                last_sport_sample = Some(Instant::now());
                                if let Some(rt) = &runtime {
                                    if let Some(id) = sport_samples.begin() {
                                        let proxy = snapshot_proxy.clone();
                                        if rt
                                            .request_sport_sample(move |result| {
                                                let _ = proxy.send_event(UiEvent::SportSample {
                                                    id,
                                                    result: result
                                                        .map_err(|error| format!("{error:#}")),
                                                });
                                            })
                                            .is_err()
                                        {
                                            sport_samples.complete(id);
                                            let _ = webview.evaluate_script(
                                                "window.receiveSport?.({available:false});",
                                            );
                                        }
                                    }
                                } else {
                                    let _ = webview.evaluate_script(
                                        "window.receiveSport?.({available:false});",
                                    );
                                }
                            }
                        }
                        Request::Pause {} => {
                            if let Some(rt) = &runtime {
                                rt.set_paused(!rt.is_paused());
                            } else {
                                demo_paused = !demo_paused;
                            }
                            refresh = true;
                        }
                        Request::Permission {} => {
                            #[cfg(target_os = "linux")]
                            if !demo && !source_mode {
                                clickpulse::dialogs::MessageDialog::new()
                                    .set_parent(&window)
                                    .set_title("ClickPulse")
                                    .set_description(locale.text("settings.permission_description"))
                                    .set_level(rfd::MessageLevel::Info)
                                    .show();
                            }
                            #[cfg(target_os = "macos")]
                            if !demo && (!source_mode || setup_pending) {
                                clickpulse::capture::request_permission();
                                if let Err(error) =
                                    clickpulse::platform::open_input_monitoring_settings()
                                {
                                    report_error(&webview, locale, "error.recording", &error);
                                }
                            }
                        }
                        #[cfg(target_os = "macos")]
                        Request::Takeover {} => {
                            if setup_pending
                                && data_job.is_none()
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
                                                    rt.set_ignore_software_input(
                                                        preferences.ignore_software_input,
                                                    );
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
                                                    runtime = restore_handoff(
                                                        &original_path,
                                                        &webview,
                                                        locale,
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
                                            runtime =
                                                restore_handoff(&original_path, &webview, locale);
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
                        #[cfg(windows)]
                        Request::Uninstall {} => {
                            if options.owned
                                && !demo
                                && !source_mode
                                && data_job.is_none()
                                && !updates.busy()
                                && confirmation::confirm(
                                    Some(&window),
                                    locale,
                                    &locale.text("removal.title"),
                                    &locale.text("removal.start"),
                                )
                            {
                                uninstall_after_save = true;
                                *flow = ControlFlow::Exit;
                            }
                        }
                        Request::Reveal {} => {
                            if let Some(rt) = &runtime
                                && let Err(error) = clickpulse::platform::reveal(rt.database_path())
                            {
                                report_error(&webview, locale, "message.reveal_failed", &error);
                            }
                        }
                        action @ (Request::StorageDirectory {} | Request::ImportHistory {}) => {
                            if data_job.is_none()
                                && !updates.installing()
                                && let Some(rt) = &runtime
                                && rt.can_manage_auto_start()
                            {
                                next_data_job = next_data_job.wrapping_add(1);
                                let id = next_data_job;
                                data_job = Some(id);
                                let proxy = snapshot_proxy.clone();
                                let reply = move |result: anyhow::Result<DataOutcome>| {
                                    let _ = proxy.send_event(UiEvent::DataFinished {
                                        id,
                                        result: result.map_err(|error| format!("{error:#}")),
                                    });
                                };
                                let result = if matches!(action, Request::StorageDirectory {}) {
                                    data_settings::select_directory(
                                        &window,
                                        rt.database_path(),
                                        locale,
                                    )
                                    .and_then(|path| {
                                        if let Some(path) = path {
                                            rt.request_directory_change(&path, move |result| {
                                                reply(result.map(DataOutcome::Relocated))
                                            })
                                        } else {
                                            Ok(false)
                                        }
                                    })
                                } else if let Some(path) =
                                    data_settings::select_import(&window, locale)
                                {
                                    rt.request_import_history(path, move |result| {
                                        reply(result.map(DataOutcome::Imported))
                                    })
                                    .map(|()| true)
                                } else {
                                    Ok(false)
                                };
                                match result {
                                    Ok(true) => {}
                                    Ok(false) => data_job = None,
                                    Err(error) => {
                                        data_job = None;
                                        data_settings::show_error(
                                            &window,
                                            locale,
                                            &format!("{error:#}"),
                                        );
                                    }
                                }
                                let _ = webview.evaluate_script(&format!("window.receiveDataBusy?.({});window.receiveSport?.({{available:false}});",data_job.is_some()));
                                snapshots.invalidate();
                                sport_samples.invalidate();
                                refresh = true;
                            }
                        }
                        Request::Quit {} => *flow = ControlFlow::Exit,
                        Request::Close {} => window_close_requested = true,
                    }
                } else {
                    toast(&webview, &locale.text("message.invalid_request"));
                }
            }
            Event::MainEventsCleared if Instant::now() >= next_refresh => {
                refresh = true;
                next_refresh =
                    Instant::now() + refresh::interval(webview.is_active(), preferences.compact);
                let now = chrono::Utc::now().timestamp();
                if data_job.is_none()
                    && !quit_after_data
                    && updates.automatic_due(
                        preferences.auto_check_updates,
                        preferences.last_update_check,
                        now,
                    )
                    && updates.check(snapshot_proxy.clone(), now, updates::CheckSource::Automatic)
                {
                    preferences.last_update_check = Some(now);
                    view_changed = true;
                    updates.publish(&webview);
                }
            }
            Event::LoopDestroyed => {
                let was_paused = runtime.as_ref().is_some_and(|rt| rt.is_paused());
                let saved = match runtime.take() {
                    Some(mut rt) => match rt.shutdown() {
                        Ok(()) => true,
                        Err(error) => {
                            eprintln!("Failed to save before exit: {error:#}");
                            false
                        }
                    },
                    None => true,
                };
                // Release the writer's profile before signalling helpers to proceed.
                drop(profile_lock.take());
                if let Some(guard) = &updates.guard
                    && let Err(error) = guard.finish_shutdown(saved, was_paused)
                {
                    eprintln!("Update handoff failed: {error:#}");
                }
                webview.dispose();
                #[cfg(target_os = "windows")]
                drop(activation.take());
                #[cfg(windows)]
                if uninstall_after_save {
                    if saved {
                        if let Err(error) = clickpulse::removal::launch() {
                            clickpulse::platform::windows::show_error(&error, "error.removal");
                        }
                    } else {
                        clickpulse::platform::windows::show_error(
                            &anyhow::anyhow!(locale.text("removal.save_failed")),
                            "error.removal",
                        );
                    }
                }
                return;
            }
            _ => {}
        }
        if window_close_requested {
            use clickpulse::presentation::CloseAction;
            match view_state::close_action(
                demo,
                cfg!(target_os = "macos"),
                preferences.close_action,
            ) {
                CloseAction::Quit => *flow = ControlFlow::Exit,
                CloseAction::Hide => {
                    webview.suspend();
                    window.set_visible(false);
                    visible = false;
                    view_changed = true;
                }
                CloseAction::Ask => {
                    close_requested = true;
                    present_window(
                        &window,
                        &mut layout,
                        &mut preferences,
                        false,
                        &mut pending_presentation,
                    );
                    visible = true;
                    view_changed = true;
                    refresh = true;
                }
            }
        }
        // Reopening restores the latest state once, including results received hidden.
        if data_job.is_none()
            && !quit_after_data
            && !matches!(*flow, ControlFlow::ExitWithCode(_))
            && updates.download_if_available(snapshot_proxy.clone())
        {
            updates.publish(&webview);
        }
        let active = view_state::renderer_active(
            visible,
            window.is_visible(),
            window.is_minimized(),
            pending_presentation.is_some(),
        );
        if active && !webview.is_active() {
            // suspend() delivered visible=false directly. A resumed renderer
            // needs visible=true even if the last published payload is identical.
            published_view.reset();
            view_changed = true;
            refresh = true;
        }
        if active && !matches!(*flow, ControlFlow::ExitWithCode(_)) {
            match webview.ensure(&window, snapshot_proxy.clone(), locale, &preferences) {
                Ok(true) => {
                    ready = false;
                    snapshots.invalidate();
                    sport_samples.invalidate();
                }
                Ok(false) => {}
                Err(error) => {
                    // Leave the tray/recorder alive and allow another Open attempt.
                    eprintln!("Unable to create interface: {error:#}");
                    window.set_visible(false);
                    visible = false;
                    clickpulse::dialogs::MessageDialog::new()
                        .set_title("ClickPulse")
                        .set_description(locale.text("error.stats"))
                        .show();
                }
            }
        }
        webview.set_active(active && visible);
        if webview.release_if_idle(
            Instant::now(),
            data_job.is_some() || updates.installing() || quit_after_data,
        ) {
            ready = false;
            snapshots.invalidate();
            sport_samples.invalidate();
        }
        // Ready may arrive after an immediate Hide. Keep restoration pending
        // until the existing view is actually resumed, not only on cold creation.
        if restore_pending
            && webview.is_ready()
            && let Ok(state) = serde_json::to_string(&session_state)
            && webview
                .evaluate_script(&format!("window.restoreSessionState?.({state});"))
                .is_ok()
        {
            restore_pending = false;
        }
        if preferences.compact != previous_compact {
            snapshots.invalidate();
            sport_samples.invalidate();
        }
        if view_changed {
            next_refresh = Instant::now() + refresh::interval(active, preferences.compact);
        }
        // A requested Quit waits for a definite commit/rollback without blocking
        // the event loop or dropping the operation's destination lock.
        if data_job.is_some() && matches!(*flow, ControlFlow::ExitWithCode(_)) {
            quit_after_data = true;
            *flow = ControlFlow::WaitUntil(next_refresh);
        }
        // A sheet attached to a hidden or Mini window can be inaccessible.
        // Keep the result until the full window is visible again.
        if data_job.is_none()
            && webview.is_ready()
            && visible
            && !preferences.compact
            && let Some(notice) = data_notice.take()
        {
            match notice {
                DataNotice::Imported(report) => {
                    data_settings::show_import_report(&window, locale, report)
                }
                DataNotice::Failed(error) => data_settings::show_error(&window, locale, &error),
            }
        }
        if ready && webview.is_active() && !preferences.compact {
            updates.publish_notice(&webview);
        }
        // Do not spend the one-time reminder on a hidden, minimized, inactive
        // or Mini window. The pending version survives until a full view opens.
        if ready
            && visible
            && !preferences.compact
            && window.is_visible()
            && !window.is_minimized()
            && window.is_focused()
            && updates.automatic_allowed()
            && !updates.installing()
            && data_job.is_none()
            && !quit_after_data
            && !matches!(*flow, ControlFlow::ExitWithCode(_))
            && let Some(version) = preferences.update_notifications.pending()
            && let Ok(version) = serde_json::to_string(version)
            && webview
                .evaluate_script(&format!("window.receiveUpdateAvailable?.({version});"))
                .is_ok()
        {
            preferences.update_notifications.mark_shown();
            view_changed = true;
        }
        webview.set_zoom(&window, &preferences);
        #[cfg(target_os = "macos")]
        webview.sync_titlebar(
            &window,
            preferences.compact,
            session_state.sidebar_collapsed.unwrap_or(false),
        );
        if view_changed
            && !demo
            && let Err(e) = persisted_preferences
                .save_if_changed(&preferences, clickpulse::presentation::Preferences::save)
        {
            report_error(&webview, locale, "message.preferences_failed", &e);
        }
        if view_changed {
            window.set_title(&locale.text(if preferences.compact {
                "native.mini_title"
            } else {
                "native.full_title"
            }));
            show.set_text(locale.text("native.open"));
            mini.set_text(locale.text("action.mini"));
            hide.set_text(locale.text("native.hide"));
            quit.set_text(locale.text("native.quit"));
            let mut view = serde_json::json!({"compact":preferences.compact,"pinned":preferences.pinned,"visible":visible,"locale":locale.code(),"language_preference":preferences.language,"timezone":preferences.timezone,"today_period":preferences.today_period,"keyboard_layout":preferences.keyboard_layout,"ui_scale":preferences.ui_scale,"theme":preferences.theme,"maximized":window.is_maximized(),"fullscreen":window.fullscreen().is_some(),"close_action":preferences.close_action,"ignore_software_input":preferences.ignore_software_input,"sport_enabled":preferences.sport_enabled,"sport_visible":preferences.sport_visible,"auto_check_updates":preferences.auto_check_updates,"last_update_check":preferences.last_update_check});
            if webview.is_ready() {
                if timezones_pending {
                    view["timezones"] =
                        serde_json::json!(clickpulse::timezone::TimeZonePreference::choices());
                }
                view["data_busy"] = serde_json::json!(data_job.is_some());
                published_view.send(view, |view| {
                    webview.evaluate_script(&format!(
                        "window.receiveView({view});window.receiveDataBusy?.({});",
                        data_job.is_some()
                    ))?;
                    timezones_pending = false;
                    #[cfg(any(target_os = "windows", target_os = "linux"))]
                    {
                        maximized_sent = Some(window.is_maximized());
                    }
                    #[cfg(target_os = "macos")]
                    {
                        fullscreen_sent = Some(window.fullscreen().is_some());
                    }
                    Ok(())
                });
                updates.publish(&webview);
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
        if close_requested
            && webview.is_ready()
            && visible
            && !preferences.compact
            && webview.evaluate_script("window.requestClose();").is_ok()
        {
            close_requested = false;
        }
        if refresh {
            // A history query must not delay acknowledgement of Pause/Resume.
            let paused = runtime.as_ref().map_or(demo_paused, Runtime::is_paused);
            if webview.is_ready() {
                published_paused.send(paused, |paused| {
                    webview.evaluate_script(&format!("window.receivePaused({paused});"))
                });
            }
            if let Some(rt) = &runtime
                && let Ok(capture) = rt.capture_status()
            {
                update_status(
                    &tray,
                    &pause,
                    &mut tray_status_sent,
                    locale,
                    source_mode,
                    rt.is_paused(),
                    &capture,
                );
                storage_alert.update(&tray, &webview, locale, &capture);
            }
            if ready && webview.is_active() && data_job.is_none() {
                if let Some(rt) = &runtime {
                    if let Some(id) = snapshots.begin() {
                        let proxy = snapshot_proxy.clone();
                        let compact = preferences.compact;
                        let shortcuts = !compact && session_state.wants_shortcuts();
                        let share_day = (!compact)
                            .then(|| session_state.share_day(preferences.timezone))
                            .flatten();
                        let reply = move |result: anyhow::Result<String>| {
                            let _ = proxy.send_event(UiEvent::Snapshot {
                                id,
                                compact,
                                shortcuts,
                                share_day,
                                result: result.map_err(|error| format!("{error:#}")),
                            });
                        };
                        let result = if compact {
                            rt.request_mini_snapshot(
                                preferences.timezone,
                                preferences.today_period,
                                reply,
                            )
                        } else if let Some(day) = share_day {
                            rt.request_share_snapshot(preferences.timezone, day, reply)
                        } else {
                            rt.request_snapshot_for_view(preferences.timezone, shortcuts, reply)
                        };
                        match result {
                            Ok(()) => {}
                            Err(error) => {
                                snapshots.complete(id);
                                report_error(&webview, locale, "error.stats", &error);
                            }
                        }
                    }
                } else if demo {
                    let mut snapshot = demo_snapshot_in_timezone(demo_paused, preferences.timezone);
                    if let Some(day) = session_state.share_day(preferences.timezone) {
                        snapshot.stats.share_shortcut = Some(clickpulse::shortcuts::DailyTop {
                            day: day.to_string(),
                            top: snapshot
                                .stats
                                .history_days
                                .iter()
                                .find(|row| row.day == day.to_string())
                                .filter(|row| row.counts.keyboard > 0)
                                .map(|row| clickpulse::shortcuts::ShortcutCount {
                                    platform: std::env::consts::OS.into(),
                                    modifiers: if cfg!(target_os = "macos") { 8 } else { 1 },
                                    key: 8,
                                    count: (row.counts.keyboard / 80).max(1),
                                }),
                        });
                    }
                    if !session_state.wants_shortcuts() {
                        snapshot.stats.shortcuts = None;
                    }
                    update_status(
                        &tray,
                        &pause,
                        &mut tray_status_sent,
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
        if !visible {
            pending_presentation = None;
        }
        if !matches!(*flow, ControlFlow::ExitWithCode(_)) {
            // Closing or hiding keeps the recorder and menu-bar icon alive.
            // Native minimization keeps `visible` true, and retains its Dock entry.
            #[cfg(target_os = "macos")]
            if !visible && dock_visible {
                _target.set_activation_policy_at_runtime(ActivationPolicy::Accessory);
                dock_visible = false;
            }
            if pending_presentation
                .is_some_and(|deadline| webview.is_ready() || Instant::now() >= deadline)
            {
                // Apply language, theme and restored navigation before exposing
                // the new renderer. A failed Ready cannot hide the app forever.
                #[cfg(target_os = "macos")]
                {
                    if !dock_visible {
                        _target.set_activation_policy_at_runtime(ActivationPolicy::Regular);
                        dock_visible = true;
                    }
                    _target.show_application();
                }
                window.set_visible(true);
                // GTK may ignore deiconify while hidden. Repeat after mapping.
                window.set_minimized(false);
                window.set_focus();
                pending_presentation = None;
            }
            *flow = ControlFlow::WaitUntil(
                pending_presentation.map_or(next_refresh, |deadline| deadline.min(next_refresh)),
            );
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
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    event_loop.run(handle_event)
}

fn update_status(
    tray: &tray_icon::TrayIcon,
    pause: &MenuItem,
    sent: &mut Option<(String, String)>,
    locale: Locale,
    source_mode: bool,
    paused: bool,
    capture: &CaptureStatus,
) {
    let label = pause_label(locale, source_mode, paused);
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
    let value = (label, format!("ClickPulse · {}", locale.text(status)));
    if sent.as_ref() != Some(&value) {
        pause.set_text(&value.0);
        if tray.set_tooltip(Some(&value.1)).is_ok() {
            *sent = Some(value);
        }
    }
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
fn report_error(webview: &VisibleWebView, locale: Locale, key: &str, error: &anyhow::Error) {
    eprintln!("{key}: {error:#}");
    let key = if error.is::<clickpulse::platform::TemporaryAppLocation>() {
        "autostart.move_app"
    } else {
        key
    };
    toast(webview, &locale.text(key));
}

fn toast(webview: &VisibleWebView, message: &str) {
    if let Ok(json) = serde_json::to_string(message) {
        let _ = webview.evaluate_script(&format!("window.toast({json});"));
    }
}

fn tray_icon() -> Result<Icon> {
    use clickpulse::branding::{TRAY_SIZE, tray_rgba};
    Ok(Icon::from_rgba(tray_rgba()?, TRAY_SIZE, TRAY_SIZE)?)
}

#[cfg(test)]
fn demo_snapshot(paused: bool) -> AppSnapshot {
    demo_snapshot_in_timezone(paused, Default::default())
}

fn demo_snapshot_in_timezone(
    paused: bool,
    timezone: clickpulse::timezone::TimeZonePreference,
) -> AppSnapshot {
    let now = timezone.at(Local::now());
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
    let mut hourly = clickpulse::hourly::empty(now);
    let mut remainder = today;
    let weights: Vec<_> = hourly
        .iter()
        .map(|hour| {
            use chrono::{TimeZone, Timelike};
            let local_hour = timezone
                .at(Local.timestamp_millis_opt(hour.start_ms).unwrap())
                .hour();
            let activity = match local_hour {
                8..=11 => 5,
                12..=13 => 2,
                14..=18 => 7,
                19..=22 => 3,
                _ => 1,
            };
            activity * (hour.end_ms - hour.start_ms)
        })
        .collect();
    let mut weight_left: i64 = weights.iter().sum();
    for (hour, weight) in hourly.iter_mut().zip(weights) {
        hour.counts = Counts {
            keyboard: remainder.keyboard * weight / weight_left,
            mouse: remainder.mouse * weight / weight_left,
        };
        weight_left -= weight;
        remainder.keyboard -= hour.counts.keyboard;
        remainder.mouse -= hour.counts.mouse;
    }
    AppSnapshot {
        timezone,
        setup_pending: false,
        auto_start_available: false,
        auto_start: false,
        demo: true,
        stats: Snapshot {
            share_shortcut: None,
            shortcuts: Some(vec![
                clickpulse::shortcuts::ShortcutCount {
                    platform: std::env::consts::OS.into(),
                    modifiers: if cfg!(target_os = "macos") { 8 } else { 1 },
                    key: 8,
                    count: 342,
                },
                clickpulse::shortcuts::ShortcutCount {
                    platform: std::env::consts::OS.into(),
                    modifiers: if cfg!(target_os = "macos") { 8 } else { 1 },
                    key: 9,
                    count: 298,
                },
            ]),
            now: now.to_rfc3339(),
            today,
            week,
            lifetime,
            mouse_buttons: clickpulse::db::MouseCounts {
                left: lifetime.mouse * 85 / 100,
                right: lifetime.mouse * 12 / 100,
                middle: lifetime.mouse * 2 / 100,
                side: lifetime.mouse
                    - lifetime.mouse * 85 / 100
                    - lifetime.mouse * 12 / 100
                    - lifetime.mouse * 2 / 100,
                other: 0,
            },
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
            hourly,
            top_keys,
            legacy_imported: false,
            keyboard_stats: KeyboardStats {
                physical,
                ..Default::default()
            },
        },
        capture: CaptureStatus {
            listening: true,
            ..Default::default()
        },
        paused,
        database_path: "Demo data · no database".into(),
        database_mode: DatabaseMode::Demo,
        source_mode: false,
    }
}

#[cfg(target_os = "macos")]
fn restore_handoff(
    path: &std::path::Path,
    view: &VisibleWebView,
    locale: Locale,
) -> Option<Runtime> {
    match Runtime::prepare_handoff(path, false) {
        Ok(runtime) => Some(runtime),
        Err(error) => {
            report_error(view, locale, "message.migration_incomplete", &error);
            // Retain the last real statistics. Never substitute demo data after
            // recovery fails, and keep the recording failure visible.
            let _ = view.evaluate_script("window.receiveCapture?.({listening:false,error:'recovery failed',error_code:'message.migration_incomplete'});");
            None
        }
    }
}

#[cfg(test)]
mod window_tests {
    use super::*;
    use tao::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn demo_timezone_keeps_dates_and_hourly_totals_consistent() {
        for name in [
            "UTC",
            "Asia/Shanghai",
            "Asia/Kathmandu",
            "America/Los_Angeles",
        ] {
            let timezone = clickpulse::timezone::TimeZonePreference::Named(name.parse().unwrap());
            let demo = demo_snapshot_in_timezone(false, timezone);
            assert_eq!(demo.timezone, timezone);
            assert_eq!(demo.stats.days.last().unwrap().day, demo.stats.now[..10]);
            let count = demo
                .stats
                .hourly
                .iter()
                .map(|h| h.counts.total())
                .sum::<i64>();
            assert_eq!(
                count,
                demo.stats
                    .windows
                    .iter()
                    .find(|w| w.seconds == 86400)
                    .unwrap()
                    .counts
                    .total()
            );
        }
    }

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
        let hourly = demo
            .stats
            .hourly
            .iter()
            .fold(Counts::default(), |mut sum, hour| {
                sum.keyboard += hour.counts.keyboard;
                sum.mouse += hour.counts.mouse;
                sum
            });
        assert_eq!(
            hourly,
            demo.stats
                .windows
                .iter()
                .find(|w| w.seconds == 86_400)
                .unwrap()
                .counts
        );

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
        #[cfg(any(target_os = "windows", target_os = "linux"))]
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
            #[cfg(any(target_os = "windows", target_os = "linux"))]
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
            #[cfg(any(target_os = "windows", target_os = "linux"))]
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
