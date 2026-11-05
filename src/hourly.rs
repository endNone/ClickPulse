//! Partition the same trailing interval as the 24-hour counter, without losing
//! the partial hour at either end or merging repeated daylight-saving hours.
use anyhow::Result;
use chrono::{DateTime, Offset, TimeZone};
use rusqlite::{Connection, params};
use serde::Serialize;

use crate::db::Counts;

pub const HOUR_MS: i64 = 3_600_000;

#[derive(Clone, Debug, Serialize)]
pub struct HourCount {
    pub start_ms: i64,
    pub end_ms: i64,
    pub counts: Counts,
    pub approximate: bool,
}

pub fn empty<Tz: TimeZone>(now: DateTime<Tz>) -> Vec<HourCount> {
    let end = now.timestamp_millis();
    let start = end - 24 * HOUR_MS;
    // Align with local hours even in zones with a :30 or :45 UTC offset.
    // Step in elapsed hours so a DST fold retains both occurrences separately.
    let offset = i64::from(now.offset().fix().local_minus_utc()) * 1000;
    let mut boundary = (start + offset).div_euclid(HOUR_MS) * HOUR_MS - offset;
    let mut hours = Vec::with_capacity(25);
    while boundary < end {
        hours.push(HourCount {
            start_ms: boundary.max(start),
            end_ms: (boundary + HOUR_MS).min(end),
            counts: Counts::default(),
            approximate: false,
        });
        boundary += HOUR_MS;
    }
    hours
}

#[derive(Clone, Copy)]
pub(crate) enum Resolution {
    Millisecond,
    Minute,
}

/// All callers supply an internal table name. SQL groups before returning rows,
/// bounding the work to 25 groups per device rather than individual key presses.
pub(crate) fn accumulate(
    conn: &Connection,
    hours: &mut [HourCount],
    table: &str,
    resolution: Resolution,
) -> Result<()> {
    let Some(first) = hours.first() else {
        return Ok(());
    };
    let start = first.start_ms;
    let end = hours.last().unwrap().end_ms;
    let anchor = first.end_ms - HOUR_MS;
    let (column, scale, comparison) = match resolution {
        Resolution::Millisecond => ("ts_ms", 1, ">"),
        Resolution::Minute => ("bucket", 60_000, ">="),
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT ({column}*{scale}-?1)/3600000 AS hour,dev,SUM(cnt)
         FROM {table} WHERE {column}{comparison}?2 AND {column}<=?3 GROUP BY hour,dev"
    ))?;
    let rows = stmt.query_map(
        params![anchor, start.div_euclid(scale), end.div_euclid(scale)],
        |r| {
            Ok((
                r.get::<_, u32>(0)? as usize,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        },
    )?;
    for row in rows {
        let (index, dev, count) = row?;
        // The exact right edge belongs to the final interval, including when
        // 'now' is itself an hour boundary. Every record is assigned once.
        let last = hours.len() - 1;
        let hour = &mut hours[index.min(last)];
        hour.counts.add(dev, count);
        hour.approximate |= matches!(resolution, Resolution::Minute) && count > 0;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    #[test]
    fn bins_cover_exactly_24_hours_and_keep_partial_edges_and_offset_hours() {
        for offset in [0, -7 * 3600, 5 * 3600 + 45 * 60, 9 * 3600 + 30 * 60] {
            let zone = FixedOffset::east_opt(offset).unwrap();
            for minute in [0, 37] {
                let now = zone.with_ymd_and_hms(2026, 11, 3, 0, minute, 0).unwrap();
                let hours = empty(now);
                assert_eq!(hours.len(), if minute == 0 { 24 } else { 25 });
                assert_eq!(
                    hours.first().unwrap().start_ms,
                    now.timestamp_millis() - 24 * HOUR_MS
                );
                assert_eq!(hours.last().unwrap().end_ms, now.timestamp_millis());
                assert_eq!(
                    hours.iter().map(|h| h.end_ms - h.start_ms).sum::<i64>(),
                    24 * HOUR_MS
                );
                for pair in hours.windows(2) {
                    assert_eq!(pair[0].end_ms, pair[1].start_ms);
                    let local = zone.timestamp_millis_opt(pair[1].start_ms).unwrap();
                    assert_eq!(local.format("%M:%S").to_string(), "00:00");
                }
            }
        }
    }

    #[test]
    fn exact_edges_and_multiple_codes_are_assigned_once_and_future_is_excluded() {
        let now = Utc.with_ymd_and_hms(2026, 11, 3, 1, 0, 0).unwrap();
        let mut hours = empty(now);
        let start = hours[0].start_ms;
        let end = now.timestamp_millis();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE recent(ts_ms INTEGER,dev INTEGER,cnt INTEGER);")
            .unwrap();
        for (stamp, dev, count) in [
            (start, 1, 100),
            (start + 1, 1, 2),
            (start + HOUR_MS, 0, 3),
            (end, 1, 5),
            (end + 1, 1, 100),
        ] {
            conn.execute(
                "INSERT INTO recent VALUES(?1,?2,?3)",
                params![stamp, dev, count],
            )
            .unwrap();
        }
        accumulate(&conn, &mut hours, "recent", Resolution::Millisecond).unwrap();
        assert_eq!(hours[0].counts.keyboard, 2);
        assert_eq!(hours[1].counts.mouse, 3);
        assert_eq!(hours[23].counts.keyboard, 5);
        assert_eq!(hours.iter().map(|h| h.counts.total()).sum::<i64>(), 10);
        assert!(hours.iter().all(|h| !h.approximate));
    }

    #[test]
    fn clock_changes_keep_distinct_elapsed_hours_without_losing_counts() {
        for (now, stamps) in [
            (
                "2026-11-01T04:37:00-08:00",
                ["2026-11-01T01:15:00-07:00", "2026-11-01T01:15:00-08:00"],
            ),
            (
                "2026-03-08T04:37:00-07:00",
                ["2026-03-08T01:15:00-08:00", "2026-03-08T03:15:00-07:00"],
            ),
        ] {
            let now = DateTime::parse_from_rfc3339(now).unwrap();
            let mut hours = empty(now);
            let conn = Connection::open_in_memory().unwrap();
            conn.execute_batch("CREATE TABLE recent(ts_ms INTEGER,dev INTEGER,cnt INTEGER);")
                .unwrap();
            for (index, stamp) in stamps.iter().enumerate() {
                conn.execute(
                    "INSERT INTO recent VALUES(?1,1,?2)",
                    params![
                        DateTime::parse_from_rfc3339(stamp)
                            .unwrap()
                            .timestamp_millis(),
                        index as i64 + 2
                    ],
                )
                .unwrap();
            }
            accumulate(&conn, &mut hours, "recent", Resolution::Millisecond).unwrap();
            let active: Vec<_> = hours.iter().filter(|h| h.counts.total() > 0).collect();
            assert_eq!(active.len(), 2);
            assert_eq!(active[0].counts.keyboard, 2);
            assert_eq!(active[1].counts.keyboard, 3);
            assert_eq!(active[1].start_ms - active[0].start_ms, HOUR_MS);
        }
    }
}
