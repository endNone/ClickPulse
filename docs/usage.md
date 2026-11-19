# Usage

This guide follows the current source, including changes awaiting release. Use database backups and history import to manage your data.

## First launch

Install the complete app using the [macOS guide](install-macos.md), [Windows guide](install-windows.md) or [Linux guide](install-linux.md). Ready-built packages need no developer tools.

On macOS, allow the app to open and enable Input Monitoring for the installed copy. These are separate steps. Reopen ClickPulse when prompted, then check **Recording** and increasing counts. Ad-hoc updates may require refreshing the permission entry; follow the macOS guide.

On Windows, extract and keep the complete portable program folder. Install WebView2 Runtime if startup reports it missing, then confirm counts with the physical keyboard and mouse.

## Everyday use

| Page or action | Purpose |
| --- | --- |
| Overview | Today, the last seven days, all-time and rolling counts, 14/30/90-day trends and the activity calendar |
| History | Daily records, year filtering and pages of older activity |
| Heatmap | Keyboard usage above mouse-button distribution; switch the keyboard to Treemap to compare block areas |
| Shortcuts | Most-used combinations, their share and the shortcut encyclopedia |
| Share | A daily image with totals, a seven-day trend and the most-used shortcut when dates align |
| Mini Sport | Optional ranks and effects for combined keyboard and mouse speed inside Mini |
| Settings | Themes, language, recording access, launch at login and data locations |

A held key counts once until released. Mouse counts track button presses, not movement, release or scrolling. Use **Compact / Full** above the heatmap to choose a layout. Your choice survives restarting the app. **Settings → Personalization → Keyboard layout** offers **Automatic**, **Compact** and **Full size**. Automatic shows navigation and number-pad keys when their recorded counts are present. The full Mac layout includes F13–F19; Windows and Linux show PrtSc, ScrLk and Pause above navigation. Recognized keys outside the selected layout remain available in **Treemap** without adding extra rows. **Other keys** shows counts that cannot be identified.

Keys keep their size when you resize the window. Scroll sideways in narrower windows, or choose an explicit **Interface size** to enlarge them.

**Heatmap** shows Keyboard above Mouse on one scrollable page. Opening it preserves the window and sidebar state. Use the top-bar button or **⌘B** (**Ctrl+B** on Windows and Linux) to hide or show the sidebar. The shortcut stays inactive while editing text or using Mini. Your sidebar choice survives changing pages, hiding and reopening the interface.

Selecting **Settings** expands section shortcuts in the sidebar. Choose a heading to jump to its controls; unavailable groups are hidden.

**Activity mix** separates keyboard use, left clicks and right clicks. Middle clicks, side buttons and other mouse buttons appear separately when used. Older records without button details remain visible as **Unclassified clicks**. Hover over a segment or legend entry for its share.

## Sport in Mini mode

Sport is off by default, including its Mini switch. In **Settings → Personalization**, enable **Sport mode** to show the switch. Sport appears only in the existing **320 × 208** Mini window. Enter **Mini mode** with the top-right button, then use the lightning switch in its top-right toolbar to enable or disable it. Combined keyboard and mouse speed replaces the status badge with the current rank and presses per second (CPS); normal activity totals remain visible.

Rank C is below 2 CPS, B starts at 2, A at 5, S at 8, and red SS at 12. Speed uses a rolling one-second window and naturally falls back when you stop clicking. Each key press and mouse button press contributes once; native held-key repeat filtering is unchanged. CPS represents their combined presses per second.

Sport uses **Warp**, a projected runway that accelerates through light gates. Red SS retains particles without shaking the window. Turning off **Sport mode** in **Settings → Personalization** hides its Mini switch and disables Sport. Your choice is remembered independently of the ordinary app theme.

Sport stops sampling and animating when disabled, hidden, expanded into the full window, or not visible. System reduced-motion preferences suppress its animation while keeping the rank and speed readable. The lightweight counter feed updates around every 250 ms; ordinary statistics refresh every 0.5 seconds.

## Activity calendar

The calendar opens on **Past year**: the latest 365 days, ending today. Days without records appear as pale squares, including dates before your first recording. The grid continues across January without splitting into separate years. Choose **All history** or a year to see older records.

## Themes and language

Choose **Blue**, **Teal**, **Violet** or **Graphite** in Settings. The preference is saved and shared by charts, keyboard views and mini mode.

**Interface size** follows the window width automatically, or uses your chosen 100%, 115%, 130% or 150%. Mini keeps its compact size. Full windows fit the display's usable area; small windows keep their contents scrollable.

