//! Passive read-only evdev capture for X11 and Wayland. Never grabs devices.
use crate::{
    input::InputEvent,
    linux_input::{LinuxInput, identity},
    runtime::{CaptureStatus, Command},
};
use anyhow::Result;
use crossbeam_channel::Sender;
use evdev::raw_stream::RawDevice;
use std::os::{
    fd::AsRawFd,
    unix::fs::{MetadataExt, OpenOptionsExt},
};
use std::{
    collections::BTreeMap,
    fs, io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

struct Reader {
    device: RawDevice,
    input: LinuxInput,
    identity: DeviceIdentity,
    events: Vec<evdev::InputEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceIdentity {
    device: u64,
    inode: u64,
    node: u64,
    changed: (i64, i64),
}
impl From<fs::Metadata> for DeviceIdentity {
    fn from(meta: fs::Metadata) -> Self {
        Self {
            device: meta.dev(),
            inode: meta.ino(),
            node: meta.rdev(),
            changed: (meta.ctime(), meta.ctime_nsec()),
        }
    }
}

fn current_identity(path: &PathBuf) -> Option<DeviceIdentity> {
    fs::metadata(path).ok().map(Into::into)
}

fn wait_for_devices(fds: &mut [libc::pollfd], timeout: Duration) -> io::Result<()> {
    // poll also sleeps with an empty descriptor list. EINTR returns to the loop
    // so shutdown and the one-second hotplug scan keep their original deadlines.
    let millis = timeout.as_nanos().div_ceil(1_000_000).clamp(1, 1000) as i32;
    let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, millis) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

impl Reader {
    fn open(path: &PathBuf) -> io::Result<Option<Self>> {
        // RawDevice::open may request write access. This collector needs only read.
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)?;
        let node_identity = file.metadata()?.into();
        let device = RawDevice::from_fd(file.into())?;
        if !device
            .supported_keys()
            .is_some_and(|keys| keys.iter().any(|key| identity(key.code()).is_some()))
        {
            return Ok(None);
        }
        let mut input = LinuxInput::default();
        input.synchronize(device.get_key_state()?.iter().map(|k| k.code()));
        Ok(Some(Self {
            device,
            input,
            identity: node_identity,
            events: Vec::new(),
        }))
    }
    fn read(&mut self, emit: impl FnMut(InputEvent)) -> io::Result<()> {
        // Retain capacity across poll wakes, including movement-only batches.
        // Buffering lets SYN_DROPPED resynchronize the device after fetch_events
        // releases its borrow, without allocating a second output vector.
        self.events.clear();
        match self.device.fetch_events() {
            Ok(events) => self.events.extend(events),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        }
        forward_events(
            &mut self.input,
            &self.events,
            |input| {
                input.synchronize(self.device.get_key_state()?.iter().map(|k| k.code()));
                Ok(())
            },
            emit,
        )
    }
}

fn forward_events(
    input: &mut LinuxInput,
    events: &[evdev::InputEvent],
    mut resynchronize: impl FnMut(&mut LinuxInput) -> io::Result<()>,
    mut emit: impl FnMut(InputEvent),
) -> io::Result<()> {
    for event in events {
        if input.is_dropped() && event.event_type().0 == 0 && event.code() == 0 {
            resynchronize(input)?;
        } else if let Some((device, code)) =
            input.event(event.event_type().0, event.code(), event.value())
        {
            emit(InputEvent {
                ts_ms: chrono::Utc::now().timestamp_millis(),
                device,
                code,
                shortcut: if device == crate::input::Device::Keyboard {
                    input.shortcut(code)
                } else {
                    None
                },
            });
        }
    }
    Ok(())
}

fn queue_failed(stop: &AtomicBool, status: &Mutex<CaptureStatus>) {
    if let Ok(mut state) = status.lock() {
        if !stop.swap(true, Ordering::Relaxed) {
            state.listening = false;
            state.error_code = Some("error.queue");
            state.error = Some("Input queue unavailable; recording stopped.".into());
        }
    } else {
        stop.store(true, Ordering::Relaxed);
    }
}

fn update_device_status(state: &mut CaptureStatus, has_devices: bool, denied: bool) {
    state.listening = has_devices;
    state.permission_needed = denied && !has_devices;
    state.error_code = if !has_devices && !denied {
        Some("error.linux_devices")
    } else {
        None // The permissions banner already explains how to recover.
    };
    state.error = state.error_code.map(str::to_string);
}

pub(crate) fn spawn(
    tx: Sender<Command>,
    paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    status: Arc<Mutex<CaptureStatus>>,
) -> Result<JoinHandle<()>> {
    let storage_blocked = status
        .lock()
        .map_err(|_| anyhow::anyhow!("Capture status is unavailable"))?
        .storage_blocked
        .clone();
    Ok(thread::Builder::new()
        .name("clickpulse-input".into())
        .spawn(move || {
            let mut devices = BTreeMap::<PathBuf, Reader>::new();
            let mut ignored = BTreeMap::<PathBuf, DeviceIdentity>::new();
            let mut paths = Vec::new();
            let mut fds = Vec::new();
            let mut devices_changed = true;
            let mut scan: Option<Instant> = None;
            while !stop.load(Ordering::Relaxed) {
                if scan.is_none_or(|last| last.elapsed() >= Duration::from_secs(1)) {
                    scan = Some(Instant::now());
                    let previous_count = devices.len();
                    devices.retain(|path, reader| current_identity(path) == Some(reader.identity));
                    devices_changed |= previous_count != devices.len();
                    ignored.retain(|path, identity| current_identity(path) == Some(*identity));
                    let mut denied = false;
                    match fs::read_dir("/dev/input") {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                if !entry.file_name().to_string_lossy().starts_with("event") {
                                    continue;
                                }
                                let path = entry.path();
                                if devices.contains_key(&path) || ignored.contains_key(&path) {
                                    continue;
                                }
                                let identity = current_identity(&path);
                                match Reader::open(&path) {
                                    Ok(Some(reader)) => {
                                        devices.insert(path, reader);
                                        devices_changed = true;
                                    }
                                    Ok(None) => {
                                        if let Some(identity) = identity {
                                            ignored.insert(path, identity);
                                        }
                                    }
                                    Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                                        denied = true
                                    }
                                    Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                                    Err(error) => eprintln!("Cannot open input device: {error}"),
                                }
                            }
                        }
                        Err(e) => {
                            denied = e.kind() == io::ErrorKind::PermissionDenied;
                        }
                    }
                    if let Ok(mut state) = status.lock()
                        && !stop.load(Ordering::Relaxed)
                    {
                        update_device_status(&mut state, !devices.is_empty(), denied);
                    }
                }
                if devices_changed {
                    paths.clear();
                    fds.clear();
                    paths.extend(devices.keys().cloned());
                    fds.extend(devices.values().map(|reader| libc::pollfd {
                        fd: reader.device.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    }));
                    devices_changed = false;
                }
                if let Err(error) = wait_for_devices(
                    &mut fds,
                    Duration::from_secs(1)
                        .saturating_sub(scan.map_or(Duration::ZERO, |last| last.elapsed())),
                ) {
                    if error.kind() != io::ErrorKind::Interrupted {
                        eprintln!("Cannot wait for input devices: {error}");
                        thread::sleep(Duration::from_secs(1));
                    }
                    continue;
                }
                for (path, fd) in paths.iter().zip(&fds) {
                    if fd.revents == 0 {
                        continue;
                    }
                    if fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                        devices.remove(path);
                        devices_changed = true;
                        continue;
                    }
                    let keep = match devices.get_mut(path).unwrap().read(|input| {
                        if !storage_blocked.load(Ordering::Relaxed)
                            && !paused.load(Ordering::Relaxed)
                            && !stop.load(Ordering::Relaxed)
                            && tx.try_send(Command::Input(input)).is_err()
                        {
                            queue_failed(&stop, &status);
                        }
                    }) {
                        Ok(()) => true,
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => true,
                        Err(_) => false,
                    };
                    if !keep {
                        devices.remove(path);
                        devices_changed = true;
                    }
                }
            }
            if let Ok(mut state) = status.lock() {
                state.listening = false;
            }
        })?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_motion_is_ignored_and_chords_use_the_state_at_each_press() {
        let mut input = LinuxInput::default();
        let motion = [
            evdev::InputEvent::new(2, 0, 12),
            evdev::InputEvent::new(0, 0, 0),
        ];
        for _ in 0..2000 {
            forward_events(
                &mut input,
                &motion,
                |_| panic!("unexpected resync"),
                |_| panic!("motion is not a click"),
            )
            .unwrap();
        }
        let batch = [
            (1, 29, 1),
            (1, 46, 1),
            (1, 46, 0),
            (1, 29, 0),
            (1, 0x110, 1),
            (1, 0x110, 0),
        ]
        .map(|(kind, code, value)| evdev::InputEvent::new(kind, code, value));
        let mut delivered = Vec::new();
        forward_events(
            &mut input,
            &batch,
            |_| panic!("unexpected resync"),
            |event| delivered.push(event),
        )
        .unwrap();
        assert_eq!(delivered.len(), 3);
        assert_eq!(
            delivered[1].shortcut.unwrap().modifiers,
            crate::shortcuts::CONTROL
        );
        assert_eq!(delivered[2].device, crate::input::Device::Mouse);
        assert_eq!(delivered[2].code, 1);
        assert!(input.shortcut(delivered[1].code).is_none());

        let dropped = [(0, 3, 0), (1, 46, 1), (0, 0, 0), (1, 46, 1)]
            .map(|(kind, code, value)| evdev::InputEvent::new(kind, code, value));
        delivered.clear();
        let mut resyncs = 0;
        forward_events(
            &mut input,
            &dropped,
            |input| {
                resyncs += 1;
                input.synchronize([29]);
                Ok(())
            },
            |event| delivered.push(event),
        )
        .unwrap();
        assert_eq!(resyncs, 1);
        assert_eq!(delivered.len(), 1);
        assert_eq!(
            delivered[0].shortcut.unwrap().modifiers,
            crate::shortcuts::CONTROL
        );
    }

    #[test]
    fn permissions_use_one_banner_and_unrelated_denied_nodes_do_not_block_capture() {
        let mut status = CaptureStatus::default();
        update_device_status(&mut status, false, true);
        assert!(!status.listening);
        assert!(status.permission_needed);
        assert_eq!(status.error_code, None);
        assert_eq!(status.error, None);
        update_device_status(&mut status, true, true);
        assert!(status.listening);
        assert!(!status.permission_needed);
        assert_eq!(status.error, None);
        update_device_status(&mut status, false, false);
        assert!(!status.permission_needed);
        assert_eq!(status.error_code, Some("error.linux_devices"));
        update_device_status(&mut status, true, false);
        assert!(status.listening);
        assert_eq!(status.error, None);
    }

    #[test]
    fn idle_wait_sleeps_and_pending_input_wakes_immediately() {
        use std::io::Write;
        use std::os::unix::net::UnixStream;
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let mut fds = [libc::pollfd {
            fd: reader.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        }];
        let started = Instant::now();
        wait_for_devices(&mut fds, Duration::from_millis(100)).unwrap();
        assert!(started.elapsed() >= Duration::from_millis(80));
        assert_eq!(fds[0].revents, 0);
        writer.write_all(b"x").unwrap();
        wait_for_devices(&mut fds, Duration::from_secs(1)).unwrap();
        assert_ne!(fds[0].revents & libc::POLLIN, 0);
    }

    #[test]
    fn ignored_path_is_reconsidered_when_node_is_replaced() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("event1");
        let old = fs::File::create(&path).unwrap();
        let cached = current_identity(&path).unwrap();
        assert_eq!(current_identity(&path), Some(cached));
        fs::remove_file(&path).unwrap();
        assert_eq!(current_identity(&path), None);
        let _new = fs::File::create(&path).unwrap();
        assert_ne!(current_identity(&path), Some(cached));
        drop(old);
    }
    #[test]
    fn queue_failure_preserves_storage_failure() {
        let stop = AtomicBool::new(true);
        let status = Mutex::new(CaptureStatus {
            error_code: Some("error.storage"),
            ..Default::default()
        });
        queue_failed(&stop, &status);
        assert_eq!(status.lock().unwrap().error_code, Some("error.storage"));
    }
}
