//! Counts and heatmaps use physical scan codes. Only eligible shortcuts resolve a
//! logical key through the foreground layout. No characters or layouts are saved.
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

// RAWKEYBOARD flags and virtual keys, kept platform independent so packet
// normalization can be tested without starting a Windows input listener.
const RI_KEY_BREAK: u16 = 1;
const VK_SHIFT: u16 = 0x10;
const VK_PAUSE: u16 = 0x13;
const VK_NUMLOCK: u16 = 0x90;
const MODIFIER_GRACE: Duration = Duration::from_millis(1500);

/// Resolve a packet without access to the layout's OEM character table.
pub fn shortcut_character(
    make: u16,
    flags: u16,
    packet_vk: u16,
    lookup: impl FnOnce(u32) -> Option<u16>,
) -> Option<char> {
    shortcut_character_with_oem(make, flags, packet_vk, lookup, |_| None)
}

/// Use the same foreground layout for both scan -> VK and OEM VK -> unshifted character.
/// None from oem_character means there is no layout; a Windows result of zero or a
/// dead key is an explicit lookup failure and falls back to the physical position.
pub fn shortcut_character_with_oem(
    make: u16,
    flags: u16,
    packet_vk: u16,
    lookup: impl FnOnce(u32) -> Option<u16>,
    oem_character: impl FnOnce(u16) -> Option<u32>,
) -> Option<char> {
    let prefix = if flags & 2 != 0 {
        0xe000
    } else if flags & 4 != 0 {
        0xe100
    } else {
        0
    };
    let mapped = (make != 0)
        .then(|| lookup(prefix | u32::from(make)))
        .flatten();
    let vk = mapped.filter(|vk| *vk != 0).unwrap_or(packet_vk);
    match vk {
        0x41..=0x5a | 0x30..=0x39 => char::from_u32(u32::from(vk)),
        0xba..=0xc0 | 0xdb..=0xdf | 0xe2 => {
            if let Some(value) = oem_character(vk) {
                // MAPVK_VK_TO_CHAR sets bit 31 for dead keys. Never strip it.
                return char::from_u32(value)
                    .filter(|&ch| crate::shortcuts::us_character_key(ch).is_some());
            }
            match vk {
                0xbd => Some('-'),
                0xbb => Some('='),
                0xbc => Some(','),
                0xbe => Some('.'),
                _ => None,
            }
        }
        _ => None,
    }
}

fn modifier_vk(key: u32) -> Option<u16> {
    match key {
        59 => Some(0xa2), // Left Control
        62 => Some(0xa3), // Right Control
        58 => Some(0xa4), // Left Alt
        61 => Some(0xa5), // Right Alt / AltGr
        56 => Some(0xa0), // Left Shift
        60 => Some(0xa1), // Right Shift
        55 => Some(0x5b), // Left Windows
        54 => Some(0x5c), // Right Windows
        _ => None,
    }
}

#[derive(Default)]
pub struct WindowsKeyboard {
    // Last make time, refreshed by repeats even when they do not add a count.
    held: HashMap<(usize, u16), Instant>,
    virtual_keys: HashMap<(usize, u16), u16>,
}
impl WindowsKeyboard {
    pub fn reset(&mut self) {
        self.held.clear();
        self.virtual_keys.clear();
    }

    pub fn reconcile_foreground(&mut self, mut is_down: impl FnMut(u16) -> bool) {
        self.held.retain(|identity, _| {
            let vk = modifier_vk(canonical_code(identity.1))
                .or_else(|| self.virtual_keys.get(identity).copied());
            vk.is_some_and(&mut is_down)
        });
        self.virtual_keys
            .retain(|identity, _| self.held.contains_key(identity));
    }

    pub fn shortcut(
        &mut self,
        device: usize,
        key: u32,
        now: Instant,
        is_down: impl FnMut(u16) -> bool,
    ) -> Option<crate::shortcuts::Chord> {
        self.shortcut_with_layout(device, key, now, is_down, |_| None)
    }

