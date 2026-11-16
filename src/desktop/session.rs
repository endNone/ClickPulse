//! Small, nonpersistent navigation state; no snapshots or entered text.
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SessionState {
    pub page: String,
    pub settings_section: String,
    pub trend_days: u16,
    pub calendar_range: String,
    pub history_range: String,
    pub history_page: u32,
    pub keyboard_view: String,
    pub scroll_top: u32,
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
            keyboard_view: "layout".into(),
            scroll_top: 0,
        }
    }
}
impl SessionState {
    pub fn valid(&self) -> bool {
        matches!(
            self.page.as_str(),
            "overview" | "history" | "keys" | "settings"
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
            && matches!(self.keyboard_view.as_str(), "layout" | "treemap")
            && self.scroll_top <= 10_000_000
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
}
