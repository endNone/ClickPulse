# Data and privacy

**English** · [简体中文](zh-CN/data-and-privacy.md)

## What is recorded (and what is not)

ClickPulse records only **device category**, **key or mouse-button identity**, **counts** and **statistical timestamps** in a local SQLite database (`events.sqlite3`).

- **Never recorded**: typed text, characters, pointer coordinates, scroll distance, screen contents, active window/app names or clipboard contents.
- **Held keys**: counted once until released; mouse movement, button release and scrolling are not clicks.
- **Software-generated input**: **Ignore software-generated input** (off by default) excludes recognized synthetic input on macOS and Windows while keeping touchpad clicks.

## Network behavior

All statistics stay strictly on your device—no account, telemetry, cloud sync or upload service.

- **Update checks**: Normal desktop sessions check GitHub Releases once per 24 hours by default and download available updates in the background, installing only after you click **Update & restart**. Turn off **Check automatically** in Settings to disable all background network requests.
- **External links**: Clicking **Shortcut Atlas**, official shortcut references, or the one-time GitHub star reminder opens the URL in your browser without sending any activity data.

## Shortcuts and report cards

- **Shortcut statistics**: On macOS and Windows, only eligible modifier combinations consult the active keyboard layout (for example, `⌘Z` on QWERTZ) to identify the shortcut; ordinary typing is never converted to characters, and no layout name or active app is saved.
- **Daily & weekly reports**: Generated entirely locally. Daily cards show a 24-hour breakdown; weekly cards cover **2–31 days** with per-day bars and peak hours. Both can show the period’s most-used shortcut. When the report and system time zones differ, the shortcut is labeled as counted by recording-local date. Typed text, app names and the time zone name stay private.
- **AI prompt**: Plain text generated locally for you to review and copy into any AI tool of your choice. ClickPulse never contacts an AI provider.

## Retention and storage

| Data | Retention |
| --- | --- |
| Minute details | Permanent |
| Rolling 24-hour precision cache | 24 hours |
| Daily, lifetime, key and shortcut summaries | Permanent |
| Diagnostic log + 3 rotated copies | Up to 10 MiB each (40 MiB max) |

History is never expired automatically. SQLite runs in WAL mode with transactional schema upgrades; normal exit drains pending counts before closing. If a database or preferences file is unavailable or damaged, ClickPulse preserves the original file as a backup and prompts you instead of silently overwriting it. See [Usage](usage.md#data-locations-backups-and-import) for platform paths and import/backup behavior.

## Literary comparisons

The activity calendar compares your keyboard press count (excluding mouse clicks, treating 1 press ≈ 1 character) against 20 classic books as a playful milestone. All reference lengths are bundled locally as small numbers—no books are downloaded. See [literary-references.md](literary-references.md).

## Public bug reports

Use `--demo` or redacted screenshots when filing issues; never attach real databases, raw logs or personal paths.
