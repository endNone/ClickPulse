# Roadmap

These are possible directions, not release-date commitments.

## Implemented

Native Rust capture, local SQLite summaries, full history, activity calendar, keyboard heatmap and treemap, CSV export, mini mode, pause, background recording, launch at login, four themes, English and Simplified Chinese, explicit migration and bounded diagnostic log rotation. The common build entry point generates Mac App/DMG/ZIP and Windows x64 portable ZIP. Program updates and removal preserve the separate user-data directory.

The 0.1.0 preview adds transactional schema versions, explicit legacy encoding metadata, preserved damaged preferences with a localized recovery notice, asynchronous snapshot delivery, strict typed IPC validation and compatibility fixes for earlier macOS 12 WebKit versions.

The 0.1.0 public preview includes complete packages, corresponding source and checksums. Automated checks and packaging pass on hosted Apple Silicon, Intel Mac and Windows x64 runners, with release tooling checked on Linux. Private security reports use GitHub's vulnerability-reporting form. Desktop interaction and physical-input coverage are recorded separately below.

## Release follow-up

1. Extend Windows hardware coverage for physical special keys, sign-out/sign-in, multi-display DPI, upgrades and removal. Record publisher-trust behavior with Smart App Control separately from capture results.
   Intel Mac packages pass native hosted checks; extend desktop, Input Monitoring, physical-input and upgrade coverage on Intel hardware.
2. For each new version, repeat Checks, Packages and Release draft verification against its exact tagged source. Keep platform assets, corresponding source and checksums together; public release tags remain immutable.
3. Evaluate signing and notarization when distribution arrangements change. The current preview uses Mac ad-hoc signing and unsigned Windows portable ZIPs.
4. Recheck the four English screenshots and mini desktop animation in the default deep blue theme after interface changes.
5. Keep package dependencies, hardware coverage and known limitations current in release notes.

## Future candidates

Prioritize backup and restore controls, optional date-based deletion with a clear preview, database size diagnostics and explicit compaction, keyboard-layout detection, accessibility and screen-size adaptation, permission diagnostics and smoother upgrades before adding unrelated features. History stays permanent by default; storage maintenance must not silently expire it.

Incremental history/keyboard payloads and background caching need profiling against long synthetic histories. The current asynchronous delivery avoids blocking the interface while preserving complete statistics.

Keep the focus on lightweight activity statistics. Exercise and game modes were dropped; cloud sync and online analysis are outside the current plan.
