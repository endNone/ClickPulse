use std::process::Command;

#[test]
fn help_and_plain_statistics_use_english_and_preserve_the_source() {
    let binary = env!("CARGO_BIN_EXE_clickpulse");
    for args in [
        &["--help"][..],
        &["stats", "--help"][..],
        &["export", "--help"][..],
        &["import", "--help"][..],
        &["migrate", "--help"][..],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("Usage:"));
        assert!(!help.chars().any(|c| ('\u{3400}'..='\u{9fff}').contains(&c)));
    }

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.sqlite3");
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,123),(2,0,1,7)").unwrap();
    drop(conn);
    let before = std::fs::read(&path).unwrap();
    let output = Command::new(binary)
        .arg("--source-db")
        .arg(&path)
        .arg("stats")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Today:"));
    assert!(text.contains("Last 7 days:"));
    assert!(text.contains("All time: 130 (keys 123 / clicks 7)"));
    assert!(!text.chars().any(|c| ('\u{3400}'..='\u{9fff}').contains(&c)));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn explicit_live_source_is_readonly_and_has_no_backup_import() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("live.sqlite3");
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,123)").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clickpulse"))
        .args(["--source-db", path.to_str().unwrap(), "stats", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["lifetime"]["keyboard"], 123);
    assert_eq!(data["legacy_imported"], false);
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
}

#[test]
fn missing_source_and_import_without_target_do_not_create_databases() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing.sqlite3");
    let binary = env!("CARGO_BIN_EXE_clickpulse");
    assert!(
        !Command::new(binary)
            .args(["--source-db", missing.to_str().unwrap(), "stats"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!missing.exists());
    let output = Command::new(binary)
        .args(["import", missing.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--db"));
}

#[test]
fn fresh_user_never_automatically_reads_the_former_developer_database() {
    let temp = tempfile::tempdir().unwrap();
    let legacy = temp
        .path()
        .join("code/individual/plugins/clickpulse/events.sqlite3");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    let conn = rusqlite::Connection::open(&legacy).unwrap();
    conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,123)").unwrap();
    drop(conn);
    let before = std::fs::read(&legacy).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clickpulse"))
        .env("HOME", temp.path())
        .env("LOCALAPPDATA", temp.path().join("local"))
        .env("XDG_DATA_HOME", temp.path().join("xdg"))
        .args(["stats", "--json"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "must require the App database, not silently show the old123 events"
    );
    assert_eq!(std::fs::read(&legacy).unwrap(), before);
    assert!(
        !temp
            .path()
            .join("Library/Application Support/ClickPulse/events.sqlite3")
            .exists()
    );
    assert!(!temp.path().join("local/ClickPulse/events.sqlite3").exists());
}

#[test]
fn explicit_migration_preserves_source_and_refuses_an_existing_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("old.sqlite3");
    let target = temp.path().join("new.sqlite3");
    let conn = rusqlite::Connection::open(&source).unwrap();
    conn.execute_batch("CREATE TABLE events(bucket INTEGER,dev INTEGER,code INTEGER,cnt INTEGER); INSERT INTO events VALUES(2,1,97,123)").unwrap();
    drop(conn);
    let before = std::fs::read(&source).unwrap();
    let invalid_target = temp.path().join("invalid-cutoff.sqlite3");
    let invalid = Command::new(env!("CARGO_BIN_EXE_clickpulse"))
        .arg("migrate")
        .arg("--source")
        .arg(&source)
        .arg("--output")
        .arg(&invalid_target)
        .args(["--legacy-key-cutoff", "2026-06-05T12:46:00"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(!invalid_target.exists());
    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_clickpulse"));
        command
            .arg("migrate")
            .arg("--source")
            .arg(&source)
            .arg("--output")
            .arg(&target)
            .args(["--legacy-key-cutoff", "1970-01-01T00:03:00Z"]);
        command
    };
    let first = command().output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
    let second = command().output().unwrap();
    assert!(!second.status.success());
    let stats = Command::new(env!("CARGO_BIN_EXE_clickpulse"))
        .arg("--source-db")
        .arg(&target)
        .args(["stats", "--json"])
        .output()
        .unwrap();
    assert!(stats.status.success());
    let conn = rusqlite::Connection::open(&target).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT value FROM clickpulse_metadata WHERE key='legacy_key_cutoff_bucket'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "3"
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&stats.stdout).unwrap()["lifetime"]["keyboard"],
        123
    );
}
