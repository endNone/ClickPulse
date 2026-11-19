//! Window preferences never change stored user statistics.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    Blue,
    Teal,
    Violet,
    Graphite,
}
impl Theme {
    pub fn code(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Teal => "teal",
            Self::Violet => "violet",
            Self::Graphite => "graphite",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SportSkin {
    #[default]
    #[serde(
        alias = "halo",
        alias = "velocity",
        alias = "neon",
        alias = "stardust",
        alias = "beat"
    )]
    Warp,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CloseAction {
    #[default]
    Ask,
    Hide,
    Quit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodayPeriod {
    #[default]
    CalendarDay,
    #[serde(rename = "rolling_24h")]
    Rolling24h,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardLayout {
    #[default]
    Auto,
    Compact,
    FullSize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum UiScale {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "100")]
    Percent100,
    #[serde(rename = "115")]
    Percent115,
    #[serde(rename = "130")]
    Percent130,
    #[serde(rename = "150")]
    Percent150,
}
impl UiScale {
    pub fn factor(self, width: f64, compact: bool) -> f64 {
        if compact {
            return 1.0;
        }
        match self {
            Self::Auto => ((width / 1160.0).clamp(1.0, 1.5) * 20.0).round() / 20.0,
            Self::Percent100 => 1.0,
            Self::Percent115 => 1.15,
            Self::Percent130 => 1.3,
            Self::Percent150 => 1.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub compact: bool,
    pub pinned: bool,
    pub language: crate::localization::LanguagePreference,
    pub timezone: crate::timezone::TimeZonePreference,
    pub today_period: TodayPeriod,
    pub keyboard_layout: KeyboardLayout,
    pub ui_scale: UiScale,
    pub theme: Theme,
    pub close_action: CloseAction,
    pub auto_check_updates: bool,
    pub last_update_check: Option<i64>,
    pub update_notifications: crate::updater::UpdateNotifications,
    pub sport_enabled: bool,
    pub sport_visible: bool,
    pub sport_skin: SportSkin,
    pub ignore_software_input: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            compact: false,
            pinned: true,
            language: Default::default(),
            timezone: Default::default(),
            today_period: Default::default(),
            keyboard_layout: Default::default(),
            ui_scale: Default::default(),
            theme: Default::default(),
            close_action: Default::default(),
            auto_check_updates: true,
            last_update_check: None,
            update_notifications: Default::default(),
            sport_enabled: false,
            sport_visible: false,
            sport_skin: Default::default(),
            ignore_software_input: false,
        }
    }
}

fn path() -> Result<PathBuf> {
    Ok(crate::platform::data_dir()?.join("presentation.json"))
}

pub struct LoadedPreferences {
    pub preferences: Preferences,
    pub recovered_from: Option<PathBuf>,
}

impl Preferences {
    /// Error dialogs may inspect language, but must not recover or rewrite
    /// preferences while an installation or failed startup is being reported.
    pub fn locale_for_error() -> crate::localization::Locale {
        path()
            .ok()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
            .unwrap_or_default()
            .language
            .resolve()
    }

    pub fn load() -> Result<LoadedPreferences> {
        Self::load_from(&path()?)
    }

    fn load_from(path: &Path) -> Result<LoadedPreferences> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LoadedPreferences {
                    preferences: Self::default(),
                    recovered_from: None,
                });
            }
            Err(error) => return Err(error).context("Unable to read window preferences"),
        };
        if let Ok(preferences) = serde_json::from_slice(&bytes) {
            return Ok(LoadedPreferences {
                preferences,
                recovered_from: None,
            });
        }
        // Preserve exact bytes before allowing defaults to replace a bad file.
        // If backup fails, startup fails rather than silently overwriting it.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let backup = path.with_extension(format!("corrupt-{}-{nonce}.json", std::process::id()));
        let mut file = std::fs::File::create_new(&backup)
            .context("Unable to preserve damaged window preferences")?;
        use std::io::Write;
        if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
            drop(file);
            let _ = std::fs::remove_file(&backup);
            return Err(error).context("Unable to preserve damaged window preferences");
        }
        drop(file);
        std::fs::remove_file(path).context("Unable to recover damaged window preferences")?;
        Ok(LoadedPreferences {
            preferences: Self::default(),
            recovered_from: Some(backup),
        })
    }
    pub fn save(&self) -> Result<()> {
        self.save_to(&path()?)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(path.parent().context("Invalid preferences directory")?)?;
        let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_period_defaults_to_today_and_survives_restart() {
        let old: Preferences = serde_json::from_str(r#"{"timezone":"Asia/Shanghai"}"#).unwrap();
        assert_eq!(old.today_period, TodayPeriod::CalendarDay);
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        let prefs = Preferences {
            today_period: TodayPeriod::Rolling24h,
            ..old
        };
        prefs.save_to(&path).unwrap();
        let restored = Preferences::load_from(&path).unwrap().preferences;
        assert_eq!(restored.today_period, TodayPeriod::Rolling24h);
        assert_eq!(restored.timezone, prefs.timezone);
        assert_eq!(
            serde_json::to_value(restored.today_period).unwrap(),
            "rolling_24h"
        );
    }

    #[test]
    fn daily_checks_migrate_old_preferences_and_retain_opt_out_and_attempt_time() {
        let old: Preferences = serde_json::from_str(r#"{"theme":"violet"}"#).unwrap();
        assert!(old.auto_check_updates);
        assert_eq!(old.last_update_check, None);
        assert_eq!(old.theme, Theme::Violet);
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        for enabled in [false, true] {
            let prefs = Preferences {
                auto_check_updates: enabled,
                last_update_check: Some(1_794_310_000),
                ..old.clone()
            };
            prefs.save_to(&path).unwrap();
            let loaded = Preferences::load_from(&path).unwrap().preferences;
            assert_eq!(loaded.auto_check_updates, enabled);
            assert_eq!(loaded.last_update_check, prefs.last_update_check);
            assert_eq!(loaded.theme, Theme::Violet);
        }
    }

    #[test]
    fn hidden_update_reminders_survive_restart_and_a_shown_version_is_not_repeated() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        let mut prefs = Preferences::default();
        assert!(prefs.update_notifications.observe(Some("9.9.9")));
        // A hidden or Mini window keeps the reminder pending when saving.
        prefs.save_to(&path).unwrap();
        let mut restarted = Preferences::load_from(&path).unwrap().preferences;
        assert_eq!(restarted.update_notifications.pending(), Some("9.9.9"));
        restarted.update_notifications.mark_shown();
        restarted.save_to(&path).unwrap();
        let mut again = Preferences::load_from(&path).unwrap().preferences;
        assert!(!again.update_notifications.observe(Some("9.9.9")));
        assert_eq!(again.update_notifications.pending(), None);
        assert!(again.update_notifications.observe(Some("10.0.0")));
        again.update_notifications.mark_shown();
        assert!(!again.update_notifications.observe(Some("9.9.9")));
        assert_eq!(again.update_notifications.pending(), None);
    }

    #[test]
    fn damaged_preferences_are_preserved_before_defaults_can_be_saved() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        let bad = b"{\"theme\":\"violet\",\"language\":\"zh-CN\",";
        std::fs::write(&path, bad).unwrap();
        let loaded = Preferences::load_from(&path).unwrap();
        let backup = loaded.recovered_from.unwrap();
        assert_eq!(std::fs::read(&backup).unwrap(), bad);
        assert!(!path.exists());
        loaded.preferences.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&backup).unwrap(), bad);
        let again = Preferences::load_from(&path).unwrap();
        assert!(again.recovered_from.is_none());
        assert_eq!(again.preferences.theme, Theme::Blue);
    }

    #[test]
    fn missing_preferences_default_but_io_errors_do_not() {
        let temp = tempfile::tempdir().unwrap();
        assert!(
            Preferences::load_from(&temp.path().join("missing.json"))
                .unwrap()
                .recovered_from
                .is_none()
        );
        assert!(Preferences::load_from(temp.path()).is_err());
    }

    #[test]
    fn existing_preferences_gain_blue_without_resetting_language_or_pin() {
        let prefs: Preferences =
            serde_json::from_str(r#"{"compact":true,"pinned":false,"language":"zh-CN"}"#).unwrap();
        assert_eq!(prefs.theme, Theme::Blue);
        assert_eq!(prefs.close_action, CloseAction::Ask);
        assert!(!prefs.sport_enabled);
        assert!(!prefs.ignore_software_input);
        assert!(!prefs.sport_visible);
        assert_eq!(prefs.sport_skin, SportSkin::Warp);
        assert!(prefs.compact);
        assert!(!prefs.pinned);
        assert_eq!(
            prefs.language,
            crate::localization::LanguagePreference::Chinese
        );
    }

    #[test]
    fn every_supported_theme_round_trips_in_saved_preferences() {
        for theme in [Theme::Blue, Theme::Teal, Theme::Violet, Theme::Graphite] {
            let prefs = Preferences {
                theme,
                ..Default::default()
            };
            let bytes = serde_json::to_vec(&prefs).unwrap();
            let loaded: Preferences = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(loaded.theme, theme);
            assert_eq!(serde_json::to_value(theme).unwrap(), theme.code());
        }
        assert!(serde_json::from_str::<Theme>(r#""unsupported""#).is_err());
    }

    #[test]
    fn remembered_close_choice_round_trips_without_resetting_other_preferences() {
        for close_action in [CloseAction::Ask, CloseAction::Hide, CloseAction::Quit] {
            let prefs = Preferences {
                close_action,
                compact: true,
                pinned: false,
                theme: Theme::Violet,
                language: crate::localization::LanguagePreference::English,
                ..Default::default()
            };
            let loaded: Preferences =
                serde_json::from_slice(&serde_json::to_vec(&prefs).unwrap()).unwrap();
            assert_eq!(loaded.close_action, close_action);
            assert!(loaded.compact);
            assert!(!loaded.pinned);
            assert_eq!(loaded.theme, Theme::Violet);
            assert_eq!(
                loaded.language,
                crate::localization::LanguagePreference::English
            );
        }
    }

    #[test]
    fn sport_defaults_off_and_retains_an_explicit_saved_choice() {
        let defaults = Preferences::default();
        assert!(!defaults.sport_enabled && !defaults.sport_visible);
        let old: Preferences = serde_json::from_str(r#"{"compact":true}"#).unwrap();
        assert!(!old.sport_enabled && !old.sport_visible);
        let opted_in: Preferences =
            serde_json::from_str(r#"{"sport_enabled":true,"sport_visible":true}"#).unwrap();
        assert!(opted_in.sport_enabled && opted_in.sport_visible);
    }

    #[test]
    fn sport_visibility_and_warp_round_trip_without_changing_other_preferences() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        let prefs = Preferences {
            sport_skin: SportSkin::Warp,
            sport_enabled: true,
            sport_visible: false,
            theme: Theme::Teal,
            pinned: false,
            ..Default::default()
        };
        prefs.save_to(&path).unwrap();
        let decoded = Preferences::load_from(&path).unwrap().preferences;
        assert_eq!(decoded.sport_skin, SportSkin::Warp);
        assert!(decoded.sport_enabled);
        assert!(!decoded.sport_visible);
        assert_eq!(decoded.theme, Theme::Teal);
        assert!(!decoded.pinned);
        assert!(serde_json::from_str::<SportSkin>(r#""unknown""#).is_err());
    }

    #[test]
    fn former_sport_skins_load_without_recovering_or_resetting_other_choices() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("presentation.json");
        for old in ["halo", "velocity", "neon", "stardust", "beat"] {
            let original = serde_json::json!({
                "sport_skin": old, "language": "ja", "pinned": false,
                "theme": "teal", "sport_enabled": true
            });
            std::fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
            let loaded = Preferences::load_from(&path).unwrap();
            assert!(loaded.recovered_from.is_none());
            assert_eq!(loaded.preferences.sport_skin, SportSkin::Warp);
            assert_eq!(
                loaded.preferences.language,
                crate::localization::LanguagePreference::Japanese
            );
            assert_eq!(loaded.preferences.theme, Theme::Teal);
            assert!(!loaded.preferences.pinned);
            assert!(loaded.preferences.sport_enabled);
            loaded.preferences.save_to(&path).unwrap();
            let saved: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert_eq!(saved["sport_skin"], "warp");
        }
    }
}

#[cfg(test)]
mod display_preference_tests {
    use super::*;
    #[test]
    fn scale_is_bounded_and_mini_always_uses_native_size() {
        for (width, scale) in [
            (900.0, 1.0),
            (1160.0, 1.0),
            (1334.0, 1.15),
            (1920.0, 1.5),
            (2560.0, 1.5),
        ] {
            assert!((UiScale::Auto.factor(width, false) - scale).abs() < 0.001);
        }
        for value in [
            UiScale::Auto,
            UiScale::Percent100,
            UiScale::Percent115,
            UiScale::Percent130,
            UiScale::Percent150,
        ] {
            assert_eq!(value.factor(2560.0, true), 1.0);
        }
        let defaults: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(defaults.keyboard_layout, KeyboardLayout::Auto);
        assert_eq!(defaults.ui_scale, UiScale::Auto);
    }

    #[test]
    fn keyboard_layout_choices_survive_saving_and_reloading_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("presentation.json");
        for layout in [KeyboardLayout::FullSize, KeyboardLayout::Compact] {
            let prefs = Preferences {
                keyboard_layout: layout,
                ui_scale: UiScale::Percent130,
                theme: Theme::Teal,
                ..Default::default()
            };
            prefs.save_to(&path).unwrap();
            let restored = Preferences::load_from(&path).unwrap().preferences;
            assert_eq!(restored.keyboard_layout, layout);
            assert_eq!(restored.ui_scale, UiScale::Percent130);
            assert_eq!(restored.theme, Theme::Teal);
        }
    }
}
