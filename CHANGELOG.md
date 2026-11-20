# Changelog

## 0.3.2 · 2026-11-20

> **macOS signing migration:** Public Mac builds now use a fixed self-signed certificate. When upgrading from 0.3.1 or earlier, remove the old ClickPulse entry in Input Monitoring and allow the updated app once; future releases signed with the same certificate keep that permission.

### New

- Track daily shortcut usage with a ranked overview, searchable list, non-US layout support (such as AZERTY, QWERTZ and Dvorak on macOS and Windows), and a link to the interactive cross-OS Shortcut Atlas on the web.
- Create daily and weekly (2–31 day) activity report cards with a floating card-deck style switcher, per-day bars, PNG export, native clipboard copy on macOS and Windows, and multi-style AI prompt recipes.
- Explore hourly input distributions in History with weekday filters, device toggles and a draggable time-range brush.
- Inspect cumulative mouse-button usage and proportions below the keyboard in Heatmap, with theme-matched shading and distinct left/right Mac modifier keys.
- Collapse the sidebar with its button or **⌘B** on Mac (**Ctrl+B** on Windows/Linux), open the project from the footer GitHub icon, and send feedback from Settings.
- Choose the counter period and font size in Settings, switch between default and full-size layouts on Heatmap, and view a full year of activity with pale empty days and varied literary comparisons.
- Keep recording quietly from the Mac menu bar after closing the window, and uninstall cleanly on Windows with explicit data-retention choices.

### Improved

- Speed up snapshot queries, reduce unchanged preference writes and unnecessary chart redraws, and trim release package sizes with thin LTO and separate debug symbols.
- Refresh Treemap key counts, the themed mouse heatmap, and History charts with unified mouse colors for clearer visual hierarchy.
- Recognize numeric keypad aliases for shortcut annotations and suggest missing shortcuts directly via the GitHub issue form.

### Fixed

- Recover unavailable or damaged storage safely without discarding existing files, preserve buffered events through short shutdown locks, and retain custom database folders and backups during uninstall.
- Keep Mac and Linux login startup working after moving the app, keep windows inside available screens, and reopen hidden or released windows cleanly after minimization.
- Restore safe default buttons in Mac and Linux confirmations, localized Windows recovery actions, and Linux recovery dialogs without requiring zenity.
- Keep Windows recording when optional session notifications are unavailable, preserve fast shortcut packet order, and attempt full file rollback if an update fails.
- Keep update checks working after temporary GitHub rate limits, freeze read-only views consistently while paused, and clean verified update staging and backups after successful updates.
- Wait for fonts before exporting report cards, omit the top shortcut when the display time zone differs from recording-local dates, and release inactive canvases cleanly.

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
