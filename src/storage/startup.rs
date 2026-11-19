//! Startup recovery primitives. The desktop caller holds profile.lock throughout.
use super::*;

#[derive(Debug)]
pub struct Failure {
    pub path: Option<PathBuf>,
    pub reason: &'static str,
    pub error: anyhow::Error,
}
impl Failure {
    pub fn database(path: PathBuf, error: anyhow::Error) -> Self {
        Self::new(Some(path), database_reason(&error), error)
    }
    fn new(path: Option<PathBuf>, reason: &'static str, error: impl Into<anyhow::Error>) -> Self {
        Self {
            path,
            reason,
            error: error.into(),
        }
    }
}

#[derive(Debug)]
enum DatabaseIssue {
    Busy,
    Integrity,
}
impl std::fmt::Display for DatabaseIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Busy => "This database is in use; close its recorder before recovery",
            Self::Integrity => "Database integrity check failed",
        })
    }
}
impl std::error::Error for DatabaseIssue {}

fn database_reason(error: &anyhow::Error) -> &'static str {
    for cause in error.chain() {
        if let Some(issue) = cause.downcast_ref::<DatabaseIssue>() {
            return match issue {
                DatabaseIssue::Busy => "storage.recovery_busy",
                DatabaseIssue::Integrity => "storage.recovery_database",
            };
        }
        if let Some(version) = cause.downcast_ref::<crate::storage_schema::UnsupportedVersion>()
            && version.version > version.supported
        {
            return "storage.recovery_newer";
        }
        if let Some(rusqlite::Error::SqliteFailure(code, _)) =
            cause.downcast_ref::<rusqlite::Error>()
        {
            use rusqlite::ErrorCode::*;
            return match code.code {
                DatabaseBusy | DatabaseLocked => "storage.recovery_busy",
                DatabaseCorrupt | NotADatabase => "storage.recovery_database",
                _ => "storage.recovery_access",
            };
        }
    }
    // Unknown errors are not evidence of corruption. Never offer to reset them.
    "storage.recovery_access"
}

fn check_integrity(conn: &Connection) -> Result<()> {
    if conn.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))? != "ok" {
        anyhow::bail!(DatabaseIssue::Integrity);
    }
    Ok(())
}

pub fn inspect(profile: &Path) -> std::result::Result<PathBuf, Failure> {
    let config = profile.join("storage.json");
    let path = match std::fs::read(&config) {
        Ok(bytes) => {
            let location: Location = serde_json::from_slice(&bytes)
                .map_err(|e| Failure::new(None, "storage.recovery_config", e))?;
            if location.version != 1 || !location.database.is_absolute() {
                return Err(Failure::new(
                    None,
                    "storage.recovery_config",
                    anyhow::anyhow!("Unsupported storage location version or path"),
                ));
            }
            match std::fs::metadata(&location.database) {
                Ok(meta) if meta.is_file() => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Failure::new(
                        Some(location.database),
                        "storage.recovery_missing",
                        error,
                    ));
                }
                Err(error) => return Err(Failure::database(location.database, error.into())),
                Ok(_) => {
                    return Err(Failure::database(
                        location.database,
                        anyhow::anyhow!("Saved database is not a file"),
                    ));
                }
            }
            location.database
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => profile.join("events.sqlite3"),
        Err(e) => return Err(Failure::new(None, "storage.recovery_access", e)),
    };
    inspect_database(&path)?;
    Ok(path)
}

pub fn inspect_database(path: &Path) -> std::result::Result<(), Failure> {
    inspect_database_with(path, false)
}

fn inspect_database_with(path: &Path, full: bool) -> std::result::Result<(), Failure> {
    let result = (|| -> Result<()> {
        if path.try_exists()? {
            let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            conn.busy_timeout(Duration::from_secs(1))?;
            crate::storage_schema::require_supported(&conn)?;
            crate::storage_schema::require_key_encoding(&conn)?;
            check_integrity(&conn)?;
            if full
                && conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? != "ok"
            {
                anyhow::bail!(DatabaseIssue::Integrity);
            }
        }
        Ok(())
    })();
    result.map_err(|error| Failure::database(path.to_owned(), error))
}

fn suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
fn backup_path(path: &Path, reason: &str) -> Result<PathBuf> {
    Ok(suffix(
        path,
        &format!(
            ".{reason}-{}-{}",
            chrono::Utc::now().format("%Y%m%dT%H%M%S%.9fZ"),
            std::process::id()
        ),
    ))
}

