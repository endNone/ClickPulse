use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow};
use chrono::Local;
use crossbeam_channel::{Receiver, Sender, bounded};
use serde::Serialize;

#[cfg(test)]
use crate::logging::{STATUS_LOG_BACKUPS, status_log_archive};
use crate::logging::{STATUS_LOG_MAX_BYTES, append_status_log};

use crate::{
    collector::CollectorDb,
    db::{EventDb, Snapshot},
    input::InputEvent,
    presentation::TodayPeriod,
    source::LiveSource,
};

pub(crate) enum Command {
    Input(InputEvent),
    Snapshot(Sender<Result<Snapshot>>),
    SnapshotAsync(SnapshotDelivery),
    MiniSnapshot(SnapshotDelivery),
    HistoryHours(
        crate::timezone::HourRange,
        Completion<Vec<crate::timezone::HistoryHour>>,
    ),
    SportSample(SportDelivery),
    Import(
        PathBuf,
        PathBuf,
        Completion<crate::history_import::ImportReport>,
    ),
    Relocate(PathBuf, PathBuf, PathBuf, Completion<()>),
    StateChanged,
    Stop,
}

/// A queued operation always completes, including worker errors and dropped queues.
pub(crate) struct Completion<T> {
    reply: Option<Box<dyn FnOnce(Result<T>) + Send>>,
}
impl<T> Completion<T> {
    fn new(reply: impl FnOnce(Result<T>) + Send + 'static) -> Self {
        Self {
            reply: Some(Box::new(reply)),
        }
    }
    fn complete(mut self, result: Result<T>) {
        if let Some(reply) = self.reply.take() {
            reply(result);
        }
    }
}
impl<T> Drop for Completion<T> {
    fn drop(&mut self) {
        if let Some(reply) = self.reply.take() {
            reply(Err(anyhow!(
                "Data operation worker stopped before completing the request"
            )));
        }
    }
}

#[derive(Debug)]
pub struct Relocation {
    path: PathBuf,
    lock: crate::storage::DatabaseLock,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CaptureStatus {
    pub listening: bool,
    pub permission_needed: bool,
    pub error: Option<String>,
    pub error_code: Option<&'static str>,
    /// Storage health is independent of device/permission health.
    #[serde(skip)]
    pub storage_error: Option<String>,
    #[serde(skip)]
    pub storage_blocked: Arc<AtomicBool>,
    #[serde(skip)]
    pub ignore_software_input: Arc<AtomicBool>,
}

impl CaptureStatus {
    fn for_display(&self) -> Self {
        let mut result = self.clone();
        if let Some(error) = &self.storage_error {
            result.error = Some(error.clone());
            result.error_code = Some(if self.storage_blocked.load(Ordering::Relaxed) {
                "error.storage_waiting"
            } else {
                "error.storage_retrying"
            });
        }
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseMode {
    Default,
    Custom,
    Source,
    Demo,
}

impl DatabaseMode {
    fn can_manage_auto_start(self) -> bool {
        self == Self::Default
    }
}

#[derive(Serialize)]
pub struct AppSnapshot {
    #[serde(flatten)]
    pub stats: Snapshot,
    pub timezone: crate::timezone::TimeZonePreference,
    pub capture: CaptureStatus,
    pub paused: bool,
    pub database_path: String,
    pub database_mode: DatabaseMode,
    pub source_mode: bool,
    pub setup_pending: bool,
    pub auto_start_available: bool,
    pub auto_start: bool,
    pub demo: bool,
}

type SnapshotCallback = Box<dyn FnOnce(Result<String>) + Send>;
type MiniCounts = (crate::db::Counts, bool);

pub(crate) struct SportDelivery {
    capture: Arc<Mutex<CaptureStatus>>,
    paused: Arc<AtomicBool>,
    reply: Option<SnapshotCallback>,
}

impl SportDelivery {
    fn complete(mut self, counts: Result<crate::db::Counts>) {
        let result = (|| {
            let counts = counts?;
            let capture = self
                .capture
                .lock()
                .map_err(|_| anyhow!("Capture status is unavailable"))?;
            Ok(serde_json::to_string(&serde_json::json!({
                "now_ms": Local::now().timestamp_millis(),
                "keyboard": counts.keyboard,
                "mouse": counts.mouse,
                "paused": self.paused.load(Ordering::Relaxed),
                "available": capture.listening && capture.error.is_none() && capture.storage_error.is_none(),
            }))?)
        })();
        if let Some(reply) = self.reply.take() {
            reply(result);
        }
    }
}

impl Drop for SportDelivery {
    fn drop(&mut self) {
        if let Some(reply) = self.reply.take() {
            reply(Err(anyhow!(
                "Sport sample worker stopped before completing the request"
            )));
        }
    }
}

pub(crate) struct SnapshotDelivery {
    share_day: Option<chrono::NaiveDate>,
    include_shortcuts: bool,
    timezone: crate::timezone::TimeZonePreference,
    today_period: TodayPeriod,
    capture: Arc<Mutex<CaptureStatus>>,
    paused: Arc<AtomicBool>,
    path: PathBuf,
    source_mode: bool,
    setup_pending: bool,
    database_mode: DatabaseMode,
    reply: Option<SnapshotCallback>,
}

impl SnapshotDelivery {
    fn complete_mini(mut self, today: Result<MiniCounts>) {
        let result = (|| {
            let (today, approximate) = today?;
            let capture = self
                .capture
                .lock()
                .map_err(|_| anyhow!("Capture status is unavailable"))?
                .for_display();
            Ok(serde_json::to_string(&serde_json::json!({
                "today": today,
                "today_period": self.today_period,
                "approximate": approximate,
                "timezone": self.timezone,
                "capture": capture,
                "paused": self.paused.load(Ordering::Relaxed),
                "source_mode": self.source_mode,
                "demo": false,
            }))?)
        })();
        if let Some(reply) = self.reply.take() {
            reply(result);
        }
    }

    fn complete(mut self, stats: Result<Snapshot>) {
        let result = (|| {
            let snapshot = AppSnapshot {
                stats: stats?,
                timezone: self.timezone,
                capture: self
                    .capture
                    .lock()
                    .map_err(|_| anyhow!("Capture status is unavailable"))?
                    .for_display(),
                paused: self.paused.load(Ordering::Relaxed),
                database_path: self.path.display().to_string(),
                database_mode: self.database_mode,
                source_mode: self.source_mode,
                setup_pending: self.setup_pending,
                auto_start_available: self.database_mode.can_manage_auto_start(),
                auto_start: self.database_mode.can_manage_auto_start() && auto_start_enabled(),
                demo: false,
            };
            Ok(serde_json::to_string(&snapshot)?)
        })();
        if let Some(reply) = self.reply.take() {
            reply(result);
        }
    }
}

impl Drop for SnapshotDelivery {
    fn drop(&mut self) {
        // A failed writer or disconnected queue must still release the UI's
        // in-flight gate instead of leaving refresh permanently pending.
        if let Some(reply) = self.reply.take() {
            reply(Err(anyhow!(
                "Snapshot worker stopped before completing the request"
            )));
        }
    }
}

pub struct Runtime {
    _instance_lock: crate::storage::DatabaseLock,
    tx: Sender<Command>,
    writer: Option<JoinHandle<Result<()>>>,
    capture: Option<JoinHandle<()>>,
    stop_capture: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
    path: PathBuf,
    source_mode: bool,
    setup_pending: bool,
    database_mode: DatabaseMode,
}

impl Runtime {
    pub fn start(path: &Path, legacy: Option<&Path>, paused: bool) -> Result<Self> {
        Self::start_storage(path, legacy, paused, false)
    }

    pub fn start_owned(path: &Path, paused: bool) -> Result<Self> {
        Self::start_storage(path, None, paused, true)
    }

    fn start_storage(
        path: &Path,
        legacy: Option<&Path>,
        paused: bool,
        owned: bool,
    ) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let instance_lock = crate::storage::lock_database(path).context(
            "ClickPulse is already using this database; open its existing window from the menu bar",
        )?;
        let mut db = if owned {
            Storage::App(CollectorDb::open(path)?)
        } else {
            let mut db = EventDb::open(path)?;
            if let Some(legacy) = legacy {
                db.import_legacy(legacy)?;
            }
            Storage::Independent(db)
        };
        db.prune_rolling_cache(Local::now().timestamp_millis())?;
        let (tx, rx) = bounded(8192);
        let status = Arc::new(Mutex::new(CaptureStatus::default()));
        let paused = Arc::new(AtomicBool::new(paused));
        let stop_capture = Arc::new(AtomicBool::new(false));
        let writer_status = status.clone();
        let writer_stop = stop_capture.clone();
        let writer_paused = paused.clone();
        #[cfg(test)]
        {
            status.lock().unwrap().listening = true;
        }
        let writer = thread::Builder::new()
            .name("clickpulse-storage".into())
            .spawn(move || {
                let result = storage_loop(db, rx, writer_status.clone(), writer_paused);
                if let Err(error) = &result {
                    writer_stop.store(true, Ordering::Relaxed);
                    if let Ok(mut status) = writer_status.lock() {
                        status.listening = false;
                        status.error_code = Some("error.storage");
                        status.error =
                            Some(format!("Saving failed and recording stopped: {error:#}"));
                    }
                }
                result
            })?;
        #[cfg(all(
            any(target_os = "macos", target_os = "windows", target_os = "linux"),
            not(test)
        ))]
        let capture = Some(crate::capture::spawn(
            tx.clone(),
            paused.clone(),
            stop_capture.clone(),
            status.clone(),
        )?);
        #[cfg(all(
            any(target_os = "macos", target_os = "windows", target_os = "linux"),
            test
        ))]
        let capture = None; // Unit tests never listen to the user's desktop input.
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let capture = {
            status.lock().unwrap().error_code = Some("error.platform");
            status.lock().unwrap().error =
                Some("Global input monitoring requires macOS or Windows; statistics and migration are available on this platform.".into());
            None
        };
        Ok(Self {
            _instance_lock: instance_lock,
            tx,
            writer: Some(writer),
            capture,
            stop_capture,
            paused,
            status,
            path: path.to_path_buf(),
            source_mode: false,
            setup_pending: false,
            database_mode: if owned {
                DatabaseMode::Default
            } else {
                DatabaseMode::Custom
            },
        })
    }

    /// Read the existing collector's live database without starting another listener.
    pub fn start_source(path: &Path, paused: bool) -> Result<Self> {
        Self::source_runtime(path, paused, false)
    }

    pub fn prepare_handoff(path: &Path, paused: bool) -> Result<Self> {
        Self::source_runtime(path, paused, true)
    }

    fn source_runtime(path: &Path, paused: bool, probe: bool) -> Result<Self> {
        let lock_path = crate::platform::data_dir()?.join("source-view.lock");
        Self::source_runtime_with_lock(path, paused, probe, &lock_path)
    }

    fn source_runtime_with_lock(
        path: &Path,
        paused: bool,
        probe: bool,
        lock_path: &Path,
    ) -> Result<Self> {
        let source = LiveSource::open(path)?;
        std::fs::create_dir_all(lock_path.parent().unwrap())?;
        let instance_lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        instance_lock.try_lock().map_err(|_| {
            anyhow!("ClickPulse is already running; open its existing window from the menu bar")
        })?;
        let (tx, rx) = bounded(8192);
        let status = Arc::new(Mutex::new(CaptureStatus::default()));
        let paused = Arc::new(AtomicBool::new(paused));
        let source_status = status.clone();
        let source_paused = paused.clone();
        let writer = thread::Builder::new()
            .name("clickpulse-live-reader".into())
            .spawn(move || source_loop_inner(source, rx, source_paused, source_status, probe))?;
        let stop_capture = Arc::new(AtomicBool::new(false));
        #[cfg(all(target_os = "macos", not(test)))]
        let capture = if probe {
            Some(crate::capture::spawn(
                tx.clone(),
                Arc::new(AtomicBool::new(false)),
                stop_capture.clone(),
                status.clone(),
            )?)
        } else {
            None
        };
        #[cfg(any(not(target_os = "macos"), test))]
        let capture = None;
        Ok(Self {
            _instance_lock: instance_lock.into(),
            tx,
            writer: Some(writer),
            capture,
            stop_capture,
            paused,
            status,
            path: path.to_path_buf(),
            source_mode: true,
            setup_pending: probe,
            database_mode: DatabaseMode::Source,
        })
    }

    pub fn snapshot(&self) -> Result<AppSnapshot> {
        let (tx, rx) = bounded(1);
        self.tx
            .send_timeout(Command::Snapshot(tx), Duration::from_secs(3))
            .map_err(|_| anyhow!("The statistics worker is unavailable"))?;
        let stats = rx
            .recv_timeout(Duration::from_secs(5))
            .context("Statistics request timed out")??;
        Ok(AppSnapshot {
            stats,
            timezone: Default::default(),
            capture: self.capture_status()?,
            paused: self.paused.load(Ordering::Relaxed),
            database_path: self.path.display().to_string(),
            database_mode: self.database_mode,
            source_mode: self.source_mode,
            setup_pending: self.setup_pending,
            auto_start_available: self.can_manage_auto_start(),
            auto_start: self.can_manage_auto_start() && auto_start_enabled(),
            demo: false,
        })
    }

    /// Queue without waiting on the UI thread. Read and serialize on the
    /// storage worker, then wake the caller through its completion callback.
    pub fn request_snapshot(
        &self,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.request_snapshot_in_timezone(Default::default(), reply)
    }

    pub fn request_snapshot_in_timezone(
        &self,
        timezone: crate::timezone::TimeZonePreference,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.request_snapshot_for_view(timezone, true, reply)
    }

    pub fn request_snapshot_for_view(
        &self,
        timezone: crate::timezone::TimeZonePreference,
        include_shortcuts: bool,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.queue_snapshot(timezone, None, include_shortcuts, None, reply)
    }

    pub fn request_history_hours(
        &self,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
        zone: crate::timezone::TimeZonePreference,
        reply: impl FnOnce(Result<Vec<crate::timezone::HistoryHour>>) + Send + 'static,
    ) {
        // Dropping an unqueued Completion also notifies the caller.
        let _ = self.tx.try_send(Command::HistoryHours(
            crate::timezone::HourRange { start, end, zone },
            Completion::new(reply),
        ));
    }

    /// Share cards request only the selected day's leading chord, never the full ranking.
    pub fn request_share_snapshot(
        &self,
        timezone: crate::timezone::TimeZonePreference,
        day: chrono::NaiveDate,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.queue_snapshot(timezone, None, false, Some(day), reply)
    }

    pub fn request_mini_snapshot(
        &self,
        timezone: crate::timezone::TimeZonePreference,
        period: TodayPeriod,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.queue_snapshot(timezone, Some(period), false, None, reply)
    }

    fn queue_snapshot(
        &self,
        timezone: crate::timezone::TimeZonePreference,
        mini: Option<TodayPeriod>,
        include_shortcuts: bool,
        share_day: Option<chrono::NaiveDate>,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        let delivery = SnapshotDelivery {
            share_day,
            include_shortcuts,
            timezone,
            today_period: mini.unwrap_or_default(),
            capture: self.status.clone(),
            paused: self.paused.clone(),
            path: self.path.clone(),
            source_mode: self.source_mode,
            setup_pending: self.setup_pending,
            database_mode: self.database_mode,
            reply: Some(Box::new(reply)),
        };
        self.tx
            .try_send(if mini.is_some() {
                Command::MiniSnapshot(delivery)
            } else {
                Command::SnapshotAsync(delivery)
            })
            .map_err(|_| anyhow!("Snapshot request queue is unavailable"))?;
        Ok(())
    }

    /// Login startup always opens the default App database. Independent and
    /// read-only sessions must not change that default recorder's startup entry.
    pub fn can_manage_auto_start(&self) -> bool {
        self.database_mode.can_manage_auto_start()
    }

    pub fn import_history(&self, source: PathBuf) -> Result<crate::history_import::ImportReport> {
        let (tx, rx) = bounded(1);
        self.request_import_history(source, move |result| {
            let _ = tx.send(result);
        })?;
        rx.recv().context("History import worker stopped")?
    }

    pub fn database_path(&self) -> &Path {
        &self.path
    }

    pub fn request_import_history(
        &self,
        source: PathBuf,
        reply: impl FnOnce(Result<crate::history_import::ImportReport>) + Send + 'static,
    ) -> Result<()> {
        anyhow::ensure!(
            self.database_mode == DatabaseMode::Default,
            "History import is only available for the normal app profile"
        );
        self.tx
            .try_send(Command::Import(
                source,
                self.path.clone(),
                Completion::new(reply),
            ))
            .map_err(|_| anyhow!("History import worker is unavailable"))?;
        Ok(())
    }

    pub fn change_directory(&mut self, directory: &Path) -> Result<bool> {
        self.change_directory_with_config(directory, &crate::storage::config_path()?)
    }

    fn change_directory_with_config(&mut self, directory: &Path, config: &Path) -> Result<bool> {
        let (tx, rx) = bounded(1);
        if !self.request_directory_with_config(directory, config, move |result| {
            let _ = tx.send(result);
        })? {
            return Ok(false);
        }
        self.finish_relocation(rx.recv().context("Storage worker stopped")??);
        Ok(true)
    }

    pub fn request_directory_change(
        &self,
        directory: &Path,
        reply: impl FnOnce(Result<Relocation>) + Send + 'static,
    ) -> Result<bool> {
        self.request_directory_with_config(directory, &crate::storage::config_path()?, reply)
    }

    fn request_directory_with_config(
        &self,
        directory: &Path,
        config: &Path,
        reply: impl FnOnce(Result<Relocation>) + Send + 'static,
    ) -> Result<bool> {
        anyhow::ensure!(
            self.database_mode == DatabaseMode::Default,
            "Storage changes are only available for the normal app profile"
        );
        let directory = directory
            .canonicalize()
            .context("Selected folder is unavailable")?;
        anyhow::ensure!(directory.is_dir(), "Select a directory");
        let output = directory.join("events.sqlite3");
        if output == self.path.canonicalize()? {
            return Ok(false);
        }
        anyhow::ensure!(
            !output.exists(),
            "Destination already contains events.sqlite3; select another folder or import that database"
        );
        let lock = crate::storage::lock_database(&output)
            .context("Another ClickPulse instance is using the selected folder")?;
        self.tx
            .try_send(Command::Relocate(
                self.path.clone(),
                output.clone(),
                config.to_owned(),
                Completion::new(move |result| {
                    reply(result.map(|()| Relocation { path: output, lock }));
                }),
            ))
            .map_err(|_| anyhow!("Storage worker is unavailable"))?;
        Ok(true)
    }

    pub fn finish_relocation(&mut self, relocation: Relocation) {
        self.path = relocation.path;
        self._instance_lock = relocation.lock;
    }

    /// Read cumulative keyboard/mouse counts on the worker, without a history snapshot.
    pub fn request_sport_sample(
        &self,
        reply: impl FnOnce(Result<String>) + Send + 'static,
    ) -> Result<()> {
        self.tx
            .try_send(Command::SportSample(SportDelivery {
                capture: self.status.clone(),
                paused: self.paused.clone(),
                reply: Some(Box::new(reply)),
            }))
            .map_err(|_| anyhow!("Sport sample request queue is unavailable"))?;
        Ok(())
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
        // Wake an idle source/recorder; entering pause must not wait for a UI refresh.
        let _ = self.tx.try_send(Command::StateChanged);
    }

    pub fn set_ignore_software_input(&self, ignore: bool) {
        if let Ok(status) = self.status.lock() {
            status
                .ignore_software_input
                .store(ignore, Ordering::Relaxed);
        }
    }

    pub fn capture_status(&self) -> Result<CaptureStatus> {
        Ok(self
            .status
            .lock()
            .map_err(|_| anyhow!("Capture status is unavailable"))?
            .for_display())
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        // Stop and join all producers before draining the storage queue.
        self.stop_capture.store(true, Ordering::Relaxed);
        let capture_result = if let Some(capture) = self.capture.take() {
            capture
                .join()
                .map_err(|_| anyhow!("The capture thread exited unexpectedly"))
        } else {
            Ok(())
        };
        // A failed producer must not skip saving its already accepted events.
        let writer_result = if let Some(writer) = self.writer.take() {
            let _ = self.tx.send(Command::Stop);
            writer
                .join()
                .map_err(|_| anyhow!("The storage thread exited unexpectedly"))
                .and_then(|result| result)
        } else {
            Ok(())
        };
        writer_result.and(capture_result)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            eprintln!("ClickPulse failed to save on exit: {error:#}");
        }
    }
}

