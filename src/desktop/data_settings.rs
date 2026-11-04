use anyhow::Result;
use clickpulse::{history_import::ImportReport, localization::Locale};
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::path::{Path, PathBuf};

fn confirm(
    window: &tao::window::Window,
    locale: Locale,
    key: &str,
    description: String,
    action: &str,
) -> bool {
    let label = locale.text(action);
    let result = MessageDialog::new()
        .set_parent(window)
        .set_title(locale.text(key))
        .set_description(description)
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancelCustom(
            label.clone(),
            locale.text("action.cancel"),
        ))
        .show();
    result == MessageDialogResult::Ok || result == MessageDialogResult::Custom(label)
}

pub(super) fn select_directory(
    window: &tao::window::Window,
    current: &Path,
    locale: Locale,
) -> Result<Option<PathBuf>> {
    // IPC may arrive while another app owns focus; make the parent active before
    // creating its sheet so file selection and keyboard commands can validate.
    window.set_focus();
    let mut dialog = rfd::FileDialog::new()
        .set_parent(window)
        .set_title(locale.text("storage.choose_title"));
    if let Some(parent) = current.parent() {
        dialog = dialog.set_directory(parent);
    }
    let Some(directory) = dialog.pick_folder() else {
        return Ok(None);
    };
    if directory.join("events.sqlite3").canonicalize().ok() == current.canonicalize().ok() {
        return Ok(None);
    }
    if directory.join("events.sqlite3").exists() {
        MessageDialog::new()
            .set_parent(window)
            .set_title(locale.text("storage.title"))
            .set_description(locale.text("storage.destination_exists"))
            .set_level(MessageLevel::Warning)
            .show();
        return Ok(None);
    }
    let description = locale
        .text("storage.move_warning")
        .replace("{path}", &directory.display().to_string());
    if !confirm(
        window,
        locale,
        "storage.choose_title",
        description,
        "action.choose_directory",
    ) {
        return Ok(None);
    }
    Ok(Some(directory))
}

pub(super) fn select_import(window: &tao::window::Window, locale: Locale) -> Option<PathBuf> {
    window.set_focus();
    let source = rfd::FileDialog::new()
        .set_parent(window)
        .set_title(locale.text("storage.import_title"))
        .add_filter("SQLite", &["sqlite3", "sqlite", "db"])
        .pick_file()?;
    let description = locale
        .text("storage.import_warning")
        .replace("{path}", &source.display().to_string());
    if !confirm(
        window,
        locale,
        "storage.import_title",
        description,
        "action.merge_history",
    ) {
        return None;
    }
    Some(source)
}

pub(super) fn show_import_report(
    window: &tao::window::Window,
    locale: Locale,
    result: ImportReport,
) {
    let description = if let Some(backup) = result.backup {
        locale
            .text("storage.imported")
            .replace("{count}", &result.added.total().to_string())
            .replace("{overlap}", &result.overlapping_rows.to_string())
            .replace("{path}", &backup.display().to_string())
    } else {
        locale.text("storage.no_new_history")
    };
    MessageDialog::new()
        .set_parent(window)
        .set_title(locale.text("storage.import_title"))
        .set_description(description)
        .show();
}

pub(super) fn show_error(window: &tao::window::Window, locale: Locale, error: &str) {
    MessageDialog::new()
        .set_parent(window)
        .set_title(locale.text("storage.title"))
        .set_description(format!("{}\n\n{error}", locale.text("storage.failed")))
        .set_level(MessageLevel::Error)
        .show();
}
