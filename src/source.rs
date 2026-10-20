//! Read the user's running collector directly. Never import a backup or start a
//! second sensor when the authoritative source is available.
use std::{collections::BTreeMap, path::Path, time::Duration};

use anyhow::{Context, Result};
use chrono::{DateTime, Days, Local, TimeZone};
use rusqlite::{Connection, OpenFlags, params};

use crate::db::{
    Counts, DayCount, EncodedKeyCount, KeyCount, KeyboardStats, Snapshot, WindowCount,
};

pub struct LiveSource {
    conn: Connection,
}

pub struct ActivityCounts {
    pub lifetime: Counts,
    pub windows: Vec<WindowCount>,
}

impl LiveSource {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("无法读取实时数据库 {}", path.display()))?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA query_only=ON")?;
        crate::storage_schema::require_key_encoding(&conn)?;
        conn.prepare("SELECT bucket,dev,code,cnt FROM events LIMIT 0")?;
        Ok(Self { conn })
    }

    pub fn snapshot(&self, now: DateTime<Local>) -> Result<Snapshot> {
        // A short read transaction includes current WAL commits without freezing
        // the collector or mixing different versions across individual queries.
        let tx = self.conn.unchecked_transaction()?;
        let today_s = now.date_naive().to_string();
        let week_s = (now.date_naive() - Days::new(6)).to_string();
        let ready = aggregates_ready(&tx)?;
        let activity = activity_counts(&tx, now.timestamp_millis(), ready)?;
        let lifetime = activity.lifetime;
        let daily_sql = if ready {
            "SELECT day,dev,cnt FROM daily_totals WHERE day<=?1 ORDER BY day"
        } else {
            "SELECT date(bucket*60,'unixepoch','localtime') AS day,dev,SUM(cnt) FROM events GROUP BY day,dev HAVING day<=?1 ORDER BY day"
        };
        let mut history = BTreeMap::new();
        let mut stmt = tx.prepare(daily_sql)?;
        for row in stmt.query_map([&today_s], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })? {
            let (day, dev, cnt) = row?;
            history
                .entry(day)
                .or_insert_with(Counts::default)
                .add(dev, cnt);
        }
        let today = history.get(&today_s).copied().unwrap_or_default();
        let mut week = Counts::default();
        for (day, c) in &history {
            if day >= &week_s && day <= &today_s {
                week.keyboard += c.keyboard;
                week.mouse += c.mouse;
            }
        }
        let days = (0..90)
            .rev()
            .map(|offset| {
                let day = (now.date_naive() - Days::new(offset)).to_string();
                DayCount {
                    counts: history.get(&day).copied().unwrap_or_default(),
                    day,
                }
            })
            .collect();
        let coverage = if ready {
            metadata_int(&tx, "precise_since_ms")?
        } else {
            None
        };
        let now_ms = now.timestamp_millis();
        let windows = activity.windows;
        let start =
            crate::db::first_valid_local_instant(&Local, now.date_naive())?.timestamp_millis();
        let mut hours = vec![Counts::default(); 24];
        let mut stmt=tx.prepare("SELECT CAST(strftime('%H',bucket*60,'unixepoch','localtime') AS INTEGER),dev,SUM(cnt) FROM events WHERE bucket>=?1 AND bucket<=?2 GROUP BY 1,2")?;
        for row in stmt.query_map(
            params![start.div_euclid(60000), now_ms.div_euclid(60000)],
            |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )? {
            let (hour, dev, cnt) = row?;
            if let Some(c) = hours.get_mut(hour as usize) {
                c.add(dev, cnt);
            }
        }
        let physical = if table_exists(&tx, "native_key_totals")? {
            let mut stmt =
                tx.prepare("SELECT code,cnt FROM native_key_totals ORDER BY cnt DESC,code")?;
            stmt.query_map([], |r| {
                Ok(KeyCount {
                    code: r.get(0)?,
                    count: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let mut before = Vec::new();
        let mut since = Vec::new();
        let persistent = table_exists(&tx, "historical_key_totals")?;
        let rows: Vec<(bool, EncodedKeyCount)> = if persistent {
            // Persistent eras do not depend on the current machine's timezone.
            let mut stmt =
                tx.prepare("SELECT era,code,cnt FROM historical_key_totals ORDER BY code")?;
            stmt.query_map([], |r| {
                Ok((
                    r.get(0)?,
                    EncodedKeyCount {
                        code: r.get(1)?,
                        count: r.get(2)?,
                    },
                ))
            })?
            .collect::<rusqlite::Result<_>>()?
        } else {
            let cutoff = crate::storage_schema::legacy_cutoff(&tx)?;
            let mut stmt = tx.prepare("SELECT CASE WHEN code>=?3 THEN 0 ELSE bucket<?1 END,code,SUM(cnt) FROM events WHERE dev=1 AND bucket<=?2 GROUP BY 1,code ORDER BY code")?;
            stmt.query_map(
                params![
                    cutoff,
                    now_ms.div_euclid(60000),
                    crate::collector::PHYSICAL_CODE_BASE
                ],
                |r| {
                    Ok((
                        r.get(0)?,
                        EncodedKeyCount {
                            code: r.get(1)?,
                            count: r.get(2)?,
                        },
                    ))
                },
            )?
            .collect::<rusqlite::Result<_>>()?
        };
        for (old, key) in rows {
            if old {
                before.push(key);
            } else {
                since.push(key);
            }
        }
        let top_keys = physical.iter().take(10).cloned().collect();
        let keyboard_stats = KeyboardStats {
            physical,
            history_before_fix: before,
            history_since_fix: since,
            history_available: true,
            physical_since: coverage
                .and_then(|ms| Local.timestamp_millis_opt(ms).single())
                .map(|date| date.to_rfc3339()),
        };
        Ok(Snapshot {
            now: now.to_rfc3339(),
            today,
            week,
            lifetime,
            windows,
            days,
            history_days: history
                .into_iter()
                .map(|(day, counts)| DayCount { day, counts })
                .collect(),
            hours,
            top_keys,
            keyboard_stats,
            legacy_imported: ready
                && tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM clickpulse_metadata WHERE key='migrated_from')",
                    [],
                    |r| r.get(0),
                )?,
        })
    }

    /// The background logger needs counters, without reading years of calendar
    /// history, hourly groups or keyboard distributions.
    pub fn activity_counts(&self, now: DateTime<Local>) -> Result<ActivityCounts> {
        let tx = self.conn.unchecked_transaction()?;
        activity_counts(&tx, now.timestamp_millis(), aggregates_ready(&tx)?)
    }

    pub fn is_recording(&self, now_ms: i64) -> Result<bool> {
        if !table_exists(&self.conn, "clickpulse_metadata")? {
            return Ok(false);
        }
        Ok(metadata_int(&self.conn, "heartbeat_ms")?
            .is_some_and(|heartbeat| (now_ms - heartbeat).abs() < 10000))
    }

    pub fn export_daily(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        let snapshot = self.snapshot(Local::now())?;
        let mut file = std::fs::File::create_new(path)?;
        writeln!(file, "date,keyboard,mouse,total")?;
        for day in snapshot.history_days {
            writeln!(
                file,
                "{},{},{},{}",
                day.day,
                day.counts.keyboard,
                day.counts.mouse,
                day.counts.total()
            )?;
        }
        file.sync_all()?;
        Ok(())
    }
}

fn aggregates_ready(conn: &Connection) -> Result<bool> {
    Ok(table_exists(conn, "clickpulse_metadata")?
        && conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM clickpulse_metadata WHERE key='aggregates_ready')",
            [],
            |r| r.get::<_, bool>(0),
        )?)
}

