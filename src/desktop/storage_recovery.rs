//! Recovery is available during foreground and background startup, before capture.
use anyhow::Result;
use clickpulse::dialogs::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use clickpulse::{
    localization::Locale,
    storage::startup::{self, Failure},
};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
enum Choice {
    Primary,
    Choose,
    Quit,
}

fn choice(result: MessageDialogResult, action: &str, choose: &str) -> Choice {
    match result {
        MessageDialogResult::Yes => Choice::Primary,
        MessageDialogResult::No => Choice::Choose,
        MessageDialogResult::Custom(label) if label == action => Choice::Primary,
        MessageDialogResult::Custom(label) if label == choose => Choice::Choose,
        _ => Choice::Quit,
    }
}

#[derive(Debug, PartialEq)]
enum Primary {
    Default,
    Restart,
    Retry,
}
fn primary(failure: &Failure) -> Primary {
    match failure.reason {
        "storage.recovery_missing" | "storage.recovery_config" => Primary::Default,
        "storage.recovery_database"
            if failure
                .path
                .as_ref()
                .is_some_and(|p| p.is_file() && !p.is_symlink()) =>
        {
            Primary::Restart
        }
        _ => Primary::Retry,
    }
}
impl Primary {
    fn label(&self) -> &'static str {
        match self {
            Self::Default => "storage.recovery_default",
            Self::Restart => "storage.recovery_restart",
            Self::Retry => "storage.recovery_retry",
        }
    }
}

// Remember what failed so Retry repeats the same selection, including publishing
// its locator. A retry must not silently return to a different database.
enum Attempt {
    Configured,
    Selected { path: PathBuf, existing: bool },
}
impl Attempt {
    fn run(&self, profile: &Path) -> std::result::Result<PathBuf, Failure> {
        let path = match self {
            Self::Configured => startup::inspect(profile)?,
            Self::Selected { path, existing } => {
                let result = if *existing {
                    startup::select_existing(profile, path)
                } else {
                    startup::select(profile, path)
                };
                result.map_err(|error| Failure::database(path.clone(), error))?;
                path.clone()
            }
        };
        startup::prepare_database(&path).map_err(|error| Failure::database(path.clone(), error))?;
        Ok(path)
    }
}

pub(super) fn resolve(locale: Locale) -> Result<Option<PathBuf>> {
    let profile = clickpulse::platform::data_dir()?;
    let mut attempt = Attempt::Configured;
    let mut state = attempt.run(&profile);
    loop {
        let failure = match state {
            Ok(path) => return Ok(Some(path)),
            Err(failure) => failure,
        };
        let primary = primary(&failure);
        let action = locale.text(primary.label());
        let choose = locale.text("storage.recovery_choose");
        let path = failure.path.as_deref().unwrap_or(&profile);
        let dialog = MessageDialog::new()
            .set_title(locale.text("error.startup"))
            .set_description(format!(
                "{}\n\n{}\n\n{}\n\n{:#}",
                locale.text(failure.reason),
                path.display(),
                locale.text(if primary == Primary::Restart {
                    "storage.recovery_preserve"
                } else {
                    "storage.recovery_unchanged"
                }),
                failure.error
            ))
            .set_level(MessageLevel::Warning)
            .set_buttons(MessageButtons::YesNoCancelCustom(
                action.clone(),
                choose.clone(),
                locale.text("native.quit"),
            ));
        #[cfg(target_os = "linux")]
        let dialog = dialog
            .set_locale(locale)
            .set_default_button(if primary == Primary::Retry { 0 } else { 2 });
        let result = dialog.show();
        match choice(result, &action, &choose) {
            Choice::Quit => return Ok(None),
            Choice::Choose => {
                let Some(directory) = rfd::FileDialog::new().set_title(&choose).pick_folder()
                else {
                    state = Err(failure);
                    continue;
                };
                attempt = Attempt::Selected {
                    path: directory.join("events.sqlite3"),
                    existing: true,
                };
            }
            Choice::Primary => match primary {
                Primary::Retry => {}
                Primary::Default => {
                    attempt = Attempt::Selected {
                        path: profile.join("events.sqlite3"),
                        existing: false,
                    }
                }
                Primary::Restart => {
                    if !super::confirmation::confirm(
                        None,
                        locale,
                        &action,
                        &format!(
                            "{}\n\n{}",
                            locale.text("storage.recovery_preserve"),
                            path.display()
                        ),
                    ) {
                        state = Err(failure);
                        continue;
                    }
                    if let Err(error) = startup::preserve_and_restart(path) {
                        state = Err(Failure::database(path.to_owned(), error));
                        continue;
                    }
                    attempt = Attempt::Selected {
                        path: path.to_owned(),
                        existing: false,
                    };
                }
            },
        }
        state = attempt.run(&profile);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_yes_is_the_primary_action() {
        assert_eq!(
            choice(MessageDialogResult::Yes, "Restart", "Choose"),
            Choice::Primary
        );
    }
    #[test]
    fn system_no_chooses_a_folder() {
        assert_eq!(
            choice(MessageDialogResult::No, "Restart", "Choose"),
            Choice::Choose
        );
    }
    #[test]
    fn cancel_closes_without_changing_data() {
        assert_eq!(
            choice(MessageDialogResult::Cancel, "Restart", "Choose"),
            Choice::Quit
        );
    }
    #[test]
    fn custom_results_match_only_the_exact_known_labels() {
        for (label, expected) in [
            ("Restart", Choice::Primary),
            ("Choose", Choice::Choose),
            ("Quit", Choice::Quit),
            ("Unknown", Choice::Quit),
        ] {
            assert_eq!(
                choice(
                    MessageDialogResult::Custom(label.into()),
                    "Restart",
                    "Choose"
                ),
                expected
            );
        }
        assert_eq!(
            choice(MessageDialogResult::Ok, "Restart", "Choose"),
            Choice::Quit
        );
    }
    #[test]
    fn only_verified_corruption_offers_restart() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.sqlite3");
        std::fs::write(&path, "corrupt").unwrap();
        for reason in [
            "storage.recovery_busy",
            "storage.recovery_access",
            "storage.recovery_newer",
        ] {
            assert_eq!(
                primary(&Failure {
                    path: Some(path.clone()),
                    reason,
                    error: anyhow::anyhow!("test")
                }),
                Primary::Retry
            );
        }
        assert_eq!(
            primary(&startup::inspect_database(&path).unwrap_err()),
            Primary::Restart
        );
    }
}
