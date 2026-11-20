use anyhow::{Result, bail};
use clickpulse::shortcuts::{ALT, CONTROL, Chord, META, PAUSE, PRINT_SCREEN, SCROLL_LOCK, SHIFT};
use url::Url;

// Format the recorded shortcut key ID using the same labels as the activity list.
#[rustfmt::skip]
const KEY_NAMES: [&str; 127] = [
    "A", "S", "D", "F", "H", "G", "Z", "X",
    "C", "V", "ISO §", "B", "Q", "W", "E", "R",
    "Y", "T", "1", "2", "3", "4", "6", "5",
    "=", "9", "7", "-", "8", "0", "]", "O",
    "U", "[", "I", "P", "Enter", "L", "J", "'",
    "K", ";", "\\", ",", "/", "N", "M", ".",
    "Tab", "Space", "`", "Backspace", "", "Esc", "", "",
    "", "", "", "", "", "", "", "",
    "F17", "Num .", "", "Num ×", "", "Num +", "", "Num Clear",
    "Volume Up", "Volume Down", "Mute", "Num ÷", "Num Enter", "", "Num −", "F18",
    "F19", "Num =", "Num 0", "Num 1", "Num 2", "Num 3", "Num 4", "Num 5",
    "Num 6", "Num 7", "F20", "Num 8", "Num 9", "JIS ¥", "JIS _", "Num ,",
    "F5", "F6", "F7", "F3", "F8", "F9", "Eisu", "F11",
    "Kana", "F13", "F16", "F14", "", "F10", "", "F12",
    "", "F15", "Insert", "Home", "PageUp", "Delete", "F4", "End",
    "F2", "PageDown", "F1", "Left", "Right", "Down", "Up",
];

pub(super) fn url(platform: &str, modifiers: u8, key: u32) -> Result<Url> {
    let meta = match platform {
        "macos" => "⌘",
        "windows" => "Win",
        "linux" => "Super",
        _ => bail!("Unknown shortcut platform"),
    };
    let chord =
        Chord::new(key, modifiers, platform).ok_or_else(|| anyhow::anyhow!("Invalid shortcut"))?;
    let fallback = format!("Key {}", chord.key);
    let key_name = match chord.key {
        PRINT_SCREEN => "PrtSc",
        SCROLL_LOCK => "Scroll Lock",
        PAUSE => "Pause",
        code => KEY_NAMES
            .get(code as usize)
            .copied()
            .filter(|name| !name.is_empty())
            .unwrap_or(&fallback),
    };
    let mut parts = Vec::with_capacity(5);
    for (bit, name) in [
        (CONTROL, "Ctrl"),
        (ALT, if platform == "macos" { "Option" } else { "Alt" }),
        (SHIFT, "Shift"),
        (META, meta),
    ] {
        if chord.modifiers & bit != 0 {
            parts.push(name);
        }
    }
    parts.push(key_name);
    let mut url = Url::parse("https://github.com/endNone/ClickPulse/issues/new")?;
    url.query_pairs_mut()
        .append_pair("template", "shortcut-suggestion.yml")
        .append_pair("platform", platform)
        .append_pair("chord", &parts.join(" + "));
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_template_platform_and_physical_chord_are_shared() {
        let link = url("macos", META, 69).unwrap();
        assert_eq!(link.host_str(), Some("github.com"));
        assert_eq!(link.path(), "/endNone/ClickPulse/issues/new");
        assert!(link.as_str().contains("%2B"));
        assert_eq!(
            link.query_pairs().collect::<Vec<_>>(),
            [
                ("template".into(), "shortcut-suggestion.yml".into()),
                ("platform".into(), "macos".into()),
                ("chord".into(), "⌘ + Num +".into()),
            ]
        );
        assert!(
            url("linux", SHIFT, 0x10054)
                .unwrap()
                .as_str()
                .ends_with("Shift+%2B+PrtSc")
        );
        assert!(
            url("windows", CONTROL, 76)
                .unwrap()
                .as_str()
                .ends_with("Ctrl+%2B+Num+Enter")
        );
    }
    #[test]
    fn rejects_unknown_platforms_and_unrecordable_combinations() {
        for (platform, mods, key) in [
            ("macos", ALT, 8),
            ("windows", SHIFT, 8),
            ("linux", 0, 8),
            ("linux", CONTROL, u32::MAX),
            ("macos&body=private", META, 8),
        ] {
            assert!(url(platform, mods, key).is_err());
        }
    }
}
