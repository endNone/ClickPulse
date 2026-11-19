//! Explicit, isolated profiling fixtures. Never open the user's profile.
use chrono::{Local, TimeZone};
use clickpulse::{
    collector::CollectorDb,
    db::EventDb,
    input::{Device, InputEvent},
    source::LiveSource,
    timezone::TimeZonePreference,
};
use sha2::{Digest, Sha256};

#[test]
#[ignore = "run explicitly for before/after timing; synthetic temporary databases only"]
fn full_snapshot_refresh_profile() {
    let temp = tempfile::tempdir().unwrap();
    let now = Local.timestamp_opt(1_793_491_200, 0).unwrap();
    let events: Vec<_> = (0..365 * 24)
        .map(|hour| InputEvent {
            ts_ms: now.timestamp_millis() - hour * 3_600_000,
            device: if hour % 4 == 0 {
                Device::Mouse
            } else {
                Device::Keyboard
            },
            code: if hour % 4 == 0 { 1 } else { 8 },
            shortcut: None,
        })
        .collect();
    let mut independent = EventDb::open(&temp.path().join("independent.sqlite3")).unwrap();
    let mut collector = CollectorDb::open(&temp.path().join("collector.sqlite3")).unwrap();
    independent.insert(&events).unwrap();
    collector.insert(&events).unwrap();
    let source = LiveSource::open(&temp.path().join("collector.sqlite3")).unwrap();
    let read_independent =
        || independent.snapshot_for_view(now, TimeZonePreference::Named(chrono_tz::UTC), false);
    let read_source =
        || source.snapshot_for_view(now, TimeZonePreference::Named(chrono_tz::UTC), false);
    let readers: [(&str, &dyn Fn() -> anyhow::Result<clickpulse::db::Snapshot>); 2] = [
        ("independent", &read_independent),
        ("collector-source", &read_source),
    ];
    for (label, snapshot) in readers {
        snapshot().unwrap();
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let view = snapshot().unwrap();
            assert_eq!(view.lifetime.total(), events.len() as i64);
            std::hint::black_box(view);
        }
        println!(
            "PROFILE {label}: {:.3} ms/full refresh (100 warm refreshes, {} events, 365 days)",
            start.elapsed().as_secs_f64() * 10.0,
            events.len()
        );
    }
    // Stable output for comparing revisions, including DST and non-hour offsets.
    // Exclude only the fixture creation time (not a statistic).
    for zone in ["UTC", "America/New_York", "Asia/Shanghai", "Asia/Kathmandu"] {
        for include_shortcuts in [false, true] {
            let preference = TimeZonePreference::Named(zone.parse().unwrap());
            for (label, view) in [
                (
                    "independent",
                    independent.snapshot_for_view(now, preference, include_shortcuts),
                ),
                (
                    "collector-source",
                    source.snapshot_for_view(now, preference, include_shortcuts),
                ),
            ] {
                let mut json = serde_json::to_value(view.unwrap()).unwrap();
                json["keyboard_stats"]
                    .as_object_mut()
                    .unwrap()
                    .remove("physical_since");
                let digest = Sha256::digest(serde_json::to_vec(&json).unwrap());
                println!("SNAPSHOT {label} {zone} shortcuts={include_shortcuts}: {digest:x}");
            }
        }
    }
}
