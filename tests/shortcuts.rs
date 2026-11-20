use chrono::Local;
use clickpulse::{
    collector::CollectorDb,
    db::EventDb,
    input::{Device, InputEvent},
    shortcuts::{CONTROL, Chord},
};

#[test]
fn date_queries_use_an_index_on_upgraded_and_reopened_databases() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("indexed.sqlite3");
    drop(CollectorDb::open(&path).unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("DROP INDEX shortcut_daily_day").unwrap();
    for _ in 0..2 {
        clickpulse::shortcuts::initialize(&conn).unwrap();
        let plan: Vec<String> = conn.prepare("EXPLAIN QUERY PLAN SELECT platform,modifiers,code,sum(cnt) FROM shortcut_daily WHERE cnt>0 AND day<>'' AND day>=?1 AND day<=?2 GROUP BY platform,modifiers,code")
            .unwrap().query_map(["2026-11-18", "2026-11-19"], |row| row.get(3)).unwrap().map(Result::unwrap).collect();
        assert!(
            plan.iter()
                .any(|line| line.contains("SEARCH shortcut_daily USING INDEX shortcut_daily_day")),
            "{plan:?}"
        );
    }
}

#[test]
fn both_storage_paths_keep_shortcuts_in_live_snapshots_and_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let now = Local::now();
    let chord = Chord::new(8, CONTROL, std::env::consts::OS).unwrap();
    let events = [
        InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Keyboard,
            code: 8,
            shortcut: Some(chord),
        },
        InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Keyboard,
            code: 8,
            shortcut: Some(chord),
        },
        InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Keyboard,
            code: 0,
            shortcut: None,
        },
        InputEvent {
            ts_ms: now.timestamp_millis(),
            device: Device::Mouse,
            code: 1,
            shortcut: None,
        },
    ];
    let collector_path = dir.path().join("collector.sqlite3");
    let app_path = dir.path().join("app.sqlite3");
    let mut collector = CollectorDb::open(&collector_path).unwrap();
    let mut app = EventDb::open(&app_path).unwrap();
    collector.insert(&events).unwrap();
    app.insert(&events).unwrap();
    let check = |snapshot: clickpulse::db::Snapshot| {
        assert_eq!(snapshot.lifetime.keyboard, 3);
        assert_eq!(snapshot.lifetime.mouse, 1);
        let shortcuts = snapshot.shortcuts.unwrap();
        assert_eq!(shortcuts.len(), 1);
        assert_eq!(shortcuts[0].count, 2);
        assert_eq!(shortcuts[0].key, 8);
        assert_eq!(shortcuts[0].modifiers, CONTROL);
    };
    check(collector.snapshot(now).unwrap());
    check(app.snapshot(now).unwrap());
    drop(collector);
    drop(app);
    check(
        CollectorDb::open(&collector_path)
            .unwrap()
            .snapshot(now)
            .unwrap(),
    );
    check(EventDb::open(&app_path).unwrap().snapshot(now).unwrap());
}

