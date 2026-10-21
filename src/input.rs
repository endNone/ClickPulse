use std::collections::HashSet;

use crossbeam_channel::TrySendError;
use serde::{Deserialize, Serialize};

pub(crate) fn input_queue_closed<T>(result: &Result<(), TrySendError<T>>) -> bool {
    matches!(result, Err(TrySendError::Disconnected(_)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(i64)]
pub enum Device {
    Mouse = 0,
    Keyboard = 1,
}

#[derive(Clone, Debug)]
pub struct InputEvent {
    pub ts_ms: i64,
    pub device: Device,
    /// Physical OS key code, never a character value or a process-randomized hash.
    pub code: u32,
}

#[derive(Default)]
pub struct KeyTracker {
    held: HashSet<u32>,
}

impl KeyTracker {
    pub fn press(&mut self, code: u32, repeat: bool) -> bool {
        if repeat {
            // Synchronize when capture starts while a key is already held.
            self.held.insert(code);
            return false;
        }
        self.held.insert(code)
    }

    pub fn release(&mut self, code: u32) {
        self.held.remove(&code);
    }

    pub fn reset(&mut self) {
        self.held.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_queue_only_stops_capture_when_disconnected() {
        let (tx, rx) = crossbeam_channel::bounded(1);
        assert!(!input_queue_closed(&tx.try_send(())));
        assert!(!input_queue_closed(&tx.try_send(())));
        drop(rx);
        assert!(input_queue_closed(&tx.try_send(())));
    }

    #[test]
    fn held_key_and_repeat_count_once_until_release() {
        let mut keys = KeyTracker::default();
        assert!(keys.press(0, false));
        assert!(!keys.press(0, true));
        assert!(!keys.press(0, false));
        keys.release(0);
        assert!(keys.press(0, false));
    }

    #[test]
    fn separate_physical_keys_and_changed_modifiers_do_not_collide() {
        let mut keys = KeyTracker::default();
        assert!(keys.press(0, false)); // macOS A
        assert!(keys.press(97, false)); // macOS F6
        keys.release(0); // same physical code even if Shift changed
        assert!(keys.press(0, false));
        assert!(!keys.press(97, false));
    }
}
