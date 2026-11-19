# Data and privacy

## What is recorded

ClickPulse counts keyboard presses and mouse-button presses. It stores device category, key or button identity, counts and statistical timestamps. It does not store typed text, pointer positions or screen contents. Mouse movement, button release and scrolling are not clicks.

A held key counts once until released. Shift does not change a physical key’s identity. Windows distinguishes input using device and scan-code information.

Statistics work offline, with no telemetry, account, cloud sync or upload service. Starting with 0.2.1, normal desktop sessions contact this project's GitHub release API once per 24-hour window by default, including after startup if the saved check is due. Failed checks can retry a few times while the app remains open. Turn off **Check automatically** in Settings to stop automatic checks and retries. **Check for updates** makes a manual request. Starting with 0.3.1, available updates download automatically from GitHub for ordinary installations; **Update & restart** confirms installation. Scoop and WinGet installations use their package manager instead. Activity data and preferences are never uploaded. Downloading build dependencies and installing WebView2 are separate steps that can require network access.

**Ignore software-generated input** is off by default. On macOS it excludes recognized software-generated events; on Windows it excludes recognized software-generated keystrokes while preserving touchpad clicks. Virtual devices and some automation tools may be indistinguishable from physical devices.

## Shortcut statistics and sharing

Shortcut statistics store daily totals for modifier combinations, beginning with this feature. Dates use the system’s local calendar day at recording time, like daily activity totals; changing the display time zone does not regroup shortcuts. Undated private-preview counts remain in all-time totals only. No time sequence, text, active applications or clipboard contents are saved. Ordinary typing is excluded; annotations describe common defaults, not confirmed actions. Each recording database has a random local counter identity so imports can merge independent counts without repeating backups. For each date and combination, imports keep the higher count from the same identity and add counts from different identities. This identity is never sent anywhere. See the [shortcut encyclopedia](shortcuts.md). Opening an official reference launches its website in your browser; no activity counts are sent.

The Heatmap page places the keyboard above the mouse and shows cumulative mouse-button counts and their proportions. Older totals without button details remain visible as unclassified clicks. No cursor path or scroll distance is collected.

Daily cards are generated on your device. They contain the chosen date, daily keyboard/mouse totals, seven daily totals, the selected day's most-used shortcut and its count (when available), and the project's GitHub address. Shortcut days follow the recording-local dates described above; undated preview counts are excluded. When the selected display time zone differs from the system time zone, cards omit the shortcut row because daily shortcut totals cannot be reprojected. Shortcut records do not retain their original time zones, so imported history or past system time-zone changes cannot be realigned. No time zone, typed content, full shortcut list, individual key counts or application names are included. On macOS and Windows, **Copy image** puts the card on your system clipboard for pasting. Nothing is uploaded; you choose whether and where to share it.

## Time and retention

By default, Overview and Mini count from midnight in the selected time zone. **Settings → Application → Counter period** can change their main count to the last 24 hours. The seven-day summary, history and calendar continue to use calendar dates. Rolling windows cover `(now − window, now]` and exclude future timestamps. Older minute-only records show `≈` until precise records cover the entire window.

The overview’s hourly chart covers that same trailing 24-hour interval, with separate keyboard and mouse counts. Intervals align with the current local hour: 24 full hours or 25 columns with partial hours at the edges. Each full interval covers one elapsed hour, including across clock changes. Hover or keyboard-focus a column for its actual date, time range and counts. Repeated daylight-saving hours stay separate; empty hours remain visible. Minute-only records are grouped once and marked approximate.

| Data | Retention |
| --- | --- |
| Minute details | Permanent |
| Temporary cache for precise rolling totals | 24 hours |
| Daily, lifetime, physical-key and historical-key summaries | Permanent |
| Status log and three rotated copies | Up to 10 MiB each (40 MiB total) |

History is not deleted by age. Only the temporary rolling cache is pruned; saved minute details and summaries remain. Permanent history means the database grows with use.