    /// Preserve Raw Input packet order for recent modifiers: a fast chord may
    /// already be physically released when its queued packets are processed.
    /// Only reconcile older state after a grace period; both time and the live
    /// query are injected so boundary cases do not require a Windows desktop.
    pub fn shortcut_with_layout(
        &mut self,
        device: usize,
        key: u32,
        now: Instant,
        mut is_down: impl FnMut(u16) -> bool,
        translate: impl FnOnce(u32) -> Option<char>,
    ) -> Option<crate::shortcuts::Chord> {
        if modifier_vk(key).is_some() {
            return None;
        }
        self.held.retain(|(held_device, scan), last_make| {
            (*held_device == 0) != (device == 0)
                || now.saturating_duration_since(*last_make) <= MODIFIER_GRACE
                || modifier_vk(canonical_code(*scan)).is_none_or(&mut is_down)
        });
        let keys = self
            .held
            .keys()
            .filter(|(d, _)| (*d == 0) == (device == 0))
            .map(|(_, scan)| canonical_code(*scan));
        // Right Alt can produce AltGr characters. Do not classify these as shortcuts.
        if keys.clone().any(|key| key == 61) {
            return None;
        }
        crate::shortcuts::Chord::with_layout(
            key,
            crate::shortcuts::modifiers(keys),
            "windows",
            translate,
        )
    }
    /// Missing keyboard device identity is treated as software input only when
    /// opted in. Never apply this rule to mouse packets: Precision Touchpads
    /// legitimately use a null device handle too.
    pub fn filtered_event(
        &mut self,
        device: usize,
        make: u16,
        flags: u16,
        vk: u16,
        ignore: bool,
    ) -> Option<u32> {
        let event = self.event(device, make, flags, vk);
        if ignore && device == 0 { None } else { event }
    }
    pub fn event(&mut self, device: usize, make: u16, flags: u16, vk: u16) -> Option<u32> {
        self.event_at(device, make, flags, vk, Instant::now())
    }

    fn event_at(
        &mut self,
        device: usize,
        make: u16,
        flags: u16,
        vk: u16,
        now: Instant,
    ) -> Option<u32> {
        if vk >= 255 || make == 0xff {
            return None;
        }
        let identity = make | ((flags & 6) << 7);
        // PrintScreen and Pause can include fake Shift/Ctrl prefix packets.
        // These are parts of one physical key, not additional key presses.
        if matches!(identity, 0x12a | 0x21d) {
            return None;
        }
        // Shift reverses NumLock. Windows can report the resulting synthetic
        // Shift transitions using the *numpad* scan code. Tracking those packets
        // would count a phantom numpad press and clear a genuinely held key.
        if vk == VK_SHIFT && matches!(identity, 0x47..=0x49 | 0x4b..=0x4d | 0x4f..=0x53) {
            return None;
        }
        if vk == VK_PAUSE {
            // Pause is not an auto-repeating key and may have no separate break
            // packet. Do not leave it held forever or confuse it with NumLock,
            // which can have the same 0x45 make code. A release can also end a
            // Ctrl+Pause press after the modifier changes.
            if flags & RI_KEY_BREAK != 0 {
                self.held.remove(&(device, 0x245));
                return None;
            }
            return (!self.held.contains_key(&(device, 0x245))).then_some(canonical_code(0x245));
        }
        let identity = if vk == VK_NUMLOCK {
            // Drivers differ on the E0 flag for NumLock; use one identity for
            // both transitions and retain the existing stored NumLock code.
            0x45
        } else if make == 0 {
            0x8000 | vk
        } else {
            // Alt+PrintScreen and Ctrl+Pause have different scan codes from
            // their unmodified keys. Normalize before tracking releases so
            // changing a modifier cannot leave the physical key held forever.
            match identity {
                0x54 => 0x137,
                0x146 => 0x245,
                _ => identity,
            }
        };
        if flags & RI_KEY_BREAK == 0 {
            self.virtual_keys.insert((device, identity), vk);
        } else {
            self.virtual_keys.remove(&(device, identity));
        }
        if flags & RI_KEY_BREAK != 0 {
            let was_held = self.held.remove(&(device, identity)).is_some();
            // Screenshot tools and Windows capture hooks can swallow PrintScreen's
            // make packet and deliver only the break packet. Count that orphan
            // release once without double-counting normal make+break presses.
            if identity == 0x137 && !was_held {
                Some(canonical_code(identity))
            } else {
                None
            }
        } else if self.held.insert((device, identity), now).is_none() {
            Some(canonical_code(identity))
        } else {
            None // Repeated make packets during a hold count once.
        }
    }
    pub fn remove_device(&mut self, device: usize) {
        self.held.retain(|(held, _), _| *held != device);
        self.virtual_keys.retain(|(held, _), _| *held != device);
    }
}

