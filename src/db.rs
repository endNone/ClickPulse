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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct MouseCounts {
    pub left: i64,
    pub right: i64,
    pub middle: i64,
    pub side_back: i64,
    pub side_forward: i64,
    pub other: i64,
}

pub(crate) fn read_mouse_counts(conn: &Connection, sql: &str) -> Result<MouseCounts> {
    let mut counts = MouseCounts::default();
    let mut stmt = conn.prepare_cached(sql)?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))? {
        let (code, count) = row?;
        match code {
            1 => counts.left += count,
            2 => counts.right += count,
            3 => counts.middle += count,
            4 => counts.side_back += count,
            5 => counts.side_forward += count,
            _ => counts.other += count,
        }
    }
    Ok(counts)
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct Snapshot {
    /// None: not requested for this view. Some([]): loaded and genuinely empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcuts: Option<Vec<crate::shortcuts::ShortcutCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<crate::report::Report>,
    pub now: String,
    pub today: Counts,
    pub week: Counts,
    pub lifetime: Counts,
    pub mouse_buttons: MouseCounts,
    pub windows: Vec<WindowCount>,
    pub days: Vec<DayCount>,
    pub history_days: Vec<DayCount>,
    pub hours: Vec<Counts>,
    pub hourly: Vec<crate::hourly::HourCount>,
    pub top_keys: Vec<KeyCount>,
    pub keyboard_stats: KeyboardStats,
    pub legacy_imported: bool,
}

const MINUTE_COUNTS: &str = "SELECT bucket,dev,SUM(cnt) FROM (SELECT bucket,dev,cnt FROM minutes WHERE bucket>=?1 AND bucket<=?2 UNION ALL SELECT bucket,dev,cnt FROM legacy_minutes WHERE bucket>=?1 AND bucket<=?2) GROUP BY bucket,dev";

pub struct EventDb {
    conn: Connection,
    projection: crate::timezone::ProjectionCache,
}

