//! Explicit native integration check; never runs in the ordinary test suite.
#![cfg(target_os = "linux")]
use clickpulse::{db::EventDb, runtime::Runtime};
use evdev::{AttributeSet, EventType, InputEvent, KeyCode, uinput::VirtualDevice};
use std::{process::Command, thread, time::Duration};

fn device() -> VirtualDevice {
    let keys: AttributeSet<KeyCode> = [
        KeyCode::KEY_A,
        KeyCode::KEY_B,
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::BTN_LEFT,
    ]
    .into_iter()
    .collect();
    let mut device = VirtualDevice::builder()
        .unwrap()
        .name("ClickPulse isolated capture test")
        .with_keys(&keys)
        .unwrap()
        .build()
        .unwrap();
    let nodes: Vec<_> = device
        .enumerate_dev_nodes_blocking()
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(!nodes.is_empty());
    assert!(
        Command::new("udevadm")
            .arg("settle")
            .status()
            .unwrap()
            .success()
    );
    for node in nodes {
        let output = Command::new("udevadm")
            .args(["info", "--query=property", "--name"])
            .arg(node)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == "LIBINPUT_IGNORE_DEVICE=1"),
            "Test device must be isolated from the desktop before emitting events"
        );
    }
    device
}
fn emit(device: &mut VirtualDevice, code: KeyCode, value: i32) {
    device
        .emit(&[InputEvent::new(EventType::KEY.0, code.code(), value)])
        .unwrap();
    thread::sleep(Duration::from_millis(40));
}
fn tap(device: &mut VirtualDevice, code: KeyCode) {
    emit(device, code, 1);
    emit(device, code, 0);
}

#[test]
#[ignore = "requires explicit isolated VM uinput test setup"]
fn native_evdev_records_repeats_pause_hotplug_and_persistent_history() {
    assert_eq!(
        std::env::var("CLICKPULSE_NATIVE_CAPTURE_TEST").as_deref(),
        Ok("1")
    );
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("events.sqlite3");
    let mut first = device();
    let mut runtime = Runtime::start(&database, None, false).unwrap();
    thread::sleep(Duration::from_millis(1300));
    let baseline = runtime.snapshot().unwrap();
    assert!(baseline.capture.listening);
    assert!(!baseline.capture.permission_needed);
    emit(&mut first, KeyCode::KEY_A, 1);
    emit(&mut first, KeyCode::KEY_A, 2);
    emit(&mut first, KeyCode::KEY_A, 2);
    emit(&mut first, KeyCode::KEY_A, 0);
    tap(&mut first, KeyCode::KEY_LEFTSHIFT);
    tap(&mut first, KeyCode::BTN_LEFT);
    let snapshot = runtime.snapshot().unwrap();
    assert_eq!(
        snapshot.stats.lifetime.keyboard - baseline.stats.lifetime.keyboard,
        2
    );
    assert_eq!(
        snapshot.stats.lifetime.mouse - baseline.stats.lifetime.mouse,
        1
    );
    runtime.set_paused(true);
    tap(&mut first, KeyCode::KEY_B);
    assert_eq!(
        runtime.snapshot().unwrap().stats.lifetime.total(),
        snapshot.stats.lifetime.total()
    );
    runtime.set_paused(false);
    tap(&mut first, KeyCode::KEY_B);
    let mut second = device();
    thread::sleep(Duration::from_millis(1300));
    tap(&mut second, KeyCode::KEY_A);
    drop(second);
    thread::sleep(Duration::from_millis(1300));
    let final_counts = runtime.snapshot().unwrap().stats.lifetime;
    assert_eq!(final_counts.keyboard - baseline.stats.lifetime.keyboard, 4);
    assert_eq!(final_counts.mouse - baseline.stats.lifetime.mouse, 1);
    runtime.shutdown().unwrap();
    drop(runtime);
    drop(first);
    let saved = EventDb::open(&database)
        .unwrap()
        .snapshot(chrono::Local::now())
        .unwrap();
    assert_eq!(saved.lifetime, final_counts);
    assert!(
        saved
            .history_days
            .iter()
            .any(|day| day.counts.keyboard >= 4 && day.counts.mouse >= 1)
    );
    println!(
        "native evdev: 4 keys + 1 click, repeats ignored, pause/resume, hotplug and history persistence passed"
    );
}