#[cfg(test)]
fn source_loop(
    source: LiveSource,
    rx: Receiver<Command>,
    paused: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
) -> Result<()> {
    source_loop_inner(source, rx, paused, status, false)
}

fn source_loop_inner(
    source: LiveSource,
    rx: Receiver<Command>,
    paused: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
    probe: bool,
) -> Result<()> {
    let mut frozen: Option<Result<crate::source::FrozenSource, String>> = None;
    let mut cached: Option<(String, bool, Snapshot)> = None;
    while let Ok(command) = rx.recv() {
        if matches!(command, Command::Stop) {
            break;
        }
        if paused.load(Ordering::Relaxed) {
            if frozen.is_none() {
                frozen = Some(source.freeze().map_err(|error| format!("{error:#}")));
                cached = None;
            }
        } else {
            frozen = None;
            cached = None;
        }
        let now = frozen
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map_or_else(Local::now, |copy| copy.now);
        let reader = || -> Result<&LiveSource> {
            match &frozen {
                Some(Ok(copy)) => Ok(&copy.source),
                Some(Err(error)) => Err(anyhow!("Cannot freeze source: {error}")),
                None => Ok(&source),
            }
        };
        let update_status = |error: Option<&anyhow::Error>| -> Result<()> {
            let mut current = status
                .lock()
                .map_err(|_| anyhow!("Source status is unavailable"))?;
            if !probe {
                current.listening = source
                    .is_recording(Local::now().timestamp_millis())
                    .unwrap_or(false);
            }
            if !probe || error.is_some() || current.error_code == Some("error.source") {
                current.error_code = error.map(|_| "error.source");
                current.error = error.map(|e| format!("Live data read failed: {e:#}"));
            }
            Ok(())
        };
        match command {
            Command::StateChanged => {
                update_status(reader().err().as_ref())?;
            }
            Command::SportSample(delivery) => {
                delivery.complete(reader().and_then(LiveSource::lifetime))
            }
            Command::HistoryHours(range, reply) => {
                reply.complete(reader().and_then(|db| db.history_hours(range)))
            }
            Command::MiniSnapshot(delivery) => {
                let result = reader().and_then(|source| match delivery.today_period {
                    TodayPeriod::CalendarDay => source
                        .today_in_timezone(now, delivery.timezone)
                        .map(|counts| (counts, false)),
                    TodayPeriod::Rolling24h => source
                        .rolling_day(now)
                        .map(|window| (window.counts, window.approximate)),
                });
                update_status(result.as_ref().err())?;
                delivery.complete_mini(result);
            }
            Command::Snapshot(_) | Command::SnapshotAsync(_) => {
                let (timezone, include_shortcuts) = match &command {
                    Command::SnapshotAsync(delivery) => {
                        (delivery.timezone, delivery.include_shortcuts)
                    }
                    _ => (Default::default(), true),
                };
                let cache_key = timezone.cache_key();
                let result = if let Some((zone, shortcuts, stats)) = &cached
                    && *zone == cache_key
                    && *shortcuts == include_shortcuts
                {
                    Ok(stats.clone())
                } else {
                    reader().and_then(|s| s.snapshot_for_view(now, timezone, include_shortcuts))
                };
                if frozen.is_some()
                    && let Ok(stats) = &result
                {
                    cached = Some((cache_key, include_shortcuts, stats.clone()));
                }
                let result = result.and_then(|mut stats| {
                    if let Command::SnapshotAsync(delivery) = &command
                        && let Some(day) = delivery.share_day
                    {
                        stats.share_shortcut = Some(reader()?.share_shortcut(day)?);
                    }
                    Ok(stats)
                });
                update_status(result.as_ref().err())?;
                match command {
                    Command::Snapshot(reply) => {
                        let _ = reply.send(result);
                    }
                    Command::SnapshotAsync(delivery) => delivery.complete(result),
                    _ => unreachable!(),
                }
            }
            Command::Import(_, _, reply) => {
                reply.complete(Err(anyhow!("Read-only source cannot import history")))
            }
            Command::Relocate(_, _, _, reply) => {
                reply.complete(Err(anyhow!("Read-only source cannot change storage")))
            }
            Command::Input(_) if probe => {}
            Command::Input(_) => {
                return Err(anyhow!(
                    "Read-only source mode must not receive captured input events"
                ));
            }
            Command::Stop => unreachable!(),
        }
    }
    Ok(())
}

