# Windows installation and updates

For **0.1.0 preview · Windows 10/11 x64**. Download the portable ZIP from the [private preview Release](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0); repository access is required. No Setup installer is offered.

Ready-to-run packages need no Rust, Python, development tools or developer account. The Windows C runtime is included in the program; no separate Visual C++ package is needed. This preview is unsigned. Use a complete package from a trusted source and compare it with `SHA256SUMS.txt`.

## Portable ZIP

1. Extract the **entire folder** from `ClickPulse-0.1.0-windows-x64.zip`.
2. Place the complete contents in a fixed program directory such as `%LOCALAPPDATA%\Programs\ClickPulse` or `D:\Apps\ClickPulse`. Keep `WebView2Loader.dll`, `Licenses`, the README and manifest beside the EXE. Use a separate folder from your statistics.
3. Open **ClickPulse.exe**. Do not run inside the ZIP or copy only the EXE.

Portable means no setup wizard is required. Statistics still live in the standard user-data directory and do not move with the EXE.

## Windows security prompts

ZIP packaging does not sign the executable. On Windows 11, **Smart App Control** can block an unsigned app and offers no individual-app exception. Keep the protection enabled and report the exact dialog if this happens. See [Microsoft's Smart App Control guidance](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions).

A **Microsoft Defender SmartScreen** reputation warning is a different prompt. Use an individual-app allow option only if it is offered and you have verified the package's source and integrity.

## First launch

ClickPulse automatically creates `%LOCALAPPDATA%\ClickPulse` for its database and preferences. It does not search a developer's old folders or import another recorder's data. The interface follows the system language and uses the default deep blue theme; change these in Settings.

**Microsoft Edge WebView2 Runtime** is required. If startup reports it missing, install the [official Microsoft runtime](https://developer.microsoft.com/microsoft-edge/webview2/#download-section) and reopen the app. The bundled Loader DLL is not the runtime. Runtime installation needs a network connection; ClickPulse statistics work offline. [Microsoft distribution guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution)

After startup, check the recording status and tray icon. Use the physical keyboard and mouse and confirm today's counts grow. A held key counts once. Remote injected input alone does not establish that physical capture works.

## Updates and directories

1. **Quit the old app from its tray menu** and retain the data folder.
2. Replace the complete program files in the same program directory.
3. Reopen and check recording, history, theme, language and close behavior. Updates do not overwrite the database or preferences.

After moving the program, disable and re-enable Launch at login from the new location to update its startup path.

| Content | Location |
| --- | --- |
| Suggested program folder | `%LOCALAPPDATA%\Programs\ClickPulse` |
| Statistics and preferences | `%LOCALAPPDATA%\ClickPulse` |
| Log | `%LOCALAPPDATA%\ClickPulse\logs\clickpulse.log` |

## Closing and removal

Window Close and the page's Quit button offer **Minimize to tray** or **Save and quit**, with an optional remembered choice. Change it in Settings → Close behavior. **Quit ClickPulse** in the tray menu always saves and exits.

To remove the portable app, disable Launch at login, quit normally and remove its program folder and any shortcuts you created. **Keep the separate history and preferences folder.** Do not delete history to reinstall or fix a missing dependency.

If you previously installed a Setup preview, remove that installation in Windows Settings → Apps → Installed apps. Its uninstaller keeps the separate user-data folder; the portable app uses the same data location.

## Old records and source builds

Other databases are accessed only through explicit `--source-db` read-only viewing or `migrate --source ... --output ...` to a new destination. Migration does not overwrite an existing database or stop another service. Stop the old recorder and keep a complete backup first; see the [development guide](development.md).

Build source separately; GitHub's Source code ZIP is not a ready-built program. See the [development guide](development.md) for prerequisites and the common build entry point. The [native Windows checklist](windows-testing.md) tracks runtime validation.