/// Preserve the previous locator before atomically publishing a new choice.
/// Validation and creation happen before switching, never as an implicit fallback.
pub fn select(profile: &Path, database: &Path) -> Result<()> {
    ensure!(
        database.is_absolute(),
        "Selected database path must be absolute"
    );
    inspect_database(database).map_err(|failure| failure.error)?;
    let _lock = database_lock(database)?;
    let db = crate::collector::CollectorDb::open(database)?;
    drop(db);
    let config = profile.join("storage.json");
    if config.try_exists()? {
        let bytes = std::fs::read(&config)?;
        let reason = match serde_json::from_slice::<Location>(&bytes) {
            Err(_) => "corrupt",
            Ok(location) if location.version != 1 => "unsupported",
            Ok(location) if !location.database.is_file() => "missing",
            Ok(_) => "previous",
        };
        let backup = backup_path(&config, reason)?;
        publish_new(&config, &backup)?;
    }
    save_location(&config, database)
}

pub fn select_existing(profile: &Path, database: &Path) -> Result<()> {
    ensure!(
        database.is_file(),
        "Choose a folder containing events.sqlite3"
    );
    select(profile, database)
}

pub fn prepare_database(path: &Path) -> Result<()> {
    let _lock = database_lock(path)?;
    drop(crate::collector::CollectorDb::open(path)?);
    Ok(())
}

/// Use the same lock name as Runtime, including canonicalization of symlinks.
fn database_lock(path: &Path) -> Result<super::DatabaseLock> {
    super::lock_database(path).map_err(|error| {
        if error
            .downcast_ref::<std::fs::TryLockError>()
            .is_some_and(|error| matches!(error, std::fs::TryLockError::WouldBlock))
        {
            anyhow::Error::new(DatabaseIssue::Busy)
        } else {
            error
        }
    })
}

/// Preserve the database AND SQLite sidecars before creating anything new.
/// If any rename fails, roll back all completed renames; never delete user data.
pub fn preserve_and_restart(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_file() && !path.is_symlink(),
        "Choose a regular database file for recovery"
    );
    let _lock = database_lock(path)?;
    // The file may have changed while the user was deciding. Only verified
    // corruption can be archived; a healthy, busy or newer database stays put.
    match inspect_database_with(path, true) {
        Err(failure) if failure.reason == "storage.recovery_database" => {}
        Err(failure) => return Err(failure.error),
        Ok(()) => anyhow::bail!("Database is healthy; restart recovery is not needed"),
    }
    let backup = backup_path(path, "damaged")?;
    // Inspect everything before moving anything. A failed sidecar inspection
    // must not leave only the main file archived; a rollback journal may hold
    // the information needed to recover a pre-WAL/legacy database.
    let mut planned = Vec::new();
    for ending in ["", "-wal", "-shm", "-journal"] {
        let original = suffix(path, ending);
        let metadata = match std::fs::symlink_metadata(&original) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        ensure!(
            metadata.file_type().is_file(),
            "Recovery requires a regular file: {}",
            original.display()
        );
        let archived = suffix(&backup, ending);
        match std::fs::symlink_metadata(&archived) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
            Ok(_) => anyhow::bail!("Recovery backup already exists"),
        }
        planned.push((original, archived));
    }
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (original, archived) in planned {
        if let Err(error) = std::fs::rename(&original, &archived) {
            for (from, to) in moved.iter().rev() {
                std::fs::rename(to, from).with_context(|| {
                    format!(
                        "Recovery rollback failed; preserved data remains at {}",
                        to.display()
                    )
                })?;
            }
            return Err(error.into());
        }
        moved.push((original, archived));
    }
    // Creating the new DB is an explicit user action; the backup remains even
    // if initialization fails (for example, if the volume becomes read-only).
    drop(crate::collector::CollectorDb::open(path)?);
    Ok(backup)
}