#[cfg(test)]
fn writer_loop(db: EventDb, rx: Receiver<Command>) -> Result<()> {
    storage_loop(
        Storage::Independent(db),
        rx,
        Default::default(),
        Default::default(),
    )
}

enum Storage {
    Independent(EventDb),
    App(CollectorDb),
}
impl Storage {
    fn busy_timeout(&self, timeout: Duration) -> Result<()> {
        match self {
            Self::Independent(db) => db.busy_timeout(timeout),
            Self::App(db) => Ok(db.conn.busy_timeout(timeout)?),
        }
    }
    fn mini_counts(
        &self,
        now: chrono::DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
        period: TodayPeriod,
    ) -> Result<MiniCounts> {
        match period {
            TodayPeriod::CalendarDay => match self {
                Self::Independent(db) => db.today_in_timezone(now, zone),
                Self::App(db) => db.today_in_timezone(now, zone),
            }
            .map(|counts| (counts, false)),
            TodayPeriod::Rolling24h => match self {
                Self::Independent(db) => db.rolling(86_400, now.timestamp_millis()),
                Self::App(db) => db.rolling_day(now),
            }
            .map(|window| (window.counts, window.approximate)),
        }
    }
    fn lifetime(&self) -> Result<crate::db::Counts> {
        match self {
            Self::Independent(db) => db.lifetime(),
            Self::App(db) => db.lifetime(),
        }
    }
    fn insert(&mut self, events: &[InputEvent]) -> Result<()> {
        match self {
            Self::Independent(db) => db.insert(events),
            Self::App(db) => db.insert(events),
        }
    }
    fn snapshot(&self, now: chrono::DateTime<Local>) -> Result<Snapshot> {
        match self {
            Self::Independent(db) => db.snapshot(now),
            Self::App(db) => db.snapshot(now),
        }
    }
    fn snapshot_for_view(
        &self,
        now: chrono::DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
        include_shortcuts: bool,
    ) -> Result<Snapshot> {
        match self {
            Self::Independent(db) => db.snapshot_for_view(now, zone, include_shortcuts),
            Self::App(db) => db.snapshot_for_view(now, zone, include_shortcuts),
        }
    }
    fn share_shortcut(&self, day: chrono::NaiveDate) -> Result<crate::shortcuts::DailyTop> {
        match self {
            Self::Independent(db) => db.share_shortcut(day),
            Self::App(db) => db.share_shortcut(day),
        }
    }
    fn prune_rolling_cache(&mut self, now: i64) -> Result<()> {
        match self {
            Self::Independent(db) => db.prune_rolling_cache(now),
            Self::App(db) => db.prune_rolling_cache(now),
        }
    }
    fn log_status(&self, log: &mut StatusLog, recording: bool) -> Result<()> {
        if let Self::App(db) = self {
            let now = Local::now();
            if recording || log.last_recording != Some(false) {
                db.heartbeat(if recording { now.timestamp_millis() } else { 0 })?;
                log.last_recording = Some(recording);
            }
            let stats = db.activity_counts(now)?;
            let windows = stats
                .windows
                .iter()
                .zip(["1m", "10m", "1h", "24h"])
                .map(|(w, label)| format!("{label:>4}: {:>8}", w.counts.total()))
                .collect::<Vec<_>>()
                .join(" | ");
            let line = format!(
                "{windows}  || total events (all time): {}\n",
                stats.lifetime.total()
            );
            log.write(&line);
        }
        Ok(())
    }
}

#[derive(Default)]
struct StatusLog {
    last_recording: Option<bool>,
    path: Option<PathBuf>,
    failure_reported: bool,
    last_written: Option<String>,
}
impl StatusLog {
    fn for_runtime() -> Self {
        Self {
            #[cfg(not(test))]
            path: crate::platform::log_path().ok(),
            #[cfg(test)]
            path: None,
            ..Default::default()
        }
    }
    fn write(&mut self, line: &str) {
        self.write_bounded(line, STATUS_LOG_MAX_BYTES);
    }
    fn write_bounded(&mut self, line: &str, limit: u64) {
        // Tests may supply a temporary log explicitly, never the user's log.
        #[cfg(test)]
        if self.path.is_none() {
            return;
        }
        if !self.failure_reported && self.last_written.as_deref() == Some(line) {
            return;
        }
        let result = (|| -> Result<()> {
            let path = self
                .path
                .as_ref()
                .context("Cannot determine the log directory")?;
            append_status_log(path, line, limit)
        })();
        if let Err(error) = result {
            if !self.failure_reported {
                eprintln!(
                    "clicklog output failed; database recording continues and logging will retry: {error:#}"
                );
            }
            self.failure_reported = true;
        } else {
            if self.failure_reported {
                eprintln!("clicklog output resumed");
            }
            self.failure_reported = false;
            self.last_written = Some(line.to_owned());
        }
    }
}

pub fn auto_start_enabled() -> bool {
    crate::platform::auto_start_enabled()
}

/// A finished data job returns the connection plus its deferred reply.
type FinishedJob = (Storage, Box<dyn FnOnce() + Send>);

type DataJob = Box<dyn FnOnce(&mut Storage) -> Box<dyn FnOnce() + Send> + Send>;
type QueuedJob = (Storage, DataJob, Sender<FinishedJob>);

/// One reusable helper per runtime; dropping it drains its current job and joins.
struct DataWorker {
    tx: Option<Sender<QueuedJob>>,
    thread: Option<JoinHandle<()>>,
}
impl DataWorker {
    fn new() -> Result<Self> {
        let (tx, rx) = bounded::<QueuedJob>(1);
        let thread = thread::Builder::new()
            .name("clickpulse-data-job".into())
            .spawn(move || {
                while let Ok((mut db, run, reply)) = rx.recv() {
                    let finish = run(&mut db);
                    let _ = reply.send((db, finish));
                }
            })?;
        Ok(Self {
            tx: Some(tx),
            thread: Some(thread),
        })
    }
}
impl Drop for DataWorker {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Data operations own the connection on a helper thread. The worker keeps
/// draining capture into memory, so a slow copy cannot fill the bounded input
/// queue and stop recording. Other requests wait until the connection returns.
struct StorageWorker {
    flush_timeout: Duration,
    helper: DataWorker,
    db: Option<Storage>,
    job: Option<Receiver<FinishedJob>>,
    deferred: std::collections::VecDeque<Command>,
    batch: Vec<InputEvent>,
}

impl StorageWorker {
    fn flush(&mut self) -> Result<()> {
        let db = self.db.as_mut().context("Storage is unavailable")?;
        db.insert(&self.batch)?;
        self.batch.clear();
        Ok(())
    }

    fn spawn_job(
        &mut self,
        run: impl FnOnce(&mut Storage) -> Box<dyn FnOnce() + Send> + Send + 'static,
    ) -> Result<()> {
        let sender = self
            .helper
            .tx
            .as_ref()
            .context("Data worker is unavailable")?;
        self.db
            .as_ref()
            .context("Storage is unavailable")?
            .busy_timeout(Duration::from_secs(5))?;
        let db = self.db.take().context("Storage is unavailable")?;
        let (tx, rx) = bounded(1);
        if let Err(error) = sender.send((db, Box::new(run), tx)) {
            // A failed handoff must not strand accepted input without a writer.
            self.db = Some(error.0.0);
            self.db.as_ref().unwrap().busy_timeout(self.flush_timeout)?;
            return Err(anyhow!("Data worker stopped"));
        }
        self.job = Some(rx);
        Ok(())
    }

    fn handle(&mut self, command: Command) -> Result<()> {
        if self.job.is_some() {
            self.deferred.push_back(command);
            return Ok(());
        }
        self.flush()?;
        let db = self.db.as_mut().context("Storage is unavailable")?;
        match command {
            Command::Snapshot(reply) => {
                let _ = reply.send(db.snapshot(Local::now()));
            }
            // Calendar regrouping may scan old minute buckets. Keep draining
            // accepted input while this read owns the database on a helper.
            Command::SnapshotAsync(delivery) => self.spawn_job(move |db| {
                let result = db
                    .snapshot_for_view(Local::now(), delivery.timezone, delivery.include_shortcuts)
                    .and_then(|mut stats| {
                        if let Some(day) = delivery.share_day {
                            stats.share_shortcut = Some(db.share_shortcut(day)?);
                        }
                        Ok(stats)
                    });
                Box::new(move || delivery.complete(result))
            })?,
            Command::HistoryHours(range, reply) => self.spawn_job(move |db| {
                let result = match db {
                    Storage::Independent(db) => db.history_hours(range),
                    Storage::App(db) => db.history_hours(range),
                };
                Box::new(move || reply.complete(result))
            })?,
            Command::MiniSnapshot(delivery) => self.spawn_job(move |db| {
                let result = db.mini_counts(Local::now(), delivery.timezone, delivery.today_period);
                Box::new(move || delivery.complete_mini(result))
            })?,
            Command::SportSample(delivery) => delivery.complete(db.lifetime()),
            Command::Import(source, current, reply) => self.spawn_job(move |db| {
                let result = match db {
                    Storage::App(db) => db.import_history(&source, &current),
                    _ => Err(anyhow!("This database cannot import history")),
                };
                Box::new(move || reply.complete(result))
            })?,
            Command::Relocate(source, output, config, reply) => self.spawn_job(move |db| {
                let result = crate::storage::relocate(&source, &output, &config).map(|new| {
                    *db = Storage::App(new);
                });
                Box::new(move || reply.complete(result))
            })?,
            Command::Input(event) => self.batch.push(event),
            Command::StateChanged | Command::Stop => {}
        }
        Ok(())
    }