The status log is optional diagnostic output, separate from activity history. It also records update failures with UTC time, request source and operation. Identical consecutive status lines are skipped. Older log versions that exceed the limit are trimmed to their most recent bytes, and rotation replaces the oldest diagnostic copy. Log failures do not stop database recording.

The desktop calendar groups all saved minute records into dates in the selected time zone, independently of the recent trend range. Time-zone changes affect the read-time view, not stored timestamps, original daily summaries or lifetime totals. Storage paths and backup steps are listed in the [usage guide](usage.md#data-locations-and-backups).

## Historical keys

Older records can contain characters, special keys and hashed codes that cannot always be mapped to a physical position. The heatmap shows recognized keys. The keyboard total also includes unresolved presses. Original records are retained.

New records use a separate physical-key encoding space. Overlapping character history and physical summaries are not added twice. Details deleted before migration cannot be recovered.

Encoding metadata and cached historical eras preserve attribution across upgrades and timezone changes. Unversioned legacy sources need their original change time and timezone for a reliable boundary; explicit migration supports an offset-bearing timestamp. See [storage compatibility](development.md#storage-compatibility).

## Literary comparisons

The activity calendar compares keyboard counts in the selected range with 20 classic books. Mouse clicks are excluded. One press is treated approximately as one character; modifiers and deletion also count. This is a playful comparison, not a measure of writing output or an inference about what you type.

A book and a phrase are picked independently when you return to Overview: 20 books and five phrases make 100 combinations. Neither repeats immediately. Both stay fixed while counts refresh or the language changes. Each book has its own approximate English-edition length, shared across interface languages. Results use one decimal place; tiny counts show a starting phrase, and no keyboard activity hides the comparison. All references are bundled as small numbers and titles; no books are downloaded. See [books and reference lengths](literary-references.md).

## Saving, updates and migration

Temporary save failures are retried while pending activity stays in memory. After a prolonged failure, recording pauses with a tray warning and a notification attempt. Check the data folder and free space; saving and recording resume automatically when possible. Keep ClickPulse running until saving recovers: unsaved activity cannot survive quitting or a power loss.

SQLite uses WAL. Save failures retry automatically as described above. A full or disconnected input queue still stops capture and shows an error. Auxiliary log failures retry without blocking the database. Normal exit stops input and drains pending records first.

First launch creates the standard user-data directory. Updates retain history and preferences. Windows **Uninstall and data** lets you keep history and settings for reinstalling while clearing caches, or explicitly delete the local profile. Databases outside the data folder are kept at the displayed location. Direct package-manager removal or deleting program files retains data; see [Windows uninstall](install-windows.md#uninstall). Startup does not automatically import a developer folder or backup.

Database schema upgrades are transactional and refuse a newer unsupported format. If the window preferences file is damaged, the app preserves its exact original bytes as a separate `presentation.corrupt-…json` file before restoring defaults and showing a notice. This affects window settings, not statistics. An unreadable file or failed preservation stops recovery instead of silently overwriting it.

An unavailable data location opens a recovery choice before recording starts, including during background startup. Choosing another location preserves the previous location setting. Starting again from a verified damaged database requires confirmation and keeps the old database, including its pending saved records, under a new backup name. Recovery never deletes that backup. Busy or inaccessible databases can be retried; databases from newer versions require a compatible ClickPulse version and cannot be reset. See [recovery choices](usage.md#save-folder-and-history-import).

Explicit migration uses SQLite backup, including committed WAL records, and validates daily totals, lifetime totals and database integrity. It creates a new destination and preserves the source. It rejects an existing destination, does not merge overlapping history and does not stop other services.

Use demo or redacted evidence in public screenshots and reports; do not attach real databases, raw logs, preferences or personal paths.

Pausing refresh in advanced `--source-db` mode creates a private temporary SQLite copy on local disk. All pages, date ranges and share-card shortcuts read that same snapshot until refresh resumes. The copy is removed on resume or normal exit; it does not hold a long-running read lock on the recorder or load the entire database into memory. The original database is never modified. Recording heartbeats are cleared when recording is paused or unavailable; repeated idle status checks do not rewrite them.
