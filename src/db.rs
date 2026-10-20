use std::{collections::BTreeMap, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Days, Local, NaiveDate, TimeZone, Timelike};
use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;

use crate::input::{Device, InputEvent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Counts {
    pub keyboard: i64,
    pub mouse: i64,
}

impl Counts {
    pub fn total(self) -> i64 {
        self.keyboard + self.mouse
    }

    pub(crate) fn add(&mut self, dev: i64, count: i64) {
        match dev {
            0 => self.mouse += count,
            1 => self.keyboard += count,
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DayCount {
    pub day: String,
    pub counts: Counts,
}

#[derive(Clone, Debug, Serialize)]
pub struct WindowCount {
    pub seconds: i64,
    pub counts: Counts,
    /// True when legacy minute buckets contribute an approximation.
    pub approximate: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct KeyCount {
    pub code: u32,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct EncodedKeyCount {
    pub code: i64,
    pub count: i64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct KeyboardStats {
    pub physical: Vec<KeyCount>,
    pub history_before_fix: Vec<EncodedKeyCount>,
    pub history_since_fix: Vec<EncodedKeyCount>,
    pub history_available: bool,
    pub physical_since: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub now: String,
    pub today: Counts,
    pub week: Counts,
    pub lifetime: Counts,
    pub windows: Vec<WindowCount>,
    pub days: Vec<DayCount>,
    pub history_days: Vec<DayCount>,
    pub hours: Vec<Counts>,
    pub top_keys: Vec<KeyCount>,
    pub keyboard_stats: KeyboardStats,
    pub legacy_imported: bool,
}

pub struct EventDb {
    conn: Connection,
}

impl EventDb {
    pub fn open(path: &Path) -> Result<Self> {
        // Detect legacy storage before issuing any schema changes or write pragmas.
        if path.is_file() && path.metadata()?.len() > 0 {
            let existing = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            crate::storage_schema::require_supported(&existing)?;
            let legacy: bool = existing.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='events')",
                [],
                |r| r.get(0),
            )?;
            if legacy {
                bail!(
                    "这是旧版数据库，请选择另一份新版数据库路径，并使用 --legacy-db 导入。原文件未修改。"
                );
            }
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        Self::from_connection(Connection::open(path)?)
    }

    fn from_connection(mut conn: Connection) -> Result<Self> {
        conn.busy_timeout(Duration::from_secs(5))?;
        crate::storage_schema::require_supported(&conn)?;
        let tx = conn.transaction()?;
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS totals(dev INTEGER PRIMARY KEY CHECK(dev IN (0,1)), cnt INTEGER NOT NULL CHECK(cnt>=0));
             CREATE TABLE IF NOT EXISTS daily(day TEXT NOT NULL, dev INTEGER NOT NULL, cnt INTEGER NOT NULL, PRIMARY KEY(day,dev));
             CREATE TABLE IF NOT EXISTS minutes(bucket INTEGER NOT NULL, dev INTEGER NOT NULL, cnt INTEGER NOT NULL, PRIMARY KEY(bucket,dev));
             CREATE TABLE IF NOT EXISTS legacy_minutes(bucket INTEGER NOT NULL, dev INTEGER NOT NULL, cnt INTEGER NOT NULL, PRIMARY KEY(bucket,dev));
             CREATE TABLE IF NOT EXISTS recent(ts_ms INTEGER NOT NULL, dev INTEGER NOT NULL, cnt INTEGER NOT NULL, PRIMARY KEY(ts_ms,dev));
             CREATE TABLE IF NOT EXISTS keys(code INTEGER PRIMARY KEY, cnt INTEGER NOT NULL);
             INSERT OR IGNORE INTO totals VALUES(0,0),(1,0);",
        )?;
        tx.pragma_update(None, "user_version", crate::storage_schema::VERSION)?;
        tx.commit()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(Self { conn })
    }

    /// Import the original Python database once, in a single destination transaction.
    /// Original codes cannot be safely mapped to physical keys, so only count/time data is imported.
    pub fn import_legacy(&mut self, path: &Path) -> Result<i64> {
        if self.legacy_imported()? {
            return Ok(0);
        }
        let legacy = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("无法只读打开旧数据库 {}", path.display()))?;
        let invalid: bool = legacy.query_row(
            "SELECT EXISTS(SELECT 1 FROM events WHERE dev NOT IN (0,1) OR cnt<0)",
            [],
            |r| r.get(0),
        )?;
        if invalid {
            bail!("旧数据库包含无效的设备或计数");
        }
        let mut stmt = legacy.prepare(
            "SELECT bucket, dev, SUM(cnt) FROM events GROUP BY bucket,dev ORDER BY bucket",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        let tx = self.conn.transaction()?;
        let mut days: BTreeMap<(String, i64), i64> = BTreeMap::new();
        let mut totals = Counts::default();
        for row in rows {
            let (bucket, dev, cnt) = row?;
            if !(0..=1).contains(&dev) || cnt < 0 {
                bail!("旧数据库包含无效的设备或计数");
            }
            let ts = bucket.checked_mul(60).context("旧数据库时间溢出")?;
            let date = Local
                .timestamp_opt(ts, 0)
                .single()
                .context("旧数据库时间无效")?;
            *days
                .entry((date.format("%Y-%m-%d").to_string(), dev))
                .or_default() += cnt;
            totals.add(dev, cnt);
            tx.execute(
                "INSERT INTO legacy_minutes VALUES(?1,?2,?3)",
                params![bucket, dev, cnt],
            )?;
        }
        for ((day, dev), cnt) in days {
            tx.execute(
                "INSERT INTO daily VALUES(?1,?2,?3) ON CONFLICT(day,dev) DO UPDATE SET cnt=cnt+excluded.cnt",
                params![day, dev, cnt],
            )?;
        }
        for (dev, cnt) in [(0, totals.mouse), (1, totals.keyboard)] {
            tx.execute(
                "UPDATE totals SET cnt=cnt+?1 WHERE dev=?2",
                params![cnt, dev],
            )?;
        }
        tx.execute(
            "INSERT INTO metadata VALUES('legacy_imported',?1)",
            [path.to_string_lossy().as_ref()],
        )?;
        tx.commit()?;
        Ok(totals.total())
    }

    pub fn legacy_imported(&self) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='legacy_imported')",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn insert(&mut self, events: &[InputEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        for event in events {
            let ts = Local
                .timestamp_millis_opt(event.ts_ms)
                .single()
                .context("事件时间无效")?;
            let dev = event.device as i64;
            let day = ts.format("%Y-%m-%d").to_string();
            let minute = event.ts_ms.div_euclid(60_000);
            tx.execute("UPDATE totals SET cnt=cnt+1 WHERE dev=?1", [dev])?;
            tx.execute(
                "INSERT INTO daily VALUES(?1,?2,1) ON CONFLICT(day,dev) DO UPDATE SET cnt=cnt+1",
                params![day, dev],
            )?;
            tx.execute("INSERT INTO minutes VALUES(?1,?2,1) ON CONFLICT(bucket,dev) DO UPDATE SET cnt=cnt+1", params![minute, dev])?;
            tx.execute(
                "INSERT INTO recent VALUES(?1,?2,1) ON CONFLICT(ts_ms,dev) DO UPDATE SET cnt=cnt+1",
                params![event.ts_ms, dev],
            )?;
            if event.device == Device::Keyboard {
                tx.execute(
                    "INSERT INTO keys VALUES(?1,1) ON CONFLICT(code) DO UPDATE SET cnt=cnt+1",
                    [event.code],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn query_counts(&self, sql: &str, values: impl rusqlite::Params) -> Result<Counts> {
        let mut counts = Counts::default();
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(values, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (dev, cnt) = row?;
            counts.add(dev, cnt);
        }
        Ok(counts)
    }

    pub fn lifetime(&self) -> Result<Counts> {
        self.query_counts("SELECT dev,cnt FROM totals", [])
    }

    pub fn export_daily(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        let mut output = std::fs::File::create_new(path)
            .with_context(|| format!("无法创建导出文件 {}（不会覆盖已有文件）", path.display()))?;
        writeln!(output, "date,keyboard,mouse,total")?;
        let mut stmt = self.conn.prepare("SELECT day,SUM(CASE WHEN dev=1 THEN cnt ELSE 0 END),SUM(CASE WHEN dev=0 THEN cnt ELSE 0 END) FROM daily GROUP BY day ORDER BY day")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (day, keyboard, mouse) = row?;
            writeln!(output, "{day},{keyboard},{mouse},{}", keyboard + mouse)?;
        }
        output.sync_all()?;
        Ok(())
    }

    pub fn rolling(&self, seconds: i64, now_ms: i64) -> Result<WindowCount> {
        if seconds <= 0 || seconds > 86_400 {
            bail!("滚动窗口必须在 1 到 86400 秒之间");
        }
        let cutoff = now_ms.checked_sub(seconds * 1000).context("时间范围溢出")?;
        // Exact half-open trailing interval: (now-window, now]. Never include future rows.
        let mut counts = self.query_counts(
            "SELECT dev,SUM(cnt) FROM recent WHERE ts_ms>?1 AND ts_ms<=?2 GROUP BY dev",
            params![cutoff, now_ms],
        )?;
        // Legacy counts lack sub-minute timestamps. Include intersecting buckets,
        // and explicitly tell callers that these particular windows are approximate.
        let mut stmt = self.conn.prepare(
            "SELECT dev,SUM(cnt) FROM legacy_minutes WHERE bucket>=?1 AND bucket<=?2 GROUP BY dev",
        )?;
        let rows = stmt.query_map(
            params![cutoff.div_euclid(60_000), now_ms.div_euclid(60_000)],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )?;
        let mut approximate = false;
        for row in rows {
            let (dev, cnt) = row?;
            counts.add(dev, cnt);
            approximate |= cnt > 0;
        }
        Ok(WindowCount {
            seconds,
            counts,
            approximate,
        })
    }

    /// Minute details and summaries are permanent; only the rolling cache expires.
    pub fn prune_rolling_cache(&mut self, now_ms: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM recent WHERE ts_ms<?1",
            [now_ms.saturating_sub(86_400_000)],
        )?;
        Ok(())
    }

    pub fn snapshot(&self, now: DateTime<Local>) -> Result<Snapshot> {
        let today_date = now.date_naive();
        let today_s = today_date.format("%Y-%m-%d").to_string();
        let week_s = (today_date - Days::new(6)).format("%Y-%m-%d").to_string();
        let today = self.query_counts("SELECT dev,cnt FROM daily WHERE day=?1", [&today_s])?;
        let week = self.query_counts(
            "SELECT dev,SUM(cnt) FROM daily WHERE day>=?1 AND day<=?2 GROUP BY dev",
            params![week_s, today_s],
        )?;
        let mut days = BTreeMap::new();
        for offset in (0..90).rev() {
            days.insert(
                (today_date - Days::new(offset))
                    .format("%Y-%m-%d")
                    .to_string(),
                Counts::default(),
            );
        }
        let first_day = days.keys().next().cloned().unwrap_or_default();
        let mut stmt = self
            .conn
            .prepare("SELECT day,dev,cnt FROM daily WHERE day>=?1 AND day<=?2")?;
        let rows = stmt.query_map(params![first_day, today_s], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        for row in rows {
            let (day, dev, cnt) = row?;
            if let Some(counts) = days.get_mut(&day) {
                counts.add(dev, cnt);
            }
        }
        let mut hours = vec![Counts::default(); 24];
        // Native timezone boundaries handle DST's 23/25-hour days.
        let (start, end) = day_bounds(today_date)?;
        let mut stmt = self.conn.prepare(
            "SELECT bucket,dev,cnt FROM minutes WHERE bucket>=?1 AND bucket<?2
             UNION ALL SELECT bucket,dev,cnt FROM legacy_minutes WHERE bucket>=?1 AND bucket<?2",
        )?;
        let rows = stmt.query_map(
            params![start.div_euclid(60_000), end.div_euclid(60_000)],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )?;
        for row in rows {
            let (bucket, dev, cnt) = row?;
            if let Some(ts) = Local.timestamp_opt(bucket * 60, 0).single() {
                hours[ts.hour() as usize].add(dev, cnt);
            }
        }
        let mut stmt = self
            .conn
            .prepare("SELECT code,cnt FROM keys ORDER BY cnt DESC,code")?;
        let physical_keys = stmt
            .query_map([], |r| {
                Ok(KeyCount {
                    code: r.get(0)?,
                    count: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let top_keys = physical_keys.iter().take(10).cloned().collect();
        // History has a different contract from recent trends: return every recorded
        // day. A new event must never shift a 90-day cutoff past imported history.
        // Send sparse records; the UI fills zero-activity calendar days as needed.
        let mut history_days = BTreeMap::new();
        let mut stmt = self
            .conn
            .prepare("SELECT day,dev,cnt FROM daily WHERE day<=?1 ORDER BY day")?;
        for row in stmt.query_map([&today_s], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })? {
            let (day, dev, cnt) = row?;
            history_days
                .entry(day)
                .or_insert_with(Counts::default)
                .add(dev, cnt);
        }
        Ok(Snapshot {
            now: now.to_rfc3339(),
            today,
            week,
            lifetime: self.lifetime()?,
            windows: [60, 600, 3600, 86400]
                .into_iter()
                .map(|s| self.rolling(s, now.timestamp_millis()))
                .collect::<Result<_>>()?,
            days: days
                .into_iter()
                .map(|(day, counts)| DayCount { day, counts })
                .collect(),
            history_days: history_days
                .into_iter()
                .map(|(day, counts)| DayCount { day, counts })
                .collect(),
            hours,
            top_keys,
            keyboard_stats: KeyboardStats {
                physical: physical_keys,
                ..Default::default()
            },
            legacy_imported: self.legacy_imported()?,
        })
    }
}

fn day_bounds(date: NaiveDate) -> Result<(i64, i64)> {
    let next = date.succ_opt().context("日期溢出")?;
    let midnight = |d: NaiveDate| {
        Local
            .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
            .earliest()
            .context("本地午夜不存在")
    };
    Ok((
        midnight(date)?.timestamp_millis(),
        midnight(next)?.timestamp_millis(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_schema_upgrade_is_idempotent_and_rejects_newer_versions() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("independent.sqlite3");
        {
            let mut db = EventDb::open(&path).unwrap();
            db.insert(&[InputEvent {
                ts_ms: Local::now().timestamp_millis(),
                device: Device::Mouse,
                code: 1,
            }])
            .unwrap();
            db.conn.pragma_update(None, "user_version", 0).unwrap();
        }
        {
            let db = EventDb::open(&path).unwrap();
            assert_eq!(db.lifetime().unwrap().mouse, 1);
            assert_eq!(
                db.conn
                    .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                    .unwrap(),
                crate::storage_schema::VERSION
            );
            db.conn.pragma_update(None, "user_version", 999).unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        assert!(EventDb::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    fn db() -> EventDb {
        EventDb::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    fn event(ts_ms: i64) -> InputEvent {
        InputEvent {
            ts_ms,
            device: Device::Keyboard,
            code: 0,
        }
    }

    #[test]
    fn trailing_window_crosses_minute_and_excludes_cutoff_and_future() {
        let mut db = db();
        db.insert(&[
            event(60_100),
            event(119_900),
            event(120_000),
            event(120_101),
        ])
        .unwrap();
        let window = db.rolling(60, 120_100).unwrap();
        assert_eq!(window.counts.keyboard, 2);
        assert!(!window.approximate);
    }

    #[test]
    fn rolling_cache_cleanup_preserves_old_details_and_totals() {
        let mut db = db();
        db.insert(&[event(1_000)]).unwrap();
        db.prune_rolling_cache(4_000 * 86_400_000).unwrap();
        assert_eq!(db.lifetime().unwrap().keyboard, 1);
        assert_eq!(
            db.conn
                .query_row("SELECT SUM(cnt) FROM minutes", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.conn
                .query_row("SELECT SUM(cnt) FROM daily", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.conn
                .query_row("SELECT COUNT(*) FROM recent", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn legacy_import_is_atomic_idempotent_and_source_is_unchanged() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("old.sqlite3");
        let old = Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,3),(2,0,1,2);").unwrap();
        drop(old);
        let before = std::fs::read(&path).unwrap();
        let mut db = db();
        assert_eq!(db.import_legacy(&path).unwrap(), 5);
        assert_eq!(db.import_legacy(&path).unwrap(), 0);
        assert_eq!(
            db.lifetime().unwrap(),
            Counts {
                keyboard: 3,
                mouse: 2
            }
        );
        assert!(db.rolling(60, 120_100).unwrap().approximate);
        db.prune_rolling_cache(4_000 * 86_400_000).unwrap();
        assert_eq!(
            db.conn
                .query_row("SELECT SUM(cnt) FROM legacy_minutes", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            5
        );
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    #[test]
    fn bad_legacy_input_rolls_back_all_rows_and_marker() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("bad.sqlite3");
        let old = Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,1,3),(3,1,1,-1);").unwrap();
        let mut db = db();
        assert!(db.import_legacy(&path).is_err());
        assert!(!db.legacy_imported().unwrap());
        assert_eq!(db.lifetime().unwrap(), Counts::default());
    }

    #[test]
    fn legacy_negative_count_cannot_be_hidden_by_aggregation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("bad.sqlite3");
        let old = Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,1,10),(2,1,2,-4);").unwrap();
        let mut db = db();
        assert_eq!(
            db.import_legacy(&path).unwrap_err().to_string(),
            "旧数据库包含无效的设备或计数"
        );
        assert!(!db.legacy_imported().unwrap());
        assert_eq!(db.lifetime().unwrap(), Counts::default());
        for table in ["legacy_minutes", "daily"] {
            assert_eq!(
                db.conn
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
    }

    #[test]
    fn csv_exports_all_history_and_refuses_to_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("activity.csv");
        let mut db = db();
        db.insert(&[
            event(1_000),
            InputEvent {
                ts_ms: 1_000,
                device: Device::Mouse,
                code: 1,
            },
        ])
        .unwrap();
        db.export_daily(&path).unwrap();
        let before = std::fs::read_to_string(&path).unwrap();
        assert!(before.starts_with("date,keyboard,mouse,total\n"));
        assert!(before.ends_with(",1,1,2\n"));
        assert!(db.export_daily(&path).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), before);
    }

    #[test]
    fn old_database_cannot_be_selected_as_new_storage() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER);",
        )
        .unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        assert!(EventDb::open(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    #[test]
    fn complete_history_survives_new_input_and_cache_cleanup() {
        let mut db = db();
        let timestamp = |year, month, day| {
            Local
                .with_ymd_and_hms(year, month, day, 12, 0, 0)
                .single()
                .unwrap()
        };
        db.insert(&[
            event(timestamp(2026, 2, 5).timestamp_millis()),
            event(timestamp(2026, 6, 3).timestamp_millis()),
        ])
        .unwrap();
        let now = timestamp(2026, 10, 7);
        db.insert(&[event(now.timestamp_millis())]).unwrap();
        let snapshot = db.snapshot(now).unwrap();
        assert_eq!(
            snapshot
                .history_days
                .iter()
                .map(|d| d.day.as_str())
                .collect::<Vec<_>>(),
            ["2026-02-05", "2026-06-03", "2026-10-07"]
        );
        assert_eq!(
            snapshot
                .history_days
                .iter()
                .map(|d| d.counts.total())
                .sum::<i64>(),
            snapshot.lifetime.total()
        );
        assert_eq!(
            snapshot
                .days
                .iter()
                .filter(|d| d.counts.total() > 0)
                .count(),
            1
        );
        let later = now + chrono::Duration::days(400);
        db.prune_rolling_cache(later.timestamp_millis()).unwrap();
        let later = db.snapshot(later).unwrap();
        assert_eq!(later.history_days.len(), 3);
        assert_eq!(later.history_days[0].day, "2026-02-05");
    }
}