fn activity_counts(conn: &Connection, now_ms: i64, ready: bool) -> Result<ActivityCounts> {
    let lifetime = if ready {
        counts(conn, "SELECT dev,cnt FROM lifetime_totals", [])?
    } else {
        counts(conn, "SELECT dev,SUM(cnt) FROM events GROUP BY dev", [])?
    };
    let coverage = if ready {
        metadata_int(conn, "precise_since_ms")?
    } else {
        None
    };
    let mut windows = Vec::new();
    for seconds in [60, 600, 3600, 86400] {
        let cutoff = now_ms - seconds * 1000;
        let precise = coverage.is_some_and(|since| cutoff >= since);
        let c = if precise {
            counts(
                conn,
                "SELECT dev,SUM(cnt) FROM recent_counts WHERE ts_ms>?1 AND ts_ms<=?2 GROUP BY dev",
                params![cutoff, now_ms],
            )?
        } else {
            counts(
                conn,
                "SELECT dev,SUM(cnt) FROM events WHERE bucket>=?1 AND bucket<=?2 GROUP BY dev",
                params![cutoff.div_euclid(60000), now_ms.div_euclid(60000)],
            )?
        };
        windows.push(WindowCount {
            seconds,
            counts: c,
            approximate: !precise && c.total() > 0,
        });
    }
    Ok(ActivityCounts { lifetime, windows })
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [name],
        |r| r.get(0),
    )?)
}

