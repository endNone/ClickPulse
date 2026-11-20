//! Daily aggregate modifier chords only. Never retain input order, text, app names or clipboard data.
use anyhow::{Result, ensure};
use rusqlite::{Connection, params};
use serde::Serialize;

pub const CONTROL: u8 = 1;
pub const ALT: u8 = 2;
pub const SHIFT: u8 = 4;
pub const META: u8 = 8;
pub const PRINT_SCREEN: u32 = 0x10137;
pub const SCROLL_LOCK: u32 = 0x10046;
pub const PAUSE: u32 = 0x10245;

const CREATE_DAILY: &str = "CREATE TABLE IF NOT EXISTS shortcut_daily(
    origin TEXT NOT NULL, platform TEXT NOT NULL,
    day TEXT NOT NULL CHECK(length(day)=10 OR day=''),
    modifiers INTEGER NOT NULL CHECK(modifiers BETWEEN 1 AND 15),
    code INTEGER NOT NULL CHECK(code BETWEEN 0 AND 126 OR code IN (65847,65606,66117)),
    cnt INTEGER NOT NULL CHECK(cnt>=0), PRIMARY KEY(origin,platform,day,modifiers,code))";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chord {
    pub modifiers: u8,
    pub key: u32,
}

/// US physical codes used to label the supported logical shortcut keys.
pub(crate) fn us_character_key(ch: char) -> Option<u32> {
    const LETTERS: [u32; 26] = [
        0, 11, 8, 2, 14, 3, 5, 4, 34, 38, 40, 37, 46, 45, 31, 35, 12, 15, 1, 17, 32, 9, 13, 7, 16,
        6,
    ];
    const DIGITS: [u32; 10] = [29, 18, 19, 20, 21, 23, 22, 26, 28, 25];
    match ch.to_ascii_lowercase() {
        c @ 'a'..='z' => Some(LETTERS[(c as u8 - b'a') as usize]),
        c @ '0'..='9' => Some(DIGITS[(c as u8 - b'0') as usize]),
        '-' => Some(27),
        '=' => Some(24),
        '[' => Some(33),
        ']' => Some(30),
        ';' => Some(41),
        '\'' => Some(39),
        ',' => Some(43),
        '.' => Some(47),
        '/' => Some(44),
        '\\' => Some(42),
        '`' => Some(50),
        _ => None,
    }
}

impl Chord {
    /// A layout lookup is permitted only for the character key of an eligible chord.
    pub fn with_layout(
        key: u32,
        modifiers: u8,
        platform: &str,
        translate: impl FnOnce(u32) -> Option<char>,
    ) -> Option<Self> {
        let physical = Self::new(key, modifiers, platform)?;
        if !matches!(physical.key, 0..=35 | 37..=47 | 50) {
            return Some(physical);
        }
        let logical = translate(physical.key)
            .and_then(us_character_key)
            .unwrap_or(physical.key);
        Self::new(logical, physical.modifiers, platform)
    }

    pub fn new(key: u32, modifiers: u8, platform: &str) -> Option<Self> {
        // Print Screen and Pause have multiple native PC scan-code sequences.
        let key = match key {
            0x10054 => PRINT_SCREEN,
            0x10146 => PAUSE,
            _ => key,
        };
        let pc_special = matches!(platform, "windows" | "linux")
            && matches!(key, PRINT_SCREEN | SCROLL_LOCK | PAUSE);
        // A modifier press alone, Caps Lock, unknown keys and ordinary typing are not chords.
        if (key > 126 && !pc_special) || matches!(key, 54..=63) || modifiers == 0 || modifiers > 15
        {
            return None;
        }
        let non_text = pc_special
            || matches!(key,
                36 | 48 | 51 | 53 | 64 | 71..=74 | 79 | 80 | 90 | 96..=101 |
                103 | 105..=107 | 109 | 111 | 113..=126
            );
        // Shift-letter and Option-letter on Mac are text input, not recorded shortcuts.
        if modifiers & (CONTROL | META) == 0
            && (!non_text && (modifiers & ALT == 0 || platform == "macos"))
        {
            return None;
        }
        Some(Self { modifiers, key })
    }
}

pub fn modifiers(keys: impl Iterator<Item = u32>) -> u8 {
    keys.fold(0, |bits, key| {
        bits | match key {
            59 | 62 => CONTROL,
            58 | 61 => ALT,
            56 | 60 => SHIFT,
            54 | 55 => META,
            _ => 0,
        }
    })
}

