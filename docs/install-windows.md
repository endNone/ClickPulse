# Windows installation and updates

For **0.1.0 initial preview · Windows 10/11 x64**. Setup and portable packages are available in the [private preview Release](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0); repository access is required. Native Windows automated code, CLI and packaging checks have passed. Desktop installation, physical input, update and uninstall validation remain pending.

Ready-to-run packages need no Rust, Python, development tools or developer account. The Windows C runtime is included in the program; no separate Visual C++ package is needed. This preview is unsigned. Use an intact package from a trusted source and compare it with its accompanying SHA256 file.

## Setup installer

1. Open `ClickPulse-0.1.0-windows-x64-Setup.exe`. The wizard supports English and Simplified Chinese.
2. The default location is `%LOCALAPPDATA%\Programs\ClickPulse` for the current user, without administrator elevation. You can choose another empty, fixed folder such as `D:\Apps\ClickPulse`. Do not select the statistics folder or a folder containing unrelated software.
3. If ClickPulse is running, choose **Quit ClickPulse** from its tray menu, then retry. Closing or hiding a window can leave recording active.
4. Open ClickPulse from the Start menu after installation. Setup does not automatically enable launch at login, grant access or change system security settings.

If Windows blocks this unsigned preview, use an individual-app allow option only when one is offered and you have verified the source and integrity. If policy does not offer that option, keep the original package and contact the maintainer or administrator rather than disabling system-wide protection.

## Portable ZIP

1. Extract the **entire folder** from `ClickPulse-0.1.0-windows-x64.zip`.
2. Place the complete contents in a fixed directory such as `%LOCALAPPDATA%\Programs\ClickPulse`. Keep `WebView2Loader.dll`, `Licenses`, the README and manifest beside the EXE.
3. Open `ClickPulse.exe`. Do not run inside the ZIP or copy only the EXE.

Setup and portable packages use the same program and data format. Portable means no setup wizard is required; statistics still live in the standard user-data directory and do not move with the EXE.

## First launch

ClickPulse automatically creates `%LOCALAPPDATA%\ClickPulse` for its database and preferences. It does not search a developer's old folders or import another recorder's data. The interface follows the system language and uses the default deep blue theme; change these in Settings.

**Microsoft Edge WebView2 Runtime** is required. If startup reports it missing, install the [official Microsoft runtime](https://developer.microsoft.com/microsoft-edge/webview2/#download-section) and reopen the app. The bundled Loader DLL is not the runtime. Runtime installation needs a network connection; ClickPulse statistics work offline. [Microsoft distribution guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution)

After startup, check the recording status and tray icon. Use the physical keyboard and mouse and confirm today's counts grow. A held key counts once. Remote injected input alone does not establish that physical capture works.

## Updates and directories

1. **Quit the old app from its tray menu** and retain the data folder.
2. For Setup, choose the existing program directory. For portable updates, replace the complete program files in the same location.
3. Reopen and check recording, history, theme, language and close behavior. Updates do not overwrite the database or preferences.

After moving the program, disable and re-enable Launch at login from the new location to update its startup path. Setup itself does not launch the recorder automatically.

| Content | Location |
| --- | --- |
| Default program | `%LOCALAPPDATA%\Programs\ClickPulse` |
| Statistics and preferences | `%LOCALAPPDATA%\ClickPulse` |
| Log | `%LOCALAPPDATA%\ClickPulse\logs\clickpulse.log` |

## Closing and uninstalling

Window Close and the page's Quit button offer **Minimize to tray** or **Save and quit**, with an optional remembered choice. Change it in Settings → Close behavior. **Quit ClickPulse** in the tray menu always saves and exits.

Uninstall Setup installations in Windows Settings → Apps → Installed apps. For portable copies, quit normally and remove the program folder. Setup uninstall removes this installation's program files, Start menu shortcut and matching startup entry. **History and preferences are kept.** Do not delete history to reinstall or fix a missing dependency.

## Old records and source builds

Other databases are accessed only through explicit `--source-db` read-only viewing or `migrate --source ... --output ...` to a new destination. Migration does not overwrite an existing database or stop another service. Stop the old recorder and keep a complete backup first; see the [development guide](development.md).

Build source separately; GitHub's Source code ZIP is not an installer. See the [development guide](development.md) for prerequisites and the common build entry point. The [native Windows checklist](windows-testing.md) tracks runtime validation.
