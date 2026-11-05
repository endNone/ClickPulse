# Usage

This guide describes current development source. Public preview build 55 does not yet include Mini Sport, the expanded language options or the new data-management controls. Current source removes CSV export, including the toolbar, history-page action and command-line command.

## First launch

Install the complete app using the [macOS guide](install-macos.md) or [Windows guide](install-windows.md). Ready-built packages need no developer tools.

On macOS, allow the app to open and enable Input Monitoring for the installed copy. These are separate steps. Reopen ClickPulse when prompted, then check **Recording** and increasing counts. Ad-hoc updates may require refreshing the permission entry; follow the macOS guide.

On Windows, extract and keep the complete portable program folder. Install WebView2 Runtime if startup reports it missing, then confirm counts with the physical keyboard and mouse.

## Everyday use

| Page or action | Purpose |
| --- | --- |
| Overview | Today, the last seven days, all-time and rolling counts, 14/30/90-day trends and the activity calendar |
| History | Daily records, year filtering and pages of older activity |
| Keyboard heatmap | Physical-key usage; switch to the treemap to compare block areas |
| Mini Sport | Optional ranks and effects for combined keyboard and mouse speed inside Mini |
| Settings | Themes, language, recording access, launch at login and data locations |

A held key counts once until released. Mouse counts track button presses, not movement, release or scrolling. Hidden extended keys remain in totals even when the macOS keyboard layout does not show them.

Selecting **Settings** expands section shortcuts in the sidebar. Choose a heading to jump to its controls; unavailable groups are hidden.

## Sport in Mini mode

Sport is off by default, including its Mini switch. In **Settings → Personalization**, enable **Sport mode** to show the switch. Sport appears only in the existing **320 × 208** Mini window. Enter **Mini mode** with the top-right button, then use the lightning switch in its top-right toolbar to enable or disable it. Combined keyboard and mouse speed replaces the status badge with the current rank and presses per second (CPS); normal activity totals remain visible.

Rank C is below 2 CPS, B starts at 2, A at 5, S at 8, and red SS at 12. Speed uses a rolling one-second window and naturally falls back when you stop clicking. Each key press and mouse button press contributes once; native held-key repeat filtering is unchanged. CPS represents their combined presses per second.

Sport uses **Warp**, a projected runway that accelerates through light gates. Red SS retains particles without shaking the window. Turning off **Sport mode** in **Settings → Personalization** hides its Mini switch and disables Sport. Your choice is remembered independently of the ordinary app theme.

Sport stops sampling and animating when disabled, hidden, expanded into the full window, or not visible. System reduced-motion preferences suppress its animation while keeping the rank and speed readable. The lightweight counter feed updates around every 250 ms; ordinary statistics retain their two-second refresh.

## Themes and language

Choose **Blue**, **Teal**, **Violet** or **Graphite** in Settings. The preference is saved and shared by charts, keyboard views and mini mode.

Language follows the system by default, unless you have saved a manual choice. English, Simplified Chinese, Traditional Chinese, Japanese, Korean, French, German and Spanish are available in Settings. Regional tags such as zh-Hant-HK, ja-JP and ko-KR map to their supported language. The first supported language in your system preference list is used; unsupported language lists fall back to English. Changing the language updates labels, menus, dates, number formatting and Mini Sport controls without resetting your statistics or skin.

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

## Save folder and history import

Settings → Data management offers **Change folder**, **Show in Finder / Open folder**, and **Import database**. Choosing a local folder copies and verifies all saved statistics before switching; the old database is retained. A folder containing `events.sqlite3` is refused to avoid overwriting history. Window preferences, logs, startup registration and `storage.json` remain in the standard profile directory. If the selected disk is unavailable on a later launch, startup reports that problem instead of creating an empty database.

Import accepts ClickPulse SQLite history, including the first public release’s `events.sqlite3`; CSV is not supported for history import. The app first saves a full SQLite backup in the current statistics folder’s `backups/` directory. After a successful import it retains that restore point and the four newest older automatic import backups. Failed imports do not prune backups; unrelated files, directories and symbolic links are left alone. Backups with SQLite journal files, unrecognized headers or noncanonical names are also retained, so more than five files may remain when cleanup would be unsafe. It merges each minute/device/key-or-button identity using the larger count, so importing the same history twice does not double totals. Independent records overlapping that same identity cannot be distinguished and may be undercounted; review the conflict warning before proceeding. New minute records are assigned to local calendar dates. Unsupported schemas, incomplete minute history and incompatible legacy key-encoding boundaries are rejected. The source is opened read-only as a consistent SQLite snapshot; other recording services are not stopped. Import and folder changes run in the background with a visible processing notice. You can continue browsing, while further data operations are disabled until completion. A requested Quit waits for the operation to commit or roll back. A locked backup fails after bounded no-progress retries instead of waiting indefinitely.


Select interface text, paths or numbers and use **⌘C** on macOS or **Ctrl+C** on Windows. macOS also provides **Edit → Copy** and **Select All**. Selected statistics remain stable during refresh until the selection is cleared. Mini’s brand area still drags the window.
