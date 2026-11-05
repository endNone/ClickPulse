//! Statistics location is separate from window preferences and logs.
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Location {
    version: u32,
    database: PathBuf,
}

/// Pin a committed source snapshot so continuous WAL writes cannot restart the
/// copy forever. Bound lock/no-progress retries instead of retrying indefinitely.
pub(crate) fn copy_snapshot(source: &Connection, dest: &mut Connection) -> Result<()> {
    copy_snapshot_with_timeout(source, dest, Duration::from_secs(5))
}

fn copy_snapshot_with_timeout(
    source: &Connection,
    dest: &mut Connection,
    stall_timeout: Duration,
) -> Result<()> {
    let _snapshot = if source.is_autocommit() {
        Some(source.unchecked_transaction()?)
    } else {
        None
    };
    source.query_row("SELECT count(*) FROM main.sqlite_schema", [], |r| {
        r.get::<_, i64>(0)
    })?;
    let backup = rusqlite::backup::Backup::new(source, dest)?;
    let mut last_progress = Instant::now();
    let mut remaining = i32::MAX;
    loop {
        use rusqlite::backup::StepResult;
        match backup.step(1024)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {
                let current = backup.progress().remaining;
                if current < remaining {
                    remaining = current;
                    last_progress = Instant::now();
                }
            }
            StepResult::Busy | StepResult::Locked => std::thread::sleep(Duration::from_millis(10)),
            _ => {}
        }
        ensure!(
            last_progress.elapsed() < stall_timeout,
            "Database backup made no progress; close other database writers and try again"
        );
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(crate::platform::data_dir()?.join("storage.json"))
}

pub fn database_path() -> Result<PathBuf> {
    resolve(&crate::platform::data_dir()?)
}

fn resolve(profile: &Path) -> Result<PathBuf> {
    let path = profile.join("storage.json");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let location: Location = serde_json::from_slice(&bytes)
                .context("Invalid storage location; original data was not changed")?;
            ensure!(
                location.version == 1 && location.database.is_absolute(),
                "Unsupported storage location"
            );
            ensure!(
                location.database.is_file(),
                "Saved database is unavailable: {}. Reconnect its disk or restore the original location; no empty database was created.",
                location.database.display()
            );
            Ok(location.database)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(profile.join("events.sqlite3"))
        }
        Err(error) => Err(error).context("Unable to read storage location"),
    }
}

pub(crate) fn unique_path(parent: &Path, prefix: &str) -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(parent.join(format!("{prefix}-{}-{nonce}.sqlite3", std::process::id())))
}

/// A reserved temporary file is removed with its SQLite sidecars on every exit.
pub(crate) struct TemporaryDatabase(pub PathBuf);
impl TemporaryDatabase {
    pub fn new(parent: &Path) -> Result<Self> {
        let path = unique_path(parent, ".clickpulse-import")?;
        std::fs::File::create_new(&path)?;
        Ok(Self(path))
    }
}
impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        remove_database(&self.0);
    }
}

pub(crate) fn remove_database(path: &Path) {
    let _ = std::fs::remove_file(path);
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_os_string();
        sidecar.push(suffix);
        let _ = std::fs::remove_file(PathBuf::from(sidecar));
    }
}

/// Publish a synced staged file without replacing an existing destination. A
/// hard link is atomic; exFAT/FAT32 and some shares lack links, so fall back to
/// an exclusive create plus copy and remove a partial copy on failure.
pub(crate) fn publish_new(staged: &Path, output: &Path) -> Result<()> {
    let error = match std::fs::hard_link(staged, output) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(error)
                .context("Unable to publish database without overwriting existing data");
        }
        Err(error) => error,
    };
    let mut file = std::fs::File::create_new(output).with_context(|| {
        format!("Unable to publish database without overwriting existing data ({error})")
    })?;
    let copied = (|| {
        std::io::copy(&mut std::fs::File::open(staged)?, &mut file)?;
        file.sync_all()
    })();
    if let Err(copy_error) = copied {
        drop(file);
        let _ = std::fs::remove_file(output);
        return Err(copy_error).context("Unable to copy database to the selected folder");
    }
    Ok(())
}

/// Online backup observes one committed SQLite snapshot, including WAL data.
/// Publish without clobbering a destination, even if another process created it.
pub(crate) fn backup_connection(source: &Connection, output: &Path) -> Result<()> {
    ensure!(
        !output.exists(),
        "Destination already exists; select a folder without events.sqlite3"
    );
    let parent = output.parent().context("Invalid destination directory")?;
    std::fs::create_dir_all(parent)?;
    let staged = TemporaryDatabase::new(parent)?;
    let mut dest = Connection::open(&staged.0)?;
    copy_snapshot(source, &mut dest)?;
    ensure!(
        dest.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))? == "ok",
        "Database integrity check failed"
    );
    dest.execute_batch("PRAGMA journal_mode=DELETE;")?;
    drop(dest);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&staged.0)?
        .sync_all()?;
    publish_new(&staged.0, output)
}

