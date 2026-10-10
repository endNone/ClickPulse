# Windows validation

The 0.1.0 preview's Windows x64 portable package passes automated Rust/interface tests, native MSVC builds and package-integrity checks on a hosted Windows runner. Native WebView2 tests cover mini-window startup and full/mini transitions. Preview desktop testing also verified clean mini-window edges on Windows 11, as recorded in [PR #2](https://github.com/endNone/ClickPulse/pull/2). The checklist below distinguishes that evidence from physical input, login startup, upgrades and display scaling. Windows downloads contain the portable ZIP only.

## Installation and data locations

| Content | Suggested location | Notes |
| --- | --- | --- |
| Program | `%LOCALAPPDATA%\Programs\ClickPulse` | Another fixed folder is allowed; keep the EXE, Loader DLL and licenses together |
| Statistics and preferences | `%LOCALAPPDATA%\ClickPulse` | Belongs to the signed-in Windows user, independently of the program folder |
| Log | `%LOCALAPPDATA%\ClickPulse\logs\clickpulse.log` | Recording status and startup/capture diagnostics |
| Test exports | A folder chosen by the tester | CSV export must not overwrite an existing file |

Install and run in the target user's interactive desktop session. Confirm the user and environment variables before choosing paths; an administrator account, SSH session or WSL shell may use a different profile. Use Windows-local history or synthetic test data.

Follow the [Windows installation guide](install-windows.md). Ready-built packages need no Rust, Python or developer tools. Microsoft Edge WebView2 Runtime is required; keep `WebView2Loader.dll` beside the EXE. The Loader DLL does not replace the Runtime.

Test complete ZIP extraction in default/custom folders and missing-runtime messages. Quit and back up before upgrading; confirm history, language, theme and close preferences survive. Disable Launch at login before removing the program folder, and retain the separate data directory. Extract again and confirm history returns. Record each result independently. With Smart App Control enabled, record the exact publisher-trust result separately: one user's successful launch does not establish that every Windows policy permits the unsigned executable.

## Desktop session and evidence

Remote desktop can show and operate the actual Windows UI. SSH can transfer packages or inspect files and logs; starting the app in a noninteractive session does not validate desktop behavior. WSL builds do not prove Windows UI or input capture works.

Record OS version, architecture, signed-in user, display scaling, program folder, existing ClickPulse processes and whether local data exists. Quit an older app normally and retain its files and complete data. For a running SQLite database, copying only the main file is not a complete backup.

Verify package SHA256 values and manifest file hashes. Extract the complete package, run `ClickPulse.exe` and inspect its window, tray and error messages. Attach actual observations or command results; mark unperformed checks as **Not tested**.

## Functional checks

| Check | Pass criteria |
| --- | --- |
| First launch | Complete main window, correct fonts and tray icon; missing dependencies produce a clear error |
| Language and theme | System language defaults; English/Chinese and all four themes switch correctly and survive restart |
| Physical input | Real keyboard/button presses update counts; holding a key does not repeat the count |
| Pause/resume | Input while paused adds no counts; recording continues after resuming |
| Main window | Native title bar is absent; app minimize, maximize/restore, close and header dragging work while scrolling |
| Mini mode | Fixed 320 × 208 logical size; rounded native corners have no white border; pin/hide/expand work |
| Display scaling | Where hardware permits, moving between displays preserves mini logical size; otherwise mark **Not tested** |
| Close choices | First close/page Quit offers minimize or save-and-quit; cancel, remembered choice, restart and resetting to Ask work in both app languages |
| Background recording | Minimizing to tray or hiding mini mode still records physical input |
| Actual exit | Save-and-quit or tray Quit saves records, stops capture and removes the process/tray icon; dedicated WebView2 children exit without affecting other apps |
| Reopen/single instance | Tray double-click or running the EXE again opens full statistics with one collector process |
| Keyboard views | Compact PC layout ends at the Space/modifier row, with no stretched navigation/numpad rows below it; both views report used visible keys consistently while preserving all stored counts |
| Persistence | Totals survive normal exit/reopen and continue increasing |
| CSV | Export agrees with displayed history and refuses to overwrite an existing path |
| Data location | Default launch uses `%LOCALAPPDATA%\ClickPulse\events.sqlite3` independently of the working directory; Settings distinguishes custom/read-only/demo sessions |
| Launch at login | Only default-database sessions expose and handle startup changes; custom `--db`, read-only and demo sessions cannot change the default startup entry. Record an actual sign-out/sign-in test separately |

Remote tools may inject input differently from physical devices and Raw Input. Report remote and local physical-input results separately. Complete capture validation requires a person at the Windows machine to press keys, click and hold a key. Missing remote-input counts alone do not prove capture failed.

### Counting and mini-window regression checks

Use a temporary database (`--db <temporary-path>`) and compare before/after counts. Press and release a normal key ten times, then hold it: expect ten plus one keyboard events. Click each available mouse button ten times; releases, pointer movement and scrolling must add no clicks. Repeat with the main window focused, another app focused, and ClickPulse hidden in the tray.

Check PrintScreen and Alt+PrintScreen, repeated Pause and Ctrl+Pause presses, and Shift with NumLock/keypad navigation separately. Release modifiers before their special key as well as after it; the next press must still count, and both variants must reach the same heatmap key. Synthetic Shift/Ctrl prefix packets must not add operations or release an actually held modifier; Pause must remain countable without a release packet. Record the physical sequence and expected delta, including any modifiers pressed by the tester.

Start directly with `--mini`, then expand and re-enter mini mode. Check the left and top edges at 100%, 150% and 200% scaling where available: the content must fill the rounded outline with no white/gray strip or clipped right/bottom edge. Confirm full-window resizing, maximizing/restoring and mini drag/pin controls still work. These manual checks remain **Not tested** until observed on the target hardware.

## Read-only totals

After the desktop app initializes its database, run in the signed-in user's PowerShell, adjusting the program path if needed:

```powershell
& "$env:LOCALAPPDATA\Programs\ClickPulse\ClickPulse.exe" stats --json
```

Compare totals before testing, before exit and after reopening. This command reads statistics without starting another collector. Report UI, remote input, local physical input, login startup and display-scaling validation independently.
