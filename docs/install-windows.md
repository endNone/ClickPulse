# Windows installation and updates

For **0.2.1 release · Windows 10/11 x64**. Download the portable ZIP from the [0.2.1 Release](https://github.com/endNone/ClickPulse/releases/tag/v0.2.1). No Setup installer is offered.

Using Scoop? [Install from the maintainer's bucket](package-managers.md#windows-scoop-bucket) to download, verify, extract and add a Start menu shortcut in one command. WebView2 and Windows security requirements still apply.

Ready-to-run packages need no Rust, Python, development tools or developer account. The Windows C runtime is included in the program; no separate Visual C++ package is needed. This package is unsigned. Use a complete package from a trusted source and compare it with `SHA256SUMS.txt`.

## Portable ZIP

1. Extract the **entire folder** from `ClickPulse-0.2.1-windows-x64.zip`.
2. Place the complete contents in a fixed program directory such as `%LOCALAPPDATA%\Programs\ClickPulse` or `D:\Apps\ClickPulse`. Keep `WebView2Loader.dll`, `Licenses`, the README and manifest beside the EXE. Use a separate folder from your statistics.
3. Open **ClickPulse.exe**. Do not run inside the ZIP or copy only the EXE.

Portable means no setup wizard is required. Statistics still live in the standard user-data directory and do not move with the EXE.

Settings → Data location shows the database actually in use and whether it is the default or a custom location. Starting from a source checkout or a different working folder does not move the default database. Advanced `--db` launches use the explicitly selected file and do not manage login startup.

## Windows security prompts

ZIP packaging does not sign the executable. On Windows 11, **Smart App Control** can block an unsigned app and offers no individual-app exception. This can affect local MSVC builds too: Windows may report “An application control policy has blocked this file” (error 4551) before ClickPulse starts. Re-extracting or rebuilding does not establish that Windows will trust it.

If you trust the source, have verified the package checksum and choose to disable this protection on your own test PC, open **Windows Security → App & browser control → Smart App Control settings**, then select **Off** and review any confirmation Windows shows. On a Chinese system the path is **Windows 安全中心 → 应用和浏览器控制 → 智能应用控制设置 → 关闭**. This disables Smart App Control for the whole device, not just ClickPulse. It is optional, not a normal installation requirement; on a managed device, ask your administrator for an approved way to run the app.

Check your Windows version and the options shown before changing the setting. Microsoft's current FAQ says recent Windows updates allow re-enabling Smart App Control in Windows Security without a clean installation when supported; older installations may differ. Do not assume it is a temporary per-app exception. See [Microsoft's Smart App Control FAQ](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions) and [App & browser control settings](https://support.microsoft.com/en-us/windows/security/windows-security/app-browser-control-in-the-windows-security-app). If you keep protection enabled, retain the exact block message and use an approved test environment.

A **Microsoft Defender SmartScreen** reputation warning is a different prompt. Use an individual-app allow option only if it is offered and you have verified the package's source and integrity.

## First launch

ClickPulse automatically creates `%LOCALAPPDATA%\ClickPulse` for its database and preferences. It does not search a developer's old folders or import another recorder's data. The interface follows the system language and uses the default deep blue theme; change these in Settings.

**Microsoft Edge WebView2 Runtime** is required. If startup reports it missing, install the [official Microsoft runtime](https://developer.microsoft.com/microsoft-edge/webview2/#download-section) and reopen the app. The bundled Loader DLL is not the runtime. Runtime installation needs a network connection; ClickPulse statistics work offline. [Microsoft distribution guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution)

After startup, check the recording status and tray icon. Use the physical keyboard and mouse and confirm today's counts grow. A held key counts once. Remote injected input alone does not establish that physical capture works.

## Updates and directories

Starting with 0.2.1, use **Settings → Application → Check for updates**, then **Update & restart** when offered. Daily checks are enabled by default and can be turned off. Upgrading from the published 0.2.0 or earlier requires the manual steps below once to obtain the updater.

For a Scoop installation, quit ClickPulse and use `scoop update clickpulse` instead. The next App release enforces this in Settings and the installer, while retaining daily/manual checks; published 0.2.1 does not yet enforce it. See [package-manager updates](package-managers.md#windows-scoop-bucket).

Check the running build before testing a fix with `ClickPulse.exe --version`, for example **clickpulse 0.2.1 (78)**. The number in parentheses is the internal build. The sidebar shows only the public version. Different preview builds can share the public version and ZIP filename.

Older builds only show `0.1.0` in the app. For those builds, inspect **ClickPulse.exe → Properties → Details → File version** (for example `0.1.0.49`), or read `internal_build` and `source_revision` in the adjacent `build-manifest.json`. Confirm the executable belongs to the folder you just extracted; downloading a package or merging a source fix does not update an already running copy.

1. **Quit the old app from its tray menu** and retain the data folder.
2. Replace the complete program files in the same program directory.
3. Reopen, verify the complete version with `ClickPulse.exe --version`, and check recording, history, theme, language and close behavior. Updates do not overwrite the database or preferences.

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
