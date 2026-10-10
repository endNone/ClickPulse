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
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locale {
    Chinese,
    English,
}
impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Self::Chinese => "zh-CN",
            Self::English => "en",
        }
    }
    pub fn text(self, key: &str) -> String {
        static MESSAGES: OnceLock<serde_json::Value> = OnceLock::new();
        let messages = MESSAGES.get_or_init(|| {
            serde_json::from_str(include_str!("../ui/locales.json"))
                .expect("invalid translation catalog")
        });
        let key = if cfg!(target_os = "windows")
            && messages[self.code()][format!("{key}.windows")].is_string()
        {
            format!("{key}.windows")
        } else {
            key.to_string()
        };
        messages[self.code()][&key]
            .as_str()
            .unwrap_or(&key)
            .to_owned()
    }
}
fn from_tag(tag: &str) -> Locale {
    if tag
        .split(['-', '_'])
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case("zh"))
    {
        Locale::Chinese
    } else {
        Locale::English
    }
}
#[cfg(any(target_os = "macos", test))]
fn from_preferences(value: &str) -> Locale {
    let first = value
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .find(|part| !part.is_empty())
        .unwrap_or("en");
    from_tag(first)
}
impl LanguagePreference {
    pub fn resolve(self) -> Locale {
        match self {
            Self::Chinese => Locale::Chinese,
            Self::English => Locale::English,
            Self::System => system_locale(),
        }
    }
}
fn system_locale() -> Locale {
    #[cfg(target_os = "windows")]
    if let Some(language) = crate::platform::windows::preferred_language() {
        return from_tag(&language);
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
    fn preferred_language_uses_first_system_choice_and_english_fallback() {
        assert_eq!(
            from_preferences("(\n\"en-US\", \"zh-Hans-US\"\n)"),
            Locale::English
        );
        assert_eq!(
            from_preferences("(\"zh-Hans-CN\", \"en-US\")"),
            Locale::Chinese
        );
        assert_eq!(
            from_preferences("(\"fr-FR\", \"zh-Hans\")"),
            Locale::English
        );
        assert_eq!(from_tag("zh_CN.UTF-8"), Locale::Chinese);
        assert_eq!(from_preferences(""), Locale::English);
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
        assert!(Locale::English.text("message.exported").contains("{path}"));
    }
}