pub fn mac_flags(flags: u64) -> u8 {
    (u8::from(flags & (1 << 18) != 0) * CONTROL)
        | (u8::from(flags & (1 << 19) != 0) * ALT)
        | (u8::from(flags & (1 << 17) != 0) * SHIFT)
        | (u8::from(flags & (1 << 20) != 0) * META)
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortcutCount {
    pub platform: String,
    pub modifiers: u8,
    pub key: u32,
    pub count: i64,
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?1 AND type='table')",
        [name],
        |r| r.get(0),
    )?)
}

pub fn initialize(conn: &Connection) -> Result<()> {
    // A savepoint works both alone and inside the caller's schema transaction.
    // The private preview's undated counts must never be assigned to a real day.
    conn.execute_batch("SAVEPOINT shortcut_daily_upgrade")?;
    let result = (|| -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clickpulse_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
            INSERT OR IGNORE INTO clickpulse_metadata VALUES('shortcut_origin',lower(hex(randomblob(16))));",
        )?;
        conn.execute_batch(CREATE_DAILY)?;
        conn.execute_batch("CREATE INDEX IF NOT EXISTS shortcut_daily_day ON shortcut_daily(day)")?;
        if table_exists(conn, "shortcut_totals")? {
            conn.execute_batch(
                "INSERT INTO shortcut_daily(origin,platform,day,modifiers,code,cnt)
                 SELECT origin,platform,'',modifiers,code,cnt FROM shortcut_totals WHERE true
                 ON CONFLICT(origin,platform,day,modifiers,code) DO UPDATE SET cnt=max(shortcut_daily.cnt,excluded.cnt);
                 DROP TABLE shortcut_totals",
            )?;
        }
        conn.execute_batch("RELEASE shortcut_daily_upgrade")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = conn
            .execute_batch("ROLLBACK TO shortcut_daily_upgrade; RELEASE shortcut_daily_upgrade");
    }
    result
}

pub fn insert(conn: &Connection, chord: Chord, day: chrono::NaiveDate) -> Result<()> {
    let platform = std::env::consts::OS;
    ensure!(
        Chord::new(chord.key, chord.modifiers, platform) == Some(chord),
        "Invalid shortcut chord"
    );
    conn.prepare_cached(
        "INSERT INTO shortcut_daily VALUES((SELECT value FROM clickpulse_metadata WHERE key='shortcut_origin'),?1,?2,?3,?4,1)
        ON CONFLICT(origin,platform,day,modifiers,code) DO UPDATE SET cnt=cnt+1",
    )?
    .execute(params![platform, day.to_string(), chord.modifiers, chord.key])?;
    Ok(())
}

pub(crate) fn validate_days(conn: &Connection) -> Result<()> {
    let mut query = conn.prepare_cached("SELECT DISTINCT day FROM shortcut_daily WHERE day<>''")?;
    for value in query.query_map([], |row| row.get::<_, String>(0))? {
        let day = value?;
        ensure!(
            day.len() == 10
                && chrono::NaiveDate::parse_from_str(&day, "%Y-%m-%d")
                    .is_ok_and(|parsed| parsed.to_string() == day),
            "Invalid shortcut date"
        );
    }
    Ok(())
}

pub fn read(conn: &Connection) -> Result<Vec<ShortcutCount>> {
    let table = if table_exists(conn, "shortcut_daily")? {
        "shortcut_daily"
    } else if table_exists(conn, "shortcut_totals")? {
        // Read-only source mode must also understand an unmigrated private copy.
        "shortcut_totals"
    } else {
        return Ok(Vec::new());
    };
    read_grouped(conn, table, None, 6001, None)
}

/// Inclusive recording-local dates, matching daily_totals. Undated preview
/// counts contribute to read(), never to a dated report. No hourly data exists.
pub fn read_range(conn: &Connection, from_day: &str, to_day: &str) -> Result<Vec<ShortcutCount>> {
    for day in [from_day, to_day] {
        ensure!(
            day.len() == 10
                && chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
                    .is_ok_and(|parsed| parsed.to_string() == day),
            "Invalid shortcut date"
        );
    }
    ensure!(from_day <= to_day, "Shortcut date range is reversed");
    if !table_exists(conn, "shortcut_daily")? {
        return Ok(Vec::new());
    }
    read_grouped(conn, "shortcut_daily", Some((from_day, to_day)), 6001, None)
}

#[derive(Clone, Debug, Serialize)]
pub struct DailyTop {
    pub day: String,
    pub top: Option<ShortcutCount>,
}

