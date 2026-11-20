//! Bounded aggregates for the visible report, read from one SQLite snapshot.
use anyhow::Result;
use chrono::{Days, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    shortcuts::{ShortcutCount, read_ranked_range},
    timezone::{HistoryHour, HourRange, TimeZonePreference},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Period {
    #[default]
    Daily,
    Weekly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Selection {
    pub day: NaiveDate,
    pub start: NaiveDate,
    pub period: Period,
}

impl From<NaiveDate> for Selection {
    fn from(day: NaiveDate) -> Self {
        Self::new(day, Period::Daily, None)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    #[serde(flatten)]
    pub selection: Selection,
    pub timezone: TimeZonePreference,
    pub hours: Vec<HistoryHour>,
    pub ranking: Vec<ShortcutCount>,
}

pub const MIN_WEEKLY_DAYS: u64 = 2;
pub const MAX_WEEKLY_DAYS: u64 = 31;

impl Selection {
    pub fn new(day: NaiveDate, period: Period, selected_start: Option<NaiveDate>) -> Self {
        let floor = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let start = match period {
            Period::Daily => day,
            Period::Weekly => {
                let earliest = day
                    .checked_sub_days(Days::new(MAX_WEEKLY_DAYS - 1))
                    .unwrap_or(floor)
                    .max(floor);
                let latest = day
                    .checked_sub_days(Days::new(MIN_WEEKLY_DAYS - 1))
                    .unwrap_or(floor)
                    .max(floor);
                selected_start
                    .unwrap_or_else(|| {
                        day.checked_sub_days(Days::new(6))
                            .unwrap_or(floor)
                            .max(floor)
                    })
                    .clamp(earliest, latest)
            }
        };
        Self { day, start, period }
    }

    pub(crate) fn read(
        self,
        db: &Connection,
        minutes_sql: &str,
        zone: TimeZonePreference,
    ) -> Result<Report> {
        let start = self.start;
        let tx = db.unchecked_transaction()?;
        let hours = HourRange {
            start,
            end: self.day,
            zone,
        }
        .read(&tx, minutes_sql)?;
        let ranking = read_ranked_range(&tx, start, self.day)?;
        Ok(Report {
            selection: self,
            timezone: zone,
            hours,
            ranking,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekly_uses_local_calendar_bounds_and_ranks_after_combining_days_and_origins() -> Result<()>
    {
        let db = Connection::open_in_memory()?;
        crate::shortcuts::initialize(&db)?;
        db.execute_batch(
            "CREATE TABLE minutes(bucket INTEGER,dev INTEGER,cnt INTEGER);
             INSERT INTO minutes VALUES
               (unixepoch('2026-10-27 06:59')/60,1,99),
               (unixepoch('2026-10-27 07:00')/60,1,4),
               (unixepoch('2026-11-01 08:30')/60,1,7),
               (unixepoch('2026-11-01 09:30')/60,0,5),
               (unixepoch('2026-11-03 07:59')/60,0,6),
               (unixepoch('2026-11-03 08:00')/60,0,99);",
        )?;
        let platform = std::env::consts::OS;
        db.execute(
            "INSERT INTO shortcut_daily VALUES
               ('a',?1,'2026-10-27',8,8,5),('b',?1,'2026-10-27',8,8,2),
               ('a',?1,'2026-11-02',8,8,4),('a',?1,'2026-11-02',8,9,8),
               ('a',?1,'2026-11-02',8,0,7),('a',?1,'2026-11-02',8,6,6),
               ('a',?1,'2026-11-02',8,1,3),('a',?1,'2026-11-02',8,3,2),
               ('a',?1,'',8,3,99),('a',?1,'2026-10-26',8,3,99),
               ('a',?1,'2026-11-02',4,123,99);",
            [platform],
        )?;
        let other = if platform == "windows" {
            "macos"
        } else {
            "windows"
        };
        // Foreign imports would otherwise occupy every place in both reports.
        for key in [8, 9, 0, 6, 1, 3] {
            db.execute(
                "INSERT INTO shortcut_daily VALUES ('b',?1,'2026-11-02',1,?2,500)",
                rusqlite::params![other, key],
            )?;
        }
        let day = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        let zone = TimeZonePreference::Named(chrono_tz::America::Los_Angeles);
        let sql = "SELECT bucket,dev,cnt FROM minutes WHERE bucket BETWEEN ?1 AND ?2";
        let weekly = Selection::new(day, Period::Weekly, None).read(&db, sql, zone)?;
        assert_eq!(
            weekly.hours,
            vec![
                (day - Days::new(6), 0, 4, 0),
                (day - Days::new(1), 1, 7, 5),
                (day, 23, 0, 6),
            ]
        );
        let ranks = |r: &Report| {
            r.ranking
                .iter()
                .map(|s| (s.key, s.count))
                .collect::<Vec<_>>()
        };
        assert_eq!(ranks(&weekly), [(8, 11), (9, 8), (0, 7), (6, 6), (1, 3)]);
        let custom =
            Selection::new(day, Period::Weekly, Some(day - Days::new(1))).read(&db, sql, zone)?;
        assert_eq!(
            custom.hours,
            [(day - Days::new(1), 1, 7, 5), (day, 23, 0, 6)]
        );
        assert_eq!(ranks(&custom), [(9, 8), (0, 7), (6, 6), (8, 4), (1, 3)]);
        let daily = Selection::from(day).read(&db, sql, zone)?;
        assert_eq!(daily.hours, [(day, 23, 0, 6)]);
        assert_eq!(ranks(&daily), [(9, 8), (0, 7), (6, 6), (8, 4), (1, 3)]);
        Ok(())
    }
}
