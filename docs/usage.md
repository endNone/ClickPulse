# Usage

## First launch

Install the complete app using the [macOS guide](install-macos.md) or [Windows guide](install-windows.md). Ready-built packages need no developer tools.

On macOS, allow the app to open and enable Input Monitoring for the installed copy. These are separate steps. Reopen ClickPulse when prompted, then check **Recording** and increasing counts. Ad-hoc updates may require refreshing the permission entry; follow the macOS guide.

On Windows, extract and keep the complete portable program folder. Install WebView2 Runtime if startup reports it missing, then confirm counts with the physical keyboard and mouse.

## Everyday use

| Page or action | Purpose |
| --- | --- |
| Overview | Today, the last seven days, all-time and rolling counts, trends and the activity calendar |
| History | Daily records, year filtering and pages of older activity |
| Keyboard heatmap | Physical-key usage; switch to the treemap to compare block areas |
| Settings | Themes, language, pause, recording access, launch at login and data locations |
| CSV export | Save all daily history without overwriting an existing file |

A held key counts once until released. Mouse counts track button presses, not movement, release or scrolling. Hidden extended keys remain in totals even when the macOS keyboard layout does not show them.

## Themes and language

Choose **Blue**, **Teal**, **Violet** or **Graphite** in Settings. The preference is saved and shared by charts, keyboard views and mini mode.

Language follows the system by default. Switch to English or Simplified Chinese in Settings to update labels, menus, dates and number formatting immediately.

Dropdown menus are rendered inside the app: use arrow keys to browse, Enter or Space to confirm, Escape to cancel, or Tab to confirm and move to the next control.

## Mini mode and background recording

Mini mode is a fixed **320 × 208 logical-size** floating window. Screen density determines its actual pixels. Drag the brand area to move it; pin, hide or expand with the top controls, and pause or resume at the bottom.

On macOS, closing the main window or hiding mini mode keeps recording. Reopen from Applications, the Dock or **Open statistics** in the menu bar. Choose **Quit ClickPulse** from the menu bar to save pending records and stop.

On Windows, drag the blank header area to move the main window; double-click it to maximize or restore. Closing the window or using its Quit action offers **Minimize to tray** or **Save and quit**. Select **Remember my choice** to reuse the decision, or change it later in **Settings → Close behavior**. **Quit ClickPulse** in the tray menu always saves and exits. Double-click the tray icon to reopen statistics; right-click it for the menu.

Launch at login starts with the window hidden. Quitting saves remaining records and stops capture.

## Data locations and backups

| Platform | Statistics and preferences | Log |
| --- | --- | --- |
| macOS | `~/Library/Application Support/ClickPulse/` | `~/Library/Logs/clickpulse.log` |
| Windows | `%LOCALAPPDATA%\ClickPulse\` | `logs\clickpulse.log` inside the data folder |

The main database is `events.sqlite3`. Use SQLite backup for a running database, or quit normally before copying it. Copying only the main file while the app is running may omit WAL records.

Diagnostic logs are limited to 10 MiB per file, with up to three copies named `clickpulse.log.1`, `.2` and `.3`; `.1` is the newest rotated copy. Logs are not needed to restore activity history.

Changing the program folder does not move or delete history. If you move the app after enabling launch at login, disable and re-enable that setting from its new location. Uninstalling program files retains statistics and preferences.

## Existing records

Normal startup uses the standard ClickPulse data directory and retains existing app history. It does not import another recorder’s database or take over its service automatically.

Advanced read-only viewing uses `--source-db`; explicit migration uses `migrate --source ... --output ...` to create a new destination. Stop the old recorder yourself and keep a complete backup first. Migration does not overwrite an existing database, merge overlapping records or stop other services. See [Development](development.md) and [Data and privacy](data-and-privacy.md).
