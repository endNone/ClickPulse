//! Standard responder-chain actions make WebKit selections usable with Cmd+C.
use anyhow::Result;
use clickpulse::localization::Locale;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub(super) struct EditMenu {
    _menu: Menu,
    edit: Submenu,
    undo: PredefinedMenuItem,
    redo: PredefinedMenuItem,
    cut: PredefinedMenuItem,
    copy: PredefinedMenuItem,
    paste: PredefinedMenuItem,
    select_all: PredefinedMenuItem,
}

impl EditMenu {
    pub(super) fn new(locale: Locale, quit: &MenuItem) -> Result<Self> {
        let menu = Menu::new();
        let app = Submenu::new("ClickPulse", true);
        // Use the same event-driven Quit as the tray so Cmd+Q drains the writer.
        app.append(quit)?;
        let edit = Submenu::new(locale.text("native.edit"), true);
        let undo = PredefinedMenuItem::undo(Some(&locale.text("native.undo")));
        let redo = PredefinedMenuItem::redo(Some(&locale.text("native.redo")));
        let cut = PredefinedMenuItem::cut(Some(&locale.text("native.cut")));
        let copy = PredefinedMenuItem::copy(Some(&locale.text("native.copy")));
        let paste = PredefinedMenuItem::paste(Some(&locale.text("native.paste")));
        let select_all = PredefinedMenuItem::select_all(Some(&locale.text("native.select_all")));
        edit.append_items(&[
            &undo,
            &redo,
            &PredefinedMenuItem::separator(),
            &cut,
            &copy,
            &paste,
            &PredefinedMenuItem::separator(),
            &select_all,
        ])?;
        menu.append_items(&[&app, &edit])?;
        menu.init_for_nsapp();
        Ok(Self {
            _menu: menu,
            edit,
            undo,
            redo,
            cut,
            copy,
            paste,
            select_all,
        })
    }

    pub(super) fn set_locale(&self, locale: Locale) {
        self.edit.set_text(locale.text("native.edit"));
        self.undo.set_text(locale.text("native.undo"));
        self.redo.set_text(locale.text("native.redo"));
        self.cut.set_text(locale.text("native.cut"));
        self.copy.set_text(locale.text("native.copy"));
        self.paste.set_text(locale.text("native.paste"));
        self.select_all.set_text(locale.text("native.select_all"));
    }
}
