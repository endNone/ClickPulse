//! Linux evdev codes mapped to the existing cross-platform physical identities.
//! Counts, heatmaps and shortcuts use physical US key positions on Linux.
//! No characters, layouts, cursor coordinates or window metadata are stored.
use crate::input::{Device, KeyTracker};

pub fn canonical_key(code: u16) -> u32 {
    match code {
        1..=83 | 87 | 88 => crate::windows_input::canonical_code(code),
        86 => 10, // ISO extra key
        96 => 76,
        97 => 62,
        98 => 75,
        99 => 0x10137,
        100 => 61,
        102 => 115,
        103 => 126,
        104 => 116,
        105 => 123,
        106 => 124,
        107 => 119,
        108 => 125,
        109 => 121,
        110 => 114,
        111 => 117,
        113 => 74,
        114 => 73,
        115 => 72,
        117 => 81,
        119 => 0x10245,
        121 => 95,
        124 => 93,
        125 => 55,
        126 => 54,
        127 | 139 => 63,
        183 => 105,
        184 => 107,
        185 => 113,
        186 => 106,
        187 => 64,
        188 => 79,
        189 => 80,
        190 => 90,
        // Unknown Linux keys have a separate namespace and retain their counts.
        other => 0x20000 + u32::from(other),
    }
}

pub fn identity(code: u16) -> Option<(Device, u32)> {
    match code {
        0 => None,
        0x110..=0x117 => Some((Device::Mouse, u32::from(code - 0x110) + 1)),
        // BTN_* ranges are gamepad, digitizer and touch state, not keys/clicks.
        1..=0xff | 0x160..=0x1ff | 0x230..=0x2bf => Some((Device::Keyboard, canonical_key(code))),
        _ => None,
    }
}

#[derive(Default)]
pub struct LinuxInput {
    held: KeyTracker,
    dropped: bool,
}
impl LinuxInput {
    pub fn shortcut(&self, key: u32) -> Option<crate::shortcuts::Chord> {
        if self.held.held_codes().any(|code| code == 100) {
            return None;
        }
        crate::shortcuts::Chord::new(
            key,
            crate::shortcuts::modifiers(
                self.held
                    .held_codes()
                    .map(|code| canonical_key(code as u16)),
            ),
            "linux",
        )
    }
    /// Seed held keys at attach/resynchronization without inventing presses.
    pub fn synchronize(&mut self, held: impl IntoIterator<Item = u16>) {
        self.held.reset();
        for code in held {
            self.held.press(u32::from(code), true);
        }
        self.dropped = false;
    }
    pub fn is_dropped(&self) -> bool {
        self.dropped
    }
    pub fn event(&mut self, kind: u16, code: u16, value: i32) -> Option<(Device, u32)> {
        if kind == 0 && code == 3 {
            self.dropped = true;
        }
        if self.dropped || kind != 1 {
            return None;
        }
        match value {
            0 => {
                self.held.release(u32::from(code));
                None
            }
            1 | 2 => self
                .held
                .press(u32::from(code), value == 2)
                .then(|| identity(code))
                .flatten(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcuts_use_state_at_press_not_after_batch_release() {
        let mut keys = LinuxInput::default();
        keys.event(1, 29, 1);
        keys.event(1, 46, 1);
        assert_eq!(keys.shortcut(8).unwrap().modifiers, 1);
        assert!(keys.event(1, 46, 2).is_none());
        keys.event(1, 29, 0);
        assert!(keys.shortcut(8).is_none());
        keys.event(1, 100, 1);
        assert!(keys.shortcut(8).is_none());
    }

    #[test]
    fn shifted_print_screen_keeps_the_linux_physical_mapping() {
        let mut keys = LinuxInput::default();
        keys.event(1, 42, 1);
        let (_, code) = keys.event(1, 99, 1).unwrap();
        let chord = keys.shortcut(code).unwrap();
        assert_eq!(chord.key, crate::shortcuts::PRINT_SCREEN);
        assert_eq!(chord.modifiers, crate::shortcuts::SHIFT);
    }

    #[test]
    fn physical_keys_repeat_modifiers_and_mouse_keep_distinct_counts() {
        let mut input = LinuxInput::default();
        assert_eq!(input.event(1, 30, 1), Some((Device::Keyboard, 0)));
        assert_eq!(input.event(1, 30, 2), None);
        assert_eq!(input.event(1, 30, 1), None);
        assert_eq!(input.event(1, 42, 1), Some((Device::Keyboard, 56)));
        assert_eq!(input.event(1, 30, 0), None);
        assert_eq!(input.event(1, 30, 1), Some((Device::Keyboard, 0)));
        for (raw, code) in [(0x110, 1), (0x111, 2), (0x112, 3), (0x113, 4), (0x114, 5)] {
            assert_eq!(input.event(1, raw, 1), Some((Device::Mouse, code)));
        }
        assert_eq!(input.event(2, 0, 20), None); // movement
        assert_eq!(input.event(3, 0, 20), None); // touch coordinates
        assert_eq!(input.event(1, 0x14a, 1), None); // BTN_TOUCH
        assert_eq!(input.event(1, 0x130, 1), None); // gamepad
    }
    #[test]
    fn attach_and_dropped_packets_do_not_create_counts() {
        let mut input = LinuxInput::default();
        input.synchronize([30]);
        assert_eq!(input.event(1, 30, 1), None);
        input.event(0, 3, 0);
        assert!(input.is_dropped());
        assert_eq!(input.event(1, 48, 1), None);
        input.synchronize([48]);
        assert_eq!(input.event(1, 48, 1), None);
        input.event(1, 48, 0);
        assert_eq!(input.event(1, 48, 1), Some((Device::Keyboard, 11)));
    }
    #[test]
    fn extended_keys_and_unknowns_do_not_collide() {
        assert_ne!(canonical_key(28), canonical_key(96));
        assert_ne!(canonical_key(29), canonical_key(97));
        assert_ne!(canonical_key(56), canonical_key(100));
        assert_ne!(canonical_key(75), canonical_key(105));
        assert_eq!(canonical_key(99), 0x10137);
        assert_eq!(canonical_key(119), 0x10245);
        assert!(canonical_key(202) >= 0x20000);
    }

    #[test]
    fn pc_system_keys_convert_evdev_codes_and_count_each_press_once() {
        let mut input = LinuxInput::default();
        for (evdev, physical) in [(99, 0x10137), (70, 0x10046), (119, 0x10245)] {
            for _ in 0..10 {
                assert_eq!(input.event(1, evdev, 1), Some((Device::Keyboard, physical)));
                assert_eq!(input.event(1, evdev, 2), None);
                assert_eq!(input.event(1, evdev, 1), None);
                assert_eq!(input.event(1, evdev, 0), None);
            }
        }
    }
}