/// One winning combination for a recording-local day, summed across origins.
/// Empty days are explicit, so the UI can distinguish them from pending reads.
pub fn read_top_for_day(conn: &Connection, day: chrono::NaiveDate) -> Result<DailyTop> {
    let day = day.to_string();
    let top = if table_exists(conn, "shortcut_daily")? {
        read_grouped(
            conn,
            "shortcut_daily",
            Some((&day, &day)),
            1,
            Some(std::env::consts::OS),
        )?
        .into_iter()
        .next()
    } else {
        None
    };
    Ok(DailyTop { day, top })
}

pub(crate) fn read_ranked_range(
    conn: &Connection,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Result<Vec<ShortcutCount>> {
    if !table_exists(conn, "shortcut_daily")? {
        return Ok(Vec::new());
    }
    read_grouped(
        conn,
        "shortcut_daily",
        Some((&start.to_string(), &end.to_string())),
        5,
        Some(std::env::consts::OS),
    )
}

fn read_grouped(
    conn: &Connection,
    table: &str,
    range: Option<(&str, &str)>,
    limit: usize,
    report_platform: Option<&str>,
) -> Result<Vec<ShortcutCount>> {
    // Bound the grouped output, not the per-origin/per-day input rows.
    let filter = if range.is_some() {
        " AND day<>'' AND day>=?1 AND day<=?2"
    } else {
        ""
    };
    let report_filter = if report_platform.is_some() {
        " AND platform=? AND modifiers<>4"
    } else {
        ""
    };
    let mut statement = conn.prepare_cached(&format!("SELECT platform,modifiers,code,sum(cnt) AS total FROM {table} WHERE cnt>0{filter}{report_filter} GROUP BY platform,modifiers,code ORDER BY total DESC,platform,modifiers,code LIMIT {limit}"))?;
    let bounds: Vec<&str> = range
        .into_iter()
        .flat_map(|(from, to)| [from, to])
        .chain(report_platform)
        .collect();
    let rows = statement.query_map(rusqlite::params_from_iter(bounds), |r| {
        Ok(ShortcutCount {
            platform: r.get(0)?,
            modifiers: r.get(1)?,
            key: r.get(2)?,
            count: r.get(3)?,
        })
    })?;
    let counts: Vec<ShortcutCount> = rows.collect::<std::result::Result<_, _>>()?;
    ensure!(counts.len() <= 6000, "Too many shortcut counters");
    for row in &counts {
        ensure!(
            matches!(row.platform.as_str(), "macos" | "windows" | "linux")
                && row.count >= 0
                && Chord::new(row.key, row.modifiers, &row.platform)
                    == Some(Chord {
                        key: row.key,
                        modifiers: row.modifiers
                    }),
            "Invalid shortcut counter"
        );
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modifier_function_keys_and_pc_special_keys_are_counted() {
        for key in [
            122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111, 105, 107, 113, 106, 64, 79, 80,
            90,
        ] {
            assert!(
                Chord::new(key, SHIFT, "macos").is_some(),
                "Shift-F key {key}"
            );
            assert!(
                Chord::new(key, ALT, "macos").is_some(),
                "Option-F key {key}"
            );
        }
        for platform in ["windows", "linux"] {
            assert_eq!(
                Chord::new(0x10054, ALT, platform).unwrap().key,
                PRINT_SCREEN
            );
            assert_eq!(Chord::new(0x10146, CONTROL, platform).unwrap().key, PAUSE);
            assert!(Chord::new(SCROLL_LOCK, SHIFT, platform).is_some());
            assert!(Chord::new(0x1ffff, META, platform).is_none());
        }
        assert!(Chord::new(PRINT_SCREEN, META, "macos").is_none());
    }
    #[test]
    fn preview_migrates_to_unknown_day_transactionally_and_idempotently() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE shortcut_totals(origin TEXT NOT NULL,platform TEXT NOT NULL,
             modifiers INTEGER NOT NULL CHECK(modifiers BETWEEN 1 AND 15),
             code INTEGER NOT NULL CHECK(code BETWEEN 0 AND 126), cnt INTEGER NOT NULL CHECK(cnt>=0),
             PRIMARY KEY(origin,platform,modifiers,code));
             INSERT INTO shortcut_totals VALUES('original','windows',1,8,12)",
        ).unwrap();
        assert_eq!(read(&conn).unwrap()[0].count, 12);
        assert!(
            read_range(&conn, "2026-11-01", "2026-11-30")
                .unwrap()
                .is_empty()
        );
        {
            let tx = conn.transaction().unwrap();
            initialize(&tx).unwrap();
            assert!(!table_exists(&tx, "shortcut_totals").unwrap());
            assert_eq!(read(&tx).unwrap()[0].count, 12);
            assert!(
                read_range(&tx, "2026-11-01", "2026-11-30")
                    .unwrap()
                    .is_empty()
            );
        }
        assert!(table_exists(&conn, "shortcut_totals").unwrap());
        assert!(!table_exists(&conn, "shortcut_daily").unwrap());
        initialize(&conn).unwrap();
        initialize(&conn).unwrap();
        assert!(!table_exists(&conn, "shortcut_totals").unwrap());
        let (origin, day, count): (String, String, i64) = conn
            .query_row("SELECT origin,day,cnt FROM shortcut_daily", [], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })
            .unwrap();
        assert_eq!((origin.as_str(), day.as_str(), count), ("original", "", 12));
        conn.execute(
            "INSERT INTO shortcut_daily VALUES('original','windows','2026-11-17',2,?1,3)",
            [PRINT_SCREEN],
        )
        .unwrap();
        assert_eq!(
            read(&conn).unwrap().iter().map(|r| r.count).sum::<i64>(),
            15
        );
        assert_eq!(
            read_range(&conn, "2026-11-17", "2026-11-17").unwrap()[0].count,
            3
        );
    }

    #[test]
    fn failed_preview_migration_does_not_drop_original_counts() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE shortcut_totals(origin,platform,modifiers,code,cnt);
            INSERT INTO shortcut_totals VALUES('keep','windows',1,8,12),('invalid','windows',1,99999,1)").unwrap();
        assert!(initialize(&conn).is_err());
        assert_eq!(
            conn.query_row("SELECT sum(cnt) FROM shortcut_totals", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            13
        );
        assert!(!table_exists(&conn, "shortcut_daily").unwrap());
    }

    #[test]
    fn date_ranges_exclude_unknown_dates_and_group_many_origins_and_days() {
        let mut conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO shortcut_daily VALUES('old','windows','',1,8,900),
            ('a','windows','2026-11-16',1,8,3),('a','windows','2026-11-17',1,8,5),
            ('b','windows','2026-11-17',1,8,7),('a','windows','2026-11-18',1,8,11)",
        )
        .unwrap();
        assert_eq!(
            read_range(&conn, "2026-11-17", "2026-11-17").unwrap()[0].count,
            12
        );
        assert_eq!(
            read_range(&conn, "2026-11-16", "2026-11-18").unwrap()[0].count,
            26
        );
        assert_eq!(read(&conn).unwrap()[0].count, 926);
        for (a, b) in [
            ("", "2026-11-17"),
            ("2026-02-30", "2026-03-01"),
            ("2026-11-18", "2026-11-17"),
        ] {
            assert!(read_range(&conn, a, b).is_err());
        }
        let tx = conn.transaction().unwrap();
        for i in 0..7000 {
            tx.execute(
                "INSERT INTO shortcut_daily VALUES(?1,'windows','2026-11-17',1,9,1)",
                [format!("origin-{i}")],
            )
            .unwrap();
        }
        tx.commit().unwrap();
        let rows = read_range(&conn, "2026-11-17", "2026-11-17").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].count, 7000);
    }
    #[test]
    fn excludes_typing_modifiers_and_mac_option_characters() {
        for key in 0..=126 {
            assert!(Chord::new(key, 0, "macos").is_none());
        }
        for key in [0, 8, 9, 12, 18] {
            assert!(Chord::new(key, SHIFT, "windows").is_none());
            assert!(Chord::new(key, ALT, "macos").is_none());
        }
        for key in 54..=63 {
            assert!(Chord::new(key, META, "macos").is_none());
        }
        assert_eq!(
            Chord::new(8, META, "macos"),
            Some(Chord {
                key: 8,
                modifiers: META
            })
        );
        assert!(Chord::new(48, ALT, "macos").is_some());
        assert!(Chord::new(8, CONTROL, "windows").is_some());
        assert_eq!(mac_flags((1 << 20) | (1 << 17)), META | SHIFT);
    }
    #[test]
    fn aggregate_only_and_roll_back_with_the_input_batch() {
        let mut db = Connection::open_in_memory().unwrap();
        assert!(read(&db).unwrap().is_empty());
        initialize(&db).unwrap();
        let chord = Chord {
            modifiers: CONTROL,
            key: 8,
        };
        {
            let tx = db.transaction().unwrap();
            insert(&tx, chord, chrono::Local::now().date_naive()).unwrap();
        }
        assert!(read(&db).unwrap().is_empty());
        insert(&db, chord, chrono::Local::now().date_naive()).unwrap();
        insert(&db, chord, chrono::Local::now().date_naive()).unwrap();
        let counts = read(&db).unwrap();
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].count, 2);
        assert_eq!(
            db.prepare("SELECT * FROM shortcut_daily")
                .unwrap()
                .column_count(),
            6
        );
    }
}