fn metadata_int(conn: &Connection, name: &str) -> Result<Option<i64>> {
    use rusqlite::OptionalExtension;
    Ok(conn
        .query_row(
            "SELECT value FROM clickpulse_metadata WHERE key=?1",
            [name],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|s| s.parse().ok()))
}

fn counts(conn: &Connection, sql: &str, values: impl rusqlite::Params) -> Result<Counts> {
    let mut result = Counts::default();
    let mut stmt = conn.prepare(sql)?;
    for row in stmt.query_map(values, |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))? {
        let (dev, cnt) = row?;
        result.add(dev, cnt);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_only_encoding_metadata_overrides_date_heuristics() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,7),(3,1,97,9); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); INSERT INTO clickpulse_metadata VALUES('legacy_key_cutoff_bucket','3'),('key_encoding_version','1');").unwrap();
        conn.execute(
            "INSERT INTO events VALUES(2,1,?1,4)",
            [crate::collector::PHYSICAL_CODE_BASE],
        )
        .unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        let reader = LiveSource::open(&path).unwrap();
        let stats = reader.snapshot(Local::now()).unwrap().keyboard_stats;
        assert_eq!(stats.history_before_fix.len(), 1);
        assert_eq!(stats.history_before_fix[0].count, 7);
        assert_eq!(
            stats
                .history_since_fix
                .iter()
                .map(|key| key.count)
                .sum::<i64>(),
            13
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn lightweight_counters_match_full_snapshot_at_window_boundaries_and_wal_updates() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("counters.sqlite3");
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); CREATE TABLE daily_totals(day TEXT,dev INTEGER,cnt INTEGER); CREATE TABLE lifetime_totals(dev INTEGER,cnt INTEGER); CREATE TABLE recent_counts(ts_ms INTEGER,dev INTEGER,cnt INTEGER); INSERT INTO clickpulse_metadata VALUES('aggregates_ready','1'); INSERT INTO lifetime_totals VALUES(1,10),(0,8);").unwrap();
        let now = Local::now();
        let ts = now.timestamp_millis();
        writer
            .execute(
                "INSERT INTO clickpulse_metadata VALUES('precise_since_ms',?1)",
                [(ts - 600_000).to_string()],
            )
            .unwrap();
        for (offset, dev, count) in [
            (60_000, 1, 4),
            (59_999, 1, 2),
            (0, 0, 3),
            (-1, 0, 9),
            (600_000, 1, 7),
            (599_999, 1, 5),
        ] {
            writer
                .execute(
                    "INSERT INTO recent_counts VALUES(?1,?2,?3)",
                    params![ts - offset, dev, count],
                )
                .unwrap();
        }
        writer
            .execute(
                "INSERT INTO events VALUES(?1,1,97,10)",
                [ts.div_euclid(60_000)],
            )
            .unwrap();
        let reader = LiveSource::open(&path).unwrap();
        let full = reader.snapshot(now).unwrap();
        let compact = reader.activity_counts(now).unwrap();
        assert_eq!(compact.lifetime, full.lifetime);
        assert_eq!(
            compact.windows[0].counts,
            Counts {
                keyboard: 2,
                mouse: 3
            }
        );
        assert_eq!(
            compact.windows[1].counts,
            Counts {
                keyboard: 11,
                mouse: 3
            }
        );
        assert!(!compact.windows[1].approximate);
        assert!(compact.windows[2].approximate);
        assert_eq!(
            serde_json::to_value(&compact.windows).unwrap(),
            serde_json::to_value(&full.windows).unwrap()
        );
        writer
            .execute("UPDATE lifetime_totals SET cnt=cnt+1 WHERE dev=0", [])
            .unwrap();
        assert_eq!(reader.activity_counts(now).unwrap().lifetime.mouse, 9);
    }

    #[test]
    fn lightweight_counter_reader_does_not_require_ui_history_tables() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("minimal.sqlite3");
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); CREATE TABLE lifetime_totals(dev INTEGER,cnt INTEGER); INSERT INTO clickpulse_metadata VALUES('aggregates_ready','1'); INSERT INTO lifetime_totals VALUES(1,42);").unwrap();
        let reader = LiveSource::open(&path).unwrap();
        assert_eq!(
            reader
                .activity_counts(Local::now())
                .unwrap()
                .lifetime
                .keyboard,
            42
        );
        assert!(reader.snapshot(Local::now()).is_err());
    }

    #[test]
    fn full_keyboard_data_partitions_history_and_keeps_keys_beyond_top_ten() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("keyboard.sqlite3");
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); CREATE TABLE native_key_totals(code INTEGER PRIMARY KEY,cnt INTEGER); INSERT INTO events VALUES(2,1,97,7),(2,0,1,100)").unwrap();
        let now = Local::now();
        writer
            .execute(
                "INSERT INTO events VALUES(?1,1,4296146943,5)",
                [now.timestamp() / 60],
            )
            .unwrap();
        for code in 0..12 {
            writer
                .execute("INSERT INTO native_key_totals VALUES(?1,1)", [code])
                .unwrap();
        }
        let snapshot = LiveSource::open(&path).unwrap().snapshot(now).unwrap();
        assert_eq!(snapshot.top_keys.len(), 10);
        assert_eq!(snapshot.keyboard_stats.physical.len(), 12);
        assert_eq!(snapshot.keyboard_stats.history_before_fix[0].count, 7);
        assert_eq!(
            snapshot.keyboard_stats.history_since_fix[0].code,
            4296146943
        );
        assert_eq!(
            snapshot
                .keyboard_stats
                .history_before_fix
                .iter()
                .chain(&snapshot.keyboard_stats.history_since_fix)
                .map(|key| key.count)
                .sum::<i64>(),
            snapshot.lifetime.keyboard
        );
    }

    #[test]
    fn readonly_source_includes_new_wal_commits_without_importing_twice() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("live.sqlite3");
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER,PRIMARY KEY(bucket,dev,code)); INSERT INTO events VALUES(2,1,97,3);").unwrap();
        let reader = LiveSource::open(&path).unwrap();
        let first = reader.snapshot(Local::now()).unwrap();
        assert_eq!(first.lifetime.total(), 3);
        writer
            .execute("INSERT INTO events VALUES(3,0,1,2)", [])
            .unwrap();
        let second = reader.snapshot(Local::now()).unwrap();
        assert_eq!(second.lifetime.total(), 5);
        assert_eq!(reader.snapshot(Local::now()).unwrap().lifetime.total(), 5);
        assert_eq!(
            second
                .history_days
                .iter()
                .map(|d| d.counts.total())
                .sum::<i64>(),
            5
        );
        assert!(reader.conn.execute("DELETE FROM events", []).is_err());
    }

    #[test]
    fn source_reads_precise_counts_without_adding_minute_counts_again() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("live.sqlite3");
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); CREATE TABLE clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT); CREATE TABLE daily_totals(day TEXT,dev INTEGER,cnt INTEGER); CREATE TABLE lifetime_totals(dev INTEGER,cnt INTEGER); CREATE TABLE recent_counts(ts_ms INTEGER,dev INTEGER,cnt INTEGER); INSERT INTO clickpulse_metadata VALUES('aggregates_ready','1'),('precise_since_ms','0'); INSERT INTO lifetime_totals VALUES(1,2);").unwrap();
        let now = Local::now();
        let ts = now.timestamp_millis();
        writer
            .execute("INSERT INTO events VALUES(?1,1,97,2)", [ts / 60000])
            .unwrap();
        writer
            .execute("INSERT INTO recent_counts VALUES(?1,1,2)", [ts - 200])
            .unwrap();
        let snapshot = LiveSource::open(&path).unwrap().snapshot(now).unwrap();
        assert_eq!(snapshot.windows[0].counts.keyboard, 2);
        assert!(!snapshot.windows[0].approximate);
    }
}
