# Roadmap

These are possible directions, not release-date commitments.

## Implemented

Native Rust capture, local SQLite summaries, full history, activity calendar, keyboard heatmap and treemap, CSV export, mini mode, pause, background recording, launch at login, four themes, English and Simplified Chinese, explicit migration and bounded diagnostic log rotation. The common build entry point generates Mac App/DMG/ZIP and Windows x64 portable ZIP. Program updates and removal preserve the separate user-data directory.

The 0.1.0 preview adds transactional schema versions, explicit legacy encoding metadata, preserved damaged preferences with a localized recovery notice, asynchronous snapshot delivery, strict typed IPC validation and compatibility fixes for earlier macOS 12 WebKit versions.

The 0.1.0 private preview is published with complete packages, corresponding source and checksums. Automated checks and packaging passed on hosted Apple Silicon, Intel Mac and Windows x64 runners, with release tooling also checked on Linux. Desktop interaction and physical-input validation are separate requirements below.

## Before the first public release

1. Confirm the repository address, artwork distribution terms and private security-reporting channel. The code license is GPL-3.0-or-later.
2. Validate Windows portable extraction, update, removal, physical input, held keys, tray, actual exit, multiple displays and login startup on a real machine, including publisher-trust behavior when Smart App Control is enabled.
   Intel Mac x64 packages were built and checked on a native hosted Intel runner; validate windows, Input Monitoring, physical input and upgrades on Intel hardware.
3. For the next candidate, repeat Checks, Packages and Release draft verification against its exact tagged source. Keep complete platform assets, corresponding source and checksums together.
4. Decide signing, notarization and distribution arrangements for both platforms.
5. Recheck the four English screenshots and mini desktop animation in the default deep blue theme after interface changes.
6. Publish packages with checksums, dependencies and known limitations.

## Future candidates

Prioritize backup and restore controls, optional date-based deletion with a clear preview, database size diagnostics and explicit compaction, keyboard-layout detection, accessibility and screen-size adaptation, permission diagnostics and smoother upgrades before adding unrelated features. History stays permanent by default; storage maintenance must not silently expire it.

Incremental history/keyboard payloads and background caching need profiling against long synthetic histories. The current asynchronous delivery avoids blocking the interface while preserving complete statistics.

Keep the focus on lightweight activity statistics. Exercise and game modes were dropped; cloud sync and online analysis are outside the current plan.
