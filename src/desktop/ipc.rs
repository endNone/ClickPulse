//! Explicit payloads reject unknown actions and incomplete or invalid options.
#[cfg(target_os = "windows")]
use clickpulse::presentation::CloseAction;
use clickpulse::{localization::LanguagePreference, presentation::Theme};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    Language {
        language: LanguagePreference,
    },
    Theme {
        theme: Theme,
    },
    Ready {},
    Compact {},
    Expand {},
    Drag {},
    Pin {},
    Hide {},
    Refresh {},
    Pause {},
    Permission {},
    Autostart {},
    Export {},
    Reveal {},
    Quit {},
    Close {},
    #[cfg(target_os = "macos")]
    Takeover {},
    #[cfg(target_os = "windows")]
    Minimize {},
    #[cfg(target_os = "windows")]
    Maximize {},
    #[cfg(target_os = "windows")]
    ClosePreference {
        close_action: CloseAction,
    },
    #[cfg(target_os = "windows")]
    CloseDecision {
        close_action: CloseAction,
        #[serde(default)]
        remember: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_actions_and_payloads_are_rejected() {
        for body in [
            r#"{"action":"unknown"}"#,
            r#"{"action":"theme"}"#,
            r#"{"action":"theme","theme":"unknown"}"#,
            r#"{"action":"language","language":null}"#,
            r#"{"action":"language","language":"fr"}"#,
            r#"{"action":"quit","theme":"blue"}"#,
        ] {
            assert!(serde_json::from_str::<Request>(body).is_err(), "{body}");
        }
        assert!(matches!(
            serde_json::from_str::<Request>(r#"{"action":"theme","theme":"teal"}"#).unwrap(),
            Request::Theme { theme: Theme::Teal }
        ));
        assert!(matches!(
            serde_json::from_str::<Request>(r#"{"action":"language","language":"zh-CN"}"#).unwrap(),
            Request::Language {
                language: LanguagePreference::Chinese
            }
        ));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn close_choices_require_a_known_choice_and_boolean_remember_flag() {
        assert!(matches!(
            serde_json::from_str::<Request>(r#"{"action":"close_decision","close_action":"hide"}"#)
                .unwrap(),
            Request::CloseDecision {
                close_action: CloseAction::Hide,
                remember: false
            }
        ));
        assert!(
            serde_json::from_str::<Request>(
                r#"{"action":"close_decision","close_action":"quit","remember":"true"}"#
            )
            .is_err()
        );
    }
}
