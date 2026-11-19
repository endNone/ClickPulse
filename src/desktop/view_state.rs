//! Persistence and renderer invalidation have different lifetimes.
use clickpulse::presentation::Preferences;

/// GTK restoration is asynchronous: pending Open must outrank the stale
/// minimized flag, so a released renderer can be created before showing it.
pub(super) fn renderer_active(
    visible: bool,
    native_visible: bool,
    minimized: bool,
    presenting: bool,
) -> bool {
    visible && (presenting || (native_visible && !minimized))
}

/// Native window close and the custom close button share one policy.
/// Explicit Quit is dispatched separately and always requests shutdown.
pub(super) fn close_action(
    demo: bool,
    macos: bool,
    preference: clickpulse::presentation::CloseAction,
) -> clickpulse::presentation::CloseAction {
    use clickpulse::presentation::CloseAction;
    if demo {
        CloseAction::Quit
    } else if macos {
        CloseAction::Hide
    } else {
        preference
    }
}

pub(super) struct PreferencePersistence(Preferences);

impl PreferencePersistence {
    pub fn new(saved: &Preferences) -> Self {
        Self(saved.clone())
    }

    pub fn save_if_changed(
        &mut self,
        current: &Preferences,
        save: impl FnOnce(&Preferences) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        if self.0 != *current {
            save(current)?;
            self.0 = current.clone();
        }
        Ok(())
    }
}

/// Only acknowledge successful delivery. Reset when a renderer is recreated.
#[derive(Default)]
pub(super) struct Published<T>(Option<T>);

impl<T: PartialEq> Published<T> {
    pub fn reset(&mut self) {
        self.0 = None;
    }
    pub fn send(&mut self, value: T, send: impl FnOnce(&T) -> wry::Result<()>) {
        if self.0.as_ref() != Some(&value) && send(&value).is_ok() {
            self.0 = Some(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn pending_open_resumes_a_hidden_minimized_renderer() {
        for native_visible in [false, true] {
            assert!(renderer_active(true, native_visible, true, true));
            assert!(!renderer_active(true, native_visible, true, false));
            assert!(!renderer_active(false, native_visible, true, true));
        }
        assert!(renderer_active(true, true, false, false));
        assert!(!renderer_active(true, false, false, false));
    }

    #[test]
    fn window_close_obeys_platform_policy_and_demo_exits() {
        use clickpulse::presentation::CloseAction;
        for choice in [CloseAction::Ask, CloseAction::Hide, CloseAction::Quit] {
            assert_eq!(close_action(false, false, choice), choice);
            assert_eq!(close_action(false, true, choice), CloseAction::Hide);
            assert_eq!(close_action(true, false, choice), CloseAction::Quit);
            assert_eq!(close_action(true, true, choice), CloseAction::Quit);
        }
    }

    #[test]
    fn view_refreshes_do_not_persist_preferences_and_failed_saves_retry() {
        let mut prefs = Preferences::default();
        let mut persisted = PreferencePersistence::new(&prefs);
        let writes = Cell::new(0);
        let save = |_: &Preferences| {
            writes.set(writes.get() + 1);
            Ok(())
        };
        for _ in 0..100 {
            persisted.save_if_changed(&prefs, save).unwrap();
        }
        assert_eq!(writes.get(), 0);
        prefs.compact = !prefs.compact;
        assert!(
            persisted
                .save_if_changed(&prefs, |_| anyhow::bail!("unavailable"))
                .is_err()
        );
        persisted.save_if_changed(&prefs, save).unwrap();
        persisted.save_if_changed(&prefs, save).unwrap();
        assert_eq!(writes.get(), 1);
    }

    #[test]
    fn unchanged_status_is_sent_once_per_renderer() {
        let mut status = Published::default();
        let sends = Cell::new(0);
        let send = |_: &bool| {
            sends.set(sends.get() + 1);
            Ok(())
        };
        status.send(false, send);
        status.send(false, send);
        status.send(true, send);
        status.reset();
        status.send(true, send);
        assert_eq!(sends.get(), 3);
    }
}
