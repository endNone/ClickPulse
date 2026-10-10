//! App-owned storage retains the original counts and encodings during handoff.
use crate::{
    db::Snapshot,
    input::{Device, InputEvent},
    source::LiveSource,
};
use anyhow::{Context, Result, bail, ensure};
use chrono::{DateTime, Local, TimeZone};
use rusqlite::{Connection, OpenFlags, params};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

// Above all Unicode, old special namespaces, and the old 32-bit hash fallback.
pub const PHYSICAL_CODE_BASE: i64 = 1 << 40;

pub fn data_path() -> Result<PathBuf> {
    Ok(crate::platform::data_dir()?.join("events.sqlite3"))
}

pub struct CollectorDb {
    conn: Connection,
    reader: LiveSource,
}

impl CollectorDb {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if path.exists() && path.metadata()?.len() > 0 {
            let check = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            crate::storage_schema::require_supported(&check)?;
            crate::storage_schema::require_key_encoding(&check)?;
            let owned = check
                .query_row(
                    "SELECT value='1' FROM clickpulse_metadata WHERE key='app_owned'",
                    [],
                    |r| r.get::<_, bool>(0),
                )
                .unwrap_or(false);
            ensure!(owned, "此文件尚未交给 App 管理，请先迁移；原文件未修改");
        }
        let mut conn = Connection::open(path)?;
        initialize(&mut conn)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")?;
        let reader = LiveSource::open(path)?;
        Ok(Self { conn, reader })
    }

    /// SQLite backup includes WAL commits. Publish only a complete validated copy,
    /// and never overwrite an existing destination or touch the original storage.
    pub fn migrate(source: &Path, output: &Path) -> Result<Snapshot> {
        Self::migrate_with_key_cutoff(source, output, None)
    }

    pub fn migrate_with_key_cutoff(
        source: &Path,
        output: &Path,
        cutoff_bucket: Option<i64>,
    ) -> Result<Snapshot> {
        ensure!(!output.exists(), "目标数据库已存在，不能重复迁移或覆盖历史");
        let parent = output.parent().context("目标目录无效")?;
        std::fs::create_dir_all(parent)?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let staged = parent.join(format!(".migration-{}-{nonce}.sqlite3", std::process::id()));
        std::fs::File::create_new(&staged)?;
        let result = (|| {
            let source_conn =
                Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            source_conn.busy_timeout(Duration::from_secs(5))?;
            source_conn.prepare("SELECT bucket,dev,code,cnt FROM events LIMIT 0")?;
            let mut dest = Connection::open(&staged)?;
            {
                let backup = rusqlite::backup::Backup::new(&source_conn, &mut dest)?;
                backup.run_to_completion(1024, Duration::from_millis(5), None)?;
            }
            ensure!(
                dest.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))? == "ok",
                "原数据完整性检查失败"
            );
            if let Some(cutoff) = cutoff_bucket {
                let cached: bool = dest.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='historical_key_totals')", [], |r| r.get(0))?;
                ensure!(
                    !cached,
                    "Historical key eras are already stored; an encoding boundary override is only for unversioned legacy sources"
                );
                dest.execute_batch("CREATE TABLE IF NOT EXISTS clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL)")?;
                dest.execute("INSERT OR REPLACE INTO clickpulse_metadata VALUES('legacy_key_cutoff_bucket',?1)", [cutoff.to_string()])?;
            }
            let before = LiveSource::open(&staged)?.snapshot(Local::now())?;
            initialize(&mut dest)?;
            dest.execute(
                "INSERT OR REPLACE INTO clickpulse_metadata VALUES('migrated_from',?1)",
                [source.to_string_lossy().as_ref()],
            )?;
            dest.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")?;
            drop(dest);
            let after = LiveSource::open(&staged)?.snapshot(Local::now())?;
            ensure!(
                before.lifetime == after.lifetime
                    && before
                        .history_days
                        .iter()
                        .map(|d| (&d.day, d.counts))
                        .collect::<Vec<_>>()
                        == after
                            .history_days
                            .iter()
                            .map(|d| (&d.day, d.counts))
                            .collect::<Vec<_>>(),
                "迁移前后历史统计不一致"
            );
            // Windows requires write access to flush a file. Flush the validated
            // staged copy before publishing, so a flush failure leaves no output.
            std::fs::OpenOptions::new()
                .write(true)
                .open(&staged)?
                .sync_all()?;
            // Same-directory hard link is atomic and fails if another destination exists.
            std::fs::hard_link(&staged, output)?;
            Ok(after)
        })();
        let _ = std::fs::remove_file(&staged);
        let _ = std::fs::remove_file(staged.with_extension("sqlite3-wal"));
        let _ = std::fs::remove_file(staged.with_extension("sqlite3-shm"));
        result
    }

    pub fn insert(&mut self, events: &[InputEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        for event in events {
            let time = Local
                .timestamp_millis_opt(event.ts_ms)
                .single()
                .context("事件时间无效")?;
            let dev = event.device as i64;
            let code = if event.device == Device::Keyboard {
                PHYSICAL_CODE_BASE + i64::from(event.code)
            } else {
                i64::from(event.code)
            };
            tx.execute("INSERT INTO events VALUES(?1,?2,?3,1) ON CONFLICT(bucket,dev,code) DO UPDATE SET cnt=cnt+1",params![event.ts_ms.div_euclid(60_000),dev,code])?;
            tx.execute("UPDATE lifetime_totals SET cnt=cnt+1 WHERE dev=?1", [dev])?;
            tx.execute("INSERT INTO daily_totals VALUES(?1,?2,1) ON CONFLICT(day,dev) DO UPDATE SET cnt=cnt+1",params![time.date_naive().to_string(),dev])?;
            tx.execute("INSERT INTO recent_counts VALUES(?1,?2,1) ON CONFLICT(ts_ms,dev) DO UPDATE SET cnt=cnt+1",params![event.ts_ms,dev])?;
            if event.device == Device::Keyboard {
                tx.execute("INSERT INTO native_key_totals VALUES(?1,1) ON CONFLICT(code) DO UPDATE SET cnt=cnt+1",[event.code])?;
                tx.execute("INSERT INTO historical_key_totals VALUES(0,?1,1) ON CONFLICT(era,code) DO UPDATE SET cnt=cnt+1",[code])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
    pub fn snapshot(&self, now: DateTime<Local>) -> Result<Snapshot> {
        self.reader.snapshot(now)
    }
    pub fn activity_counts(&self, now: DateTime<Local>) -> Result<crate::source::ActivityCounts> {
        self.reader.activity_counts(now)
    }
    pub fn export_daily(&self, path: &Path) -> Result<()> {
        self.reader.export_daily(path)
    }
    pub fn heartbeat(&self, now: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO clickpulse_metadata VALUES('heartbeat_ms',?1)",
            [now.to_string()],
        )?;
        Ok(())
    }
    /// Only the duplicate trailing-window cache expires; history is permanent.
    pub fn prune_rolling_cache(&mut self, now: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM recent_counts WHERE ts_ms<?1",
            [now.saturating_sub(86_400_000)],
        )?;
        Ok(())
    }
}

fn initialize(conn: &mut Connection) -> Result<()> {
    conn.busy_timeout(Duration::from_secs(5))?;
    crate::storage_schema::require_supported(conn)?;
    crate::storage_schema::require_key_encoding(conn)?;
    let tx = conn.transaction()?;
    tx.execute_batch("
        CREATE TABLE IF NOT EXISTS events(bucket INTEGER NOT NULL,dev INTEGER NOT NULL,code INTEGER NOT NULL,cnt INTEGER NOT NULL,PRIMARY KEY(bucket,dev,code));
        CREATE TABLE IF NOT EXISTS clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS daily_totals(day TEXT NOT NULL,dev INTEGER NOT NULL,cnt INTEGER NOT NULL,PRIMARY KEY(day,dev));
        CREATE TABLE IF NOT EXISTS lifetime_totals(dev INTEGER PRIMARY KEY,cnt INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS recent_counts(ts_ms INTEGER NOT NULL,dev INTEGER NOT NULL,cnt INTEGER NOT NULL,PRIMARY KEY(ts_ms,dev));
        CREATE TABLE IF NOT EXISTS native_key_totals(code INTEGER PRIMARY KEY,cnt INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS historical_key_totals(era INTEGER NOT NULL,code INTEGER NOT NULL,cnt INTEGER NOT NULL,PRIMARY KEY(era,code));")?;
    let bad: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM events WHERE dev NOT IN(0,1) OR cnt<0)",
        [],
        |r| r.get(0),
    )?;
    if bad {
        bail!("历史中存在无效计数，迁移已取消");
    }
    let aggregates: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM clickpulse_metadata WHERE key='aggregates_ready')",
        [],
        |r| r.get(0),
    )?;
    if !aggregates {
        tx.execute("INSERT INTO daily_totals SELECT date(bucket*60,'unixepoch','localtime'),dev,SUM(cnt) FROM events GROUP BY 1,2",[])?;
        tx.execute(
            "INSERT INTO lifetime_totals SELECT dev,SUM(cnt) FROM events GROUP BY dev",
            [],
        )?;
        tx.execute(
            "INSERT INTO clickpulse_metadata VALUES('aggregates_ready','1')",
            [],
        )?;
    }
    tx.execute(
        "INSERT OR IGNORE INTO lifetime_totals VALUES(0,0),(1,0)",
        [],
    )?;
    let owned: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM clickpulse_metadata WHERE key='app_owned')",
        [],
        |r| r.get(0),
    )?;
    let cutoff = if crate::storage_schema::metadata(&tx, "legacy_key_cutoff_bucket")?.is_some() {
        crate::storage_schema::legacy_cutoff(&tx)?
    } else {
        let legacy_keys: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM events WHERE dev=1 AND code<?1)",
            [PHYSICAL_CODE_BASE],
            |r| r.get(0),
        )?;
        if legacy_keys {
            crate::storage_schema::legacy_cutoff(&tx)?
        } else {
            0
        }
    };
    tx.execute(
        "INSERT OR IGNORE INTO clickpulse_metadata VALUES('legacy_key_cutoff_bucket',?1)",
        [cutoff.to_string()],
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO clickpulse_metadata VALUES('key_encoding_version',?1)",
        [crate::storage_schema::KEY_ENCODING_VERSION.to_string()],
    )?;
    if !owned {
        tx.execute("INSERT INTO historical_key_totals SELECT CASE WHEN code>=?2 THEN 0 ELSE bucket<?1 END,code,SUM(cnt) FROM events WHERE dev=1 GROUP BY 1,2",params![cutoff,PHYSICAL_CODE_BASE])?;
        tx.execute(
            "INSERT INTO clickpulse_metadata VALUES('app_owned','1')",
            [],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO clickpulse_metadata VALUES('precise_since_ms',?1)",
            [Local::now().timestamp_millis().to_string()],
        )?;
    }
    tx.pragma_update(None, "user_version", crate::storage_schema::VERSION)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_schema_upgrade_rolls_back_new_tables_and_version() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("invalid-owned.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,-1); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO clickpulse_metadata VALUES('app_owned','1');").unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        assert!(CollectorDb::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
    }

    #[test]
    fn existing_owned_schema_upgrades_without_reclassifying_cached_history() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("owned.sqlite3");
        {
            let mut db = CollectorDb::open(&path).unwrap();
            db.insert(&[InputEvent {
                ts_ms: Local::now().timestamp_millis(),
                device: Device::Keyboard,
                code: 0,
            }])
            .unwrap();
            db.conn.execute_batch("PRAGMA user_version=0; DELETE FROM clickpulse_metadata WHERE key IN('key_encoding_version','legacy_key_cutoff_bucket');").unwrap();
        }
        let before = LiveSource::open(&path)
            .unwrap()
            .snapshot(Local::now())
            .unwrap();
        let db = CollectorDb::open(&path).unwrap();
        let after = db.snapshot(Local::now()).unwrap();
        assert_eq!(before.lifetime, after.lifetime);
        assert_eq!(
            serde_json::to_value(before.keyboard_stats).unwrap(),
            serde_json::to_value(after.keyboard_stats).unwrap()
        );
        assert_eq!(
            db.conn
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            crate::storage_schema::VERSION
        );
        assert_eq!(
            crate::storage_schema::metadata(&db.conn, "key_encoding_version")
                .unwrap()
                .as_deref(),
            Some("1")
        );
        drop(db);
        assert_eq!(
            CollectorDb::open(&path)
                .unwrap()
                .snapshot(Local::now())
                .unwrap()
                .lifetime
                .keyboard,
            1
        );
    }

    #[test]
    fn future_versions_are_rejected_before_any_database_write() {
        for schema in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("future.sqlite3");
            {
                let db = CollectorDb::open(&path).unwrap();
                if schema {
                    db.conn.pragma_update(None, "user_version", 999).unwrap();
                } else {
                    db.conn.execute("UPDATE clickpulse_metadata SET value='999' WHERE key='key_encoding_version'", []).unwrap();
                }
            }
            let before = std::fs::read(&path).unwrap();
            assert!(CollectorDb::open(&path).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), before);
        }
    }

    #[test]
    fn migration_uses_explicit_absolute_encoding_boundary_and_preserves_it() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("legacy.sqlite3");
        let output = temp.path().join("owned.sqlite3");
        let conn = Connection::open(&source).unwrap();
        conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,7),(3,1,97,9); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO clickpulse_metadata VALUES('legacy_key_cutoff_bucket','3');").unwrap();
        drop(conn);
        let snapshot = CollectorDb::migrate(&source, &output).unwrap();
        assert_eq!(snapshot.keyboard_stats.history_before_fix[0].count, 7);
        assert_eq!(snapshot.keyboard_stats.history_since_fix[0].count, 9);
        let db = CollectorDb::open(&output).unwrap();
        assert_eq!(
            crate::storage_schema::metadata(&db.conn, "legacy_key_cutoff_bucket")
                .unwrap()
                .as_deref(),
            Some("3")
        );
        // Cached eras remain authoritative even if later metadata changes.
        db.conn
            .execute(
                "UPDATE clickpulse_metadata SET value='0' WHERE key='legacy_key_cutoff_bucket'",
                [],
            )
            .unwrap();
        assert_eq!(
            db.snapshot(Local::now())
                .unwrap()
                .keyboard_stats
                .history_before_fix[0]
                .count,
            7
        );
    }

    #[test]
    fn wal_migration_preserves_permanent_totals_and_full_key_history_without_overlap() {
        let t = tempfile::tempdir().unwrap();
        let old = t.path().join("old.sqlite3");
        let new = t.path().join("new.sqlite3");
        let source = Connection::open(&old).unwrap();
        source.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER,PRIMARY KEY(bucket,dev,code)); INSERT INTO events VALUES(2,1,97,3),(2,0,1,2);
        CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO clickpulse_metadata VALUES('aggregates_ready','1');
        CREATE TABLE daily_totals(day TEXT,dev INTEGER,cnt INTEGER,PRIMARY KEY(day,dev)); INSERT INTO daily_totals VALUES('1970-01-01',1,9),('1970-01-01',0,2);
        CREATE TABLE lifetime_totals(dev INTEGER PRIMARY KEY,cnt INTEGER); INSERT INTO lifetime_totals VALUES(1,9),(0,2);").unwrap();
        let before = std::fs::read(&old).unwrap();
        let baseline = CollectorDb::migrate(&old, &new).unwrap();
        assert_eq!(baseline.lifetime.total(), 11);
        assert!(CollectorDb::migrate(&old, &new).is_err());
        assert_eq!(std::fs::read(&old).unwrap(), before);
        let mut db = CollectorDb::open(&new).unwrap();
        let now = Local::now();
        db.insert(&[InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Keyboard,
            code: 0,
        }])
        .unwrap();
        let s = db.snapshot(now).unwrap();
        assert_eq!(s.lifetime.total(), 12);
        assert_eq!(s.keyboard_stats.physical[0].count, 1);
        assert_eq!(
            s.keyboard_stats
                .history_before_fix
                .iter()
                .map(|k| k.count)
                .sum::<i64>()
                + s.keyboard_stats
                    .history_since_fix
                    .iter()
                    .map(|k| k.count)
                    .sum::<i64>(),
            4
        );
        assert_eq!(
            s.keyboard_stats.history_since_fix[0].code,
            PHYSICAL_CODE_BASE
        );
        db.prune_rolling_cache(now.timestamp_millis() + 400 * 86_400_000)
            .unwrap();
        let s = db.snapshot(now).unwrap();
        assert_eq!(s.lifetime.total(), 12);
        assert_eq!(s.keyboard_stats.history_before_fix[0].count, 3);
        assert_eq!(s.keyboard_stats.history_since_fix[0].count, 1);
        assert_eq!(
            db.conn
                .query_row("SELECT SUM(cnt) FROM events", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            6
        );
    }
    #[test]
    fn cache_cleanup_keeps_all_minute_history_and_recent_window_boundaries() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("permanent.sqlite3");
        let mut db = CollectorDb::open(&path).unwrap();
        let now = 4_000 * 86_400_000;
        let cutoff = now - 86_400_000;
        // This fixture covers a complete precise window, independent of the
        // real creation time used by the collector's coverage metadata.
        db.conn
            .execute(
                "UPDATE clickpulse_metadata SET value=?1 WHERE key='precise_since_ms'",
                [cutoff.to_string()],
            )
            .unwrap();
        db.insert(&[
            InputEvent {
                ts_ms: 1_000,
                device: Device::Keyboard,
                code: 0,
            },
            InputEvent {
                ts_ms: cutoff - 1,
                device: Device::Mouse,
                code: 1,
            },
            InputEvent {
                ts_ms: cutoff,
                device: Device::Keyboard,
                code: 1,
            },
            InputEvent {
                ts_ms: now - 1_000,
                device: Device::Mouse,
                code: 1,
            },
            InputEvent {
                ts_ms: now + 1_000,
                device: Device::Keyboard,
                code: 2,
            },
        ])
        .unwrap();
        let history = || {
            db.conn
                .prepare("SELECT bucket,dev,code,cnt FROM events ORDER BY bucket,dev,code")
                .unwrap()
                .query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
        };
        let before = history();
        db.prune_rolling_cache(now).unwrap();
        assert_eq!(
            db.conn
                .query_row("SELECT COUNT(*) FROM recent_counts", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
        let snapshot = db
            .snapshot(Local.timestamp_millis_opt(now).single().unwrap())
            .unwrap();
        let window = snapshot
            .windows
            .iter()
            .find(|window| window.seconds == 86_400)
            .unwrap();
        assert!(!window.approximate);
        assert_eq!(window.counts.keyboard, 0);
        assert_eq!(window.counts.mouse, 1);
        db.prune_rolling_cache(now + 4_000 * 86_400_000).unwrap();
        drop(db);
        let db = CollectorDb::open(&path).unwrap();
        let after = db
            .conn
            .prepare("SELECT bucket,dev,code,cnt FROM events ORDER BY bucket,dev,code")
            .unwrap()
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            db.snapshot(Local.timestamp_millis_opt(now).single().unwrap())
                .unwrap()
                .lifetime
                .total(),
            5
        );
        assert_eq!(
            db.conn
                .query_row("SELECT SUM(cnt) FROM native_key_totals", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            3
        );
    }
    #[test]
    fn invalid_source_never_publishes_or_modifies_original() {
        let t = tempfile::tempdir().unwrap();
        let old = t.path().join("bad.sqlite3");
        let new = t.path().join("new.sqlite3");
        let c = Connection::open(&old).unwrap();
        c.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER);INSERT INTO events VALUES(1,1,97,-4)").unwrap();
        drop(c);
        let bytes = std::fs::read(&old).unwrap();
        assert!(CollectorDb::migrate(&old, &new).is_err());
        assert!(!new.exists());
        assert_eq!(std::fs::read(&old).unwrap(), bytes);
        assert!(CollectorDb::open(&old).is_err());
    }
}
