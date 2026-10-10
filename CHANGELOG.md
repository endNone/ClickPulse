# Changelog

## 0.1.0 — Preview · 2026-10-10

- Keep toolbar actions centered at the top while scrolling, with repeated page names removed.
- Match the Windows heatmap to the compact six-row layout, keeping navigation/numpad history without stretching extra rows below Space.
- Recognize modified PrintScreen/Pause scan codes and their previously stored aliases without rewriting or double-counting history.
- Keep the selected dropdown option when opening with ArrowUp; subsequent arrows preview adjacent options.
- Label default/custom/read-only/demo storage in Settings, reject relative Windows data roots, and restrict login-startup controls to the default App database.
- Show the internal build alongside the public version in Settings, the sidebar and `--version`, so preview binaries can be distinguished when verifying fixes.
- Remove native shadow insets from the Windows mini window and align its rounded region with the actual client area, including frame changes and display scaling.
- Ignore synthetic keyboard prefixes around PrintScreen and Shift/NumLock keypad input, preserving physically held keys; count repeated Pause presses without requiring a release packet.

- Restore dashboard startup on earlier macOS 12 WebKit versions by avoiding newer JavaScript helpers; regression checks run the real interface modules without those helpers.
- Upgrade existing databases transactionally to schema version 1, reject future formats before writes, and preserve counts and historical key classifications.
- Persist key encoding metadata and support an explicit timestamp with UTC offset when migrating unversioned legacy data across timezones.
- Preserve damaged preferences in a separate recovery file and show a translated notice when the window is visible; file access errors no longer silently reset settings.
- Move periodic snapshot reads and serialization off the interface thread, coalesce refresh requests, and deliver slow results and storage failures reliably.
- Validate typed desktop requests, reject malformed actions with a translated error, and separate window, IPC and refresh handling.
- Use English for CLI help, statistics and success messages; keep English and Simplified Chinese desktop interfaces.
- Native Rust keyboard and mouse counts, held-key deduplication, local SQLite storage and permanent history.
- Today, seven-day, lifetime and rolling totals; daily trends, activity calendar, full history and CSV export.
- Physical keyboard heatmap and usage treemap, distinct left/right modifiers and preserved hidden-key and historical totals.
- Fixed-size mini mode with drag, pin, pause, background recording, expand and launch at login.
- English and Simplified Chinese, system-language defaults and Blue, Teal, Violet and Graphite themes.
- Windows tray and remembered close choices; macOS Input Monitoring controls.
- Bounded diagnostic logs with three rotated copies and duplicate-output suppression; permanent activity history is unaffected.
- Reliable shutdown after capture failures, isolated storage tests, consistent native demo totals and portable payload checks.
- Database migration flushes the validated copy through a writable handle before publishing, fixing a Windows access-denied error without changing the source history.
- Consistent source checkout bytes and static Windows C runtime linking, with import checks that reject missing developer runtime dependencies.
- Explicit UTF-8 reads keep project metadata and dependency license text intact on Windows.
- Mac App/DMG/ZIP and Windows x64 portable ZIP with checksums, manifests and licenses. The former Setup download has been withdrawn; Windows release builds now produce ZIP only. Updates and program removal retain data.
- English installation, usage, privacy, development and release guides; four blue-theme screenshots and a mini desktop animation, automated checks, platform packaging and Release draft workflows.

Internal build 55 is the first public preview, including all fixes above. Apple Silicon, Intel Mac and Windows x64 packages pass automated tests, Clippy, release builds and package verification on native hosted runners. Hardware-specific input, login and display-scaling coverage is recorded in the Windows validation guide and roadmap. Mac packages are ad-hoc signed; Windows packages are unsigned. Full-history payload improvements, backup/restore controls and optional storage maintenance remain future work.
