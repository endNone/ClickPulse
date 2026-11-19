# Changelog

## Unreleased

Suggested scope: prioritize stability and resource use in **0.3.2**; stage the larger interface and statistics additions for **0.3.3**. These are planning labels, not released versions or separately prepared builds. The current working tree still contains both groups.

### Fixed

- **0.3.2:** Recover unavailable or damaged storage without discarding existing files; distinguish temporary locks, permissions and databases from newer versions. Keep final buffered events through short shutdown lock conflicts.
- **0.3.2:** Keep Mac and Linux login startup working after moving the app; keep windows within the available screen and disable unwanted Windows browser actions.
- **0.3.2:** Restore safe default buttons in Mac and Linux confirmations, localized Windows recovery actions, and Linux recovery dialogs without requiring zenity. Reopen hidden or released windows after minimization.
- **0.3.2:** Keep Windows recording when optional session/desktop notifications are unavailable. Correct Windows uninstall cleanup, preserve requested history and settings, and clean verified update staging and backups after successful updates.
- **0.3.2:** Reduce duplicate database work, unchanged preference writes and frontend redraws. In a controlled 365-day synthetic-history benchmark, median full-snapshot time fell **73.8%** for standalone databases and **59.2%** for collector views; these are query/aggregation timings, not whole-app CPU or memory savings.
- **0.3.2:** Reduce package size with thin LTO and separate debug symbols. Same-source build-profile comparisons measured **10.3%** smaller macOS ARM64 DMGs, **31.7%** smaller Ubuntu ARM64 ZIPs and **50.9%** smaller Windows x64 **MinGW cross-built** ZIPs. These compare private build profiles, not published versions; final release sizes and Windows MSVC savings still need measurement.
- **0.3.2:** Restore tabular numerals, font spacing and white brand titles; correct Settings control sizes and overlay layering, calendar labels and translated accessibility text.
- **0.3.2:** Keep update checks working after temporary GitHub rate limits; handle data handoff errors without crashing and reap external desktop launchers.
- **0.3.2:** Preserve buffered data when background storage jobs cannot start, retain custom database folders and recovery backups during uninstall, and attempt every file restore if an update rolls back.
- **0.3.2:** Freeze all read-only source views consistently while paused, including Mini and share cards; stop repeated heartbeat writes while recording is paused or unavailable.
- **0.3.3:** Preserve fast Windows shortcut packet order; reconcile stale modifiers after 1.5 seconds, retain held keys across foreground changes, and reset state on secure-desktop or session changes. Search all available shortcut-title and context translations.
- **0.3.3:** Release inactive share canvases without interrupting exports; wait for fonts, keep long footer text clear of the project link, and omit the top shortcut when its local-date basis differs from the card's selected time zone.

### New

- **0.3.3:** Discover daily shortcut counts with an offline, source-linked reference library. Use **Show Shift combinations** to include Shift-only entries in the usage list; the choice lasts for the current session and recording is unchanged.
- **0.3.3:** Create daily activity cards with the day's most-used shortcut and count when date bases agree. Save PNGs, or **copy the image** directly to the native clipboard on macOS and Windows.
- **0.3.3:** Explore cumulative mouse-button usage and proportions below the keyboard in Heatmap, including individual buttons and distinct left/right Mac modifier keys.
- **0.3.3:** Collapse the sidebar with its button or **⌘B** on Mac (**Ctrl+B** on Windows/Linux). Open the project from the footer GitHub icon and send feedback from Settings.
- **0.3.3:** Choose the counter period, interface size and compact or full keyboard layout in reorganized Settings.
- **0.3.3:** View a full year of activity with pale empty days and varied literary comparisons; keep recording quietly from the Mac menu bar after closing the window.

## 0.3.1 · 2026-11-15

### New

- Exclude recognized software-generated input with an optional setting on macOS and Windows.

### Improved

- Download updates automatically; install and restart only when you confirm.
- Use less memory in the background; reopen your previous page when needed.
- Download smaller packages with lighter fonts and icons.
- Reduce Linux background power use; install `fonts-noto-cjk` for Chinese, Japanese and Korean.
- Keep WinGet installations current through `winget upgrade --id endNone.ClickPulse --exact`.

### Fixed

- Resume saving automatically after temporary failures, with a warning when recording must pause.
- Keep Windows launch-at-login working through upgrades and repair missing startup paths.
- Keep macOS idle CPU use low.
- Open recreated windows after the interface is ready.
- Keep data controls available during downloads and explain when updates require manual installation.

## 0.3.0 · 2026-11-12

### New

- Track keyboard and mouse activity on Ubuntu 22.04 and 24.04.

### Improved

- Read consistent navigation labels, icons and Settings controls across platforms.
- Use a cleaner Linux window without the extra title bar.
- Keep Scoop installations current with `scoop update clickpulse`.
- Reduce background refreshes and disk activity.

### Fixed

- Open Mini with inactive Sport effects hidden from the first frame.
- See a clear, consistent checkmark when remembering your close preference.
- Switch between Mini and full view without carrying over text selection.
- Count physical Caps Lock presses correctly on macOS.

## 0.2.1 · 2026-11-11

### New

- Find updates in Settings, or automatically each day; daily checks can be disabled.
- Choose when to install with **Update & restart**.

### Improved

- Read larger Settings controls and translated system time zones.
- Enjoy smoother Sport previews.
- Keep your history and preferences when updating.

### Fixed

- Browse long histories without slow calendar loading.
- Download Windows updates successfully.
- Receive each new-version reminder once.

> Upgrading from 0.2.0 or earlier? Download 0.2.1 manually once; later updates install inside the app.

## 0.2.0 · 2026-11-05

### New

- Turn typing and clicking into Sport effects, with ranks from C to SS.
- Explore your last 24 hours and compare 14-, 30- or 90-day trends.
- Use eight languages and choose your preferred time zone.
- Move your data anywhere or merge history from another computer, with automatic backups.

### Improved

- Keep recording smoothly while importing or browsing long histories.
- Copy live counts as they continue updating.
- Reach Settings sections faster with clearer controls and shortcuts.

### Fixed

- Import history reliably without opening the same database twice.
- Keep dropdown choices and scroll position during refreshes.

CSV export has been removed; history imports and backups remain available.

## 0.1.0 — Preview · 2026-10-10

### New

- Track keyboard and mouse activity locally, with complete daily history.
- Discover patterns through trends, calendars, keyboard heatmaps and treemaps.
- Keep live counts nearby with a floating, pinnable Mini window.
- Personalize your view with four themes and English or Simplified Chinese.
- Continue recording in the background and start automatically when you log in.

### Improved

- Reach window controls while scrolling.
- See the complete Windows keyboard at smaller window sizes.
- Keep history when updating or removing the app.

### Fixed

- Open the dashboard on macOS 12.
- Count special Windows keys correctly without repeated held-key counts.
- Switch between Mini and the full window without misplaced borders.
- Recover preferences when a saved settings file cannot be read.
