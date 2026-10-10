# Changelog

## Unreleased

- Remove Windows mini-mode shadow insets so the WebView fills the rounded window without white or gray edges; restore the normal shadow when expanding.
- Ignore synthetic Windows keyboard prefixes around PrintScreen, Pause and keypad input, and count repeated Pause presses without requiring a release packet.

## 0.1.0 — Private preview · 2026-10-09

- Restore dashboard startup on earlier macOS 12 WebKit versions by avoiding newer JavaScript helpers; regression checks run the real interface modules without those helpers.
- Upgrade existing databases transactionally to schema version 1, reject future formats before writes, and preserve counts and historical key classifications.
- Persist key encoding metadata and support an explicit timestamp with UTC offset when migrating unversioned legacy data across timezones.
- Preserve damaged preferences in a separate recovery file and show a translated notice when the window is visible; file access errors no longer silently reset settings.
- Move periodic snapshot reads and serialization off the interface thread, coalesce refresh requests, and deliver slow results and storage failures reliably.
- Validate typed desktop requests, reject malformed actions with a translated error, and separate window, IPC and refresh handling.
- Retry transient Windows installer-tool setup failures and require the compiler to exist before packaging.
- Use English for CLI help, statistics and success messages; keep English and Simplified Chinese desktop and installer interfaces.
- Native Rust keyboard and mouse counts, held-key deduplication, local SQLite storage and permanent history.
- Today, seven-day, lifetime and rolling totals; daily trends, activity calendar, full history and CSV export.
- Physical keyboard heatmap and usage treemap, distinct left/right modifiers and preserved hidden-key and historical totals.
- Fixed-size mini mode with drag, pin, pause, background recording, expand and launch at login.
- English and Simplified Chinese, system-language defaults and Blue, Teal, Violet and Graphite themes.
- Windows tray and remembered close choices; macOS Input Monitoring controls.
- Bounded diagnostic logs with three rotated copies and duplicate-output suppression; permanent activity history is unaffected.
- Reliable shutdown after capture failures, isolated storage tests, consistent native demo totals and stricter installer payload checks.
- Database migration flushes the validated copy through a writable handle before publishing, fixing a Windows access-denied error without changing the source history.
- Consistent source checkout bytes and static Windows C runtime linking, with import checks that reject missing developer runtime dependencies.
- Explicit UTF-8 reads keep installer translations, project metadata and dependency license text intact on Windows.
- Mac App/DMG/ZIP and Windows x64 Setup/portable ZIP with checksums, manifests and licenses. Updates and uninstalling retain data.
- English installation, usage, privacy, development and release guides; four blue-theme screenshots and a mini desktop animation, automated checks, platform packaging and Release draft workflows.

Apple Silicon, Intel Mac and Windows x64 automated checks and packaging have passed on GitHub. Internal build 49 also includes schema, preference recovery, asynchronous refresh and early WebKit compatibility fixes. Full-history payloads, backup/restore controls and optional storage maintenance remain future work. Desktop installation, physical input and upgrade validation on Windows/Intel hardware remain pending before public release. Mac packages are ad-hoc signed; Windows packages are unsigned.
