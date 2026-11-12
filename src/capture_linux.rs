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
use std::os::unix::fs::OpenOptionsExt;
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
}
impl Reader {
    fn open(path: &PathBuf) -> io::Result<Option<Self>> {
        // RawDevice::open may request write access. This collector needs only read.
        let device = RawDevice::from_fd(
            fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(path)?
                .into(),
        )?;
        if !device
            .supported_keys()
            .is_some_and(|keys| keys.iter().any(|key| identity(key.code()).is_some()))
        {
            return Ok(None);
        }
        let mut input = LinuxInput::default();
        input.synchronize(device.get_key_state()?.iter().map(|k| k.code()));
        Ok(Some(Self { device, input }))
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
    Ok(thread::Builder::new()
        .name("clickpulse-input".into())
        .spawn(move || {
            let mut devices = BTreeMap::<PathBuf, Reader>::new();
            let mut scan = Instant::now() - Duration::from_secs(2);
            while !stop.load(Ordering::Relaxed) {
                if scan.elapsed() >= Duration::from_secs(1) {
                    scan = Instant::now();
                    devices.retain(|path, _| path.exists());
                    let mut denied = false;
                    let mut failed = false;
                    match fs::read_dir("/dev/input") {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                if !entry.file_name().to_string_lossy().starts_with("event") {
                                    continue;
                                }
                                let path = entry.path();
                                if devices.contains_key(&path) {
                                    continue;
                                }
                                match Reader::open(&path) {
                                    Ok(Some(reader)) => {
                                        devices.insert(path, reader);
                                    }
                                    Ok(None) => {}
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
                devices.retain(|_, reader| match reader.read() {
                    Ok(events) => {
                        for (device, code) in events {
                            if !paused.load(Ordering::Relaxed) && !stop.load(Ordering::Relaxed) {
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
                });
                thread::sleep(Duration::from_millis(8));
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
