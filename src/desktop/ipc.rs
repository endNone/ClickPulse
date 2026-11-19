//! Explicit payloads reject unknown actions and incomplete or invalid options.
#[cfg(any(target_os = "windows", target_os = "linux"))]
use clickpulse::presentation::CloseAction;
use clickpulse::{localization::LanguagePreference, presentation::Theme};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    Language {
        language: LanguagePreference,
    },
    Timezone {
        timezone: clickpulse::timezone::TimeZonePreference,
    },
    TodayPeriod {
        period: clickpulse::presentation::TodayPeriod,
    },
    KeyboardLayout {
        layout: clickpulse::presentation::KeyboardLayout,
    },
    UiScale {
        scale: clickpulse::presentation::UiScale,
    },
    Theme {
        theme: Theme,
    },
    CheckUpdates {},
    AutomaticUpdates {
        enabled: bool,
    },
    InstallUpdate {},
    UpdateRelease {},
    ProjectHome {},
    SendFeedback {},
    ShortcutReference {
        source: String,
    },
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    CopyShareCard {
        png: String,
        day: String,
    },
    SaveShareCard {
        png: String,
        day: String,
    },
    Ready {},
    SessionState {
        state: super::session::SessionState,
    },
    Compact {},
    Expand {},
    Drag {},
    Pin {},
    Hide {},
    Refresh {},
    SportSample {},
    HistoryHours {
        id: u32,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
        timezone: clickpulse::timezone::TimeZonePreference,
    },
    Sport {
        enabled: bool,
    },
    SportVisible {
        visible: bool,
    },
    Pause {},
    IgnoreSoftwareInput {
        enabled: bool,
    },
    Permission {},
    Autostart {},
    Reveal {},
    StorageDirectory {},
    ImportHistory {},
    #[cfg(windows)]
    Uninstall {},
    Quit {},
    Close {},
    #[cfg(target_os = "macos")]
    Takeover {},
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Minimize {},
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    Maximize {},
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    ClosePreference {
        close_action: CloseAction,
    },
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    CloseDecision {
        close_action: CloseAction,
        #[serde(default)]
        remember: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn copy_card_ipc_requires_image_and_date_fields() {
        assert!(matches!(serde_json::from_str::<Request>(r#"{"action":"copy_share_card","png":"data:image/png;base64,AA==","day":"2026-11-17"}"#).unwrap(), Request::CopyShareCard { .. }));
        assert!(
            serde_json::from_str::<Request>(r#"{"action":"copy_share_card","day":"2026-11-17"}"#)
                .is_err()
        );
    }
    #[test]
    fn invalid_actions_and_payloads_are_rejected() {
        for body in [
            r#"{"action":"unknown"}"#,
            r#"{"action":"check_updates","url":"https://example.com"}"#,
            r#"{"action":"automatic_updates"}"#,
            r#"{"action":"automatic_updates","enabled":"true"}"#,
            r#"{"action":"automatic_updates","enabled":true,"interval":1}"#,
            r#"{"action":"install_update","path":"/tmp/other"}"#,
            r#"{"action":"update_release","url":"https://example.com"}"#,
            r#"{"action":"project_home","url":"https://example.com"}"#,
            r#"{"action":"send_feedback","url":"https://example.com"}"#,
            r#"{"action":"timezone","timezone":"not/a-zone"}"#,
            r#"{"action":"timezone"}"#,
            r#"{"action":"timezone","timezone":null}"#,
            r#"{"action":"today_period"}"#,
            r#"{"action":"today_period","period":"all"}"#,
            r#"{"action":"today_period","period":"rolling_24h","hours":48}"#,
            r#"{"action":"ui_scale","scale":150}"#,
            r#"{"action":"ui_scale","scale":"500"}"#,
            r#"{"action":"ui_scale"}"#,
            r#"{"action":"keyboard_layout","layout":"unknown"}"#,
            r#"{"action":"keyboard_layout"}"#,
            r#"{"action":"export"}"#,
            r#"{"action":"theme"}"#,
            r#"{"action":"theme","theme":"unknown"}"#,
            r#"{"action":"language","language":null}"#,
            r#"{"action":"language","language":"it"}"#,
            r#"{"action":"quit","theme":"blue"}"#,
            r#"{"action":"sport","enabled":"true"}"#,
            r#"{"action":"ignore_software_input","enabled":"true"}"#,
            r#"{"action":"ignore_software_input"}"#,
            r#"{"action":"sport_skin","skin":"unknown"}"#,
            // Warp is the only Sport effect; there is no skin choice to send.
            r#"{"action":"sport_skin","skin":"warp"}"#,
            r#"{"action":"sport_sample","mouse":99999}"#,
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

    #[test]
    fn timezone_payload_uses_validated_iana_names() {
        for name in ["system", "Asia/Shanghai", "America/Los_Angeles"] {
            let body = serde_json::json!({"action":"timezone","timezone":name}).to_string();
            assert!(matches!(
                serde_json::from_str::<Request>(&body).unwrap(),
                Request::Timezone { .. }
            ));
        }
        for period in ["calendar_day", "rolling_24h"] {
            let body = serde_json::json!({"action":"today_period","period":period}).to_string();
            assert!(matches!(
                serde_json::from_str::<Request>(&body).unwrap(),
                Request::TodayPeriod { .. }
            ));
        }
    }

    #[test]
    fn display_preferences_accept_only_supported_choices() {
        for scale in ["auto", "100", "115", "130", "150"] {
            let body = serde_json::json!({"action":"ui_scale","scale":scale}).to_string();
            assert!(matches!(
                serde_json::from_str::<Request>(&body).unwrap(),
                Request::UiScale { .. }
            ));
        }
        for layout in ["auto", "compact", "full_size"] {
            let body = serde_json::json!({"action":"keyboard_layout","layout":layout}).to_string();
            assert!(matches!(
                serde_json::from_str::<Request>(&body).unwrap(),
                Request::KeyboardLayout { .. }
            ));
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
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