Language follows the system by default, unless you have saved a manual choice. English, Simplified Chinese, Traditional Chinese, Japanese, Korean, French, German and Spanish are available in Settings. Regional tags such as zh-Hant-HK, ja-JP and ko-KR map to their supported language. The first supported language in your system preference list is used; unsupported language lists fall back to English. Changing the language updates labels, menus, dates, number formatting and Mini Sport controls without resetting your statistics or skin.

Dropdown menus are rendered inside the app: use arrow keys to browse, Enter or Space to confirm, Escape to cancel, or Tab to confirm and move to the next control.

## Check for updates

In **Settings → Application → App updates**, click **Check for updates**, or leave daily checks enabled. In 0.3.1, a newer version downloads automatically in the background. When it is ready, click **Update & restart** to save pending activity, install and reopen. Downloading alone never installs or restarts the app.

Turn off **Check automatically** to stop scheduled checks; manual checks remain available. Checks run once per 24-hour window while the app is open, including in the background. The last attempt is remembered across restarts. Temporary connection failures retry a few times without interrupting recording.

Scoop and WinGet installations show their own upgrade command in Settings. Quit ClickPulse before running it. If a manual installation cannot update in its current location, Settings explains how to proceed. Demo and read-only sessions do not download updates or change your saved update preference.

