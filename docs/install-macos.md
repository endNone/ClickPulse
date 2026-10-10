# Manual installation on macOS

For **0.1.0 initial preview · macOS 12 or later · Apple Silicon and Intel**. DMG and App ZIP packages for both architectures are available in the [0.1.0 Release](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0). Both architectures pass builds and checks on their corresponding GitHub macOS runners. This guide covers first launch, Input Monitoring and updates.

Check Apple menu → About This Mac: choose `macos-arm64` for an Apple M-series chip, or `macos-x64` for an Intel processor.

Using a built App requires no Rust, Python or other developer tools, and no developer account or membership purchase. The preview is locally ad-hoc signed; it has not been signed with Apple Developer ID or notarized.

## 1. Place the App in Applications

1. Open the matching `ClickPulse-0.1.0-macos-arm64.dmg` or `ClickPulse-0.1.0-macos-x64.dmg`, or extract the matching App ZIP and locate `ClickPulse.app`. GitHub Source code downloads and `source-draft.zip` contain source rather than an installable App; see the [development guide](development.md) to build it.
2. In Finder, drag the entire `ClickPulse.app` into Applications (`/Applications`). For your user only, `~/Applications` is also suitable: press `⇧⌘G`, enter `~` to open your home folder, then create or open its `Applications` folder.
3. Open the App from that location. Keep it there after granting access or enabling launch at login, and avoid opening another copy from Downloads.

The DMG's Applications shortcut points to the application directory; drag the App onto it, then eject the disk image after copying. Quit any running older version from the menu bar before replacing it.

First launch uses `~/Library/Application Support/ClickPulse/`. Existing history and preferences are retained; startup does not select a developer's or another recorder's old directory. To uninstall, quit and remove only the App; history is kept.

## 2. Allow the first launch

After confirming the package came from the maintainer and has not been modified, add an opening exception for **ClickPulse**:

1. Double-click `ClickPulse.app` in its installed location. If it opens normally, continue to the next section.
2. If macOS cannot verify the developer or check the App for malicious software, dismiss the alert and open **System Settings → Privacy & Security**, then scroll to Security.
3. Find the message about ClickPulse being blocked and select **Open Anyway**.
4. Complete any Touch ID or Mac login-password prompt yourself, then select **Open**. Later launches can usually use a normal double-click.

**macOS 12:** use **System Preferences → Security & Privacy → General → Open Anyway**. Labels may vary by macOS version. See [Apple's current instructions](https://support.apple.com/en-us/102445) and the [macOS 12 instructions](https://support.apple.com/guide/mac-help/mh40616/12.0/mac/12.0).

This adds an exception for the selected App without disabling system-wide protection. If the alert says the App **will damage your computer** or **is damaged**, stop, obtain the original package again and contact the maintainer. Those alerts need a different response from an unidentified-developer message. [Apple's explanation of these alerts](https://support.apple.com/en-us/102445)

## 3. Grant Input Monitoring access

After allowing the App to open, grant access for keyboard and mouse statistics:

1. In ClickPulse, open **Settings → Recording and access**, then select **Grant access** or **Manage access** beside Input Monitoring. You can also open **System Settings → Privacy & Security → Input Monitoring** directly.
2. Enable **ClickPulse**. If it is missing, select **+**, choose `ClickPulse.app` from its installed location and enable it. Complete any authentication prompt yourself.
3. Follow any **Quit & Reopen** prompt. If it does not reopen automatically, choose **Quit ClickPulse** from its menu-bar menu, then open it from the installed location.
4. Confirm that the App shows **Recording**. Press a few keys and click a few times in another application, wait briefly, then check that today's keyboard and mouse counts increase.

**macOS 12:** open **System Preferences → Security & Privacy → Privacy → Input Monitoring**, unlock the pane and select ClickPulse. See [Apple's Input Monitoring instructions](https://support.apple.com/guide/mac-help/mchl4cedafb6/mac) and the [macOS 12 location](https://support.apple.com/guide/mac-help/mchl4cedafb6/12.0/mac/12.0).

The window's close button only hides it; recording continues. Use **Quit ClickPulse** when restarting for access changes. The App stores key/button identities, counts and statistical timestamps, without typed text, pointer positions or screen contents.

## 4. Update and refresh access

1. Choose **Quit ClickPulse** from its menu-bar menu, then replace the old App with the complete new `ClickPulse.app` in the same directory.
2. Open it from that location. If macOS blocks it again, follow section 2. If it does not show Recording, check access as described in section 3.
3. If Input Monitoring is enabled but the App still reports missing access, quit ClickPulse first. Select the old ClickPulse entry in Input Monitoring and remove it with **−**, then use **+** to add the **updated App from its current installed location**. Enable access and reopen it.

Updating the preview changes its ad-hoc signature, so an old permission entry may not apply to the new App. Confirm both Recording status and increasing counts. Replacing the App retains history in `~/Library/Application Support/ClickPulse/`; deleting that directory is not needed to refresh permissions.

If you move the App, confirm recording works in its new location, then turn **Settings → Application → Launch at login** off and back on to update the startup location. Login startup runs in the menu bar with the window hidden.

## Common situations

| Situation | What to do |
| --- | --- |
| Open Anyway is missing | Try opening the installed App again, then return to security settings. A work- or school-managed Mac may restrict this action; contact its administrator. |
| Access is enabled but recording has not started | Check whether recording is paused, quit normally and reopen. If access is still missing, remove the old entry and add the updated App as described in section 4. |
| Closing the window hides the interface | Reopen from Applications, the Dock or the menu-bar Open statistics action. Closing does not stop recording. |
| You received source code or an incompatible package | Choose arm64 for an Apple M-series chip or x64 for Intel, and use the complete App. Source requires a developer build. |

See the [usage guide](usage.md) and [data and privacy guide](data-and-privacy.md) for further details.
