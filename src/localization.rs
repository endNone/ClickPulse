//! Shared App translations; choosing a language never changes collection or data.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum LanguagePreference {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "zh-CN")]
    Chinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-TW")]
    TraditionalChinese,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "es")]
    Spanish,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locale {
    Chinese,
    English,
    TraditionalChinese,
    Japanese,
    Korean,
    French,
    German,
    Spanish,
}
impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Self::Chinese => "zh-CN",
            Self::English => "en",
            Self::TraditionalChinese => "zh-TW",
            Self::Japanese => "ja",
            Self::Korean => "ko",
            Self::French => "fr",
            Self::German => "de",
            Self::Spanish => "es",
        }
    }
    pub fn text(self, key: &str) -> String {
        static MESSAGES: OnceLock<serde_json::Value> = OnceLock::new();
        let messages = MESSAGES.get_or_init(|| {
            serde_json::from_str(include_str!("../ui/locales.json"))
                .expect("invalid translation catalog")
        });
        let key = if cfg!(target_os = "windows")
            && (messages[self.code()][format!("{key}.windows")].is_string()
                || messages["en"][format!("{key}.windows")].is_string())
        {
            format!("{key}.windows")
        } else {
            key.to_string()
        };
        messages[self.code()][&key]
            .as_str()
            .or_else(|| messages["en"][&key].as_str())
            .unwrap_or(&key)
            .to_owned()
    }
}
fn supported_tag(tag: &str) -> Option<Locale> {
    let tag = tag
        .trim()
        .split(['.', '@'])
        .next()
        .unwrap_or("")
        .replace('_', "-")
        .to_ascii_lowercase();
    let parts: Vec<_> = tag.split('-').collect();
    match parts[0] {
        "en" => Some(Locale::English),
        "ja" => Some(Locale::Japanese),
        "ko" => Some(Locale::Korean),
        "fr" => Some(Locale::French),
        "de" => Some(Locale::German),
        "es" => Some(Locale::Spanish),
        "zh" => {
            if parts.contains(&"hans") {
                Some(Locale::Chinese)
            } else if parts
                .iter()
                .any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo"))
            {
                Some(Locale::TraditionalChinese)
            } else {
                Some(Locale::Chinese)
            }
        }
        _ => None,
    }
}
fn from_tag(tag: &str) -> Locale {
    supported_tag(tag).unwrap_or(Locale::English)
}
#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn from_preferences(value: &str) -> Locale {
    value
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .find_map(supported_tag)
        .unwrap_or(Locale::English)
}
impl LanguagePreference {
    pub fn resolve(self) -> Locale {
        match self {
            Self::Chinese => Locale::Chinese,
            Self::English => Locale::English,
            Self::TraditionalChinese => Locale::TraditionalChinese,
            Self::Japanese => Locale::Japanese,
            Self::Korean => Locale::Korean,
            Self::French => Locale::French,
            Self::German => Locale::German,
            Self::Spanish => Locale::Spanish,
            Self::System => system_locale(),
        }
    }
}
fn system_locale() -> Locale {
    #[cfg(target_os = "windows")]
    if let Some(languages) = crate::platform::windows::preferred_languages() {
        return from_preferences(&languages.join(","));
    }
    #[cfg(target_os = "macos")]
    for arguments in [
        ["read", "app.clickpulse.desktop", "AppleLanguages"],
        ["read", "-g", "AppleLanguages"],
    ] {
        if let Ok(output) = std::process::Command::new("/usr/bin/defaults")
            .args(arguments)
            .output()
            && output.status.success()
            && !output.stdout.is_empty()
        {
            return from_preferences(&String::from_utf8_lossy(&output.stdout));
        }
    }
    for variable in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(variable)
            && !value.is_empty()
        {
            return from_tag(&value);
        }
    }
    Locale::English
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferred_language_uses_first_supported_choice_and_english_fallback() {
        assert_eq!(
            from_preferences("(\n\"en-US\", \"zh-Hans-US\"\n)"),
            Locale::English
        );
        assert_eq!(
            from_preferences("(\"zh-Hans-CN\", \"en-US\")"),
            Locale::Chinese
        );
        assert_eq!(
            from_preferences("(\"it-IT\", \"zh-Hans\")"),
            Locale::Chinese
        );
        assert_eq!(from_tag("zh_CN.UTF-8"), Locale::Chinese);
        assert_eq!(from_preferences(""), Locale::English);
    }
    #[test]
    fn regional_language_tags_and_explicit_scripts_resolve_consistently() {
        for (tag, expected) in [
            ("zh-Hant-HK", Locale::TraditionalChinese),
            ("zh_TW.UTF-8", Locale::TraditionalChinese),
            ("ZH-hk", Locale::TraditionalChinese),
            ("zh-MO", Locale::TraditionalChinese),
            ("zh-Hans-TW", Locale::Chinese),
            ("zh-SG", Locale::Chinese),
            ("ja-JP", Locale::Japanese),
            ("ko_KR.UTF-8", Locale::Korean),
            (" en-GB ", Locale::English),
            ("pt-BR", Locale::English),
            ("fr-CA", Locale::French),
            ("de_DE.UTF-8", Locale::German),
            ("es-MX", Locale::Spanish),
        ] {
            assert_eq!(from_tag(tag), expected, "{tag}");
        }
        assert_eq!(from_preferences("(it-IT, ja-JP, en-US)"), Locale::Japanese);
        assert_eq!(
            LanguagePreference::TraditionalChinese.resolve(),
            Locale::TraditionalChinese
        );
        assert_eq!(LanguagePreference::Japanese.resolve(), Locale::Japanese);
        assert_eq!(LanguagePreference::Korean.resolve(), Locale::Korean);
    }
    #[test]
    fn old_window_preferences_default_to_system_language() {
        let prefs: crate::presentation::Preferences =
            serde_json::from_str(r#"{"compact":true,"pinned":false}"#).unwrap();
        assert!(prefs.compact);
        assert!(!prefs.pinned);
        assert_eq!(prefs.language, LanguagePreference::System);
        assert_eq!(
            serde_json::to_value(LanguagePreference::Chinese).unwrap(),
            "zh-CN"
        );
        assert!(serde_json::from_str::<LanguagePreference>(r#""unknown""#).is_err());
    }
    #[test]
    fn shared_catalog_localizes_native_controls_and_preserves_placeholders() {
        assert_eq!(Locale::English.text("native.open"), "Open statistics");
        assert_eq!(Locale::Chinese.text("native.open"), "打开统计");
        assert!(Locale::English.text("storage.imported").contains("{path}"));
        for (locale, label) in [
            (Locale::TraditionalChinese, "個人化"),
            (Locale::Japanese, "カスタマイズ"),
            (Locale::Korean, "개인 설정"),
            (Locale::French, "Personnalisation"),
            (Locale::German, "Personalisierung"),
            (Locale::Spanish, "Personalización"),
        ] {
            assert_eq!(locale.text("settings.personalization"), label);
            assert!(locale.text("sport.mini_readout").contains("{speed}"));
        }
    }
}
