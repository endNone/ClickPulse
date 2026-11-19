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
    conn.execute_batch(
        "INSERT INTO shortcut_daily VALUES
        ('a','macos','',8,8,10000),
        ('a','macos','2026-11-17',4,36,99999),
        ('a','macos','2026-11-16',8,8,9000),
        ('a','macos','2026-11-17',8,8,3),
        ('b','macos','2026-11-17',8,8,4),
        ('a','macos','2026-11-17',8,9,6),
        ('a','windows','2026-11-17',1,8,7)",
    )
    .unwrap();
    let day = chrono::NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    let result = clickpulse::shortcuts::read_top_for_day(&conn, day).unwrap();
    assert_eq!(result.day, "2026-11-17");
    let top = result.top.unwrap();
    assert_eq!(
        (top.platform.as_str(), top.modifiers, top.key, top.count),
        ("macos", 8, 8, 7)
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
