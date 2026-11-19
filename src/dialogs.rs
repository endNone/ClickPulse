//! Message dialogs must not depend on an optional external desktop command.
#[cfg(target_os = "linux")]
pub use linux::MessageDialog;
#[cfg(not(target_os = "linux"))]
pub use rfd::MessageDialog;
pub use rfd::{MessageButtons, MessageDialogResult, MessageLevel};

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::localization::Locale;
    use gtk::prelude::*;
    use tao::platform::unix::WindowExtUnix;

    #[derive(Default)]
    pub struct MessageDialog {
        title: String,
        description: String,
        level: MessageLevel,
        buttons: MessageButtons,
        parent: Option<gtk::ApplicationWindow>,
        locale: Option<Locale>,
        default_button: Option<usize>,
    }
    impl MessageDialog {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn set_title(mut self, value: impl Into<String>) -> Self {
            self.title = value.into();
            self
        }
        pub fn set_description(mut self, value: impl Into<String>) -> Self {
            self.description = value.into();
            self
        }
        pub fn set_level(mut self, value: MessageLevel) -> Self {
            self.level = value;
            self
        }
        pub fn set_buttons(mut self, value: MessageButtons) -> Self {
            self.buttons = value;
            self
        }
        pub fn set_locale(mut self, locale: Locale) -> Self {
            self.locale = Some(locale);
            self
        }
        pub fn set_default_button(mut self, index: usize) -> Self {
            self.default_button = Some(index);
            self
        }
        pub fn set_parent(mut self, window: &tao::window::Window) -> Self {
            self.parent = Some(window.gtk_window().clone());
            self
        }
        pub fn show(self) -> MessageDialogResult {
            // Startup recovery runs before Tao creates the GTK event loop.
            if let Err(error) = gtk::init() {
                eprintln!(
                    "Unable to display dialog: {error}: {}: {}",
                    self.title, self.description
                );
                return MessageDialogResult::Cancel;
            }
            let dialog = gtk::MessageDialog::builder()
                .modal(true)
                .message_type(match self.level {
                    MessageLevel::Info => gtk::MessageType::Info,
                    MessageLevel::Warning => gtk::MessageType::Warning,
                    MessageLevel::Error => gtk::MessageType::Error,
                })
                .text(&self.title)
                .secondary_text(&self.description)
                .build();
            if let Some(parent) = &self.parent {
                dialog.set_transient_for(Some(parent));
            }
            let buttons = button_results(
                self.buttons,
                self.locale
                    .unwrap_or_else(crate::presentation::Preferences::locale_for_error),
            );
            for (index, (label, _)) in buttons.iter().enumerate() {
                // Labels are literal translated text, not GTK underscore mnemonics.
                dialog.add_action_widget(
                    &gtk::Button::builder()
                        .label(label)
                        .can_default(true)
                        .visible(true)
                        .build(),
                    gtk::ResponseType::Other(index as u16),
                );
            }
            // Destructive choices stay opt-in; a caller can make safe Retry the default.
            let default = self
                .default_button
                .filter(|i| *i < buttons.len())
                .unwrap_or(buttons.len() - 1);
            dialog.set_default_response(gtk::ResponseType::Other(default as u16));
            dialog.set_focus(
                dialog
                    .widget_for_response(gtk::ResponseType::Other(default as u16))
                    .as_ref(),
            );
            let response = dialog.run();
            unsafe {
                dialog.destroy();
            }
            match response {
                gtk::ResponseType::Other(index) => buttons
                    .into_iter()
                    .nth(index as usize)
                    .map(|(_, result)| result)
                    .unwrap_or_default(),
                _ => MessageDialogResult::Cancel,
            }
        }
    }
    fn button_results(
        buttons: MessageButtons,
        locale: Locale,
    ) -> Vec<(String, MessageDialogResult)> {
        use MessageDialogResult as R;
        let cancel = locale.text("action.cancel");
        let ok = locale.text("action.ok");
        let yes = locale.text("action.yes");
        let no = locale.text("action.no");
        match buttons {
            MessageButtons::Ok => vec![(ok, R::Ok)],
            MessageButtons::OkCancel => vec![(ok, R::Ok), (cancel, R::Cancel)],
            MessageButtons::YesNo => vec![(yes, R::Yes), (no, R::No)],
            MessageButtons::YesNoCancel => vec![(yes, R::Yes), (no, R::No), (cancel, R::Cancel)],
            MessageButtons::OkCustom(a) => vec![(a.clone(), R::Custom(a))],
            MessageButtons::OkCancelCustom(a, b) => {
                vec![(a.clone(), R::Custom(a)), (b.clone(), R::Custom(b))]
            }
            MessageButtons::YesNoCancelCustom(a, b, c) => vec![
                (a.clone(), R::Custom(a)),
                (b.clone(), R::Custom(b)),
                (c.clone(), R::Custom(c)),
            ],
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        #[ignore = "requires an isolated GTK display (xvfb-run), never the user's desktop"]
        fn native_dialogs_work_without_zenity() {
            gtk::init().unwrap();
            for (response, expected) in [(0, "Retry"), (1, "Choose"), (2, "Quit")] {
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(30),
                    move || {
                        let dialog = gtk::Window::list_toplevels()
                            .into_iter()
                            .find_map(|window| window.downcast::<gtk::MessageDialog>().ok())
                            .unwrap();
                        for (index, label) in ["Retry", "Choose", "Quit"].iter().enumerate() {
                            let button = dialog
                                .widget_for_response(gtk::ResponseType::Other(index as u16))
                                .unwrap()
                                .downcast::<gtk::Button>()
                                .unwrap();
                            assert_eq!(button.label().as_deref(), Some(*label));
                        }
                        dialog.response(gtk::ResponseType::Other(response));
                    },
                );
                assert_eq!(
                    MessageDialog::new()
                        .set_title("Recovery test")
                        .set_description("Isolated test only")
                        .set_buttons(MessageButtons::YesNoCancelCustom(
                            "Retry".into(),
                            "Choose".into(),
                            "Quit".into()
                        ))
                        .show(),
                    MessageDialogResult::Custom(expected.into())
                );
            }
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(30), || {
                let dialog = gtk::Window::list_toplevels()
                    .into_iter()
                    .find_map(|window| window.downcast::<gtk::MessageDialog>().ok())
                    .unwrap();
                dialog.response(gtk::ResponseType::DeleteEvent);
            });
            assert_eq!(
                MessageDialog::new().set_title("Close test").show(),
                MessageDialogResult::Cancel
            );
            // Exercise the GTK default response, not a direct response ID.
            for default in [0, 2] {
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(30),
                    move || {
                        let dialog = gtk::Window::list_toplevels()
                            .into_iter()
                            .find_map(|window| window.downcast::<gtk::MessageDialog>().ok())
                            .unwrap();
                        let button = dialog
                            .widget_for_response(gtk::ResponseType::Other(default as u16))
                            .unwrap()
                            .downcast::<gtk::Button>()
                            .unwrap();
                        assert!(button.is_visible());
                        assert!(button.is_mapped());
                        assert!(!button.uses_underline());
                        assert_eq!(dialog.default_widget().as_ref(), Some(button.upcast_ref()));
                        assert!(dialog.activate_default());
                    },
                );
                let result = MessageDialog::new()
                    .set_locale(Locale::English)
                    .set_buttons(MessageButtons::YesNoCancelCustom(
                        "Re_try".into(),
                        "Choose".into(),
                        "Quit".into(),
                    ))
                    .set_default_button(default)
                    .show();
                assert_eq!(
                    result,
                    MessageDialogResult::Custom(
                        if default == 0 { "Re_try" } else { "Quit" }.into()
                    )
                );
            }
        }
        #[test]
        fn generic_buttons_use_the_selected_app_language() {
            for locale in [
                Locale::Chinese,
                Locale::TraditionalChinese,
                Locale::Japanese,
                Locale::Korean,
                Locale::English,
                Locale::French,
                Locale::German,
                Locale::Spanish,
            ] {
                let buttons = button_results(MessageButtons::YesNoCancel, locale);
                assert_eq!(
                    buttons[0],
                    (locale.text("action.yes"), MessageDialogResult::Yes)
                );
                assert_eq!(
                    buttons[1],
                    (locale.text("action.no"), MessageDialogResult::No)
                );
                assert_eq!(
                    buttons[2],
                    (locale.text("action.cancel"), MessageDialogResult::Cancel)
                );
                assert_eq!(
                    button_results(MessageButtons::Ok, locale)[0].0,
                    locale.text("action.ok")
                );
            }
        }
        #[test]
        fn recovery_buttons_return_their_actual_custom_labels() {
            let buttons = button_results(
                MessageButtons::YesNoCancelCustom("Retry".into(), "Choose".into(), "Quit".into()),
                Locale::English,
            );
            assert_eq!(
                buttons
                    .iter()
                    .map(|(_, result)| result.clone())
                    .collect::<Vec<_>>(),
                [
                    MessageDialogResult::Custom("Retry".into()),
                    MessageDialogResult::Custom("Choose".into()),
                    MessageDialogResult::Custom("Quit".into())
                ]
            );
        }
    }
}
