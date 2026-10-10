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
pub enum CloseAction {
    #[default]
    Ask,
    Hide,
    Quit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub compact: bool,
    pub pinned: bool,
    pub language: crate::localization::LanguagePreference,
    pub theme: Theme,
    pub close_action: CloseAction,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            compact: false,
            pinned: true,
            language: Default::default(),
            theme: Default::default(),
            close_action: Default::default(),
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
        std::fs::create_dir_all(path.parent().unwrap())?;
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp)?;
            file.write_all(&serde_json::to_vec_pretty(self)?)?;
            file.sync_all()?;
            std::fs::rename(&temp, path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temp);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