    /// Input captured during the job lands after it, then queued requests resume.
    fn finish(&mut self, finished: FinishedJob) -> Result<()> {
        let (db, reply) = finished;
        self.db = Some(db);
        self.job = None;
        let input_result = self
            .db
            .as_ref()
            .unwrap()
            .busy_timeout(self.flush_timeout)
            .and_then(|()| self.flush());
        // The job has already committed or rolled back. An unrelated buffered
        // input write failure must not turn a committed import into "failed".
        reply();
        input_result?;
        while self.job.is_none()
            && let Some(command) = self.deferred.pop_front()
        {
            self.handle(command)?;
        }
        Ok(())
    }

    fn wait_for_job(&mut self) -> Result<()> {
        while let Some(job) = self.job.clone() {
            let finished = job
                .recv()
                .map_err(|_| anyhow!("Data operation worker stopped"))?;
            self.finish(finished)?;
        }
        Ok(())
    }
}

/// Retry without blocking capture delivery. Persistent failure pauses new input;
/// already accepted events remain buffered until the database can save again.
#[derive(Default)]
struct StorageRetry {
    since: Option<Instant>,
    next: Option<Instant>,
}

impl StorageRetry {
    fn failed(&mut self, status: &Mutex<CaptureStatus>, now: Instant, error: &anyhow::Error) {
        let first = self.since.is_none();
        self.since.get_or_insert(now);
        let mut state = status.lock().unwrap_or_else(|e| e.into_inner());
        state.storage_error = Some(format!("Saving failed; retrying: {error:#}"));
        let blocked = now.duration_since(self.since.unwrap()) >= Duration::from_secs(60)
            || state.storage_blocked.load(Ordering::Relaxed);
        state.storage_blocked.store(blocked, Ordering::Relaxed);
        self.next = Some(now + Duration::from_secs(if blocked { 60 } else { 1 }));
        if first {
            eprintln!("ClickPulse cannot save; retaining pending input: {error:#}");
        }
    }

    fn limit_buffer(&self, status: &Mutex<CaptureStatus>, pending: usize) {
        // Includes a further bounded channel's worth of in-flight events.
        // Normally the 60-second deadline is reached long before this limit.
        if self.since.is_some() && pending >= 65_536 {
            status
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .storage_blocked
                .store(true, Ordering::Relaxed);
        }
    }