impl EventDb {
    pub fn empty_snapshot(now: DateTime<Local>) -> Result<Snapshot> {
        Self::from_connection(Connection::open_in_memory()?)?.snapshot(now)
    }
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
                    "This is a legacy database. Select a new database path and import with --legacy-db. The source was not modified."
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
             CREATE TABLE IF NOT EXISTS mouse_buttons(code INTEGER PRIMARY KEY, cnt INTEGER NOT NULL);
             INSERT OR IGNORE INTO totals VALUES(0,0),(1,0);",
        )?;
        tx.pragma_update(None, "user_version", crate::storage_schema::VERSION)?;
        crate::shortcuts::initialize(&tx)?;
        tx.commit()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(Self {
            conn,
            projection: Default::default(),
        })
    }

    /// Import the original Python database once, in a single destination transaction.
    /// Original codes cannot be safely mapped to physical keys, so only count/time data is imported.
    pub fn import_legacy(&mut self, path: &Path) -> Result<i64> {
        if self.legacy_imported()? {
            return Ok(0);
        }
        let legacy = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| {
                format!("Cannot open legacy database read-only: {}", path.display())
            })?;
        let invalid: bool = legacy.query_row(
            "SELECT EXISTS(SELECT 1 FROM events WHERE dev NOT IN (0,1) OR cnt<0)",
            [],
            |r| r.get(0),
        )?;
        if invalid {
            bail!("The legacy database contains invalid devices or counts");
        }
        let mut stmt = legacy.prepare_cached(
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
                bail!("The legacy database contains invalid devices or counts");
            }
            let ts = bucket
                .checked_mul(60)
                .context("Legacy database timestamp overflow")?;
            let date = Local
                .timestamp_opt(ts, 0)
                .single()
                .context("Invalid legacy database timestamp")?;
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
        self.projection.clear();
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
        {
            let mut totals = tx.prepare_cached("UPDATE totals SET cnt=cnt+1 WHERE dev=?1")?;
            let mut daily = tx.prepare_cached(
                "INSERT INTO daily VALUES(?1,?2,1) ON CONFLICT(day,dev) DO UPDATE SET cnt=cnt+1",
            )?;
            let mut minutes = tx.prepare_cached("INSERT INTO minutes VALUES(?1,?2,1) ON CONFLICT(bucket,dev) DO UPDATE SET cnt=cnt+1")?;
            let mut recent = tx.prepare_cached(
                "INSERT INTO recent VALUES(?1,?2,1) ON CONFLICT(ts_ms,dev) DO UPDATE SET cnt=cnt+1",
            )?;
            let mut keys = tx.prepare_cached(
                "INSERT INTO keys VALUES(?1,1) ON CONFLICT(code) DO UPDATE SET cnt=cnt+1",
            )?;
            let mut mouse = tx.prepare_cached(
                "INSERT INTO mouse_buttons VALUES(?1,1) ON CONFLICT(code) DO UPDATE SET cnt=cnt+1",
            )?;
            let mut last_date = None;
            let mut day = String::new();
            for event in events {
                let date = Local
                    .timestamp_millis_opt(event.ts_ms)
                    .single()
                    .context("Invalid event timestamp")?
                    .date_naive();
                if last_date != Some(date) {
                    day = date.to_string();
                    last_date = Some(date);
                }
                let dev = event.device as i64;
                totals.execute([dev])?;
                daily.execute(params![day, dev])?;
                minutes.execute(params![event.ts_ms.div_euclid(60_000), dev])?;
                recent.execute(params![event.ts_ms, dev])?;
                if event.device == Device::Keyboard {
                    if let Some(chord) = event.shortcut {
                        crate::shortcuts::insert(&tx, chord, date)?;
                    }
                    keys.execute([event.code])?;
                } else {
                    mouse.execute([event.code])?;
                }
            }
        }
        tx.commit()?;
        self.projection.record(events);
        Ok(())
    }

    pub(crate) fn busy_timeout(&self, timeout: Duration) -> Result<()> {
        Ok(self.conn.busy_timeout(timeout)?)
    }

    fn query_counts(&self, sql: &str, values: impl rusqlite::Params) -> Result<Counts> {
        let mut counts = Counts::default();
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map(values, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (dev, cnt) = row?;
            counts.add(dev, cnt);
        }
        Ok(counts)
    }

    pub fn today_in_timezone(
        &self,
        now: DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
    ) -> Result<Counts> {
        crate::timezone::today_counts(
            &self.conn,
            now,
            zone,
            "SELECT bucket,dev,cnt FROM minutes WHERE bucket>=?1 AND bucket<=?2 UNION ALL SELECT bucket,dev,cnt FROM legacy_minutes WHERE bucket>=?1 AND bucket<=?2",
        )
    }

    pub fn lifetime(&self) -> Result<Counts> {
        self.query_counts("SELECT dev,cnt FROM totals", [])
    }

    pub fn rolling(&self, seconds: i64, now_ms: i64) -> Result<WindowCount> {
        if seconds <= 0 || seconds > 86_400 {
            bail!("The rolling window must be between 1 and 86400 seconds");
        }
        let cutoff = now_ms
            .checked_sub(seconds * 1000)
            .context("Time range overflow")?;
        // Exact half-open trailing interval: (now-window, now]. Never include future rows.
        let mut counts = self.query_counts(
            "SELECT dev,SUM(cnt) FROM recent WHERE ts_ms>?1 AND ts_ms<=?2 GROUP BY dev",
            params![cutoff, now_ms],
        )?;
        // Legacy counts lack sub-minute timestamps. Include intersecting buckets,
        // and explicitly tell callers that these particular windows are approximate.
        let mut stmt = self.conn.prepare_cached(
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

    pub fn snapshot_in_timezone(
        &self,
        now: DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
    ) -> Result<Snapshot> {
        self.snapshot_for_view(now, zone, true)
    }

    pub(crate) fn history_hours(
        &self,
        range: crate::timezone::HourRange,
    ) -> Result<Vec<crate::timezone::HistoryHour>> {
        range.read(&self.conn, MINUTE_COUNTS)
    }

    pub fn report(
        &self,
        selection: crate::report::Selection,
        zone: crate::timezone::TimeZonePreference,
    ) -> Result<crate::report::Report> {
        selection.read(&self.conn, MINUTE_COUNTS, zone)
    }

    pub fn snapshot_for_view(
        &self,
        now: DateTime<Local>,
        zone: crate::timezone::TimeZonePreference,
        include_shortcuts: bool,
    ) -> Result<Snapshot> {
        let _tx = self.conn.unchecked_transaction()?;
        let mut snapshot = self.snapshot_base(now, include_shortcuts)?;
        crate::timezone::project(
            &self.conn,
            &mut snapshot,
            zone,
            now,
            MINUTE_COUNTS,
            &[
                ("recent", crate::hourly::Resolution::Millisecond),
                ("legacy_minutes", crate::hourly::Resolution::Minute),
            ],
            crate::timezone::Projection::Writer(&self.projection),
        )?;
        Ok(snapshot)
    }

    pub fn snapshot(&self, now: DateTime<Local>) -> Result<Snapshot> {
        self.snapshot_with_shortcuts(now, true)
    }

    fn snapshot_base(&self, now: DateTime<Local>, include_shortcuts: bool) -> Result<Snapshot> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT code,cnt FROM keys ORDER BY cnt DESC,code")?;
        let physical_keys = stmt
            .query_map([], |r| {
                Ok(KeyCount {
                    code: r.get(0)?,
                    count: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let top_keys = physical_keys.iter().take(10).cloned().collect();
        Ok(Snapshot {
            now: now.to_rfc3339(),
            shortcuts: include_shortcuts
                .then(|| crate::shortcuts::read(&self.conn))
                .transpose()?,
            lifetime: self.lifetime()?,
            mouse_buttons: read_mouse_counts(&self.conn, "SELECT code,cnt FROM mouse_buttons")?,
            windows: [60, 600, 3600, 86400]
                .into_iter()
                .map(|s| self.rolling(s, now.timestamp_millis()))
                .collect::<Result<_>>()?,
            top_keys,
            keyboard_stats: KeyboardStats {
                physical: physical_keys,
                ..Default::default()
            },
            legacy_imported: self.legacy_imported()?,
            ..Default::default()
        })
    }

    fn snapshot_with_shortcuts(
        &self,
        now: DateTime<Local>,
        include_shortcuts: bool,
    ) -> Result<Snapshot> {
        let base = self.snapshot_base(now, include_shortcuts)?;
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
            .prepare_cached("SELECT day,dev,cnt FROM daily WHERE day>=?1 AND day<=?2")?;
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
        let mut stmt = self.conn.prepare_cached(
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
        // History has a different contract from recent trends: return every recorded
        // day. A new event must never shift a 90-day cutoff past imported history.
        // Send sparse records; the UI fills zero-activity calendar days as needed.
        let mut history_days = BTreeMap::new();
        let mut stmt = self
            .conn
            .prepare_cached("SELECT day,dev,cnt FROM daily WHERE day<=?1 ORDER BY day")?;
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
        let mut hourly = crate::hourly::empty(now);
        crate::hourly::accumulate(
            &self.conn,
            &mut hourly,
            "recent",
            crate::hourly::Resolution::Millisecond,
        )?;
        crate::hourly::accumulate(
            &self.conn,
            &mut hourly,
            "legacy_minutes",
            crate::hourly::Resolution::Minute,
        )?;
        Ok(Snapshot {
            today,
            week,
            hours,
            hourly,
            days: days
                .into_iter()
                .map(|(day, counts)| DayCount { day, counts })
                .collect(),
            history_days: history_days
                .into_iter()
                .map(|(day, counts)| DayCount { day, counts })
                .collect(),
            ..base
        })
    }
}

fn day_bounds(date: NaiveDate) -> Result<(i64, i64)> {
    let next = date.succ_opt().context("Date overflow")?;
    Ok((
        first_valid_local_instant(&Local, date)?.timestamp_millis(),
        first_valid_local_instant(&Local, next)?.timestamp_millis(),
    ))
}

pub(crate) fn first_valid_local_instant<T: TimeZone>(
    timezone: &T,
    date: NaiveDate,
) -> Result<DateTime<T>> {
    let midnight = date.and_hms_opt(0, 0, 0).unwrap();
    // Search by seconds so historical transitions with sub-minute offsets work too.
    (0..86400)
        .find_map(|seconds| {
            timezone
                .from_local_datetime(&(midnight + chrono::Duration::seconds(seconds)))
                .earliest()
        })
        .context("Local midnight does not exist")
}

#[cfg(test)]
mod tests {
    #[test]
    fn independent_mouse_buttons_survive_reopening_without_guessing_old_totals() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("buttons.sqlite3");
        let mut db = EventDb::open(&path).unwrap();
        // Old independent profiles have a total but did not store button codes.
        db.conn
            .execute("UPDATE totals SET cnt=20 WHERE dev=0", [])
            .unwrap();
        for code in [1, 1, 2, 3, 5] {
            db.insert(&[InputEvent {
                shortcut: None,
                ts_ms: Local::now().timestamp_millis(),
                device: Device::Mouse,
                code,
            }])
            .unwrap();
        }
        drop(db);
        let db = EventDb::open(&path).unwrap();
        let snapshot = db.snapshot(Local::now()).unwrap();
        assert_eq!(snapshot.lifetime.mouse, 25);
        assert_eq!(
            snapshot.mouse_buttons,
            MouseCounts {
                left: 2,
                right: 1,
                middle: 1,
                side_back: 0,
                side_forward: 1,
                other: 0
            }
        );
    }
    use super::*;
    #[derive(Clone)]
    struct MidnightTransition {
        ambiguous: bool,
    }

    impl TimeZone for MidnightTransition {
        type Offset = chrono::FixedOffset;

        fn from_offset(_: &Self::Offset) -> Self {
            Self { ambiguous: false }
        }

        fn offset_from_local_date(&self, date: &NaiveDate) -> chrono::LocalResult<Self::Offset> {
            self.offset_from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        }

        fn offset_from_local_datetime(
            &self,
            date: &chrono::NaiveDateTime,
        ) -> chrono::LocalResult<Self::Offset> {
            let offset = chrono::FixedOffset::east_opt(3600).unwrap();
            if date.hour() != 0 {
                chrono::LocalResult::Single(offset)
            } else if self.ambiguous {
                chrono::LocalResult::Ambiguous(offset, chrono::FixedOffset::east_opt(0).unwrap())
            } else {
                chrono::LocalResult::None
            }
        }

        fn offset_from_utc_date(&self, _: &NaiveDate) -> Self::Offset {
            chrono::FixedOffset::east_opt(3600).unwrap()
        }

        fn offset_from_utc_datetime(&self, date: &chrono::NaiveDateTime) -> Self::Offset {
            self.offset_from_utc_date(&date.date())
        }
    }

    #[test]
    fn day_start_handles_missing_and_ambiguous_midnight() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 6).unwrap();
        let start =
            first_valid_local_instant(&MidnightTransition { ambiguous: false }, date).unwrap();
        assert_eq!(start.naive_local(), date.and_hms_opt(1, 0, 0).unwrap());
        let start =
            first_valid_local_instant(&MidnightTransition { ambiguous: true }, date).unwrap();
        assert_eq!(start.naive_local(), date.and_hms_opt(0, 0, 0).unwrap());
        assert_eq!(start.offset().local_minus_utc(), 3600);
        let start = first_valid_local_instant(&chrono::Utc, date).unwrap();
        assert_eq!(start.naive_local(), date.and_hms_opt(0, 0, 0).unwrap());
    }

    #[test]
    fn independent_schema_upgrade_is_idempotent_and_rejects_newer_versions() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("independent.sqlite3");
        {
            let mut db = EventDb::open(&path).unwrap();
            db.insert(&[InputEvent {
                shortcut: None,
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

    #[test]
    fn hourly_combines_native_and_legacy_once_and_matches_the_rolling_day() {
        let mut db = db();
        let now = DateTime::parse_from_rfc3339("2026-11-03T00:37:15-08:00")
            .unwrap()
            .with_timezone(&Local);
        let end = now.timestamp_millis();
        let start = end - 86_400_000;
        for (stamp, device) in [
            (start, Device::Keyboard),
            (start + 1, Device::Keyboard),
            (end, Device::Mouse),
            (end + 1, Device::Mouse),
        ] {
            db.insert(&[InputEvent {
                shortcut: None,
                ts_ms: stamp,
                device,
                code: 0,
            }])
            .unwrap();
        }
        db.conn
            .execute(
                "INSERT INTO legacy_minutes VALUES(?1,1,7),(?2,0,3),(?3,1,99)",
                params![
                    start.div_euclid(60_000),
                    (end - 3_600_000).div_euclid(60_000),
                    end.div_euclid(60_000) + 1
                ],
            )
            .unwrap();
        let stats = db.snapshot(now).unwrap();
        let counts = stats
            .hourly
            .iter()
            .fold(Counts::default(), |mut sum, hour| {
                sum.keyboard += hour.counts.keyboard;
                sum.mouse += hour.counts.mouse;
                sum
            });
        assert_eq!(
            counts,
            Counts {
                keyboard: 8,
                mouse: 4
            }
        );
        assert_eq!(counts, db.rolling(86_400, end).unwrap().counts);
        assert_eq!(stats.hourly.iter().filter(|h| h.approximate).count(), 2);
    }
    fn event(ts_ms: i64) -> InputEvent {
        InputEvent {
            shortcut: None,
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
            "The legacy database contains invalid devices or counts"
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