pub fn canonical_code(scan: u16) -> u32 {
    match scan {
        0x56 => 10,
        0x120 => 74,
        0x12e => 73,
        0x130 => 72,
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
        0x64 => 105, // F13
        0x65 => 107,
        0x66 => 113,
        0x67 => 106,
        0x68 => 64,
        0x69 => 79,
        0x6a => 80,
        0x6b => 90, // F20
        0x11c => 76,
        0x11d => 62,
        0x135 => 75,
        0x54 | 0x137 => 0x10137,
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
        0x146 | 0x245 => 0x10245,
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
    fn foreground_reconciliation_preserves_held_chords_and_repeat_suppression() {
        let mut keys = WindowsKeyboard::default();
        keys.event(1, 0x38, 0, 0x12); // Alt
        keys.event(1, 0x2e, 0, 0x43); // C, held through the focus change
        keys.event(1, 0x1d, 0, 0x11); // Ctrl, release lost in another process
        keys.reconcile_foreground(|vk| matches!(vk, 0xa4 | 0x43));
        assert_eq!(keys.held.len(), 2);
        assert!(keys.event(1, 0x2e, 0, 0x43).is_none());
        let tab = keys.event(1, 0x0f, 0, 0x09).unwrap();
        assert_eq!(
            keys.shortcut(1, tab, Instant::now(), |_| true)
                .unwrap()
                .modifiers,
            crate::shortcuts::ALT
        );
        keys.reset();
        assert!(keys.held.is_empty());
        assert!(keys.virtual_keys.is_empty());
    }

    #[test]
    fn extended_function_keys_have_shared_identities() {
        for (scan, code) in (0x64..=0x6b).zip([105, 107, 113, 106, 64, 79, 80, 90]) {
            assert_eq!(canonical_code(scan), code);
        }
    }

    #[test]
    fn recent_control_c_never_queries_live_state_even_at_grace_boundary() {
        let start = Instant::now();
        for age in [0, 100, 1499, 1500] {
            for device in [0, 1] {
                let mut keys = WindowsKeyboard::default();
                keys.event_at(device, 0x1d, 0, 0x11, start);
                let now = start + Duration::from_millis(age);
                let c = keys.event_at(device, 0x2e, 0, 0x43, now).unwrap();
                // Ctrl is physically up already, but its break packet is still queued.
                let mut queries = 0;
                let chord = keys.shortcut(device, c, now, |_| {
                    queries += 1;
                    false
                });
                assert_eq!(chord.unwrap().modifiers, crate::shortcuts::CONTROL);
                assert_eq!(queries, 0, "live state queried at {age} ms");
            }
        }
    }

    #[test]
    fn control_held_for_three_seconds_still_forms_paste() {
        let start = Instant::now();
        let mut keys = WindowsKeyboard::default();
        keys.event_at(1, 0x1d, 0, 0x11, start);
        let now = start + Duration::from_secs(3);
        let v = keys.event_at(1, 0x2f, 0, 0x56, now).unwrap();
        let mut queries = Vec::new();
        let chord = keys.shortcut(1, v, now, |vk| {
            queries.push(vk);
            true
        });
        assert_eq!(queries, vec![0xa2]);
        assert_eq!(chord.unwrap().modifiers, crate::shortcuts::CONTROL);
        assert!(keys.held.contains_key(&(1, 0x1d)));
    }

    #[test]
    fn stale_released_alt_is_removed_before_forming_a_chord() {
        let start = Instant::now();
        let mut keys = WindowsKeyboard::default();
        keys.event_at(1, 0x38, 0, 0x12, start);
        let now = start + Duration::from_millis(1501);
        let c = keys.event_at(1, 0x2e, 0, 0x43, now).unwrap();
        let mut queries = Vec::new();
        assert!(
            keys.shortcut(1, c, now, |vk| {
                queries.push(vk);
                false
            })
            .is_none()
        );
        assert_eq!(queries, vec![0xa4]);
        assert!(!keys.held.contains_key(&(1, 0x38)));
        assert!(
            keys.held.contains_key(&(1, 0x2e)),
            "ordinary held keys remain tracked"
        );
        // A new physical Alt press is counted once after stale state is removed.
        assert_eq!(keys.event_at(1, 0x38, 0, 0x12, now), Some(58));
        assert_eq!(keys.event_at(1, 0x38, 0, 0x12, now), None);
        assert_eq!(
            keys.shortcut(1, c, now, |_| panic!("recent modifier"))
                .unwrap()
                .modifiers,
            crate::shortcuts::ALT
        );
    }

    #[test]
    fn repeated_modifier_make_refreshes_grace_without_adding_a_press() {
        let start = Instant::now();
        let mut keys = WindowsKeyboard::default();
        assert_eq!(keys.event_at(1, 0x1d, 0, 0x11, start), Some(59));
        let repeated = start + Duration::from_millis(2900);
        assert_eq!(keys.event_at(1, 0x1d, 0, 0x11, repeated), None);
        let now = start + Duration::from_secs(3);
        let c = keys.event_at(1, 0x2e, 0, 0x43, now).unwrap();
        assert_eq!(
            keys.shortcut(1, c, now, |_| panic!("repeat refreshed grace"))
                .unwrap()
                .modifiers,
            crate::shortcuts::CONTROL
        );
    }

    #[test]
    fn reconciliation_checks_each_side_and_skips_modifier_only_presses() {
        let start = Instant::now();
        let now = start + Duration::from_secs(3);
        for (make, flags, vk, state_vk) in [
            (0x1d, 0, 0x11, 0xa2),
            (0x1d, 2, 0x11, 0xa3),
            (0x38, 0, 0x12, 0xa4),
            (0x38, 2, 0x12, 0xa5),
            (0x2a, 0, 0x10, 0xa0),
            (0x36, 0, 0x10, 0xa1),
            (0x5b, 2, 0x5b, 0x5b),
            (0x5c, 2, 0x5c, 0x5c),
        ] {
            let mut keys = WindowsKeyboard::default();
            let modifier = keys.event_at(1, make, flags, vk, start).unwrap();
            assert!(
                keys.shortcut(1, modifier, now, |_| panic!("modifier-only packet"))
                    .is_none()
            );
            let tab = keys.event_at(1, 0x0f, 0, 0x09, now).unwrap();
            let mut queried = Vec::new();
            assert!(
                keys.shortcut(1, tab, now, |key| {
                    queried.push(key);
                    false
                })
                .is_none()
            );
            assert_eq!(queried, vec![state_vk]);
            assert_eq!(keys.held.len(), 1);
        }
    }

    #[test]
    fn stale_checks_keep_software_and_physical_input_domains_separate() {
        let start = Instant::now();
        let now = start + Duration::from_secs(3);
        let mut keys = WindowsKeyboard::default();
        keys.event_at(0, 0x38, 0, 0x12, start);
        let c = keys.event_at(1, 0x2e, 0, 0x43, now).unwrap();
        assert!(
            keys.shortcut(1, c, now, |_| panic!("unrelated device domain"))
                .is_none()
        );
        assert!(keys.held.contains_key(&(0, 0x38)));
        assert!(keys.shortcut(0, c, now, |_| false).is_none());
        assert!(!keys.held.contains_key(&(0, 0x38)));
    }

    #[test]
    fn foreground_discontinuity_discards_swallowed_releases() {
        let mut keys = WindowsKeyboard::default();
        for (scan, vk) in [(0x1d, 0x11), (0x38, 0x12)] {
            keys.event(1, scan, 0, vk);
        }
        // Foreground changed into an elevated window; its break packets are absent.
        keys.reset();
        for (scan, vk) in [(0x2e, 0x43), (0x1e, 0x41)] {
            let key = keys.event(1, scan, 0, vk).unwrap();
            assert!(keys.shortcut(1, key, Instant::now(), |_| true).is_none());
            keys.event(1, scan, RI_KEY_BREAK, vk);
        }
        // The next physical modifier/chord is counted normally, once.
        assert_eq!(keys.event(1, 0x1d, 0, 0x11), Some(59));
        assert_eq!(keys.event(1, 0x1d, 0, 0x11), None);
        let c = keys.event(1, 0x2e, 0, 0x43).unwrap();
        assert_eq!(
            keys.shortcut(1, c, Instant::now(), |_| true)
                .unwrap()
                .modifiers,
            crate::shortcuts::CONTROL
        );
    }
    #[test]
    fn queued_fast_chord_keeps_modifiers_until_its_break_packet() {
        for device in [0, 1] {
            let mut keys = WindowsKeyboard::default();
            keys.event(device, 0x1d, 0, 0x11);
            let c = keys.event(device, 0x2e, 0, 0x43).unwrap();
            // No live-state lookup: the physical chord may already be released.
            assert_eq!(
                keys.shortcut(device, c, Instant::now(), |_| true)
                    .unwrap()
                    .modifiers,
                1
            );
            keys.event(device, 0x2e, RI_KEY_BREAK, 0x43);
            keys.event(device, 0x1d, RI_KEY_BREAK, 0x11);
            assert!(keys.shortcut(device, c, Instant::now(), |_| true).is_none());
        }
    }
    #[test]
    fn desktop_and_session_reset_remove_swallowed_modifier_releases() {
        for (make, flags, vk) in [(0x5b, 2, 0x5b), (0x1d, 0, 0xa2), (0x38, 0, 0xa4)] {
            let mut keys = WindowsKeyboard::default();
            keys.event(1, make, flags, vk);
            keys.reset();
            assert_eq!(keys.event(1, 0x2e, 0, 0x43), Some(8));
            assert!(keys.shortcut(1, 8, Instant::now(), |_| true).is_none());
            assert!(keys.event(1, make, flags, vk).is_some());
            assert!(keys.event(1, make, flags, vk).is_none());
        }
    }
    #[test]
    fn shortcuts_follow_modifiers_releases_and_ignore_altgr() {
        let mut keys = WindowsKeyboard::default();
        keys.event(1, 0x1d, 0, 0x11);
        assert_eq!(keys.event(1, 0x2e, 0, 0x43), Some(8));
        assert_eq!(
            keys.shortcut(1, 8, Instant::now(), |_| true)
                .unwrap()
                .modifiers,
            1
        );
        assert!(keys.event(1, 0x2e, 0, 0x43).is_none());
        assert!(keys.shortcut(0, 8, Instant::now(), |_| true).is_none());
        keys.event(1, 0x1d, RI_KEY_BREAK, 0x11);
        assert!(keys.shortcut(1, 8, Instant::now(), |_| true).is_none());
        keys.event(1, 0x1d, 0, 0x11);
        keys.event(1, 0x38, 2, 0x12);
        assert!(keys.shortcut(1, 8, Instant::now(), |_| true).is_none());
    }

    #[test]
    fn alternate_print_screen_packet_keeps_its_alt_chord() {
        let mut keys = WindowsKeyboard::default();
        keys.event(1, 0x38, 0, 0x12);
        let code = keys.event(1, 0x54, 0, 0x2c).unwrap();
        let chord = keys.shortcut(1, code, Instant::now(), |_| true).unwrap();
        assert_eq!(chord.key, crate::shortcuts::PRINT_SCREEN);
        assert_eq!(chord.modifiers, crate::shortcuts::ALT);
    }

    #[test]
    fn software_filter_is_optional_and_preserves_physical_keys_and_release_state() {
        let mut keys = WindowsKeyboard::default();
        assert!(keys.filtered_event(0, 0x1e, 0, 0x41, false).is_some());
        assert!(
            keys.filtered_event(0, 0x1e, RI_KEY_BREAK, 0x41, true)
                .is_none()
        );
        assert!(keys.filtered_event(0, 0x1e, 0, 0x41, true).is_none());
        assert!(keys.filtered_event(10, 0x1e, 0, 0x41, true).is_some());
        // Turning the filter off during a hold does not turn repeats into presses.
        assert!(keys.filtered_event(0, 0x1e, 0, 0x41, false).is_none());
        keys.filtered_event(0, 0x1e, RI_KEY_BREAK, 0x41, false);
        assert!(keys.filtered_event(0, 0x1e, 0, 0x41, false).is_some());
    }
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
    fn shifted_numpad_packets_count_only_the_physical_presses() {
        let mut keyboard = WindowsKeyboard::default();
        let mut counts = Vec::new();
        // Hold Shift and tap Numpad1 twice with NumLock on. Each tap is wrapped
        // in fake Shift transitions whose scan code misleadingly says Numpad1.
        let packets = [
            (0x2a, 0, VK_SHIFT),
            (0x4f, 1, VK_SHIFT),
            (0x4f, 0, 0x23),
            (0x4f, 0, 0x23), // auto-repeat
            (0x4f, 1, 0x23),
            (0x4f, 0, VK_SHIFT),
            (0x4f, 1, VK_SHIFT),
            (0x4f, 0, 0x23),
            (0x4f, 1, 0x23),
            (0x4f, 0, VK_SHIFT),
            (0x2a, 1, VK_SHIFT),
            (0x4f, 0, 0x61), // physical Numpad1 again, after releasing Shift
            (0x4f, 1, 0x61),
        ];
        for (make, flags, vk) in packets {
            counts.extend(keyboard.event(1, make, flags, vk));
        }
        assert_eq!(counts, vec![56, 83, 83, 83]);
    }
    #[test]
    fn fake_shift_packets_do_not_release_a_held_numpad_key() {
        let mut keyboard = WindowsKeyboard::default();
        for (make, code) in [(0x47, 89), (0x4c, 87), (0x53, 65)] {
            assert_eq!(keyboard.event(1, make, 0, 0x60), Some(code));
            assert_eq!(keyboard.event(1, make, 1, VK_SHIFT), None);
            assert_eq!(keyboard.event(1, make, 0, VK_SHIFT), None);
            assert_eq!(keyboard.event(1, make, 0, 0x60), None);
            assert_eq!(keyboard.event(1, make, 1, 0x60), None);
            assert_eq!(keyboard.event(1, make, 0, 0x60), Some(code));
        }
    }
    #[test]
    fn print_screen_prefixes_do_not_add_shift_counts() {
        let mut keyboard = WindowsKeyboard::default();
        for _ in 0..2 {
            assert_eq!(keyboard.event(1, 0x2a, 2, VK_SHIFT), None);
            assert_eq!(keyboard.event(1, 0x37, 2, 0x2c), Some(0x10137));
            assert_eq!(keyboard.event(1, 0x37, 2, 0x2c), None);
            assert_eq!(keyboard.event(1, 0x37, 3, 0x2c), None);
            assert_eq!(keyboard.event(1, 0x2a, 3, VK_SHIFT), None);
        }
        // A real Shift key remains independent of the ignored prefix.
        assert_eq!(keyboard.event(1, 0x2a, 0, VK_SHIFT), Some(56));
    }
    #[test]
    fn alt_print_screen_keeps_one_identity_across_modifier_changes() {
        let mut keyboard = WindowsKeyboard::default();
        let mut counts = Vec::new();
        let packets = [
            (0x38, 0, 0x12), // Alt
            (0x54, 0, 0x2c), // Alt+PrintScreen
            (0x54, 0, 0x2c), // repeated make
            (0x38, 1, 0x12), // release Alt before PrintScreen
            (0x37, 2, 0x2c), // same key still held under its ordinary scan code
            (0x37, 3, 0x2c),
            (0x37, 2, 0x2c), // next physical press, without Alt
            (0x54, 1, 0x2c), // either release form clears the same held key
            (0x54, 0, 0x2c),
            (0x54, 1, 0x2c),
        ];
        for (make, flags, vk) in packets {
            counts.extend(keyboard.event(1, make, flags, vk));
        }
        assert_eq!(counts, vec![58, 0x10137, 0x10137, 0x10137]);
        assert!(keyboard.held.is_empty());
    }
    #[test]
    fn orphan_print_screen_break_counts_once_when_make_is_swallowed() {
        let mut keyboard = WindowsKeyboard::default();
        // When a screenshot tool swallows PrintScreen's make packet, Windows
        // still delivers the break packet (0x37 with E0|BREAK = 3).
        assert_eq!(keyboard.event(1, 0x37, 3, 0x2c), Some(0x10137));
        // A normal make+break press still counts only once.
        assert_eq!(keyboard.event(1, 0x37, 2, 0x2c), Some(0x10137));
        assert_eq!(keyboard.event(1, 0x37, 3, 0x2c), None);
    }
    #[test]
    fn control_pause_counts_repeated_presses_and_releases_under_either_identity() {
        let mut keyboard = WindowsKeyboard::default();
        assert_eq!(keyboard.event(1, 0x1d, 0, 0x11), Some(59));
        for _ in 0..2 {
            // Ctrl+Pause is reported as E0 46 / VK_CANCEL, with a break.
            assert_eq!(keyboard.event(1, 0x46, 2, 0x03), Some(0x10245));
            assert_eq!(keyboard.event(1, 0x46, 2, 0x03), None);
            assert_eq!(keyboard.event(1, 0x46, 3, 0x03), None);
        }
        assert_eq!(keyboard.event(1, 0x46, 2, 0x03), Some(0x10245));
        assert_eq!(keyboard.event(1, 0x1d, 1, 0x11), None);
        assert_eq!(keyboard.event(1, 0x45, 0, VK_PAUSE), None);
        assert_eq!(keyboard.event(1, 0x45, 1, VK_PAUSE), None);
        // The next ordinary Pause presses must still work without key-up.
        assert_eq!(keyboard.event(1, 0x45, 0, VK_PAUSE), Some(0x10245));
        assert_eq!(keyboard.event(1, 0x45, 0, VK_PAUSE), Some(0x10245));
        assert_eq!(keyboard.event(1, 0x46, 2, 0x03), Some(0x10245));
        assert_eq!(keyboard.event(1, 0x46, 3, 0x03), None);
        assert!(keyboard.held.is_empty());
    }
    #[test]
    fn pause_counts_each_press_without_a_break_and_stays_separate_from_numlock() {
        let mut keyboard = WindowsKeyboard::default();
        assert_eq!(keyboard.event(1, 0x45, 0, VK_NUMLOCK), Some(71));
        for _ in 0..2 {
            assert_eq!(keyboard.event(1, 0x1d, 4, 0x11), None);
            assert_eq!(keyboard.event(1, 0x45, 0, VK_PAUSE), Some(0x10245));
        }
        assert_eq!(keyboard.event(1, 0x45, 4, VK_PAUSE), Some(0x10245));
        assert_eq!(keyboard.event(1, 0x45, 5, VK_PAUSE), None);
        // A changed extended flag must not leave NumLock stuck or recount it.
        assert_eq!(keyboard.event(1, 0x45, 2, VK_NUMLOCK), None);
        assert_eq!(keyboard.event(1, 0x45, 3, VK_NUMLOCK), None);
        assert_eq!(keyboard.event(1, 0x45, 2, VK_NUMLOCK), Some(71));
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

#[cfg(test)]
mod supplemental_mapping_tests {
    use super::*;
    #[test]
    fn iso_and_audio_scan_codes_share_cross_platform_identities() {
        for (scan, code) in [(0x56, 10), (0x120, 74), (0x12e, 73), (0x130, 72)] {
            assert_eq!(canonical_code(scan), code);
        }
    }
}