The updater replaces program files and keeps a recovery copy; history, save-folder settings and preferences stay in place. If saving or verification fails, the update is cancelled. A read-only installation or an independent/demo session offers the release download page instead. On macOS, an ad-hoc signed update usually invalidates the existing Input Monitoring grant even though its switch still appears on. After restarting, ClickPulse shows **Allow Input Monitoring again after the update**: quit ClickPulse, remove its entry in Privacy & Security → Input Monitoring with **−**, add the App again with **+**, then reopen it. See [refreshing the permission entry](install-macos.md#4-update-and-refresh-access). The app shows the public version; `--version` includes its internal build for diagnostics.

The updater is included starting with 0.2.1. Install 0.2.1 manually when upgrading from the published 0.2.0 or earlier; those packages do not include this control.

## Counter period and time zone

In **Settings → Application**, choose **Calendar day** under **Counter period** to reveal **Time zone** below it. **System time zone** is the default. Choose an IANA zone such as **Asia/Shanghai** or **America/Los_Angeles** to use it throughout the desktop statistics. Type a city name while the menu is open to find a zone quickly. The choice is saved independently of language and theme. While read-only refresh is paused, changing the time zone, page or counter period reprojects the same frozen data. Resume refresh to read new activity.

Calendar day, the seven-day summary, daily trends, history, the calendar and hourly labels use the same selected zone. A zone change can move activity across a date boundary. It does not rewrite timestamps, saved summaries or lifetime counts. System mode follows operating-system time-zone changes. Repeated daylight-saving hours remain separate, and rolling counters still cover the same elapsed intervals. Legacy minute records retain their minute precision.

In **Settings → Application → Counter period**, choose what Overview and Mini display:

- **Calendar day** counts from midnight in the selected time zone. This is the default.
- **Last 24 hours** counts the 24 hours leading up to now, including yesterday.

The heading changes with your choice, which is remembered after restarting. **Time zone** is hidden for **Last 24 hours**; switching back to **Calendar day** restores the selected zone. History and calendar dates stay unchanged. On daylight-saving transitions, a calendar day can span 23 or 25 hours; the rolling option always covers 24 hours.

## Mini mode and background recording

Mini mode is a fixed **320 × 208 logical-size** floating window. Screen density determines its actual pixels. Drag the brand area to move it; pin, hide or expand with the top controls, and pause or resume at the bottom.

On macOS, closing the main window or hiding Mini keeps recording from the menu bar and removes the running Dock icon. Opening either window restores the Dock icon; minimizing with the yellow button keeps it. Icons pinned to the Dock remain. Reopen from Applications, a pinned Dock icon or **Open statistics** in the menu bar. Choose **Quit ClickPulse** or press **⌘Q** to save pending records and stop.

On Windows, drag the blank header area to move the main window; double-click it to maximize or restore. Closing the window or using its Quit action offers **Minimize to tray** or **Save and quit**. Select **Remember my choice** to reuse the decision, or change it later in **Settings → Close behavior**. **Quit ClickPulse** in the tray menu always saves and exits. Double-click the tray icon to reopen statistics; right-click it for the menu.

Launch at login starts with the window hidden. Quitting saves remaining records and stops capture.

## Data locations and backups

| Platform | Statistics and preferences | Log |
| --- | --- | --- |
| macOS | `~/Library/Application Support/ClickPulse/` | `~/Library/Logs/clickpulse.log` |
| Windows | `%LOCALAPPDATA%\ClickPulse\` | `logs\clickpulse.log` inside the data folder |
| Linux | `$XDG_DATA_HOME/ClickPulse/`, normally `~/.local/share/ClickPulse/` | `logs/clickpulse.log` inside the data folder |

The main database is `events.sqlite3`. Use SQLite backup for a running database, or quit normally before copying it. Copying only the main file while the app is running may omit WAL records.

Diagnostic logs are limited to 10 MiB per file, with up to three copies named `clickpulse.log.1`, `.2` and `.3`; `.1` is the newest rotated copy. Logs are not needed to restore activity history.

Changing the program folder does not move or delete history. Opening the moved app repairs an enabled login-startup entry whose previous executable no longer exists. Existing entries pointing to another valid location are left alone; disable and re-enable the setting to use the new location. Removing program files alone retains statistics and preferences. On Windows, **Settings → Uninstall** offers a choice to retain history and settings or delete them, and cleans disposable caches in either case.

## Existing records

Normal startup uses the standard ClickPulse data directory and retains existing app history. It does not import another recorder’s database or take over its service automatically.

Advanced read-only viewing uses `--source-db`; explicit migration uses `migrate --source ... --output ...` to create a new destination. Stop the old recorder yourself and keep a complete backup first. Migration does not overwrite an existing database, merge overlapping records or stop other services. See [Development](development.md) and [Data and privacy](data-and-privacy.md).

## Save folder and history import

Settings → Data management offers **Change folder**, **Show in Finder / Open folder**, and **Import database**. Choosing a local folder copies and verifies all saved statistics before switching; the old database is retained. A folder containing `events.sqlite3` is refused to avoid overwriting history. Choosing a folder inside the app requires confirmation, since moving or updating the app can make that history unavailable. Window preferences and the saved location stay in the standard profile directory.

If the saved location is missing or its setting is damaged, startup offers **Use default location**, **Choose another folder** or **Quit**. Reconnect the disk to keep using its history. The default location reuses any history already there; it never silently replaces a missing database. Choosing another folder requires an existing `events.sqlite3`. Only a verified damaged database can be renamed as a backup before starting again, after a second confirmation. Busy databases and access problems offer **Retry**. A database from a newer version must be opened with a compatible ClickPulse version; it cannot be reset. The original database and its pending saved records stay together in that backup.

On macOS and Linux, opening an app that has moved repairs its missing login-startup path. Existing startup entries for another location are left alone. On macOS, move ClickPulse into Applications before enabling login startup from a mounted installer or a temporary system location.

Import accepts ClickPulse SQLite history, including the first public release’s `events.sqlite3`; CSV is not supported for history import. The app first saves a full SQLite backup in the current statistics folder’s `backups/` directory. After a successful import it retains that restore point and the four newest older automatic import backups. Failed imports do not prune backups; unrelated files, directories and symbolic links are left alone. Backups with SQLite journal files, unrecognized headers or noncanonical names are also retained, so more than five files may remain when cleanup would be unsafe. It merges each minute/device/key-or-button identity using the larger count, so importing the same history twice does not double totals. Independent records overlapping that same identity cannot be distinguished and may be undercounted; review the conflict warning before proceeding. New minute records are assigned to local calendar dates. Unsupported schemas, incomplete minute history and incompatible legacy key-encoding boundaries are rejected. The source is opened read-only as a consistent SQLite snapshot; other recording services are not stopped. Import and folder changes run in the background with a visible processing notice. You can continue browsing, while further data operations are disabled until completion. A requested Quit waits for the operation to commit or roll back. A locked backup fails after bounded no-progress retries instead of waiting indefinitely.


Select interface text, paths or numbers and use **⌘C** on macOS or **Ctrl+C** on Windows. macOS also provides **Edit → Copy** and **Select All**. Selected statistics remain stable during refresh until the selection is cleared. Mini’s brand area still drags the window.


## Shortcuts, Heatmap and Share

**Shortcuts** shows cumulative counts, the eight most-used combinations, and the top five’s share. The system filter applies to the whole dashboard; search narrows only the detailed list. Counts begin when this feature is installed. Shift-only combinations are hidden by default; **Show Shift combinations** includes them in the dashboard and list. This choice lasts for the current app session and does not change recording or the encyclopedia.

Switch to **Shortcut encyclopedia** to browse the bundled reference. An annotation describes a common default, not a confirmed command. Custom mappings and applications can differ. The [handbook](shortcuts.md) explains coverage and links to official references.

The **Mouse** section below Keyboard in **Heatmap** separates left, right, middle and side buttons. Its distribution covers all recorded history.

**Share** lets you choose a calendar date and a card style, then **Save PNG**. On macOS and Windows, **Copy image** lets you paste the card directly into another app. The image uses that date's real totals and the seven-day trend ending on it. Today's totals are marked as incomplete; a day without records stays at zero. A demo card is labeled as demo data. The footer shows the project's GitHub address, without your time zone. Exporting does not send anything online.