pub(crate) fn backup_file(source: &Path, output: &Path) -> Result<()> {
    let source = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    source.busy_timeout(Duration::from_secs(5))?;
    crate::storage_schema::require_supported(&source)?;
    crate::storage_schema::require_key_encoding(&source)?;
    backup_connection(&source, output)
}

fn save_location(config: &Path, database: &Path) -> Result<()> {
    let parent = config.parent().context("Invalid storage settings path")?;
    std::fs::create_dir_all(parent)?;
    let staged = config.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = std::fs::File::create_new(&staged)?;
        file.write_all(&serde_json::to_vec_pretty(&Location {
            version: 1,
            database: database.to_owned(),
        })?)?;
        file.sync_all()?;
        std::fs::rename(&staged, config)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

/// Caller holds both database locks and has flushed accepted input. Keep the old
/// database as a backup; commit the locator only after the copy can be reopened.
pub(crate) fn relocate(
    source: &Path,
    output: &Path,
    config: &Path,
) -> Result<crate::collector::CollectorDb> {
    backup_file(source, output)?;
    let result = (|| {
        let db = crate::collector::CollectorDb::open(output)?;
        save_location(config, output)?;
        Ok(db)
    })();
    if result.is_err() {
        remove_database(output);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collector::CollectorDb,
        input::{Device, InputEvent},
    };

    #[test]
    fn a_locked_backup_returns_an_error_instead_of_retrying_forever() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let source = Connection::open_in_memory().unwrap();
            let mut output = Connection::open_in_memory().unwrap();
            source
                .execute_batch("CREATE TABLE data(n); BEGIN IMMEDIATE; INSERT INTO data VALUES(1);")
                .unwrap();
            tx.send(
                copy_snapshot_with_timeout(&source, &mut output, Duration::from_millis(50))
                    .is_err(),
            )
            .unwrap();
        });
        assert!(rx.recv_timeout(Duration::from_secs(2)).unwrap());
    }

    #[test]
    fn relocation_keeps_wal_history_and_refuses_overwrite_or_missing_selected_storage() {
        let temp = tempfile::tempdir().unwrap();
        let profile = temp.path().join("profile");
        let source = temp.path().join("original.sqlite3");
        let mut old = CollectorDb::open(&source).unwrap();
        old.insert(&[InputEvent {
            ts_ms: chrono::Local::now().timestamp_millis(),
            device: Device::Keyboard,
            code: 0,
        }])
        .unwrap();
        let output = temp.path().join("自定义目录/events.sqlite3");
        let new = relocate(&source, &output, &profile.join("storage.json")).unwrap();
        assert_eq!(new.lifetime().unwrap().keyboard, 1);
        assert_eq!(old.lifetime().unwrap().keyboard, 1);
        assert_eq!(resolve(&profile).unwrap(), output);
        assert!(relocate(&source, &output, &profile.join("storage.json")).is_err());
        drop(new);
        remove_database(&output);
        assert!(
            resolve(&profile).is_err(),
            "never silently start empty when a selected disk is missing"
        );
    }

    #[test]
    fn publishing_never_replaces_an_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let staged = temp.path().join("staged");
        let output = temp.path().join("events.sqlite3");
        std::fs::write(&staged, b"new").unwrap();
        std::fs::write(&output, b"keep").unwrap();
        assert!(publish_new(&staged, &output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"keep");
        std::fs::remove_file(&output).unwrap();
        publish_new(&staged, &output).unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), b"new");
    }

    #[test]
    fn failed_locator_write_removes_only_new_copy_and_retains_previous_selection() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.sqlite3");
        let db = CollectorDb::open(&source).unwrap();
        let output = temp.path().join("new/events.sqlite3");
        let invalid = temp.path().join("not-a-folder");
        std::fs::write(&invalid, b"keep").unwrap();
        assert!(relocate(&source, &output, &invalid.join("storage.json")).is_err());
        assert!(!output.exists());
        assert_eq!(std::fs::read(invalid).unwrap(), b"keep");
        assert_eq!(db.lifetime().unwrap().total(), 0);
        assert_eq!(
            resolve(temp.path()).unwrap(),
            temp.path().join("events.sqlite3")
        );
    }
}
