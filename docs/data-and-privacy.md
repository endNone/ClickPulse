# Data and privacy

## What is recorded

ClickPulse counts keyboard presses and mouse-button presses. It stores device category, key or button identity, counts and statistical timestamps. It does not store typed text, pointer positions or screen contents. Mouse movement, button release and scrolling are not clicks.

A held key counts once until released. Shift does not change a physical key’s identity. Windows distinguishes input using device and scan-code information.

Statistics work offline, with no telemetry, account, cloud sync or upload service. Starting with 0.2.1, normal desktop sessions contact this project's GitHub release API once per 24-hour window by default, including after startup if the saved check is due. Failed checks can retry a few times while the app remains open. Turn off **Check automatically every day** in Settings to stop automatic checks and retries. **Check for updates** makes a manual request; **Update & restart** explicitly downloads the matching package from GitHub. Activity data and preferences are never uploaded. Downloading build dependencies and installing WebView2 are separate steps that can require network access.

## Time and retention

Today and the last seven days follow local calendar days. Rolling windows cover `(now − window, now]` and exclude future timestamps. Older minute-only records show `≈` until precise records cover the entire window.

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

Older records can contain characters, special keys and hashed codes that cannot always be mapped to a physical position. Known identities map to the keyboard; unresolved counts remain in the keyboard total without creating an extra key. Visible-key counts can therefore add up to less than the keyboard total. Original records are retained.

New records use a separate physical-key encoding space. Overlapping character history and physical summaries are not added twice. Details deleted before migration cannot be recovered.

Encoding metadata and cached historical eras preserve attribution across upgrades and timezone changes. Unversioned legacy sources need their original change time and timezone for a reliable boundary; explicit migration supports an offset-bearing timestamp. See [storage compatibility](development.md#storage-compatibility).

## Literary comparisons

The activity calendar’s literary phrase uses only keyboard counts in the selected range. It does not include mouse clicks or read or infer written content. One press is treated approximately as one character for a playful comparison with the English text of *Pride and Prejudice*. Modifiers and deletion also count, so this is not a measure of writing output.

The reference is [Project Gutenberg’s English edition, number 1342](https://www.gutenberg.org/ebooks/1342). The fixed baseline is 700,000 characters, rounded from roughly 685,000 after removing illustration captions and folding repeated whitespace between the first sentence and the ebook end marker. Results display one decimal place. All interface languages use the same baseline; the app does not download the book. Small counts show a short starting phrase, and no keyboard activity hides the comparison.

## Saving, updates and migration

SQLite uses WAL. A persistence or queue failure stops capture and shows an error. Auxiliary log failures retry without blocking the database. Normal exit stops input and drains pending records first.

First launch creates the standard user-data directory. Program updates and uninstalling retain its history and preferences; startup does not automatically import a developer folder or backup.

Database schema upgrades are transactional and refuse a newer unsupported format. If the window preferences file is damaged, the app preserves its exact original bytes as a separate `presentation.corrupt-…json` file before restoring defaults and showing a notice. This affects window settings, not statistics. An unreadable file or failed preservation stops recovery instead of silently overwriting it.

Explicit migration uses SQLite backup, including committed WAL records, and validates daily totals, lifetime totals and database integrity. It creates a new destination and preserves the source. It rejects an existing destination, does not merge overlapping history and does not stop other services.

Use demo or redacted evidence in public screenshots and reports; do not attach real databases, raw logs, preferences or personal paths.
