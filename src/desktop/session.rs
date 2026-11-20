//! Small, nonpersistent navigation state; no snapshots or entered text.
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct InsightsState {
    #[serde(rename = "shortcut_library", skip_serializing)]
    _legacy_library: bool,
    // Accept older session payloads. The renderer now uses the native platform.
    #[serde(skip_serializing)]
    shortcut_system: String,
    pub shortcut_rows: u16,
    #[serde(rename = "shortcut_show_shift", skip_serializing)]
    _legacy_show_shift: bool,
    pub share_style: String,
    pub share_period: clickpulse::report::Period,
    pub share_day: Option<String>,
    pub share_start: Option<chrono::NaiveDate>,
}
impl Default for InsightsState {
    fn default() -> Self {
        Self {
            _legacy_library: false,
            shortcut_system: String::new(),
            shortcut_rows: 100,
            _legacy_show_shift: false,
            share_style: "blue".into(),
            share_period: Default::default(),
            share_day: None,
            share_start: None,
        }
    }
}
impl InsightsState {
    fn valid(&self) -> bool {
        matches!(
            self.shortcut_system.as_str(),
            "" | "macos" | "windows" | "linux"
        ) && (100..=6000).contains(&self.shortcut_rows)
            && self.shortcut_rows.is_multiple_of(100)
            && matches!(
                self.share_style.as_str(),
                "blue" | "paper" | "ink" | "pine" | "clay" | "lavender" | "rose"
            )
            && self.share_day.as_ref().is_none_or(|day| {
                day.len() == 10 && chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").is_ok()
            })
            && self.share_start.is_none_or(|start| {
                (1..=9999).contains(&chrono::Datelike::year(&start))
                    && (self.share_period != clickpulse::report::Period::Weekly
                        || self
                            .share_day
                            .as_deref()
                            .is_none_or(|end| start.to_string().as_str() <= end))
            })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SessionState {
    pub page: String,
    pub settings_section: String,
    pub trend_days: u16,
    pub calendar_range: String,
    pub history_range: String,
    pub history_page: u32,
    // Old renderers included the chart view here; the preference now owns it.
    #[serde(default, rename = "keyboard_view", skip_serializing)]
    _legacy_keyboard_view: Option<clickpulse::presentation::KeyboardView>,
    // Accept old renderer state; Heatmap no longer has a device tab to restore.
    #[serde(default, skip_serializing)]
    pub heatmap_page: Option<String>,
    #[serde(default)]
    pub sidebar_collapsed: Option<bool>,
    pub scroll_top: u32,
    #[serde(default)]
    pub insights: InsightsState,
}
impl Default for SessionState {
    fn default() -> Self {
        Self {
            page: "overview".into(),
            settings_section: "personalization-settings".into(),
            trend_days: 14,
            calendar_range: "365".into(),
            history_range: "all".into(),
            history_page: 0,
            _legacy_keyboard_view: None,
            heatmap_page: None,
            sidebar_collapsed: None,
            scroll_top: 0,
            insights: InsightsState::default(),
        }
    }
}
impl SessionState {
    pub fn report_selection(
        &self,
        zone: clickpulse::timezone::TimeZonePreference,
    ) -> Option<clickpulse::report::Selection> {
        (self.page == "share").then(|| {
            let day = self
                .insights
                .share_day
                .as_deref()
                .and_then(|day| chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").ok())
                .unwrap_or_else(|| zone.at(chrono::Local::now()).date_naive());
            clickpulse::report::Selection::new(
                day,
                self.insights.share_period,
                self.insights.share_start.filter(|start| *start <= day),
            )
        })
    }
    pub fn wants_shortcuts(&self) -> bool {
        self.page == "shortcuts"
    }
    pub fn valid(&self) -> bool {
        matches!(
            self.page.as_str(),
            "overview" | "history" | "keys" | "shortcuts" | "mouse" | "share" | "settings"
        ) && matches!(
            self.settings_section.as_str(),
            "personalization-settings"
                | "handoff-settings"
                | "capture-settings"
                | "storage-settings"
                | "app-settings"
        ) && matches!(self.trend_days, 14 | 30 | 90)
            && valid_range(&self.calendar_range, true)
            && valid_range(&self.history_range, false)
            && self.history_page <= 100_000
            && self
                .heatmap_page
                .as_deref()
                .is_none_or(|page| matches!(page, "keys" | "mouse"))
            && self.scroll_top <= 10_000_000
            && self.insights.valid()
    }
}
fn valid_range(value: &str, recent: bool) -> bool {
    value == "all"
        || (recent && matches!(value, "90" | "365"))
        || (value.len() == 4 && value.bytes().all(|b| b.is_ascii_digit()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_scope_follows_the_visible_activity() {
        let mut state = SessionState::default();
        assert!(!state.wants_shortcuts());
        state.page = "shortcuts".into();
        assert!(state.wants_shortcuts());
        for page in ["overview", "history", "keys", "mouse", "share", "settings"] {
            state.page = page.into();
            assert!(!state.wants_shortcuts());
        }
    }
    #[test]
    fn insights_restore_choices_without_retaining_search_text() {
        let mut value = serde_json::to_value(SessionState::default()).unwrap();
        value.as_object_mut().unwrap().remove("insights");
        let old: SessionState = serde_json::from_value(value.clone()).unwrap();
        assert!(old.valid());
        assert_eq!(old.insights.share_style, "blue");
        assert_eq!(old.insights.shortcut_rows, 100);
        value["insights"] = serde_json::json!({"shortcut_library":true,"shortcut_system":"linux","shortcut_show_shift":true,"share_style":"ink","share_day":"2026-11-16"});
        let restored: SessionState = serde_json::from_value(value.clone()).unwrap();
        assert!(restored.valid());
        assert_eq!(restored.insights.share_day.as_deref(), Some("2026-11-16"));
        assert!(
            serde_json::to_value(&restored.insights)
                .unwrap()
                .get("shortcut_show_shift")
                .is_none()
        );
        assert!(
            serde_json::to_value(&restored.insights)
                .unwrap()
                .get("shortcut_system")
                .is_none()
        );
        value["insights"]["shortcut_rows"] = 200.into();
        assert!(
            serde_json::from_value::<SessionState>(value.clone())
                .unwrap()
                .valid()
        );
        for invalid in [0, 99, 101, 6100] {
            value["insights"]["shortcut_rows"] = invalid.into();
            assert!(
                !serde_json::from_value::<SessionState>(value.clone())
                    .unwrap()
                    .valid()
            );
        }
        value["insights"]["shortcut_rows"] = 200.into();
        value["insights"]["share_day"] = "2026-02-30".into();
        assert!(
            !serde_json::from_value::<SessionState>(value.clone())
                .unwrap()
                .valid()
        );
        value["insights"]["share_day"] = serde_json::Value::Null;
        value["insights"]["query"] = "private text".into();
        assert!(serde_json::from_value::<SessionState>(value).is_err());
    }
    #[test]
    fn session_state_accepts_navigation_but_rejects_arbitrary_content() {
        let mut state = SessionState::default();
        assert_eq!(state.calendar_range, "365");
        assert!(state.valid());
        state.history_range = "365".into();
        assert!(!state.valid());
        state.history_range = "all".into();
        state.page = "settings".into();
        state.settings_section = "app-settings".into();
        state.calendar_range = "90".into();
        state.history_range = "2026".into();
        assert!(state.valid());
        state.history_range = "90".into();
        assert!(!state.valid());
        state.history_range = "all".into();
        state.scroll_top = u32::MAX;
        assert!(!state.valid());
        state.scroll_top = 0;
        state.page = "<script>".into();
        assert!(!state.valid());
        assert!(
            serde_json::from_value::<SessionState>(serde_json::json!({"page":"settings"})).is_err()
        );
    }

    #[test]
    fn sidebar_choice_roundtrips_and_old_sessions_remain_valid() {
        let mut value = serde_json::to_value(SessionState::default()).unwrap();
        value.as_object_mut().unwrap().remove("sidebar_collapsed");
        let old: SessionState = serde_json::from_value(value.clone()).unwrap();
        assert!(old.valid());
        assert_eq!(old.sidebar_collapsed, None);
        for collapsed in [true, false] {
            value["sidebar_collapsed"] = collapsed.into();
            let state: SessionState = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(state.sidebar_collapsed, Some(collapsed));
            assert_eq!(
                serde_json::to_value(state).unwrap()["sidebar_collapsed"],
                collapsed
            );
        }
        value["sidebar_collapsed"] = "true".into();
        assert!(serde_json::from_value::<SessionState>(value).is_err());
    }

    #[test]
    fn report_style_options_are_valid_session_choices() {
        let mut state = SessionState::default();
        for style in ["blue", "paper", "ink", "pine", "clay", "lavender", "rose"] {
            state.insights.share_style = style.into();
            assert!(state.valid(), "{style}");
        }
        state.insights.share_style = "unknown".into();
        assert!(!state.valid());
    }

    #[test]
    fn old_heatmap_device_state_is_accepted_but_not_republished() {
        let mut value = serde_json::to_value(SessionState::default()).unwrap();
        value.as_object_mut().unwrap().remove("heatmap_page");
        let old: SessionState = serde_json::from_value(value.clone()).unwrap();
        assert!(old.valid());
        assert_eq!(old.heatmap_page, None);
        for page in ["keys", "mouse"] {
            value["heatmap_page"] = page.into();
            let state: SessionState = serde_json::from_value(value.clone()).unwrap();
            assert!(state.valid());
            assert!(
                serde_json::to_value(state)
                    .unwrap()
                    .get("heatmap_page")
                    .is_none()
            );
        }
        value["heatmap_page"] = "share".into();
        assert!(
            !serde_json::from_value::<SessionState>(value)
                .unwrap()
                .valid()
        );
    }
}
