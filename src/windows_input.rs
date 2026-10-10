//! Normalize physical PC scan codes to the shared keyboard identities. This
//! never converts input to characters and is testable without native capture.
use std::collections::HashSet;

#[derive(Default)]
pub struct WindowsKeyboard {
    held: HashSet<(usize, u16)>,
}
impl WindowsKeyboard {
    pub fn event(&mut self, device: usize, make: u16, flags: u16, vk: u16) -> Option<u32> {
        if vk == 255 || make == 0xff {
            return None;
        }
        let identity = make | ((flags & 6) << 7);
        let identity = if make == 0 { 0x8000 | vk } else { identity };
        if flags & 1 != 0 {
            self.held.remove(&(device, identity));
            None
        } else if self.held.insert((device, identity)) {
            Some(canonical_code(identity))
        } else {
            None // Repeated make packets during a hold count once.
        }
    }
    pub fn remove_device(&mut self, device: usize) {
        self.held.retain(|(held, _)| *held != device);
    }
}

pub fn canonical_code(scan: u16) -> u32 {
    match scan {
        0x01 => 53,
        0x02 => 18,
        0x03 => 19,
        0x04 => 20,
        0x05 => 21,
        0x06 => 23,
        0x07 => 22,
        0x08 => 26,
        0x09 => 28,
        0x0a => 25,
        0x0b => 29,
        0x0c => 27,
        0x0d => 24,
        0x0e => 51,
        0x0f => 48,
        0x10 => 12,
        0x11 => 13,
        0x12 => 14,
        0x13 => 15,
        0x14 => 17,
        0x15 => 16,
        0x16 => 32,
        0x17 => 34,
        0x18 => 31,
        0x19 => 35,
        0x1a => 33,
        0x1b => 30,
        0x1c => 36,
        0x1d => 59,
        0x1e => 0,
        0x1f => 1,
        0x20 => 2,
        0x21 => 3,
        0x22 => 5,
        0x23 => 4,
        0x24 => 38,
        0x25 => 40,
        0x26 => 37,
        0x27 => 41,
        0x28 => 39,
        0x29 => 50,
        0x2a => 56,
        0x2b => 42,
        0x2c => 6,
        0x2d => 7,
        0x2e => 8,
        0x2f => 9,
        0x30 => 11,
        0x31 => 45,
        0x32 => 46,
        0x33 => 43,
        0x34 => 47,
        0x35 => 44,
        0x36 => 60,
        0x37 => 67,
        0x38 => 58,
        0x39 => 49,
        0x3a => 57,
        0x3b => 122,
        0x3c => 120,
        0x3d => 99,
        0x3e => 118,
        0x3f => 96,
        0x40 => 97,
        0x41 => 98,
        0x42 => 100,
        0x43 => 101,
        0x44 => 109,
        0x45 => 71,
        0x46 => 0x10046,
        0x47 => 89,
        0x48 => 91,
        0x49 => 92,
        0x4a => 78,
        0x4b => 86,
        0x4c => 87,
        0x4d => 88,
        0x4e => 69,
        0x4f => 83,
        0x50 => 84,
        0x51 => 85,
        0x52 => 82,
        0x53 => 65,
        0x57 => 103,
        0x58 => 111,
        0x11c => 76,
        0x11d => 62,
        0x135 => 75,
        0x137 => 0x10137,
        0x138 => 61,
        0x147 => 115,
        0x148 => 126,
        0x149 => 116,
        0x14b => 123,
        0x14d => 124,
        0x14f => 119,
        0x150 => 125,
        0x151 => 121,
        0x152 => 114,
        0x153 => 117,
        0x15b => 55,
        0x15c => 54,
        0x15d => 63,
        0x245 => 0x10245,
        other => 0x10000 + u32::from(other),
    }
}

pub fn mouse_buttons(flags: u16) -> impl Iterator<Item = u32> {
    [(1, 1), (4, 2), (16, 3), (64, 4), (256, 5)]
        .into_iter()
        .filter_map(move |(flag, code)| (flags & flag != 0).then_some(code))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeats_shifted_releases_and_distinct_devices_keep_physical_identity() {
        let mut keyboard = WindowsKeyboard::default();
        assert_eq!(keyboard.event(1, 0x1e, 0, 0x41), Some(0));
        assert_eq!(keyboard.event(1, 0x1e, 0, 0x41), None);
        assert_eq!(keyboard.event(2, 0x1e, 0, 0x41), Some(0));
        assert_eq!(keyboard.event(1, 0x1e, 1, 0x41), None);
        assert_eq!(keyboard.event(1, 0x1e, 0, 0x41), Some(0));
        keyboard.remove_device(2);
        assert_eq!(keyboard.event(2, 0x1e, 0, 0x41), Some(0));
    }
    #[test]
    fn extended_modifiers_arrows_and_keypad_do_not_collide() {
        let mut keyboard = WindowsKeyboard::default();
        assert_eq!(keyboard.event(1, 0x1d, 0, 0x11), Some(59));
        assert_eq!(keyboard.event(1, 0x1d, 2, 0x11), Some(62));
        assert_eq!(keyboard.event(1, 0x38, 0, 0x12), Some(58));
        assert_eq!(keyboard.event(1, 0x38, 2, 0x12), Some(61));
        assert_eq!(keyboard.event(1, 0x4b, 0, 0x25), Some(86));
        assert_eq!(keyboard.event(1, 0x4b, 2, 0x25), Some(123));
        assert_eq!(keyboard.event(1, 0x1c, 0, 0x0d), Some(36));
        assert_eq!(keyboard.event(1, 0x1c, 2, 0x0d), Some(76));
    }
    #[test]
    fn overrun_packets_are_ignored_and_unknown_keys_remain_disjoint() {
        let mut keyboard = WindowsKeyboard::default();
        assert_eq!(keyboard.event(1, 0xff, 0, 255), None);
        assert!(keyboard.event(1, 0x70, 0, 0x15).unwrap() > 0xffff);
        assert!(keyboard.event(1, 0, 0, 0xad).unwrap() > 0xffff);
    }
    #[test]
    fn mouse_counts_press_flags_without_releases_movement_or_wheel() {
        assert_eq!(
            mouse_buttons(1 | 4 | 16 | 64 | 256).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            mouse_buttons(2 | 8 | 32 | 128 | 512 | 1024 | 2048).count(),
            0
        );
    }
}
