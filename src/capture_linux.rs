//! Passive read-only evdev capture for X11 and Wayland. Never grabs devices.
use crate::{
    input::InputEvent,
    linux_input::{LinuxInput, identity},
    runtime::{CaptureStatus, Command},
};
use anyhow::Result;
use chrono::Local;
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
        }))
    }
    fn read(&mut self) -> io::Result<Vec<(crate::input::Device, u32)>> {
        let events = match self.device.fetch_events() {
            Ok(events) => events.collect::<Vec<_>>(),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut result = Vec::new();
        for event in events {
            if self.input.is_dropped() && event.event_type().0 == 0 && event.code() == 0 {
                self.input
                    .synchronize(self.device.get_key_state()?.iter().map(|k| k.code()));
            } else if let Some(input) =
                self.input
                    .event(event.event_type().0, event.code(), event.value())
            {
                result.push(input);
            }
        }
        Ok(result)
    }
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
            let mut scan = Instant::now() - Duration::from_secs(2);
            while !stop.load(Ordering::Relaxed) {
                if scan.elapsed() >= Duration::from_secs(1) {
                    scan = Instant::now();
                    devices.retain(|path, reader| current_identity(path) == Some(reader.identity));
                    ignored.retain(|path, identity| current_identity(path) == Some(*identity));
                    let mut denied = false;
                    let mut failed = false;
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
                                    Err(_) => failed = true,
                                }
                            }
                        }
                        Err(e) => {
                            denied = e.kind() == io::ErrorKind::PermissionDenied;
                            failed = !denied;
                        }
                    }
                    if let Ok(mut state) = status.lock()
                        && !stop.load(Ordering::Relaxed)
                    {
                        state.listening = !devices.is_empty();
                        state.permission_needed = denied;
                        state.error_code = if denied {
                            Some("error.linux_permission")
                        } else if failed || devices.is_empty() {
                            Some("error.linux_devices")
                        } else {
                            None
                        };
                        state.error = state.error_code.map(|key| key.to_string());
                    }
                }
                let paths: Vec<_> = devices.keys().cloned().collect();
                let mut fds: Vec<_> = devices
                    .values()
                    .map(|reader| libc::pollfd {
                        fd: reader.device.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    })
                    .collect();
                if let Err(error) = wait_for_devices(
                    &mut fds,
                    Duration::from_secs(1).saturating_sub(scan.elapsed()),
                ) {
                    if error.kind() != io::ErrorKind::Interrupted {
                        eprintln!("Cannot wait for input devices: {error}");
                        thread::sleep(Duration::from_secs(1));
                    }
                    continue;
                }
                for (path, fd) in paths.into_iter().zip(fds) {
                    if fd.revents == 0 {
                        continue;
                    }
                    if fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                        devices.remove(&path);
                        continue;
                    }
                    let keep = match devices.get_mut(&path).unwrap().read() {
                        Ok(events) => {
                            for (device, code) in events {
                                if !storage_blocked.load(Ordering::Relaxed)
                                    && !paused.load(Ordering::Relaxed)
                                    && !stop.load(Ordering::Relaxed)
                                {
                                    let input = InputEvent {
                                        ts_ms: Local::now().timestamp_millis(),
                                        device,
                                        code,
                                    };
                                    if tx.try_send(Command::Input(input)).is_err() {
                                        queue_failed(&stop, &status);
                                        break;
                                    }
                                }
                            }
                            true
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => true,
                        Err(_) => false,
                    };
                    if !keep {
                        devices.remove(&path);
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