#[test]
fn both_writers_bucket_shortcuts_by_the_event_local_date_and_skip_unrequested_rows() {
    use chrono::{Duration, TimeZone};
    use clickpulse::{shortcuts, source::LiveSource};
    use rusqlite::Connection;
    let dir = tempfile::tempdir().unwrap();
    let midnight = Local.with_ymd_and_hms(2026, 11, 16, 0, 0, 0).unwrap();
    let chord = Chord::new(8, CONTROL, std::env::consts::OS).unwrap();
    let events: Vec<_> = [-1, 0, 1]
        .into_iter()
        .map(|seconds| InputEvent {
            ts_ms: (midnight + Duration::seconds(seconds)).timestamp_millis(),
            device: Device::Keyboard,
            code: 8,
            shortcut: Some(chord),
        })
        .collect();
    let collector_path = dir.path().join("collector.sqlite3");
    let app_path = dir.path().join("app.sqlite3");
    let mut collector = CollectorDb::open(&collector_path).unwrap();
    let mut app = EventDb::open(&app_path).unwrap();
    collector.insert(&events).unwrap();
    app.insert(&events).unwrap();
    let check = |snapshot: clickpulse::db::Snapshot| {
        assert_eq!(snapshot.lifetime.keyboard, 3);
        assert!(snapshot.shortcuts.is_none());
        assert!(
            serde_json::to_value(snapshot)
                .unwrap()
                .get("shortcuts")
                .is_none()
        );
    };
    check(
        collector
            .snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    check(
        app.snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    let source = LiveSource::open(&collector_path).unwrap();
    check(
        source
            .snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    for path in [&collector_path, &app_path] {
        let conn = Connection::open(path).unwrap();
        assert_eq!(
            shortcuts::read_range(&conn, "2026-11-15", "2026-11-15").unwrap()[0].count,
            1
        );
        assert_eq!(
            shortcuts::read_range(&conn, "2026-11-16", "2026-11-16").unwrap()[0].count,
            2
        );
        assert_eq!(shortcuts::read(&conn).unwrap()[0].count, 3);
        // Corrupt only the shortcut data: unrelated views must not read it.
        conn.execute(
            "INSERT INTO shortcut_daily VALUES('bad','invalid','2026-11-16',1,8,1)",
            [],
        )
        .unwrap();
    }
    check(
        collector
            .snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    check(
        app.snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    check(
        source
            .snapshot_for_view(midnight, Default::default(), false)
            .unwrap(),
    );
    assert!(
        collector
            .snapshot_for_view(midnight, Default::default(), true)
            .is_err()
    );
    assert!(
        app.snapshot_for_view(midnight, Default::default(), true)
            .is_err()
    );
    assert!(
        source
            .snapshot_for_view(midnight, Default::default(), true)
            .is_err()
    );
}

#[test]
fn share_leader_groups_origins_excludes_undated_counts_and_breaks_ties_stably() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    clickpulse::shortcuts::initialize(&conn).unwrap();
    let platform = std::env::consts::OS;
    let other = if platform == "macos" {
        "windows"
    } else {
        "macos"
    };
    conn.execute(
        "INSERT INTO shortcut_daily VALUES
        ('a',?1,'',8,8,10000),
        ('a',?1,'2026-11-17',4,36,99999),
        ('a',?1,'2026-11-16',8,8,9000),
        ('a',?1,'2026-11-17',8,8,3),
        ('b',?1,'2026-11-17',8,8,4),
        ('a',?1,'2026-11-17',8,9,6),
        ('a',?2,'2026-11-17',1,8,999)",
        [platform, other],
    )
    .unwrap();
    let day = chrono::NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    let result = clickpulse::shortcuts::read_top_for_day(&conn, day).unwrap();
    assert_eq!(result.day, "2026-11-17");
    let top = result.top.unwrap();
    assert_eq!(
        (top.platform.as_str(), top.modifiers, top.key, top.count),
        (platform, 8, 8, 7)
    );
    assert!(
        clickpulse::shortcuts::read_top_for_day(&conn, day.succ_opt().unwrap())
            .unwrap()
            .top
            .is_none()
    );
    let source = rusqlite::Connection::open_in_memory().unwrap();
    source.execute_batch("CREATE TABLE shortcut_totals(origin TEXT,platform TEXT,modifiers INTEGER,code INTEGER,cnt INTEGER); INSERT INTO shortcut_totals VALUES('a','macos',8,8,99)").unwrap();
    assert!(
        clickpulse::shortcuts::read_top_for_day(&source, day)
            .unwrap()
            .top
            .is_none()
    );
}

#[test]
fn mac_shortcuts_use_layout_characters_but_keep_modifier_bits() {
    use clickpulse::shortcuts::{META, SHIFT};
    for (physical, character, expected) in [
        (12, 'a', 0),
        (16, 'z', 6),
        (34, 'c', 8),
        (12, 'я', 12),
        (24, '=', 24),
    ] {
        for mods in [META, META | SHIFT] {
            assert_eq!(
                Chord::with_layout(physical, mods, "macos", |key| {
                    assert_eq!(key, physical);
                    Some(character)
                }),
                Some(Chord {
                    modifiers: mods,
                    key: expected
                })
            );
        }
    }
}

#[test]
fn us_shortcut_positions_are_identity_mapped() {
    use clickpulse::shortcuts::META;
    // Physical rows of a US ANSI keyboard, including every supported punctuation key.
    let rows: &[(&str, &[u32])] = &[
        (
            "`1234567890-=",
            &[50, 18, 19, 20, 21, 23, 22, 26, 28, 25, 29, 27, 24],
        ),
        (
            "qwertyuiop[]\\",
            &[12, 13, 14, 15, 17, 16, 32, 34, 31, 35, 33, 30, 42],
        ),
        ("asdfghjkl;'", &[0, 1, 2, 3, 5, 4, 38, 40, 37, 41, 39]),
        ("zxcvbnm,./", &[6, 7, 8, 9, 11, 45, 46, 43, 47, 44]),
    ];
    for &(row, codes) in rows {
        for (ch, &key) in row.chars().zip(codes) {
            let mut called = false;
            let chord = Chord::with_layout(key, META, "macos", |_| {
                called = true;
                Some(ch)
            });
            assert!(called);
            assert_eq!(chord, Chord::new(key, META, "macos"));
        }
    }
}

#[test]
fn ordinary_input_and_noncharacter_keys_never_consult_layout() {
    use clickpulse::shortcuts::{ALT, META, PRINT_SCREEN, SHIFT};
    let forbidden = |_| -> Option<char> { panic!("unexpected layout lookup") };
    for platform in ["macos", "windows"] {
        for mods in [0, SHIFT] {
            for key in 0..=50 {
                let _ = Chord::with_layout(key, mods, platform, forbidden);
            }
        }
        for key in [
            36,
            48,
            49,
            51,
            53,
            56,
            65,
            69,
            71,
            76,
            78,
            81,
            82,
            96,
            123,
            126,
            PRINT_SCREEN,
        ] {
            assert_eq!(
                Chord::with_layout(key, META, platform, forbidden),
                Chord::new(key, META, platform)
            );
        }
    }
    for mods in [ALT, ALT | SHIFT] {
        assert!(Chord::with_layout(12, mods, "macos", forbidden).is_none());
    }
}

#[test]
fn azerty_event_counts_physical_q_and_logical_select_all_in_both_databases() {
    use clickpulse::shortcuts::META;
    let dir = tempfile::tempdir().unwrap();
    let now = Local::now();
    let event = InputEvent {
        ts_ms: now.timestamp_millis(),
        device: Device::Keyboard,
        code: 12,
        shortcut: Chord::with_layout(12, META, "macos", |_| Some('a')),
    };
    let mut app = EventDb::open(&dir.path().join("app.db")).unwrap();
    let mut collector = CollectorDb::open(&dir.path().join("collector.db")).unwrap();
    app.insert(std::slice::from_ref(&event)).unwrap();
    collector.insert(&[event]).unwrap();
    for snapshot in [app.snapshot(now).unwrap(), collector.snapshot(now).unwrap()] {
        assert_eq!(snapshot.lifetime.keyboard, 1);
        assert_eq!(
            snapshot
                .top_keys
                .iter()
                .map(|k| (k.code, k.count))
                .collect::<Vec<_>>(),
            [(12, 1)]
        );
        let rows = snapshot.shortcuts.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].modifiers, rows[0].key, rows[0].count),
            (META, 0, 1)
        );
    }
}

#[test]
fn windows_oem_shortcuts_use_unshifted_layout_characters_including_the_iso_key() {
    use clickpulse::{
        shortcuts::CONTROL,
        windows_input::{canonical_code, shortcut_character_with_oem},
    };
    for (scan, vk, mapped, expected) in [
        (0x2c, 0xba, ';' as u32, 41), // Dvorak: US Z position.
        (0x33, 0xbe, ';' as u32, 41), // French punctuation need not match its US VK name.
        (0x56, 0xe2, 92, 42),         // ISO extra key -> backslash.
        (0x56, 0xe2, '<' as u32, 10), // Unsupported character stays physical.
        (0x2c, 0xba, 0x8000_0060, 6), // Dead key.
        (0x2c, 0xba, 0, 6),
    ] {
        let key = canonical_code(scan);
        let chord = Chord::with_layout(key, CONTROL, "windows", |_| {
            shortcut_character_with_oem(
                scan,
                0,
                0x5a,
                |_| Some(vk),
                |code| {
                    assert_eq!(code, vk);
                    Some(mapped)
                },
            )
        })
        .unwrap();
        assert_eq!(chord.key, expected);
    }
    assert_eq!(
        Chord::with_layout(10, CONTROL, "macos", |_| Some('/'))
            .unwrap()
            .key,
        44
    );
}

#[test]
fn windows_shortcuts_resolve_foreground_scan_then_packet_then_physical() {
    use clickpulse::{
        shortcuts::CONTROL,
        windows_input::{WindowsKeyboard, shortcut_character},
    };
    use std::time::Instant;
    let now = Instant::now();
    for (scan, raw, foreground, expected) in [
        (0x10, 0x51, Some(0x41), 0),
        (0x15, 0x59, Some(0x5a), 6),
        (0x27, 0x41, Some(0xba), 41),
        (0x10, 0x41, None, 0),
        (0x10, 0x41, Some(0), 0),
        (0x10, 0, None, 12),
    ] {
        let mut keyboard = WindowsKeyboard::default();
        keyboard.event(1, 0x1d, 0, 0xa2);
        let key = keyboard.event(1, scan, 0, raw).unwrap();
        let result = keyboard.shortcut_with_layout(
            1,
            key,
            now,
            |_| true,
            |_| {
                shortcut_character(scan, 0, raw, |extended| {
                    assert_eq!(extended, u32::from(scan));
                    foreground
                })
            },
        );
        assert_eq!(
            result,
            Some(Chord {
                modifiers: CONTROL,
                key: expected
            })
        );
    }
    assert_eq!(
        shortcut_character(0x10, 2, 0, |s| {
            assert_eq!(s, 0xe010);
            Some(0xbb)
        }),
        Some('=')
    );
    assert_eq!(
        shortcut_character(0, 0, 0xbd, |_| panic!("no scan to map")),
        Some('-')
    );
    let mut keyboard = WindowsKeyboard::default();
    keyboard.event(1, 0x38, 2, 0xa5);
    assert!(
        keyboard
            .shortcut_with_layout(1, 12, now, |_| true, |_| panic!("AltGr composition"))
            .is_none()
    );
}
