//! Event codes for counts and heatmaps are physical. On macOS and Windows, only an
//! eligible shortcut carries a layout-resolved key ID. No typed characters are stored.
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[repr(i64)]
pub enum Device {
    Mouse = 0,
    Keyboard = 1,
}

#[derive(Clone, Debug)]
pub struct InputEvent {
    pub shortcut: Option<crate::shortcuts::Chord>,
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
    pub fn held_codes(&self) -> impl Iterator<Item = u32> + '_ {
        self.held.iter().copied()
    }

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

    /// FlagsChanged carries the state at the event timestamp. A later global
    /// key-state query can already describe the release or a different device.
    /// Device masks are defined by Apple's IOKit/hidsystem/IOLLEvent.h.
    pub fn mac_modifier(&mut self, code: u32, flags: u64) -> bool {
        if code == 57 {
            return false;
        } // Independent physical Caps Lock stream.
        let mask = match code {
            56 => Some(0x2),
            60 => Some(0x4),
            59 => Some(0x1),
            62 => Some(0x2000),
            58 => Some(0x20),
            61 => Some(0x40),
            55 => Some(0x8),
            54 => Some(0x10),
            63 => Some(0x800000),
            _ => None,
        };
        let down = mask.map_or_else(|| !self.held.contains(&code), |mask| flags & mask != 0);
        if down {
            self.press(code, false)
        } else {
            self.release(code);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mac_modifiers_count_both_sides_once_even_while_the_other_side_is_held() {
        for (left, right, lflag, rflag) in [
            (56, 60, 2, 4),
            (59, 62, 1, 0x2000),
            (58, 61, 0x20, 0x40),
            (55, 54, 8, 0x10),
        ] {
            let mut tracker = KeyTracker::default();
            for _ in 0..10 {
                assert!(tracker.mac_modifier(left, lflag));
                assert!(tracker.mac_modifier(right, lflag | rflag));
                assert!(!tracker.mac_modifier(left, rflag));
                assert!(!tracker.mac_modifier(right, 0));
            }
        }
        let mut tracker = KeyTracker::default();
        assert!(!tracker.mac_modifier(57, u64::MAX));
        assert!(tracker.mac_modifier(63, 0x800000));
        assert!(!tracker.mac_modifier(63, 0));
        assert!(tracker.mac_modifier(999, 0));
        assert!(!tracker.mac_modifier(999, 0));
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