    fn recovered(&mut self, status: &Mutex<CaptureStatus>) {
        if self.since.take().is_some() {
            let mut state = status.lock().unwrap_or_else(|e| e.into_inner());
            state.storage_error = None;
            state.storage_blocked.store(false, Ordering::Relaxed);
            eprintln!("ClickPulse saving recovered; pending input saved.");
        }
        self.next = None;
    }
}

fn reject_while_retrying(command: Command) {
    let error = || anyhow!("Saving is temporarily unavailable; pending input is being retained");
    match command {
        Command::Snapshot(reply) => {
            let _ = reply.send(Err(error()));
        }
        Command::SnapshotAsync(reply) => reply.complete(Err(error())),
        Command::MiniSnapshot(reply) => reply.complete_mini(Err(error())),
        Command::HistoryHours(_, reply) => reply.complete(Err(error())),
        Command::SportSample(reply) => reply.complete(Err(error())),
        Command::Import(_, _, reply) => reply.complete(Err(error())),
        Command::Relocate(_, _, _, reply) => reply.complete(Err(error())),
        Command::StateChanged => {}
        Command::Input(_) | Command::Stop => unreachable!(),
    }
}

fn storage_loop(
    db: Storage,
    rx: Receiver<Command>,
    status: Arc<Mutex<CaptureStatus>>,
    paused: Arc<AtomicBool>,
) -> Result<()> {
    // Short busy waits let the loop drain capture while another process holds a
    // write lock. User-requested data jobs restore their normal five-second wait.
    db.busy_timeout(Duration::from_millis(100))?;
    let mut worker = StorageWorker {
        flush_timeout: Duration::from_millis(100),
        helper: DataWorker::new()?,
        db: Some(db),
        job: None,
        deferred: Default::default(),
        batch: Vec::with_capacity(128),
    };
    let idle = crossbeam_channel::never();
    let mut flush = Instant::now();
    let mut cache_cleanup = Instant::now();
    let mut log = Instant::now();
    let mut status_log = StatusLog::for_runtime();
    let mut retry = StorageRetry::default();
    let mut heartbeat_due = true;
    loop {
        let job = worker.job.clone().unwrap_or_else(|| idle.clone());
        let mut select = crossbeam_channel::Select::new();
        let command_index = select.recv(&rx);
        select.recv(&job);
        let mut wait = crate::collector::HEARTBEAT_INTERVAL
            .saturating_sub(log.elapsed())
            .min(Duration::from_secs(3600).saturating_sub(cache_cleanup.elapsed()));
        if heartbeat_due {
            wait = Duration::ZERO;
        }
        if let Some(next) = retry.next {
            wait = next.saturating_duration_since(Instant::now());
        } else if worker.job.is_some() {
            // Completion/input channels wake us; database deadlines wait for the job.
            wait = crate::collector::HEARTBEAT_INTERVAL;
        } else if !worker.batch.is_empty() {
            wait = wait.min(Duration::from_millis(200).saturating_sub(flush.elapsed()));
        }
        match select.select_timeout(wait) {
            Ok(operation) if operation.index() == command_index => match operation.recv(&rx) {
                Ok(Command::Input(event)) => worker.batch.push(event),
                Ok(Command::StateChanged) => heartbeat_due = true,
                Ok(Command::Stop) | Err(_) => {
                    // Requests queued before Stop still complete, as before.
                    worker.flush_timeout = Duration::from_secs(5);
                    worker.wait_for_job()?;
                    if let Some(db) = &worker.db {
                        db.busy_timeout(worker.flush_timeout)?;
                    }
                    worker.flush()?;
                    if let Some(Storage::App(db)) = &worker.db {
                        db.heartbeat(0)?;
                    }
                    break;
                }
                Ok(command) if retry.since.is_some() => reject_while_retrying(command),
                Ok(command) => {
                    if let Err(error) = worker.handle(command) {
                        retry.failed(&status, Instant::now(), &error);
                    }
                }
            },
            Ok(operation) => {
                let finished = operation
                    .recv(&job)
                    .map_err(|_| anyhow!("Data operation worker stopped"))?;
                if let Err(error) = worker.finish(finished) {
                    retry.failed(&status, Instant::now(), &error);
                }
            }
            Err(_) => {}
        }
        // Periodic database work waits while a job owns the connection.
        if worker.db.is_none() {
            anyhow::ensure!(worker.job.is_some(), "Storage connection was lost");
            continue;
        }
        retry.limit_buffer(&status, worker.batch.len());
        if retry.next.is_some_and(|next| Instant::now() < next) {
            continue;
        }
        let attempt = (|| -> Result<()> {
            if retry.since.is_some()
                || worker.batch.len() >= 128
                || flush.elapsed() >= Duration::from_millis(200)
            {
                // insert is transactional; only a successful commit clears the batch.
                worker.flush()?;
                flush = Instant::now();
            }
            let db = worker.db.as_mut().unwrap();
            if cache_cleanup.elapsed() >= Duration::from_secs(3600) {
                db.prune_rolling_cache(Local::now().timestamp_millis())?;
                cache_cleanup = Instant::now();
            }
            let recording = !paused.load(Ordering::Relaxed)
                && status
                    .lock()
                    .map(|s| s.listening && !s.storage_blocked.load(Ordering::Relaxed))
                    .unwrap_or(false);
            if heartbeat_due
                || status_log.last_recording != Some(recording)
                || log.elapsed() >= crate::collector::HEARTBEAT_INTERVAL
            {
                db.log_status(&mut status_log, recording)?;
                heartbeat_due = false;
                log = Instant::now();
            }
            // Requests queued during a data job resume only after its captured
            // input is safely committed. Newly arriving requests fail promptly.
            while worker.job.is_none()
                && let Some(command) = worker.deferred.pop_front()
            {
                worker.handle(command)?;
            }
            Ok(())
        })();
        match attempt {
            Ok(()) => retry.recovered(&status),
            Err(error) => retry.failed(&status, Instant::now(), &error),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_lock_for_ten_seconds_retains_input_and_recovers() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("locked.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        runtime.snapshot().unwrap();
        let external = rusqlite::Connection::open(&path).unwrap();
        external.execute_batch("BEGIN EXCLUSIVE").unwrap();
        let started = Instant::now();
        let mut count = 0;
        while started.elapsed() < Duration::from_secs(10) {
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    shortcut: None,
                    ts_ms: Local::now().timestamp_millis(),
                    device: crate::input::Device::Keyboard,
                    code: 4,
                }))
                .unwrap();
            count += 1;
            thread::sleep(Duration::from_millis(50));
        }
        assert!(runtime.capture_status().unwrap().storage_error.is_some());
        assert!(!runtime.stop_capture.load(Ordering::Relaxed));
        assert_eq!(
            external
                .query_row("SELECT SUM(cnt) FROM lifetime_totals", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        runtime.set_paused(true);
        external.execute_batch("ROLLBACK").unwrap();
        let deadline = Instant::now() + Duration::from_secs(4);
        while runtime.capture_status().unwrap().storage_error.is_some() {
            assert!(Instant::now() < deadline, "storage failed to recover");
            thread::sleep(Duration::from_millis(20));
        }
        assert!(runtime.is_paused());
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.keyboard, count);
        runtime.shutdown().unwrap();
        assert_eq!(
            external
                .query_row("SELECT SUM(cnt) FROM lifetime_totals", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            count
        );
    }

    #[test]
    fn storage_retry_backoff_preserves_device_errors() {
        let state = Mutex::new(CaptureStatus {
            error_code: Some("error.capture"),
            error: Some("device disconnected".into()),
            ..Default::default()
        });
        let mut retry = StorageRetry::default();
        let start = Instant::now();
        retry.failed(&state, start, &anyhow!("disk unavailable"));
        assert_eq!(retry.next, Some(start + Duration::from_secs(1)));
        assert_eq!(
            state.lock().unwrap().for_display().error_code,
            Some("error.storage_retrying")
        );
        retry.failed(
            &state,
            start + Duration::from_secs(60),
            &anyhow!("disk unavailable"),
        );
        assert_eq!(retry.next, Some(start + Duration::from_secs(120)));
        assert!(
            state
                .lock()
                .unwrap()
                .storage_blocked
                .load(Ordering::Relaxed)
        );
        retry.recovered(&state);
        assert!(
            !state
                .lock()
                .unwrap()
                .storage_blocked
                .load(Ordering::Relaxed)
        );
        assert_eq!(
            state.lock().unwrap().for_display().error_code,
            Some("error.capture")
        );
    }

    #[test]
    fn failed_transaction_retries_without_double_counting() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("transaction.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        runtime.snapshot().unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TRIGGER fail_recent BEFORE INSERT ON recent_counts BEGIN SELECT RAISE(FAIL,'test failure after totals'); END;").unwrap();
        for _ in 0..20 {
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    shortcut: None,
                    ts_ms: Local::now().timestamp_millis(),
                    device: crate::input::Device::Keyboard,
                    code: 4,
                }))
                .unwrap();
        }
        // This query flushes, fails, and returns promptly while the worker stays alive.
        assert!(runtime.snapshot().is_err());
        assert_eq!(
            conn.query_row("SELECT SUM(cnt) FROM lifetime_totals", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        conn.execute_batch("DROP TRIGGER fail_recent;").unwrap();
        let deadline = Instant::now() + Duration::from_secs(4);
        while runtime.capture_status().unwrap().storage_error.is_some() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.keyboard, 20);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn mini_snapshot_flushes_counts_without_history_and_heartbeat_clears_on_exit() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("mini.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        let now = Local::now();
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: now.timestamp_millis(),
                device: crate::input::Device::Keyboard,
                code: 4,
            }))
            .unwrap();
        // Establish startup completion before reading the external heartbeat.
        assert_eq!(runtime.snapshot().unwrap().stats.today.keyboard, 1);
        let reader = LiveSource::open(&path).unwrap();
        assert!(reader.is_recording(now.timestamp_millis()).unwrap());
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("DROP TABLE daily_totals; DROP TABLE native_key_totals; DROP TABLE historical_key_totals;").unwrap();
        for period in [TodayPeriod::CalendarDay, TodayPeriod::Rolling24h] {
            let (tx, rx) = bounded(1);
            runtime
                .request_mini_snapshot(Default::default(), period, move |r| {
                    tx.send(r).unwrap();
                })
                .unwrap();
            let json = rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["today"]["keyboard"], 1);
            assert_eq!(value["today_period"], serde_json::to_value(period).unwrap());
            assert!(value.get("history_days").is_none());
            assert!(value.get("keyboard_stats").is_none());
            assert!(value.get("auto_start").is_none());
            assert!(json.len() < 1024);
        }
        runtime.shutdown().unwrap();
        assert!(
            !reader
                .is_recording(Local::now().timestamp_millis())
                .unwrap()
        );
    }

    #[test]
    fn counter_period_matches_full_snapshot_across_midnight_and_dst() {
        use crate::input::Device;
        use chrono::DateTime;
        let zone =
            crate::timezone::TimeZonePreference::Named("America/Los_Angeles".parse().unwrap());
        for (time, calendar_total) in [
            ("2026-03-09T06:59:30Z", 23),
            ("2026-11-02T07:59:30Z", 25),
            ("2026-11-03T08:00:30Z", 1),
        ] {
            let now = DateTime::parse_from_rfc3339(time)
                .unwrap()
                .with_timezone(&Local);
            let temp = tempfile::tempdir().unwrap();
            let app = CollectorDb::open(&temp.path().join("app.sqlite3")).unwrap();
            app.conn
                .execute(
                    "UPDATE clickpulse_metadata SET value=?1 WHERE key='precise_since_ms'",
                    [(now.timestamp_millis() - 86_400_000).to_string()],
                )
                .unwrap();
            for mut db in [
                Storage::App(app),
                Storage::Independent(
                    EventDb::open(&temp.path().join("independent.sqlite3")).unwrap(),
                ),
            ] {
                db.insert(
                    &(0..28)
                        .map(|h| InputEvent {
                            shortcut: None,
                            ts_ms: (now - chrono::Duration::hours(h)).timestamp_millis(),
                            device: Device::Keyboard,
                            code: 4,
                        })
                        .collect::<Vec<_>>(),
                )
                .unwrap();
                for zone in [
                    zone,
                    crate::timezone::TimeZonePreference::Named("Asia/Shanghai".parse().unwrap()),
                ] {
                    let full = db.snapshot_for_view(now, zone, true).unwrap();
                    let day = db.mini_counts(now, zone, TodayPeriod::CalendarDay).unwrap();
                    assert_eq!(day.0, full.today);
                    let rolling = db.mini_counts(now, zone, TodayPeriod::Rolling24h).unwrap();
                    let expected = full.windows.iter().find(|w| w.seconds == 86_400).unwrap();
                    assert_eq!(rolling.0, expected.counts);
                    assert_eq!(rolling.1, expected.approximate);
                    assert_eq!(rolling.0.total(), 24);
                }
                assert_eq!(
                    db.mini_counts(now, zone, TodayPeriod::CalendarDay)
                        .unwrap()
                        .0
                        .total(),
                    calendar_total
                );
            }
        }
    }

    #[test]
    fn paused_source_invalidates_mini_counts_when_counter_period_changes() {
        use crate::input::Device;
        use chrono::{TimeZone, Timelike};
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source-period.sqlite3");
        let mut writer = CollectorDb::open(&path).unwrap();
        let now = Local::now();
        // Keep yesterday's final minute inside the rolling window at any UTC hour.
        let zone = crate::timezone::TimeZonePreference::Named(
            if now.with_timezone(&chrono::Utc).hour() < 12 {
                "UTC"
            } else {
                "Pacific/Honolulu"
            }
            .parse()
            .unwrap(),
        );
        let projected = zone.at(now);
        let midnight = projected
            .timezone()
            .from_local_datetime(&projected.date_naive().and_hms_opt(0, 0, 0).unwrap())
            .single()
            .unwrap();
        let events = [now.timestamp_millis(), midnight.timestamp_millis() - 60_000].map(|ts_ms| {
            InputEvent {
                shortcut: None,
                ts_ms,
                device: Device::Keyboard,
                code: 4,
            }
        });
        writer.insert(&events).unwrap();
        let mut runtime = Runtime::source_runtime_with_lock(
            &path,
            true,
            false,
            &temp.path().join("source-view.lock"),
        )
        .unwrap();
        let read = |period| {
            let (tx, rx) = bounded(1);
            runtime
                .request_mini_snapshot(zone, period, move |r| {
                    tx.send(r).unwrap();
                })
                .unwrap();
            let value: serde_json::Value =
                serde_json::from_str(&rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap())
                    .unwrap();
            value["today"]["keyboard"].as_i64().unwrap()
        };
        assert_eq!(read(TodayPeriod::CalendarDay), 1);
        assert_eq!(read(TodayPeriod::Rolling24h), 2);
        writer.insert(&events[..1]).unwrap();
        assert_eq!(read(TodayPeriod::Rolling24h), 2);
        assert_eq!(read(TodayPeriod::CalendarDay), 1);
        runtime.set_paused(false);
        assert_eq!(read(TodayPeriod::CalendarDay), 2);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn inactive_recorder_clears_heartbeat_once_without_idle_writes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("heartbeat.sqlite3");
        let storage = Storage::App(CollectorDb::open(&path).unwrap());
        let observer = rusqlite::Connection::open(&path).unwrap();
        let version = || {
            observer
                .query_row("PRAGMA data_version", [], |r| r.get::<_, i64>(0))
                .unwrap()
        };
        let mut log = StatusLog::default();
        storage.log_status(&mut log, true).unwrap();
        assert!(
            LiveSource::open(&path)
                .unwrap()
                .is_recording(Local::now().timestamp_millis())
                .unwrap()
        );
        storage.log_status(&mut log, false).unwrap();
        assert!(
            !LiveSource::open(&path)
                .unwrap()
                .is_recording(Local::now().timestamp_millis())
                .unwrap()
        );
        let idle = version();
        for _ in 0..3 {
            storage.log_status(&mut log, false).unwrap();
        }
        assert_eq!(version(), idle);
        storage.log_status(&mut log, true).unwrap();
        assert!(version() > idle);
    }

    #[test]
    fn failed_data_handoff_retains_connection_and_pending_events() {
        let temp = tempfile::tempdir().unwrap();
        let (tx, rx) = bounded::<QueuedJob>(1);
        drop(rx);
        let mut worker = StorageWorker {
            flush_timeout: Duration::from_millis(100),
            helper: DataWorker {
                tx: Some(tx),
                thread: None,
            },
            db: Some(Storage::Independent(
                EventDb::open(&temp.path().join("handoff.sqlite3")).unwrap(),
            )),
            job: None,
            deferred: Default::default(),
            batch: vec![InputEvent {
                shortcut: None,
                ts_ms: chrono::Utc::now().timestamp_millis(),
                device: crate::input::Device::Keyboard,
                code: 8,
            }],
        };
        assert!(
            worker
                .spawn_job(|_| Box::new(|| panic!("disconnected job ran")))
                .is_err()
        );
        assert!(worker.db.is_some());
        assert!(worker.job.is_none());
        assert_eq!(worker.batch.len(), 1);
        worker.flush().unwrap();
        assert_eq!(worker.db.as_ref().unwrap().lifetime().unwrap().keyboard, 1);
        worker.helper.tx = None;
        assert!(worker.spawn_job(|_| Box::new(|| {})).is_err());
        assert!(worker.db.is_some());
        worker.db = None;
        assert!(worker.flush().is_err());
    }

    #[test]
    fn consecutive_data_jobs_reuse_one_thread_and_return_the_connection() {
        let temp = tempfile::tempdir().unwrap();
        let mut worker = StorageWorker {
            flush_timeout: Duration::from_millis(100),
            helper: DataWorker::new().unwrap(),
            db: Some(Storage::Independent(
                EventDb::open(&temp.path().join("jobs.sqlite3")).unwrap(),
            )),
            job: None,
            deferred: Default::default(),
            batch: Vec::new(),
        };
        let mut ids = Vec::new();
        for _ in 0..5 {
            let (tx, rx) = bounded(1);
            worker
                .spawn_job(move |db| {
                    assert_eq!(db.lifetime().unwrap().total(), 0);
                    let id = thread::current().id();
                    Box::new(move || tx.send(id).unwrap())
                })
                .unwrap();
            worker.wait_for_job().unwrap();
            ids.push(rx.recv_timeout(Duration::from_secs(1)).unwrap());
        }
        assert!(ids.iter().all(|id| *id == ids[0]));
        assert_ne!(ids[0], thread::current().id());
    }

    #[test]
    fn queued_sample_and_shutdown_wait_for_import_and_all_accepted_input() {
        let temp = tempfile::tempdir().unwrap();
        let current = temp.path().join("current.sqlite3");
        let source = temp.path().join("locked.sqlite3");
        let lock = rusqlite::Connection::open(&source).unwrap();
        lock.execute_batch(&format!("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER,PRIMARY KEY(bucket,dev,code)); INSERT INTO events VALUES(28000000,1,{},5); BEGIN EXCLUSIVE;", crate::collector::PHYSICAL_CODE_BASE)).unwrap();
        let mut runtime = Runtime::start_owned(&current, true).unwrap();
        let (reply, imported) = bounded(1);
        runtime
            .request_import_history(source, move |result| {
                let _ = reply.send(result);
            })
            .unwrap();
        assert!(imported.recv_timeout(Duration::from_millis(50)).is_err());
        for _ in 0..20_000 {
            runtime
                .tx
                .send_timeout(
                    Command::Input(InputEvent {
                        shortcut: None,
                        ts_ms: Local::now().timestamp_millis(),
                        device: crate::input::Device::Mouse,
                        code: 1,
                    }),
                    Duration::from_secs(1),
                )
                .unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        while runtime.tx.is_full() && Instant::now() < deadline {
            thread::yield_now();
        }
        assert!(!runtime.tx.is_full());
        let (reply, sampled) = bounded(1);
        let start = Instant::now();
        runtime
            .request_sport_sample(move |result| {
                let _ = reply.send(result);
            })
            .unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(sampled.recv_timeout(Duration::from_millis(50)).is_err());
        lock.execute_batch("ROLLBACK").unwrap();
        // Quit must still finish the accepted import, buffered input and queued read.
        runtime.shutdown().unwrap();
        assert_eq!(
            imported
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .unwrap()
                .added
                .keyboard,
            5
        );
        let sample: serde_json::Value = serde_json::from_str(
            &sampled
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(sample["keyboard"], 5);
        assert_eq!(sample["mouse"], 20_000);
        assert_eq!(
            LiveSource::open(&current).unwrap().lifetime().unwrap(),
            crate::db::Counts {
                keyboard: 5,
                mouse: 20_000
            }
        );
    }

    #[test]
    fn committed_job_reply_survives_a_later_buffered_input_write_failure() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("committed.sqlite3");
        let mut db = CollectorDb::open(&path).unwrap();
        db.insert(&[InputEvent {
            shortcut: None,
            ts_ms: Local::now().timestamp_millis(),
            device: crate::input::Device::Keyboard,
            code: 0,
        }])
        .unwrap();
        db.conn.execute_batch("CREATE TRIGGER fail_later_input BEFORE INSERT ON events WHEN NEW.dev=0 BEGIN SELECT RAISE(ABORT,'test input flush failure'); END;").unwrap();
        let (tx, rx) = bounded(1);
        let completion = Completion::new(move |result: Result<usize>| {
            let _ = tx.send(result);
        });
        let mut worker = StorageWorker {
            flush_timeout: Duration::from_millis(100),
            helper: DataWorker::new().expect("create data worker"),
            db: None,
            job: None,
            deferred: Default::default(),
            batch: vec![InputEvent {
                shortcut: None,
                ts_ms: Local::now().timestamp_millis(),
                device: crate::input::Device::Mouse,
                code: 1,
            }],
        };
        assert!(
            worker
                .finish((
                    Storage::App(db),
                    Box::new(move || completion.complete(Ok(1)))
                ))
                .is_err()
        );
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap().unwrap(), 1);
        assert_eq!(
            LiveSource::open(&path).unwrap().lifetime().unwrap(),
            crate::db::Counts {
                keyboard: 1,
                mouse: 0
            }
        );
    }
    #[test]
    fn asynchronous_import_returns_before_a_source_lock_and_keeps_queued_input() {
        let temp = tempfile::tempdir().unwrap();
        let current = temp.path().join("current.sqlite3");
        let source = temp.path().join("locked.sqlite3");
        let source_conn = rusqlite::Connection::open(&source).unwrap();
        source_conn.execute_batch(&format!("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER,PRIMARY KEY(bucket,dev,code)); INSERT INTO events VALUES(28000000,1,{},5); BEGIN EXCLUSIVE;",crate::collector::PHYSICAL_CODE_BASE)).unwrap();
        let mut runtime = Runtime::start_owned(&current, true).unwrap();
        let (reply, receive) = bounded(1);
        let start = Instant::now();
        runtime
            .request_import_history(source, move |result| {
                let _ = reply.send(result);
            })
            .unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(receive.recv_timeout(Duration::from_millis(50)).is_err());
        // More than the 8192-slot input queue: a blocked import must not stall it.
        for _ in 0..20_000 {
            runtime
                .tx
                .send_timeout(
                    Command::Input(InputEvent {
                        shortcut: None,
                        ts_ms: Local::now().timestamp_millis(),
                        device: crate::input::Device::Mouse,
                        code: 1,
                    }),
                    Duration::from_secs(1),
                )
                .unwrap();
        }
        source_conn.execute_batch("ROLLBACK").unwrap();
        assert_eq!(
            receive
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
                .unwrap()
                .added
                .keyboard,
            5
        );
        assert_eq!(
            runtime.snapshot().unwrap().stats.lifetime,
            crate::db::Counts {
                keyboard: 5,
                mouse: 20_000
            }
        );
        runtime.shutdown().unwrap();
    }

    #[test]
    fn dropping_a_queued_operation_notifies_the_interface() {
        let (reply, receive) = bounded(1);
        let completion = Completion::<()>::new(move |result| {
            let _ = reply.send(result);
        });
        drop(completion);
        assert!(
            receive
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .is_err()
        );
    }
    #[test]
    fn relocation_drains_pending_input_retains_pause_and_routes_future_input_to_new_file() {
        let temp = tempfile::tempdir().unwrap();
        let original = temp.path().join("old.sqlite3");
        let directory = temp.path().join("新目录");
        std::fs::create_dir(&directory).unwrap();
        let config = temp.path().join("profile/storage.json");
        let mut runtime = Runtime::start_owned(&original, true).unwrap();
        let now = Local::now().timestamp_millis();
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: now,
                device: crate::input::Device::Keyboard,
                code: 0,
            }))
            .unwrap();
        assert!(
            runtime
                .change_directory_with_config(&directory, &config)
                .unwrap()
        );
        assert!(runtime.is_paused());
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.keyboard, 1);
        assert!(
            !runtime
                .change_directory_with_config(&directory, &config)
                .unwrap()
        );
        assert!(Runtime::start_owned(&directory.join("events.sqlite3"), true).is_err());
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: now + 1,
                device: crate::input::Device::Mouse,
                code: 1,
            }))
            .unwrap();
        runtime.shutdown().unwrap();
        assert_eq!(
            LiveSource::open(&original).unwrap().lifetime().unwrap(),
            crate::db::Counts {
                keyboard: 1,
                mouse: 0
            }
        );
        assert_eq!(
            LiveSource::open(&directory.join("events.sqlite3"))
                .unwrap()
                .lifetime()
                .unwrap(),
            crate::db::Counts {
                keyboard: 1,
                mouse: 1
            }
        );
    }

    #[test]
    fn import_flushes_pending_input_and_continues_recording_after_duplicate_import() {
        let temp = tempfile::tempdir().unwrap();
        let current = temp.path().join("current.sqlite3");
        let source = temp.path().join("source.sqlite3");
        let mut source_db = CollectorDb::open(&source).unwrap();
        let at = Local::now().timestamp_millis() - 120_000;
        source_db
            .insert(&[InputEvent {
                shortcut: None,
                ts_ms: at,
                device: crate::input::Device::Keyboard,
                code: 0,
            }])
            .unwrap();
        let mut runtime = Runtime::start_owned(&current, false).unwrap();
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: at,
                device: crate::input::Device::Mouse,
                code: 1,
            }))
            .unwrap();
        assert_eq!(
            runtime
                .import_history(source.clone())
                .unwrap()
                .added
                .keyboard,
            1
        );
        assert_eq!(runtime.import_history(source).unwrap().added.total(), 0);
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: at,
                device: crate::input::Device::Mouse,
                code: 1,
            }))
            .unwrap();
        runtime.shutdown().unwrap();
        assert_eq!(
            LiveSource::open(&current).unwrap().lifetime().unwrap(),
            crate::db::Counts {
                keyboard: 1,
                mouse: 2
            }
        );
    }
    use crate::input::Device;

    #[test]
    fn sport_sample_flushes_both_devices_without_materializing_history() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("sport.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        for device in [Device::Mouse, Device::Mouse, Device::Keyboard] {
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    shortcut: None,
                    ts_ms: Local::now().timestamp_millis(),
                    device,
                    code: 1,
                }))
                .unwrap();
        }
        let caller = thread::current().id();
        let (reply, response) = bounded(1);
        runtime
            .request_sport_sample(move |result| {
                reply.send((thread::current().id(), result)).unwrap();
            })
            .unwrap();
        let (worker, result) = response.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_ne!(worker, caller);
        let value: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(value["mouse"], 2);
        assert_eq!(value["keyboard"], 1);
        assert!(value.get("history_days").is_none());
        assert_eq!(value.as_object().unwrap().len(), 5);
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.keyboard, 1);
        // A broken history table must not block this lightweight counter path.
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute("DROP TABLE daily_totals", []).unwrap();
        let (reply, response) = bounded(1);
        runtime
            .request_sport_sample(move |result| {
                reply.send(result).unwrap();
            })
            .unwrap();
        let result = response
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["mouse"], 2);
        assert_eq!(value["keyboard"], 1);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn abandoned_sport_delivery_releases_the_ui_gate() {
        let (reply, response) = bounded(1);
        drop(SportDelivery {
            capture: Arc::new(Mutex::new(CaptureStatus::default())),
            paused: Arc::new(AtomicBool::new(false)),
            reply: Some(Box::new(move |result| {
                reply.send(result.is_err()).unwrap();
            })),
        });
        assert!(response.recv_timeout(Duration::from_millis(500)).unwrap());
    }

    #[test]
    fn async_snapshot_serializes_on_worker_after_flushing_pending_input() {
        let temp = tempfile::tempdir().unwrap();
        let mut runtime = Runtime::start_owned(&temp.path().join("owned.sqlite3"), false).unwrap();
        for _ in 0..10 {
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    shortcut: None,
                    ts_ms: Local::now().timestamp_millis(),
                    device: Device::Keyboard,
                    code: 0,
                }))
                .unwrap();
        }
        let caller = thread::current().id();
        let (reply, response) = bounded(1);
        runtime
            .request_snapshot(move |result| {
                reply.send((thread::current().id(), result)).unwrap();
            })
            .unwrap();
        // Assert delivery and counts, without a two-second performance budget
        // on busy native CI hosts. Thread isolation is verified below.
        let (worker, result) = response.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_ne!(worker, caller);
        let value: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(value["lifetime"]["keyboard"], 10);
        assert_eq!(value["keyboard_stats"]["physical"][0]["count"], 10);
        assert_eq!(value["database_mode"], "default");
        assert_eq!(value["auto_start_available"], true);
        assert!(runtime.can_manage_auto_start());
        runtime.shutdown().unwrap();
    }

    #[test]
    fn custom_database_cannot_manage_or_report_default_recorder_login_startup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("independent.sqlite3");
        let mut runtime = Runtime::start(&path, None, true).unwrap();
        assert!(!runtime.can_manage_auto_start());
        let snapshot = runtime.snapshot().unwrap();
        assert_eq!(snapshot.database_mode, DatabaseMode::Custom);
        assert_eq!(snapshot.database_path, path.display().to_string());
        assert!(!snapshot.auto_start_available);
        assert!(!snapshot.auto_start);

        let (reply, response) = bounded(1);
        runtime
            .request_snapshot(move |result| reply.send(result).unwrap())
            .unwrap();
        let json = response
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["database_mode"], "custom");
        assert_eq!(value["auto_start_available"], false);
        assert_eq!(value["auto_start"], false);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn async_snapshot_returns_without_waiting_on_a_full_queue() {
        let (tx, rx) = bounded(1);
        tx.send(Command::Input(InputEvent {
            shortcut: None,
            ts_ms: 0,
            device: Device::Mouse,
            code: 1,
        }))
        .unwrap();
        let runtime = Runtime {
            _instance_lock: tempfile::tempfile().unwrap().into(),
            tx,
            writer: None,
            capture: None,
            stop_capture: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            status: Arc::new(Mutex::new(CaptureStatus::default())),
            path: PathBuf::from("fixture.sqlite3"),
            source_mode: false,
            setup_pending: false,
            database_mode: DatabaseMode::Custom,
        };
        let (reply, response) = bounded(1);
        let worker = thread::spawn(move || {
            reply
                .send(runtime.request_snapshot(|_| {}).is_err())
                .unwrap();
        });
        let result = response.recv_timeout(Duration::from_millis(500));
        // Release a mistakenly blocking implementation before joining so this
        // test reports a failure instead of leaving a permanently stuck thread.
        drop(rx);
        worker.join().unwrap();
        assert!(result.unwrap());
    }

    #[test]
    fn failed_storage_completes_pending_async_requests_with_an_error() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("failure.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute("DROP TABLE daily_totals", []).unwrap();
        runtime
            .tx
            .send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: Local::now().timestamp_millis(),
                device: Device::Keyboard,
                code: 0,
            }))
            .unwrap();
        let (reply, response) = bounded(1);
        runtime
            .request_snapshot(move |result| {
                reply.send(result.is_err()).unwrap();
            })
            .unwrap();
        assert!(response.recv_timeout(Duration::from_secs(2)).unwrap());
        assert!(runtime.shutdown().is_err());
    }

    #[test]
    fn status_log_rotation_keeps_three_archives_in_recency_order() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("clickpulse.log");
        let mut log = StatusLog {
            path: Some(path.clone()),
            ..Default::default()
        };
        for number in 0..8 {
            log.write_bounded(&format!("line{number}\n"), 6);
            assert!(!log.failure_reported);
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "line7\n");
        for index in 1..=STATUS_LOG_BACKUPS {
            let archive = status_log_archive(&path, index);
            assert_eq!(
                std::fs::read_to_string(&archive).unwrap(),
                format!("line{}\n", 7 - index)
            );
            assert!(archive.metadata().unwrap().len() <= 6);
        }
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 4);
    }

    #[test]
    fn status_log_bounds_legacy_files_and_rejects_oversized_lines() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("clickpulse.log");
        std::fs::write(&path, "older diagnostics\nrecent").unwrap();
        for index in 1..=STATUS_LOG_BACKUPS {
            std::fs::write(
                status_log_archive(&path, index),
                format!("older diagnostics\nprev0{index}"),
            )
            .unwrap();
        }
        append_status_log(&path, "now\n", 6).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "now\n");
        for (index, expected) in [(1, "recent"), (2, "prev01"), (3, "prev02")] {
            assert_eq!(
                std::fs::read_to_string(status_log_archive(&path, index)).unwrap(),
                expected
            );
        }
        assert!(append_status_log(&path, "too-long\n", 6).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "now\n");
        let retained_bytes: u64 = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|file| file.unwrap().metadata().unwrap().len())
            .sum();
        assert!(retained_bytes <= 4 * 6);

        // A slightly oversized legacy file exercises overlapping, multi-buffer
        // compaction without allocating a full production-sized log fixture.
        let large_path = temp.path().join("legacy.log");
        let legacy: Vec<u8> = (0..70_007).map(|index| (index % 251) as u8).collect();
        std::fs::write(&large_path, &legacy).unwrap();
        append_status_log(&large_path, "now\n", 70_000).unwrap();
        assert_eq!(
            std::fs::read(status_log_archive(&large_path, 1)).unwrap(),
            legacy[7..]
        );
    }

    #[test]
    fn status_log_skips_successful_duplicates_but_retries_after_failure() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("clickpulse.log");
        let mut log = StatusLog {
            path: Some(path.clone()),
            ..Default::default()
        };
        log.write_bounded("old\n", 8);
        log.write_bounded("old\n", 8);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "old\n");
        let blocked_archive = status_log_archive(&path, STATUS_LOG_BACKUPS);
        std::fs::create_dir(&blocked_archive).unwrap();
        log.write_bounded("new\n", 8);
        assert!(log.failure_reported);
        assert_eq!(log.last_written.as_deref(), Some("old\n"));
        std::fs::remove_dir(blocked_archive).unwrap();
        // Even the last successfully written value must retry after a failure.
        log.write_bounded("old\n", 8);
        assert!(!log.failure_reported);
        log.write_bounded("old\n", 8);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "old\nold\n");
    }

    #[test]
    fn archive_failure_does_not_stop_database_counts_or_heartbeat() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let log_path = temp.path().join("clickpulse.log");
        let blocked_archive = status_log_archive(&log_path, STATUS_LOG_BACKUPS);
        std::fs::create_dir(&blocked_archive).unwrap();
        let mut log = StatusLog {
            path: Some(log_path.clone()),
            ..Default::default()
        };
        let mut db = Storage::App(CollectorDb::open(&path).unwrap());
        db.insert(&[InputEvent {
            shortcut: None,
            ts_ms: Local::now().timestamp_millis(),
            device: Device::Mouse,
            code: 1,
        }])
        .unwrap();
        let before_status = Local::now().timestamp_millis();
        db.log_status(&mut log, true).unwrap();
        assert!(log.failure_reported);
        assert_eq!(db.snapshot(Local::now()).unwrap().lifetime.mouse, 1);
        let heartbeat = rusqlite::Connection::open(&path).unwrap();
        let last_heartbeat: i64 = heartbeat
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM clickpulse_metadata WHERE key='heartbeat_ms'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(last_heartbeat >= before_status);
        std::fs::remove_dir(blocked_archive).unwrap();
        db.log_status(&mut log, true).unwrap();
        assert!(!log.failure_reported);
        assert!(
            std::fs::read_to_string(log_path)
                .unwrap()
                .contains("total events (all time): 1")
        );
    }

    #[test]
    fn unavailable_log_does_not_stop_database_counts_and_recovers_on_next_write() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let log_path = temp.path().join("clickpulse.log");
        std::fs::create_dir(&log_path).unwrap(); // A directory cannot be appended as a file.
        let mut log = StatusLog {
            path: Some(log_path.clone()),
            ..Default::default()
        };
        let mut db = Storage::App(CollectorDb::open(&path).unwrap());
        db.log_status(&mut log, true).unwrap();
        assert!(log.failure_reported);
        let event = InputEvent {
            shortcut: None,
            ts_ms: Local::now().timestamp_millis(),
            device: Device::Mouse,
            code: 1,
        };
        db.insert(&[event]).unwrap();
        assert_eq!(db.snapshot(Local::now()).unwrap().lifetime.mouse, 1);
        db.log_status(&mut log, true).unwrap();
        assert!(log.failure_reported);
        std::fs::remove_dir(&log_path).unwrap();
        db.log_status(&mut log, true).unwrap();
        assert!(!log.failure_reported);
        assert!(
            std::fs::read_to_string(log_path)
                .unwrap()
                .contains("total events (all time): 1")
        );
    }

    #[test]
    fn handoff_probe_discards_inputs_without_modifying_live_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source.sqlite3");
        let writer = rusqlite::Connection::open(&path).unwrap();
        writer.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER);INSERT INTO events VALUES(2,1,97,7)").unwrap();
        let (tx, rx) = bounded(512);
        let source = LiveSource::open(&path).unwrap();
        let worker = thread::spawn(move || {
            source_loop_inner(
                source,
                rx,
                Arc::new(AtomicBool::new(false)),
                Arc::new(Mutex::new(CaptureStatus::default())),
                true,
            )
        });
        for _ in 0..257 {
            tx.send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: 123456,
                device: Device::Keyboard,
                code: 0,
            }))
            .unwrap();
        }
        let (reply, result) = bounded(1);
        tx.send(Command::Snapshot(reply)).unwrap();
        assert_eq!(result.recv().unwrap().unwrap().lifetime.total(), 7);
        tx.send(Command::Stop).unwrap();
        worker.join().unwrap().unwrap();
        assert_eq!(
            writer
                .query_row("SELECT SUM(cnt) FROM events", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            7
        );
    }

    #[test]
    fn paused_source_cache_respects_shortcut_scope_on_navigation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source.sqlite3");
        let writer = CollectorDb::open(&path).unwrap();
        drop(writer);
        let writer = rusqlite::Connection::open(&path).unwrap();
        crate::shortcuts::insert(
            &writer,
            crate::shortcuts::Chord {
                key: 8,
                modifiers: 1,
            },
            Local::now().date_naive(),
        )
        .unwrap();
        let source = LiveSource::open(&path).unwrap();
        let (tx, rx) = bounded(4);
        let paused = Arc::new(AtomicBool::new(true));
        let capture = Arc::new(Mutex::new(CaptureStatus::default()));
        let worker_paused = paused.clone();
        let worker_capture = capture.clone();
        let worker = thread::spawn(move || {
            source_loop_inner(source, rx, worker_paused, worker_capture, true)
        });
        let today = Local::now().date_naive();
        for (include_shortcuts, share_day) in [
            (false, None),
            (true, None),
            (false, Some(today)),
            (false, Some(today.pred_opt().unwrap())),
            (false, Some(today)),
            (false, None),
            (true, None),
        ] {
            let (reply, result) = bounded(1);
            tx.send(Command::SnapshotAsync(SnapshotDelivery {
                share_day,
                include_shortcuts,
                timezone: Default::default(),
                today_period: Default::default(),
                capture: capture.clone(),
                paused: paused.clone(),
                path: path.clone(),
                source_mode: true,
                setup_pending: true,
                database_mode: DatabaseMode::Source,
                reply: Some(Box::new(move |value| {
                    reply.send(value).unwrap();
                })),
            }))
            .unwrap();
            let value: serde_json::Value = serde_json::from_str(
                &result
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(value.get("share_shortcut").is_some(), share_day.is_some());
            if let Some(day) = share_day {
                assert_eq!(value["share_shortcut"]["day"], day.to_string());
                if day == today {
                    assert_eq!(value["share_shortcut"]["top"]["count"], 1);
                } else {
                    assert!(value["share_shortcut"]["top"].is_null());
                }
            }
            assert_eq!(value.get("shortcuts").is_some(), include_shortcuts);
            if include_shortcuts {
                assert_eq!(value["shortcuts"][0]["count"], 1);
            }
        }
        tx.send(Command::Stop).unwrap();
        worker.join().unwrap().unwrap();
    }

    #[test]
    fn async_writer_omits_shortcuts_only_when_the_view_does_not_request_them() {
        let temp = tempfile::tempdir().unwrap();
        for owned in [false, true] {
            let path = temp.path().join(if owned {
                "owned.sqlite3"
            } else {
                "independent.sqlite3"
            });
            let mut runtime = if owned {
                Runtime::start_owned(&path, true)
            } else {
                Runtime::start(&path, None, true)
            }
            .unwrap();
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    ts_ms: Local::now().timestamp_millis(),
                    device: Device::Keyboard,
                    code: 8,
                    shortcut: Some(crate::shortcuts::Chord {
                        key: 8,
                        modifiers: 1,
                    }),
                }))
                .unwrap();
            for include in [false, true, false, true] {
                let (tx, rx) = bounded(1);
                runtime
                    .request_snapshot_for_view(Default::default(), include, move |r| {
                        tx.send(r).unwrap();
                    })
                    .unwrap();
                let value: serde_json::Value = serde_json::from_str(
                    &rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap(),
                )
                .unwrap();
                assert_eq!(value["lifetime"]["keyboard"], 1);
                assert_eq!(value.get("shortcuts").is_some(), include);
                if include {
                    assert_eq!(value["shortcuts"][0]["count"], 1);
                }
            }
            let today = Local::now().date_naive();
            for day in [today, today.pred_opt().unwrap(), today] {
                let (tx, rx) = bounded(1);
                runtime
                    .request_share_snapshot(Default::default(), day, move |r| {
                        tx.send(r).unwrap();
                    })
                    .unwrap();
                let value: serde_json::Value = serde_json::from_str(
                    &rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap(),
                )
                .unwrap();
                assert!(value.get("shortcuts").is_none());
                assert_eq!(value["share_shortcut"]["day"], day.to_string());
                if day == today {
                    assert_eq!(value["share_shortcut"]["top"]["count"], 1);
                } else {
                    assert!(value["share_shortcut"]["top"].is_null());
                }
            }
            let (tx, rx) = bounded(1);
            runtime
                .request_snapshot_for_view(Default::default(), false, move |r| {
                    tx.send(r).unwrap();
                })
                .unwrap();
            let value: serde_json::Value =
                serde_json::from_str(&rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap())
                    .unwrap();
            assert!(value.get("share_shortcut").is_none());
            runtime.shutdown().unwrap();
        }
    }

    #[test]
    fn source_pause_freezes_display_then_resumes_latest_committed_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("live.sqlite3");
        let writer = rusqlite::Connection::open(&path).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,3)").unwrap();
        let source = LiveSource::open(&path).unwrap();
        let (tx, rx) = bounded(4);
        let paused = Arc::new(AtomicBool::new(false));
        let worker_paused = paused.clone();
        let worker = thread::spawn(move || {
            source_loop(
                source,
                rx,
                worker_paused,
                Arc::new(Mutex::new(CaptureStatus::default())),
            )
        });
        let snapshot = || {
            let (reply, result) = bounded(1);
            tx.send(Command::Snapshot(reply)).unwrap();
            result.recv().unwrap().unwrap()
        };
        assert_eq!(snapshot().lifetime.total(), 3);
        paused.store(true, Ordering::Relaxed);
        tx.send(Command::StateChanged).unwrap();
        assert_eq!(snapshot().lifetime.total(), 3); // pause has reached the worker
        writer
            .execute("INSERT INTO events VALUES(3,0,1,2)", [])
            .unwrap();
        assert_eq!(snapshot().lifetime.total(), 3);
        paused.store(false, Ordering::Relaxed);
        assert_eq!(snapshot().lifetime.total(), 5);
        tx.send(Command::Stop).unwrap();
        worker.join().unwrap().unwrap();
    }

    #[test]
    fn paused_source_keeps_mini_share_and_changed_scope_on_one_copy() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("source.sqlite3");
        let now = Local::now();
        let mut writer = CollectorDb::open(&path).unwrap();
        let event = InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Keyboard,
            code: 8,
            shortcut: crate::shortcuts::Chord::new(8, crate::shortcuts::CONTROL, "windows"),
        };
        writer.insert(std::slice::from_ref(&event)).unwrap();
        let mut runtime = Runtime::source_runtime_with_lock(
            &path,
            false,
            false,
            &temporary.path().join("source-view.lock"),
        )
        .unwrap();
        runtime.set_paused(true);
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.total(), 1);
        writer.insert(&[event.clone(), event]).unwrap();
        let zone = crate::timezone::TimeZonePreference::Named(chrono_tz::UTC);
        for include_shortcuts in [false, true] {
            let (send, receive) = bounded(1);
            runtime
                .request_snapshot_for_view(zone, include_shortcuts, move |reply| {
                    send.send(reply).unwrap();
                })
                .unwrap();
            let value: serde_json::Value =
                serde_json::from_str(&receive.recv().unwrap().unwrap()).unwrap();
            assert_eq!(value["lifetime"]["keyboard"], 1);
        }
        let (send, receive) = bounded(1);
        runtime
            .request_mini_snapshot(zone, TodayPeriod::Rolling24h, move |reply| {
                send.send(reply).unwrap();
            })
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&receive.recv().unwrap().unwrap()).unwrap();
        assert_eq!(value["today"]["keyboard"], 1);
        let (send, receive) = bounded(1);
        runtime
            .request_share_snapshot(zone, now.date_naive(), move |reply| {
                send.send(reply).unwrap();
            })
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&receive.recv().unwrap().unwrap()).unwrap();
        assert_eq!(value["share_shortcut"]["top"]["count"], 1);
        runtime.set_paused(false);
        assert_eq!(runtime.snapshot().unwrap().stats.lifetime.total(), 3);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn storage_failure_propagates_and_rolls_back_partial_transaction() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("fail.sqlite3");
        let db = EventDb::open(&path).unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TRIGGER fail BEFORE INSERT ON recent BEGIN SELECT RAISE(FAIL,'simulated write failure'); END;").unwrap();
        let (tx, rx) = bounded(2);
        let writer = thread::spawn(move || writer_loop(db, rx));
        tx.send(Command::Input(InputEvent {
            shortcut: None,
            ts_ms: 123_456,
            device: Device::Mouse,
            code: 1,
        }))
        .unwrap();
        tx.send(Command::Stop).unwrap();
        assert!(
            writer
                .join()
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("simulated write failure")
        );
        assert_eq!(EventDb::open(&path).unwrap().lifetime().unwrap().total(), 0);
    }

    #[test]
    fn one_runtime_per_database_and_lock_releases_on_exit() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let first = Runtime::start(&path, None, true).unwrap();
        assert!(Runtime::start(&path, None, true).is_err());
        drop(first);
        let second = Runtime::start(&path, None, true).unwrap();
        assert!(second.snapshot().unwrap().paused);
    }

    #[cfg(unix)]
    #[test]
    fn database_symlink_shares_instance_lock_and_releases_on_exit() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let alias = temp.path().join("alias.sqlite3");
        let first = Runtime::start(&path, None, true).unwrap();
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(Runtime::start(&alias, None, true).is_err());
        drop(first);
        let second = Runtime::start(&alias, None, true).unwrap();
        assert_eq!(
            second.snapshot().unwrap().database_path,
            alias.display().to_string()
        );
    }

    #[cfg(unix)]
    #[test]
    fn new_database_in_symlinked_directory_uses_canonical_lock() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("real");
        let alias = temp.path().join("alias");
        std::fs::create_dir(&directory).unwrap();
        std::os::unix::fs::symlink(&directory, &alias).unwrap();
        let first = Runtime::start(&alias.join("db.sqlite3"), None, true).unwrap();
        assert!(Runtime::start(&directory.join("db.sqlite3"), None, true).is_err());
        drop(first);
        assert!(Runtime::start(&directory.join("db.sqlite3"), None, true).is_ok());
    }

    #[test]
    fn storage_tests_never_start_native_input_capture() {
        let temp = tempfile::tempdir().unwrap();
        let mut runtime =
            Runtime::start(&temp.path().join("isolated.sqlite3"), None, false).unwrap();
        assert!(runtime.capture.is_none());
        assert!(StatusLog::for_runtime().path.is_none());
        runtime.shutdown().unwrap();
    }

    #[test]
    fn capture_thread_failure_still_drains_and_joins_storage() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("shutdown.sqlite3");
        let mut runtime = Runtime::start(&path, None, true).unwrap();
        runtime.capture = Some(thread::spawn(|| panic!("simulated capture failure")));
        for _ in 0..17 {
            runtime
                .tx
                .send(Command::Input(InputEvent {
                    shortcut: None,
                    ts_ms: 123_456,
                    device: Device::Mouse,
                    code: 1,
                }))
                .unwrap();
        }
        assert!(runtime.shutdown().is_err());
        assert!(
            runtime.writer.is_none(),
            "storage must join even when capture panics"
        );
        assert_eq!(EventDb::open(&path).unwrap().lifetime().unwrap().mouse, 17);
        runtime.shutdown().unwrap();
    }

    #[test]
    fn startup_keeps_decade_old_details_in_both_storage_modes() {
        for owned in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("permanent.sqlite3");
            let mut storage = if owned {
                Storage::App(CollectorDb::open(&path).unwrap())
            } else {
                Storage::Independent(EventDb::open(&path).unwrap())
            };
            storage
                .insert(&[InputEvent {
                    shortcut: None,
                    ts_ms: Local::now().timestamp_millis() - 4_000 * 86_400_000,
                    device: Device::Keyboard,
                    code: 0,
                }])
                .unwrap();
            drop(storage);
            let mut runtime = if owned {
                Runtime::start_owned(&path, true).unwrap()
            } else {
                Runtime::start(&path, None, true).unwrap()
            };
            assert_eq!(runtime.snapshot().unwrap().stats.lifetime.keyboard, 1);
            let conn = rusqlite::Connection::open_with_flags(
                &path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            let details = if owned { "events" } else { "minutes" };
            let cache = if owned { "recent_counts" } else { "recent" };
            assert_eq!(
                conn.query_row(&format!("SELECT SUM(cnt) FROM {details}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1
            );
            assert_eq!(
                conn.query_row(&format!("SELECT COUNT(*) FROM {cache}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
            runtime.shutdown().unwrap();
        }
    }

    #[test]
    fn stop_waits_for_a_short_external_lock_and_saves_the_final_batch() {
        for owned in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("final.sqlite3");
            let db = if owned {
                Storage::App(CollectorDb::open(&path).unwrap())
            } else {
                Storage::Independent(EventDb::open(&path).unwrap())
            };
            let external = rusqlite::Connection::open(&path).unwrap();
            external.execute_batch("BEGIN EXCLUSIVE").unwrap();
            let (tx, rx) = bounded(4);
            tx.send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: Local::now().timestamp_millis(),
                device: Device::Mouse,
                code: 1,
            }))
            .unwrap();
            tx.send(Command::Stop).unwrap();
            let writer =
                thread::spawn(move || storage_loop(db, rx, Default::default(), Default::default()));
            thread::sleep(Duration::from_millis(600));
            external.execute_batch("ROLLBACK").unwrap();
            writer.join().unwrap().unwrap();
            let table = if owned { "lifetime_totals" } else { "totals" };
            assert_eq!(
                external
                    .query_row(&format!("SELECT SUM(cnt) FROM {table}"), [], |row| row
                        .get::<_, i64>(0))
                    .unwrap(),
                1
            );
            if owned {
                assert_eq!(
                    external
                        .query_row(
                            "SELECT value FROM clickpulse_metadata WHERE key='heartbeat_ms'",
                            [],
                            |row| row.get::<_, String>(0)
                        )
                        .unwrap(),
                    "0"
                );
            }
        }
    }

    #[test]
    fn stop_drains_events_and_flushes_partial_batch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let db = EventDb::open(&path).unwrap();
        let (tx, rx) = bounded(512);
        let writer = thread::spawn(move || writer_loop(db, rx));
        for _ in 0..257 {
            tx.send(Command::Input(InputEvent {
                shortcut: None,
                ts_ms: 123_456,
                device: Device::Mouse,
                code: 1,
            }))
            .unwrap();
        }
        tx.send(Command::Stop).unwrap();
        writer.join().unwrap().unwrap();
        assert_eq!(EventDb::open(&path).unwrap().lifetime().unwrap().mouse, 257);
    }

    #[test]
    fn zoned_async_snapshot_is_tagged_and_shutdown_keeps_queued_input() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("zone.sqlite3");
        let mut runtime = Runtime::start_owned(&path, false).unwrap();
        let event = InputEvent {
            shortcut: None,
            ts_ms: Local::now().timestamp_millis(),
            device: Device::Keyboard,
            code: 1,
        };
        for _ in 0..5 {
            runtime.tx.send(Command::Input(event.clone())).unwrap();
        }
        let (tx, rx) = bounded(1);
        let timezone = crate::timezone::TimeZonePreference::Named("Asia/Shanghai".parse().unwrap());
        runtime
            .request_snapshot_in_timezone(timezone, move |r| {
                tx.send(r).unwrap();
            })
            .unwrap();
        for _ in 0..20000 {
            runtime.tx.send(Command::Input(event.clone())).unwrap();
        }
        let result = rx.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        let value: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["timezone"], "Asia/Shanghai");
        assert_eq!(value["lifetime"]["keyboard"], 5);
        assert_eq!(
            chrono::DateTime::parse_from_rfc3339(value["now"].as_str().unwrap())
                .unwrap()
                .offset()
                .local_minus_utc(),
            8 * 3600
        );
        runtime.shutdown().unwrap();
        assert_eq!(
            CollectorDb::open(&path)
                .unwrap()
                .lifetime()
                .unwrap()
                .keyboard,
            20005
        );
    }
    #[test]
    fn missing_explicit_legacy_database_fails_startup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let missing = temp.path().join("missing.sqlite3");
        assert!(Runtime::start(&path, Some(&missing), true).is_err());
        assert!(!missing.exists());
    }

    #[test]
    fn imported_legacy_database_can_be_missing_on_restart() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let legacy = temp.path().join("legacy.sqlite3");
        let conn = rusqlite::Connection::open(&legacy).unwrap();
        conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,3),(2,0,1,2);").unwrap();
        drop(conn);
        let first = Runtime::start(&path, Some(&legacy), true).unwrap();
        assert_eq!(first.snapshot().unwrap().stats.lifetime.total(), 5);
        drop(first);
        std::fs::remove_file(&legacy).unwrap();
        let second = Runtime::start(&path, Some(&legacy), true).unwrap();
        let snapshot = second.snapshot().unwrap();
        assert!(snapshot.stats.legacy_imported);
        assert_eq!(snapshot.stats.lifetime.total(), 5);
        assert!(!legacy.exists());
    }
}
