//! Read-time calendar projection; never rewrite event timestamps or saved totals.
use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Datelike, Days, FixedOffset, Local, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{cell::RefCell, collections::BTreeMap};

use crate::{
    db::{Counts, DayCount, Snapshot},
    hourly::{self, Resolution},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimeZonePreference {
    #[default]
    System,
    Named(Tz),
}

impl TimeZonePreference {
    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Named(zone) => zone.name(),
        }
    }

    pub fn at<T: TimeZone>(self, instant: DateTime<T>) -> DateTime<FixedOffset> {
        match self {
            Self::System => instant.with_timezone(&Local).fixed_offset(),
            Self::Named(zone) => instant.with_timezone(&zone).fixed_offset(),
        }
    }

    // A paused read-only view may be reprojected after the system zone changes.
    pub(crate) fn cache_key(self) -> String {
        match self {
            Self::System => format!(
                "system:{}:{}",
                std::env::var("TZ").unwrap_or_default(),
                iana_time_zone::get_timezone()
                    .unwrap_or_else(|_| Local::now().offset().to_string())
            ),
            Self::Named(_) => self.code().to_owned(),
        }
    }

    pub fn choices() -> Vec<&'static str> {
        let mut names: Vec<_> = chrono_tz::TZ_VARIANTS
            .iter()
            .copied()
            .map(Tz::name)
            .collect();
        names.sort_unstable();
        names
    }
}

impl Serialize for TimeZonePreference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code())
    }
}

impl<'de> Deserialize<'de> for TimeZonePreference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        if name == "system" {
            return Ok(Self::System);
        }
        name.parse()
            .map(Self::Named)
            .map_err(serde::de::Error::custom)
    }
}

/// Only exclusive app writers use this cache. Read-only external collectors
/// still query their source on every refresh. Imports/reopens discard the cache.
#[derive(Default)]
pub(crate) struct ProjectionCache(RefCell<Option<CalendarHistory>>);

struct CalendarHistory {
    key: String,
    zone: TimeZonePreference,
    days: BTreeMap<String, Counts>,
}

impl ProjectionCache {
    pub(crate) fn clear(&self) {
        *self.0.borrow_mut() = None;
    }

    /// Called only after a successful input transaction, including late events.
    pub(crate) fn record(&self, events: &[crate::input::InputEvent]) {
        let mut entry = self.0.borrow_mut();
        if let Some(cache) = entry.as_mut() {
            for event in events {
                let Ok(local) = minute_in_zone(event.ts_ms.div_euclid(60_000), cache.zone) else {
                    *entry = None;
                    return;
                };
                cache
                    .days
                    .entry(local.date_naive().to_string())
                    .or_default()
                    .add(event.device as i64, 1);
            }
        }
    }
}

fn minute_in_zone(bucket: i64, zone: TimeZonePreference) -> Result<DateTime<FixedOffset>> {
    let instant = Utc
        .timestamp_opt(
            bucket.checked_mul(60).context("Event timestamp overflow")?,
            0,
        )
        .single()
        .context("Invalid event timestamp")?;
    ensure!(
        (0..=9999).contains(&instant.year()),
        "Event date outside supported calendar range"
    );
    let local = zone.at(instant);
    ensure!(
        (0..=9999).contains(&local.year()),
        "Local date outside supported calendar range"
    );
    Ok(local)
}

