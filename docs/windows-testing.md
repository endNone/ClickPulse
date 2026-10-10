# Windows validation

The published 0.1.0 private preview's Windows x64 Setup and portable packages passed automated Rust/interface tests, native MSVC builds and package-integrity checks on a hosted Windows runner. Interactive installation, upgrades, uninstalling and complete desktop/input validation remain pending. This checklist describes the required tests; it is not evidence that they have passed.

## Installation and data locations

| Content | Suggested location | Notes |
| --- | --- | --- |
| Program | `%LOCALAPPDATA%\Programs\ClickPulse` | Another fixed folder is allowed; keep the EXE, Loader DLL and licenses together |
| Statistics and preferences | `%LOCALAPPDATA%\ClickPulse` | Belongs to the signed-in Windows user, independently of the program folder |
| Log | `%LOCALAPPDATA%\ClickPulse\logs\clickpulse.log` | Recording status and startup/capture diagnostics |
| Test exports | A folder chosen by the tester | CSV export must not overwrite an existing file |

Install and run in the target user's interactive desktop session. Confirm the user and environment variables before choosing paths; an administrator account, SSH session or WSL shell may use a different profile. Use Windows-local history or synthetic test data.

Follow the [Windows installation guide](install-windows.md). Ready-built packages need no Rust, Python or developer tools. Microsoft Edge WebView2 Runtime is required; keep `WebView2Loader.dll` beside the EXE. The Loader DLL does not replace the Runtime.

Test Setup and portable packages separately: default/custom folders, English and Chinese installer options, retry while an older version is running, cancellation and missing-runtime messages. Installation must reject the statistics directory and prevent capture before Setup finishes. Quit and back up before upgrading; confirm history, language, theme and close preferences survive. Uninstalling must remove program files and the matching startup entry while retaining data. Reinstall and confirm history returns. Record each result independently.

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
| Keyboard views | Correct PC layout, matching used-key totals in both views and no Other keys card |
| Persistence | Totals survive normal exit/reopen and continue increasing |
| CSV | Export agrees with displayed history and refuses to overwrite an existing path |
| Launch at login | Startup points to the fixed program folder; record an actual sign-out/sign-in test separately |

Remote tools may inject input differently from physical devices and Raw Input. Report remote and local physical-input results separately. Complete capture validation requires a person at the Windows machine to press keys, click and hold a key. Missing remote-input counts alone do not prove capture failed.

## Read-only totals

After the desktop app initializes its database, run in the signed-in user's PowerShell, adjusting the program path if needed:

```powershell
& "$env:LOCALAPPDATA\Programs\ClickPulse\ClickPulse.exe" stats --json
```

Compare totals before testing, before exit and after reopening. This command reads statistics without starting another collector. Report UI, remote input, local physical input, login startup and display-scaling validation independently.