pub fn inside_program_directory(directory: &Path, executable: &Path) -> Result<bool> {
    let executable = executable.canonicalize()?;
    let parent = executable
        .parent()
        .context("Executable directory is unavailable")?;
    // macOS stores its binary inside Contents/MacOS; warn for the entire bundle.
    let root = executable
        .ancestors()
        .find(|p| p.extension().is_some_and(|e| e == "app"))
        .unwrap_or(parent);
    Ok(directory.canonicalize()?.starts_with(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broken_locator_and_missing_database_do_not_create_empty_history() {
        let temp = tempfile::tempdir().unwrap();
        let profile = temp.path();
        let config = profile.join("storage.json");
        for value in [
            "broken",
            r#"{"version":99,"database":"/absent/events.sqlite3"}"#,
            r#"{"version":1,"database":"/absent/events.sqlite3"}"#,
        ] {
            std::fs::write(&config, value).unwrap();
            assert!(inspect(profile).is_err());
            assert!(!profile.join("events.sqlite3").exists());
            assert_eq!(std::fs::read_to_string(&config).unwrap(), value);
        }
    }
    #[test]
    fn selecting_default_keeps_existing_history_and_backs_up_bad_locator() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let mut db = crate::collector::CollectorDb::open(&path).unwrap();
        db.insert(&[crate::input::InputEvent {
            shortcut: None,
            ts_ms: chrono::Utc::now().timestamp_millis(),
            device: crate::input::Device::Keyboard,
            code: 12,
        }])
        .unwrap();
        drop(db);
        std::fs::write(temp.path().join("storage.json"), "broken").unwrap();
        select(temp.path(), &path).unwrap();
        assert_eq!(
            crate::collector::CollectorDb::open(&path)
                .unwrap()
                .lifetime()
                .unwrap()
                .keyboard,
            1
        );
        assert_eq!(inspect(temp.path()).unwrap(), path);
        assert!(std::fs::read_dir(temp.path()).unwrap().flatten().any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("storage.json.corrupt-")
        }));
    }
    #[test]
    fn choosing_an_existing_folder_never_creates_history_and_missing_locator_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("missing/events.sqlite3");
        let locator = serde_json::json!({"version":1,"database":missing}).to_string();
        std::fs::write(temp.path().join("storage.json"), &locator).unwrap();
        let selected = temp.path().join("another/events.sqlite3");
        assert!(select_existing(temp.path(), &selected).is_err());
        assert!(!selected.exists());
        assert_eq!(
            std::fs::read_to_string(temp.path().join("storage.json")).unwrap(),
            locator
        );
        select(temp.path(), &temp.path().join("events.sqlite3")).unwrap();
        let backup = std::fs::read_dir(temp.path())
            .unwrap()
            .flatten()
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("storage.json.missing-")
            })
            .unwrap()
            .path();
        assert_eq!(std::fs::read_to_string(backup).unwrap(), locator);
        assert!(!missing.exists());
    }
    #[test]
    fn corrupt_and_newer_databases_are_preserved_and_active_database_is_not_renamed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        std::fs::write(&path, "damaged database").unwrap();
        std::fs::write(suffix(&path, "-wal"), "preserved WAL").unwrap();
        std::fs::write(suffix(&path, "-shm"), "preserved SHM").unwrap();
        assert!(inspect_database(&path).is_err());
        // SQLite can refresh its shared-memory index during a read-only open.
        // Recovery must preserve the index present when the reset is chosen.
        let shared_memory = std::fs::read(suffix(&path, "-shm")).unwrap();
        let held = database_lock(&path).unwrap();
        assert!(preserve_and_restart(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "damaged database");
        drop(held);
        let backup = preserve_and_restart(&path).unwrap();
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "damaged database"
        );
        assert_eq!(
            std::fs::read_to_string(suffix(&backup, "-wal")).unwrap(),
            "preserved WAL"
        );
        assert_eq!(
            std::fs::read(suffix(&backup, "-shm")).unwrap(),
            shared_memory
        );
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA user_version=999;").unwrap();
        drop(conn);
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_newer"
        );
        assert!(preserve_and_restart(&path).is_err());
        assert_eq!(
            Connection::open(&path)
                .unwrap()
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            999
        );
    }

    #[test]
    fn recovery_preserves_rollback_journal_and_preflights_all_sidecars() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        std::fs::write(&path, "damaged database").unwrap();
        std::fs::write(suffix(&path, "-wal"), "WAL bytes").unwrap();
        let shm = suffix(&path, "-shm");
        std::fs::create_dir(&shm).unwrap();
        assert!(preserve_and_restart(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "damaged database");
        assert_eq!(
            std::fs::read_to_string(suffix(&path, "-wal")).unwrap(),
            "WAL bytes"
        );
        assert!(
            !std::fs::read_dir(temp.path())
                .unwrap()
                .flatten()
                .any(|e| { e.file_name().to_string_lossy().contains(".damaged-") })
        );
        std::fs::remove_dir(&shm).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(temp.path().join("missing-sidecar"), &shm).unwrap();
            assert!(preserve_and_restart(&path).is_err());
            assert!(path.is_file());
            std::fs::remove_file(&shm).unwrap();
        }
        // An unfinished hot journal requires write access, not a destructive reset.
        std::fs::write(suffix(&path, "-journal"), "rollback bytes").unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_access"
        );
        assert!(preserve_and_restart(&path).is_err());
        let journal = vec![0_u8; 512];
        std::fs::write(suffix(&path, "-journal"), &journal).unwrap();
        let backup = preserve_and_restart(&path).unwrap();
        assert_eq!(std::fs::read(suffix(&backup, "-journal")).unwrap(), journal);
        assert!(!suffix(&path, "-journal").exists());
        assert!(inspect_database(&path).is_ok());
    }

    #[test]
    fn recovery_classifies_busy_access_newer_and_verified_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE sample(value INTEGER CHECK(value>0)); BEGIN EXCLUSIVE")
            .unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_busy"
        );
        conn.execute_batch("ROLLBACK").unwrap();
        let held = database_lock(&path).unwrap();
        assert_eq!(
            Failure::database(path.clone(), prepare_database(&path).unwrap_err()).reason,
            "storage.recovery_busy"
        );
        drop(held);
        let readonly =
            Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let error = readonly
            .execute("INSERT INTO sample VALUES(1)", [])
            .unwrap_err();
        assert_eq!(database_reason(&error.into()), "storage.recovery_access");
        for raw in [
            rusqlite::ffi::SQLITE_BUSY,
            rusqlite::ffi::SQLITE_LOCKED,
            rusqlite::ffi::SQLITE_READONLY,
            rusqlite::ffi::SQLITE_PERM,
            rusqlite::ffi::SQLITE_CANTOPEN,
        ] {
            let error = rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(raw), None);
            let expected = if matches!(
                raw,
                rusqlite::ffi::SQLITE_BUSY | rusqlite::ffi::SQLITE_LOCKED
            ) {
                "storage.recovery_busy"
            } else {
                "storage.recovery_access"
            };
            assert_eq!(
                database_reason(&anyhow::Error::from(error).context("opening data")),
                expected
            );
        }
        assert_eq!(
            database_reason(&std::io::Error::from(std::io::ErrorKind::PermissionDenied).into()),
            "storage.recovery_access"
        );
        conn.execute_batch("PRAGMA user_version=999").unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_newer"
        );
        assert!(preserve_and_restart(&path).is_err());
        conn.execute_batch("PRAGMA user_version=1; CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY, value TEXT); INSERT INTO clickpulse_metadata VALUES('key_encoding_version', '999')").unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_newer"
        );
        assert!(preserve_and_restart(&path).is_err());
        conn.execute("UPDATE clickpulse_metadata SET value='1'", [])
            .unwrap();
        drop(readonly);
        drop(conn);
        // A broken freelist produces a non-ok quick_check on a read-only handle.
        // CHECK constraints are deliberately skipped by SQLite in read-only mode.
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[36..40].copy_from_slice(&1_u32.to_be_bytes());
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_database"
        );
    }
    #[test]
    fn recovery_checks_index_integrity_before_archiving() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE saved(value INTEGER, other INTEGER); INSERT INTO saved VALUES(42,43); CREATE INDEX saved_index ON saved(value); PRAGMA writable_schema=ON; UPDATE sqlite_schema SET sql='CREATE INDEX saved_index ON saved(other)' WHERE name='saved_index'; PRAGMA writable_schema=OFF;").unwrap();
        drop(conn);
        // Index contents disagree with their declared column, without corrupt table pages.
        assert!(inspect_database(&path).is_ok());
        assert!(inspect_database_with(&path, true).is_err());
        let backup = preserve_and_restart(&path).unwrap();
        assert!(backup.is_file());
        assert!(inspect_database(&path).is_ok());
    }

    #[test]
    fn healthy_or_replaced_database_is_not_archived_after_the_dialog() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        std::fs::write(&path, "damaged").unwrap();
        assert_eq!(
            inspect_database(&path).unwrap_err().reason,
            "storage.recovery_database"
        );
        std::fs::remove_file(&path).unwrap();
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE saved(value); INSERT INTO saved VALUES(42)")
            .unwrap();
        drop(conn);
        assert!(preserve_and_restart(&path).is_err());
        assert_eq!(
            Connection::open(&path)
                .unwrap()
                .query_row("SELECT value FROM saved", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            42
        );
        assert!(
            !std::fs::read_dir(temp.path())
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().contains(".damaged-"))
        );
    }

    #[test]
    fn program_directory_warning_includes_the_whole_bundle_and_resolved_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("ClickPulse.app");
        let directory = bundle.join("Contents/MacOS");
        std::fs::create_dir_all(&directory).unwrap();
        let binary = directory.join("clickpulse");
        std::fs::write(&binary, "test executable").unwrap();
        assert!(inside_program_directory(&bundle, &binary).unwrap());
        assert!(inside_program_directory(&directory, &binary).unwrap());
        assert!(!inside_program_directory(temp.path(), &binary).unwrap());
        #[cfg(unix)]
        {
            let alias = temp.path().join("shortcut");
            std::os::unix::fs::symlink(&bundle, &alias).unwrap();
            assert!(inside_program_directory(&alias, &binary).unwrap());
        }
    }
}