/// The caller holds a consistent read transaction. Aggregate minute/device rows
/// before conversion so key count does not multiply timezone conversions.
pub(crate) fn project(
    conn: &Connection,
    snapshot: &mut Snapshot,
    zone: TimeZonePreference,
    now: DateTime<Local>,
    minutes_sql: &str,
    hourly_tables: &[(&str, Resolution)],
    cache: Option<&ProjectionCache>,
) -> Result<()> {
    let local_now = zone.at(now);
    let today = local_now.date_naive();
    let today_s = today.to_string();
    let week_s = (today - Days::new(6)).to_string();
    let key = zone.cache_key();
    let cached = cache.and_then(|cache| {
        cache
            .0
            .borrow()
            .as_ref()
            .filter(|c| c.key == key)
            .map(|c| c.days.clone())
    });
    let warm = cached.is_some();
    let mut history = cached.unwrap_or_default();
    let mut hours = vec![Counts::default(); 24];
    let mut stmt = conn.prepare(minutes_sql)?;
    // A warm refresh only reads today's hour buckets. All possible zone offsets
    // fit within a day of UTC midnight; filter the exact local date below.
    let start = if warm {
        today
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
            .div_euclid(60)
            - 1440
    } else {
        i64::MIN
    };
    let end = if warm {
        now.timestamp().div_euclid(60)
    } else {
        i64::MAX
    };
    let rows = stmt.query_map([start, end], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (bucket, dev, count) = row?;
        let local = minute_in_zone(bucket, zone)?;
        let day = local.date_naive();
        if !warm {
            history.entry(day.to_string()).or_default().add(dev, count);
        }
        if day == today && local.timestamp_millis() <= now.timestamp_millis() {
            hours[local.hour() as usize].add(dev, count);
        }
    }
    if !warm && let Some(cache) = cache {
        *cache.0.borrow_mut() = Some(CalendarHistory {
            key,
            zone,
            days: history.clone(),
        });
    }
    snapshot.now = local_now.to_rfc3339();
    snapshot.today = history.get(&today_s).copied().unwrap_or_default();
    snapshot.week = Counts::default();
    for (_, c) in history.range(week_s..=today_s) {
        snapshot.week.keyboard += c.keyboard;
        snapshot.week.mouse += c.mouse;
    }
    snapshot.days = (0..90)
        .rev()
        .map(|offset| {
            let day = (today - Days::new(offset)).to_string();
            DayCount {
                counts: history.get(&day).copied().unwrap_or_default(),
                day,
            }
        })
        .collect();
    snapshot.history_days = history
        .into_iter()
        .filter(|(day, _)| day <= &today.to_string())
        .map(|(day, counts)| DayCount { day, counts })
        .collect();
    snapshot.hours = hours;
    snapshot.hourly = hourly::empty(local_now);
    for (table, resolution) in hourly_tables {
        hourly::accumulate(conn, &mut snapshot.hourly, table, *resolution)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collector::CollectorDb,
        db::EventDb,
        input::{Device, InputEvent},
    };

    fn instant(value: &str) -> DateTime<Local> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Local)
    }
    fn zone(name: &str) -> TimeZonePreference {
        TimeZonePreference::Named(name.parse().unwrap())
    }
    fn events() -> Vec<InputEvent> {
        [
            ("2026-11-03T16:00:00Z", Device::Keyboard, 2),
            ("2026-11-03T23:30:00Z", Device::Mouse, 3),
            ("2026-11-04T00:15:00Z", Device::Keyboard, 5),
        ]
        .into_iter()
        .flat_map(|(at, device, count)| {
            (0..count).map(move |_| InputEvent {
                ts_ms: instant(at).timestamp_millis(),
                device,
                code: 1,
            })
        })
        .collect()
    }
    fn check(snapshot: &Snapshot, name: &str, today: i64) {
        assert_eq!(snapshot.today.total(), today, "{name}");
        assert_eq!(snapshot.lifetime.total(), 10, "{name}");
        assert_eq!(
            snapshot
                .history_days
                .iter()
                .map(|d| d.counts.total())
                .sum::<i64>(),
            10
        );
        assert_eq!(
            snapshot
                .hourly
                .iter()
                .map(|h| h.counts.total())
                .sum::<i64>(),
            10
        );
        assert_eq!(
            snapshot
                .windows
                .iter()
                .find(|w| w.seconds == 86400)
                .unwrap()
                .counts
                .total(),
            10
        );
        assert_eq!(snapshot.days.last().unwrap().day, snapshot.now[..10]);
    }

    #[test]
    fn zones_regroup_both_schemas_and_readonly_sources_without_changing_data() {
        let temp = tempfile::tempdir().unwrap();
        let now = instant("2026-11-04T00:30:00Z");
        let app_path = temp.path().join("app.sqlite3");
        let mut app = CollectorDb::open(&app_path).unwrap();
        app.insert(&events()).unwrap();
        let mut independent = EventDb::open(&temp.path().join("independent.sqlite3")).unwrap();
        independent.insert(&events()).unwrap();
        let read = crate::source::LiveSource::open(&app_path).unwrap();
        let before = app.snapshot(now).unwrap();
        for (name, today, date) in [
            ("UTC", 5, "2026-11-04"),
            ("America/Los_Angeles", 10, "2026-11-03"),
            ("Asia/Shanghai", 10, "2026-11-04"),
            ("Asia/Kathmandu", 8, "2026-11-04"),
        ] {
            for snapshot in [
                app.snapshot_in_timezone(now, zone(name)).unwrap(),
                independent.snapshot_in_timezone(now, zone(name)).unwrap(),
                read.snapshot_in_timezone(now, zone(name)).unwrap(),
            ] {
                check(&snapshot, name, today);
                assert_eq!(&snapshot.now[..10], date);
                assert!(
                    snapshot
                        .hourly
                        .windows(2)
                        .all(|p| p[0].end_ms == p[1].start_ms)
                );
            }
        }
        let after = app.snapshot(now).unwrap();
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(after).unwrap()
        );
    }

    #[test]
    fn named_zone_dst_fold_keeps_both_repeated_hours() {
        let temp = tempfile::tempdir().unwrap();
        let mut db = EventDb::open(&temp.path().join("events.sqlite3")).unwrap();
        db.insert(
            &["2026-11-01T08:15:00Z", "2026-11-01T09:15:00Z"].map(|stamp| InputEvent {
                ts_ms: instant(stamp).timestamp_millis(),
                device: Device::Keyboard,
                code: 1,
            }),
        )
        .unwrap();
        let zone = zone("America/Los_Angeles");
        let snapshot = db
            .snapshot_in_timezone(instant("2026-11-01T12:37:00Z"), zone)
            .unwrap();
        let active: Vec<_> = snapshot
            .hourly
            .iter()
            .filter(|h| h.counts.total() > 0)
            .collect();
        assert_eq!(active.len(), 2);
        let a = zone.at(Utc.timestamp_millis_opt(active[0].start_ms).unwrap());
        let b = zone.at(Utc.timestamp_millis_opt(active[1].start_ms).unwrap());
        assert_eq!(a.hour(), 1);
        assert_eq!(b.hour(), 1);
        assert_ne!(a.offset(), b.offset());
        assert_eq!(snapshot.today.keyboard, 2);
    }

    #[test]
    fn preference_defaults_system_and_validates_named_iana_zones() {
        let prefs: crate::presentation::Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(prefs.timezone, TimeZonePreference::System);
        for name in [
            "system",
            "UTC",
            "Asia/Shanghai",
            "America/New_York",
            "Asia/Kathmandu",
        ] {
            let parsed: TimeZonePreference =
                serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), name);
        }
        for invalid in [
            serde_json::json!("GMT-8"),
            serde_json::json!("not/a-zone"),
            serde_json::json!(null),
        ] {
            assert!(serde_json::from_value::<TimeZonePreference>(invalid).is_err());
        }
    }

    #[test]
    #[ignore = "manual large-history projection check"]
    fn three_million_rows_keep_timezone_history_and_rolling_totals_consistent() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("large.sqlite3");
        let app = CollectorDb::open(&path).unwrap();
        drop(app);
        let conn = Connection::open(&path).unwrap();
        let now = Local::now();
        let minute = now.timestamp() / 60;
        conn.execute_batch("PRAGMA synchronous=OFF;").unwrap();
        conn.execute("WITH RECURSIVE m(n) AS (VALUES(0) UNION ALL SELECT n+1 FROM m WHERE n<99999),k(n) AS (VALUES(0) UNION ALL SELECT n+1 FROM k WHERE n<29) INSERT INTO events SELECT ?1-m.n,1,k.n+1099511627776,1 FROM m CROSS JOIN k",[minute]).unwrap();
        conn.execute("UPDATE lifetime_totals SET cnt=3000000 WHERE dev=1", [])
            .unwrap();
        conn.execute_batch("INSERT INTO daily_totals SELECT date(bucket*60,'unixepoch','localtime'),dev,SUM(cnt) FROM events GROUP BY 1,2;").unwrap();
        drop(conn);
        let read = crate::source::LiveSource::open(&path).unwrap();
        let started = std::time::Instant::now();
        let snapshot = read
            .snapshot_in_timezone(now, zone("Asia/Kathmandu"))
            .unwrap();
        eprintln!(
            "three-million-row timezone projection: {:?}",
            started.elapsed()
        );
        assert_eq!(
            snapshot
                .history_days
                .iter()
                .map(|d| d.counts.total())
                .sum::<i64>(),
            3000000
        );
        assert_eq!(snapshot.lifetime.keyboard, 3000000);
        assert_eq!(
            snapshot
                .hourly
                .iter()
                .map(|h| h.counts.total())
                .sum::<i64>(),
            snapshot
                .windows
                .iter()
                .find(|w| w.seconds == 86400)
                .unwrap()
                .counts
                .total()
        );
        let mut owned = CollectorDb::open(&path).unwrap();
        let cold = std::time::Instant::now();
        owned
            .snapshot_in_timezone(now, zone("Asia/Kathmandu"))
            .unwrap();
        eprintln!("owned cold projection: {:?}", cold.elapsed());
        let warm = std::time::Instant::now();
        owned
            .insert(&[InputEvent {
                ts_ms: now.timestamp_millis(),
                device: Device::Mouse,
                code: 1,
            }])
            .unwrap();
        let updated = owned
            .snapshot_in_timezone(now, zone("Asia/Kathmandu"))
            .unwrap();
        eprintln!("owned insert + warm projection: {:?}", warm.elapsed());
        assert_eq!(updated.lifetime.total(), 3_000_001);
        assert_eq!(
            updated
                .history_days
                .iter()
                .map(|d| d.counts.total())
                .sum::<i64>(),
            3_000_001
        );
    }

    fn calendar(snapshot: &Snapshot) -> serde_json::Value {
        serde_json::json!([
            snapshot.now,
            snapshot.today,
            snapshot.week,
            snapshot.lifetime,
            snapshot.days,
            snapshot.history_days,
            snapshot.hours,
            snapshot.hourly
        ])
    }

    #[test]
    fn owned_calendar_cache_matches_fresh_reads_after_late_input_midnight_dst_and_zone_changes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("owned.sqlite3");
        let independent_path = temp.path().join("independent.sqlite3");
        let mut owned = CollectorDb::open(&path).unwrap();
        let mut independent = EventDb::open(&independent_path).unwrap();
        let mut recorded = Vec::new();
        // Repeated DST hour, future input becoming present, and late old input.
        for (at, now) in [
            ("2026-11-01T08:15:00Z", "2026-11-01T08:30:00Z"),
            ("2026-11-01T09:15:00Z", "2026-11-01T08:30:00Z"),
            ("2026-10-01T23:59:00Z", "2026-11-02T09:30:00Z"),
        ] {
            let event = InputEvent {
                ts_ms: instant(at).timestamp_millis(),
                device: Device::Keyboard,
                code: 1,
            };
            owned.insert(std::slice::from_ref(&event)).unwrap();
            independent.insert(std::slice::from_ref(&event)).unwrap();
            recorded.push(event);
            for name in [
                "America/Los_Angeles",
                "Asia/Shanghai",
                "Asia/Kathmandu",
                "America/Los_Angeles",
            ] {
                for _ in 0..2 {
                    let now = instant(now);
                    let expected = crate::source::LiveSource::open(&path)
                        .unwrap()
                        .snapshot_in_timezone(now, zone(name))
                        .unwrap();
                    assert_eq!(
                        calendar(&owned.snapshot_in_timezone(now, zone(name)).unwrap()),
                        calendar(&expected)
                    );
                    let fresh_dir = tempfile::tempdir().unwrap();
                    let mut fresh = EventDb::open(&fresh_dir.path().join("fresh.sqlite3")).unwrap();
                    fresh.insert(&recorded).unwrap();
                    assert_eq!(
                        calendar(&independent.snapshot_in_timezone(now, zone(name)).unwrap()),
                        calendar(&fresh.snapshot_in_timezone(now, zone(name)).unwrap())
                    );
                }
            }
        }
        let now = instant("2026-11-02T09:30:00Z");
        for _ in 0..2 {
            let expected = crate::source::LiveSource::open(&path)
                .unwrap()
                .snapshot_in_timezone(now, TimeZonePreference::System)
                .unwrap();
            assert_eq!(
                calendar(
                    &owned
                        .snapshot_in_timezone(now, TimeZonePreference::System)
                        .unwrap()
                ),
                calendar(&expected)
            );
        }
        // A transaction that rolls back after one accepted row must not update the cache.
        owned.conn.execute_batch("CREATE TRIGGER reject_mouse BEFORE INSERT ON events WHEN NEW.dev=0 BEGIN SELECT RAISE(ABORT,'test rollback'); END;").unwrap();
        let mut rejected = recorded[0].clone();
        rejected.device = Device::Mouse;
        assert!(owned.insert(&[recorded[0].clone(), rejected]).is_err());
        let expected = crate::source::LiveSource::open(&path)
            .unwrap()
            .snapshot_in_timezone(now, TimeZonePreference::System)
            .unwrap();
        assert_eq!(
            calendar(
                &owned
                    .snapshot_in_timezone(now, TimeZonePreference::System)
                    .unwrap()
            ),
            calendar(&expected)
        );
    }

    #[test]
    fn calendar_cache_is_rebuilt_after_import_and_relocation_without_double_counting() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("owned.sqlite3");
        let incoming = temp.path().join("incoming.sqlite3");
        let mut owned = CollectorDb::open(&path).unwrap();
        let mut source = CollectorDb::open(&incoming).unwrap();
        let now = instant("2026-11-05T12:00:00Z");
        let at = instant("2020-01-01T12:00:00Z").timestamp_millis();
        let event = InputEvent {
            ts_ms: at,
            device: Device::Mouse,
            code: 1,
        };
        owned.insert(std::slice::from_ref(&event)).unwrap();
        source
            .insert(&[event.clone(), event.clone(), event.clone()])
            .unwrap();
        owned
            .snapshot_in_timezone(now, zone("America/Los_Angeles"))
            .unwrap();
        owned.import_history(&incoming, &path).unwrap();
        owned.import_history(&incoming, &path).unwrap();
        let expected = crate::source::LiveSource::open(&path)
            .unwrap()
            .snapshot_in_timezone(now, zone("America/Los_Angeles"))
            .unwrap();
        assert_eq!(expected.lifetime.total(), 3);
        let mut independent = EventDb::open(&temp.path().join("independent.sqlite3")).unwrap();
        independent
            .snapshot_in_timezone(now, zone("Asia/Shanghai"))
            .unwrap();
        assert_eq!(independent.import_legacy(&incoming).unwrap(), 3);
        assert_eq!(independent.import_legacy(&incoming).unwrap(), 0);
        assert_eq!(
            independent
                .snapshot_in_timezone(now, zone("Asia/Shanghai"))
                .unwrap()
                .history_days[0]
                .counts
                .total(),
            3
        );
        assert_eq!(
            calendar(
                &owned
                    .snapshot_in_timezone(now, zone("America/Los_Angeles"))
                    .unwrap()
            ),
            calendar(&expected)
        );
        let moved = temp.path().join("moved.sqlite3");
        let mut relocated =
            crate::storage::relocate(&path, &moved, &temp.path().join("storage.json")).unwrap();
        relocated.insert(&[event]).unwrap();
        assert_eq!(
            relocated
                .snapshot_in_timezone(now, zone("America/Los_Angeles"))
                .unwrap()
                .history_days[0]
                .counts
                .total(),
            4
        );
    }

    #[test]
    fn malformed_future_dates_return_errors_without_panicking_or_rewriting_data() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("bad-date.sqlite3");
        let db = CollectorDb::open(&path).unwrap();
        let bucket = DateTime::<Utc>::MAX_UTC.timestamp() / 60;
        db.conn
            .execute("INSERT INTO events VALUES(?1,1,1099511627777,2)", [bucket])
            .unwrap();
        db.conn
            .execute("UPDATE lifetime_totals SET cnt=2 WHERE dev=1", [])
            .unwrap();
        let result = db.snapshot_in_timezone(Local::now(), zone("Pacific/Kiritimati"));
        assert!(format!("{:#}", result.unwrap_err()).contains("supported calendar range"));
        assert_eq!(db.lifetime().unwrap().keyboard, 2);
        assert_eq!(
            db.conn
                .query_row("SELECT bucket FROM events", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            bucket
        );
    }
}
