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

use crate::{
    collector::CollectorDb,
    db::{EventDb, Snapshot},
    input::InputEvent,
    source::LiveSource,
};

pub(crate) enum Command {
    Input(InputEvent),
    Snapshot(Sender<Result<Snapshot>>),
    SnapshotAsync(SnapshotDelivery),
    SportSample(SportDelivery),
    Import(
        PathBuf,
        PathBuf,
        Completion<crate::history_import::ImportReport>,
    ),
    Relocate(PathBuf, PathBuf, PathBuf, Completion<()>),
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
    lock: std::fs::File,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CaptureStatus {
    pub listening: bool,
    pub permission_needed: bool,
    pub error: Option<String>,
    pub error_code: Option<&'static str>,
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
                "available": capture.listening && capture.error.is_none(),
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
    timezone: crate::timezone::TimeZonePreference,
    capture: Arc<Mutex<CaptureStatus>>,
    paused: Arc<AtomicBool>,
    path: PathBuf,
    source_mode: bool,
    setup_pending: bool,
    database_mode: DatabaseMode,
    reply: Option<SnapshotCallback>,
}

impl SnapshotDelivery {
    fn complete(mut self, stats: Result<Snapshot>) {
        let result = (|| {
            let snapshot = AppSnapshot {
                stats: stats?,
                timezone: self.timezone,
                capture: self
                    .capture
                    .lock()
                    .map_err(|_| anyhow!("Capture status is unavailable"))?
                    .clone(),
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
    _instance_lock: std::fs::File,
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
        let lock_path = if path.exists() {
            std::fs::canonicalize(path)?
        } else {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            std::fs::canonicalize(parent)?.join(path.file_name().unwrap_or_default())
        }
        .with_extension("lock");
        let instance_lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        instance_lock
            .try_lock()
            .map_err(|_| anyhow!("此数据库已有 ClickPulse 正在运行，请从菜单栏打开原窗口"))?;
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
        let writer = thread::Builder::new()
            .name("clickpulse-storage".into())
            .spawn(move || {
                let result = storage_loop(db, rx);
                if let Err(error) = &result {
                    writer_stop.store(true, Ordering::Relaxed);
                    if let Ok(mut status) = writer_status.lock() {
                        status.listening = false;
                        status.error_code = Some("error.storage");
                        status.error = Some(format!("保存失败，记录已停止：{error:#}"));
                    }
                }
                result
            })?;
        #[cfg(all(any(target_os = "macos", target_os = "windows"), not(test)))]
        let capture = Some(crate::capture::spawn(
            tx.clone(),
            paused.clone(),
            stop_capture.clone(),
            status.clone(),
        )?);
        #[cfg(all(any(target_os = "macos", target_os = "windows"), test))]
        let capture = None; // Unit tests never listen to the user's desktop input.
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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
        let source = LiveSource::open(path)?;
        let lock_path = crate::platform::data_dir()?.join("source-view.lock");
        std::fs::create_dir_all(lock_path.parent().unwrap())?;
        let instance_lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        instance_lock
            .try_lock()
            .map_err(|_| anyhow!("ClickPulse 已在运行，请从菜单栏打开原窗口"))?;
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
            _instance_lock: instance_lock,
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
            .map_err(|_| anyhow!("统计线程不可用"))?;
        let stats = rx
            .recv_timeout(Duration::from_secs(5))
            .context("统计读取超时")??;
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
        self.tx
            .try_send(Command::SnapshotAsync(SnapshotDelivery {
                timezone,
                capture: self.status.clone(),
                paused: self.paused.clone(),
                path: self.path.clone(),
                source_mode: self.source_mode,
                setup_pending: self.setup_pending,
                database_mode: self.database_mode,
                reply: Some(Box::new(reply)),
            }))
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
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(output.with_extension("lock"))?;
        lock.try_lock()
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
    }

    pub fn capture_status(&self) -> Result<CaptureStatus> {
        Ok(self
            .status
            .lock()
            .map_err(|_| anyhow!("监听状态不可用"))?
            .clone())
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        // Stop and join all producers before draining the storage queue.
        self.stop_capture.store(true, Ordering::Relaxed);
        let capture_result = if let Some(capture) = self.capture.take() {
            capture.join().map_err(|_| anyhow!("监听线程异常退出"))
        } else {
            Ok(())
        };
        // A failed producer must not skip saving its already accepted events.
        let writer_result = if let Some(writer) = self.writer.take() {
            let _ = self.tx.send(Command::Stop);
            writer
                .join()
                .map_err(|_| anyhow!("保存线程异常退出"))
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
            eprintln!("ClickPulse 退出时保存失败：{error:#}");
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
    let mut cached: Option<(String, Snapshot)> = None;
    while let Ok(command) = rx.recv() {
        match command {
            Command::SportSample(delivery) => delivery.complete(source.lifetime()),
            Command::Snapshot(_) | Command::SnapshotAsync(_) => {
                let now = Local::now();
                let timezone = match &command {
                    Command::SnapshotAsync(delivery) => delivery.timezone,
                    _ => Default::default(),
                };
                let cache_key = timezone.cache_key();
                let result = if paused.load(Ordering::Relaxed)
                    && let Some((cached_zone, stats)) = &cached
                    && *cached_zone == cache_key
                {
                    Ok(stats.clone())
                } else {
                    source.snapshot_in_timezone(now, timezone)
                };
                if let Ok(stats) = &result {
                    cached = Some((cache_key, stats.clone()));
                }
                let mut current = status.lock().map_err(|_| anyhow!("同步状态不可用"))?;
                if !probe {
                    current.listening =
                        source.is_recording(now.timestamp_millis()).unwrap_or(false);
                }
                if !probe || result.is_err() {
                    current.error_code = result.is_err().then_some("error.source");
                    current.error = result
                        .as_ref()
                        .err()
                        .map(|e| format!("读取实时数据失败：{e:#}"));
                }
                drop(current);
                match command {
                    Command::Snapshot(reply) => {
                        let _ = reply.send(result);
                    }
                    Command::SnapshotAsync(delivery) => delivery.complete(result),
                    _ => unreachable!(),
                }
            }
            Command::Import(_, _, reply) => {
                reply.complete(Err(anyhow!("Read-only source cannot import history")));
            }
            Command::Relocate(_, _, _, reply) => {
                reply.complete(Err(anyhow!("Read-only source cannot change storage")));
            }
            Command::Stop => break,
            Command::Input(_) if probe => {} // Probe only: the old collector still owns persistence.
            Command::Input(_) => return Err(anyhow!("实时数据读取模式不应接收采集事件")),
        }
    }
    Ok(())
}

#[cfg(test)]
fn writer_loop(db: EventDb, rx: Receiver<Command>) -> Result<()> {
    storage_loop(Storage::Independent(db), rx)
}

enum Storage {
    Independent(EventDb),
    App(CollectorDb),
}
impl Storage {
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
    fn snapshot_in_timezone(
        &self,
        now: chrono::DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
    ) -> Result<Snapshot> {
        match self {
            Self::Independent(db) => db.snapshot_in_timezone(now, zone),
            Self::App(db) => db.snapshot_in_timezone(now, zone),
        }
    }
    fn prune_rolling_cache(&mut self, now: i64) -> Result<()> {
        match self {
            Self::Independent(db) => db.prune_rolling_cache(now),
            Self::App(db) => db.prune_rolling_cache(now),
        }
    }
    fn log_status(&self, log: &mut StatusLog) -> Result<()> {
        if let Self::App(db) = self {
            let now = Local::now();
            db.heartbeat(now.timestamp_millis())?;
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

const STATUS_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
const STATUS_LOG_BACKUPS: usize = 3;

fn status_log_archive(path: &Path, index: usize) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    name.into()
}

fn bound_status_log(path: &Path, limit: u64) -> Result<u64> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    anyhow::ensure!(metadata.is_file(), "日志路径不是普通文件");
    if metadata.len() <= limit {
        return Ok(metadata.len());
    }
    // Keep the most recent diagnostics from older, unbounded log versions.
    // Copy forward with a small buffer; history remains entirely in SQLite.
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;
    let start = metadata.len() - limit;
    let mut copied = 0;
    let mut buffer = [0_u8; 64 * 1024];
    while copied < limit {
        let length = (limit - copied).min(buffer.len() as u64) as usize;
        file.seek(SeekFrom::Start(start + copied))?;
        file.read_exact(&mut buffer[..length])?;
        file.seek(SeekFrom::Start(copied))?;
        file.write_all(&buffer[..length])?;
        copied += length as u64;
    }
    file.set_len(limit)?;
    Ok(limit)
}

fn append_status_log(path: &Path, line: &str, limit: u64) -> Result<()> {
    use std::io::Write;
    anyhow::ensure!(line.len() as u64 <= limit, "单条日志超过大小限制");
    std::fs::create_dir_all(path.parent().context("日志路径无效")?)?;
    let current_size = bound_status_log(path, limit)?;
    for index in 1..=STATUS_LOG_BACKUPS {
        bound_status_log(&status_log_archive(path, index), limit)?;
    }
    if current_size + line.len() as u64 > limit {
        let oldest = status_log_archive(path, STATUS_LOG_BACKUPS);
        match std::fs::remove_file(oldest) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        for index in (1..STATUS_LOG_BACKUPS).rev() {
            match std::fs::rename(
                status_log_archive(path, index),
                status_log_archive(path, index + 1),
            ) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        std::fs::rename(path, status_log_archive(path, 1))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

#[derive(Default)]
struct StatusLog {
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
            let path = self.path.as_ref().context("无法确定日志目录")?;
            append_status_log(path, line, limit)
        })();
        if let Err(error) = result {
            if !self.failure_reported {
                eprintln!("clicklog 输出失败，数据库记录继续；将自动重试：{error:#}");
            }
            self.failure_reported = true;
        } else {
            if self.failure_reported {
                eprintln!("clicklog 输出已恢复");
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

/// Data operations own the connection on a helper thread. The worker keeps
/// draining capture into memory, so a slow copy cannot fill the bounded input
/// queue and stop recording. Other requests wait until the connection returns.
struct StorageWorker {
    db: Option<Storage>,
    job: Option<Receiver<FinishedJob>>,
    deferred: std::collections::VecDeque<Command>,
    batch: Vec<InputEvent>,
}

impl StorageWorker {
    fn flush(&mut self) -> Result<()> {
        if let Some(db) = &mut self.db {
            db.insert(&self.batch)?;
            self.batch.clear();
        }
        Ok(())
    }

    fn spawn_job(
        &mut self,
        run: impl FnOnce(&mut Storage) -> Box<dyn FnOnce() + Send> + Send + 'static,
    ) -> Result<()> {
        let mut db = self.db.take().context("Storage is unavailable")?;
        let (tx, rx) = bounded(1);
        thread::Builder::new()
            .name("clickpulse-data-job".into())
            .spawn(move || {
                let finish = run(&mut db);
                let _ = tx.send((db, finish));
            })?;
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
                let result = db.snapshot_in_timezone(Local::now(), delivery.timezone);
                Box::new(move || delivery.complete(result))
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
            Command::Stop => {}
        }
        Ok(())
    }

    /// Input captured during the job lands after it, then queued requests resume.
    fn finish(&mut self, finished: FinishedJob) -> Result<()> {
        let (db, reply) = finished;
        self.db = Some(db);
        self.job = None;
        let input_result = self.flush();
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

fn storage_loop(db: Storage, rx: Receiver<Command>) -> Result<()> {
    let mut worker = StorageWorker {
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
    loop {
        let job = worker.job.clone().unwrap_or_else(|| idle.clone());
        let mut select = crossbeam_channel::Select::new();
        let command_index = select.recv(&rx);
        select.recv(&job);
        match select.select_timeout(Duration::from_millis(100)) {
            Ok(operation) if operation.index() == command_index => match operation.recv(&rx) {
                Ok(Command::Input(event)) => worker.batch.push(event),
                Ok(Command::Stop) | Err(_) => {
                    // Requests queued before Stop still complete, as before.
                    worker.wait_for_job()?;
                    worker.flush()?;
                    break;
                }
                Ok(command) => worker.handle(command)?,
            },
            Ok(operation) => {
                let finished = operation
                    .recv(&job)
                    .map_err(|_| anyhow!("Data operation worker stopped"))?;
                worker.finish(finished)?;
            }
            Err(_) => {}
        }
        // Periodic database work waits while a job owns the connection.
        let Some(db) = &mut worker.db else {
            continue;
        };
        // A deadline flushes even during continuous input: no dependence on idle gaps.
        if worker.batch.len() >= 128 || flush.elapsed() >= Duration::from_millis(200) {
            db.insert(&worker.batch)?;
            worker.batch.clear();
            flush = Instant::now();
        }
        if cache_cleanup.elapsed() >= Duration::from_secs(3600) {
            db.prune_rolling_cache(Local::now().timestamp_millis())?;
            cache_cleanup = Instant::now();
        }
        if log.elapsed() >= Duration::from_secs(2) {
            db.log_status(&mut status_log)?;
            log = Instant::now();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            db: None,
            job: None,
            deferred: Default::default(),
            batch: vec![InputEvent {
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
                ts_ms: at,
                device: crate::input::Device::Keyboard,
                code: 0,
            }])
            .unwrap();
        let mut runtime = Runtime::start_owned(&current, false).unwrap();
        runtime
            .tx
            .send(Command::Input(InputEvent {
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
            ts_ms: 0,
            device: Device::Mouse,
            code: 1,
        }))
        .unwrap();
        let runtime = Runtime {
            _instance_lock: tempfile::tempfile().unwrap(),
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
            ts_ms: Local::now().timestamp_millis(),
            device: Device::Mouse,
            code: 1,
        }])
        .unwrap();
        let before_status = Local::now().timestamp_millis();
        db.log_status(&mut log).unwrap();
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
        db.log_status(&mut log).unwrap();
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
        db.log_status(&mut log).unwrap();
        assert!(log.failure_reported);
        let event = InputEvent {
            ts_ms: Local::now().timestamp_millis(),
            device: Device::Mouse,
            code: 1,
        };
        db.insert(&[event]).unwrap();
        assert_eq!(db.snapshot(Local::now()).unwrap().lifetime.mouse, 1);
        db.log_status(&mut log).unwrap();
        assert!(log.failure_reported);
        std::fs::remove_dir(&log_path).unwrap();
        db.log_status(&mut log).unwrap();
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
    fn storage_failure_propagates_and_rolls_back_partial_transaction() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("fail.sqlite3");
        let db = EventDb::open(&path).unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TRIGGER fail BEFORE INSERT ON recent BEGIN SELECT RAISE(FAIL,'simulated write failure'); END;").unwrap();
        let (tx, rx) = bounded(2);
        let writer = thread::spawn(move || writer_loop(db, rx));
        tx.send(Command::Input(InputEvent {
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
    fn stop_drains_events_and_flushes_partial_batch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("db.sqlite3");
        let db = EventDb::open(&path).unwrap();
        let (tx, rx) = bounded(512);
        let writer = thread::spawn(move || writer_loop(db, rx));
        for _ in 0..257 {
            tx.send(Command::Input(InputEvent {
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
