# Install with Homebrew or Scoop

These are **maintainer-owned** package sources in [endNone/ClickPulse](https://github.com/endNone/ClickPulse), maintained by Wuhan Zhang. They are not entries in Homebrew's central cask repository, Scoop Extras or winget. Both install the same versioned files published in GitHub Releases and verify SHA256 checksums.

## macOS: Homebrew Cask

With [Homebrew](https://brew.sh) already installed:

```sh
brew tap endnone/clickpulse https://github.com/endNone/ClickPulse && brew install --cask endnone/clickpulse/clickpulse
```

The Cask selects Apple Silicon or Intel automatically and places `ClickPulse.app` in `/Applications`. If you already installed ClickPulse manually, keep that installation and use its updater, or quit it and move that App to Trash before switching to Homebrew. Do not leave two copies in different Applications folders. The separate data folder is preserved.

For a user-only installation, add `--appdir="$HOME/Applications"` to `brew install`. Keep that location for later upgrades and Input Monitoring access.

The App is currently ad-hoc signed and **not notarized**. Homebrew does not notarize it or remove the first-launch requirement. Follow [macOS first launch and Input Monitoring](install-macos.md#2-allow-the-first-launch). The Cask does not disable Gatekeeper or remove quarantine attributes.

Quit ClickPulse before updating or removing it:

```sh
brew update
brew upgrade --cask --greedy endnone/clickpulse/clickpulse
# Or uninstall the program while retaining history:
brew uninstall --cask endnone/clickpulse/clickpulse
```

`--greedy` includes this Cask even though the App also offers its own updater. Prefer Homebrew updates for a Homebrew installation so its recorded version stays consistent. Updates can require refreshing the existing Input Monitoring grant.

## Windows: Scoop bucket

With [Scoop](https://scoop.sh) installed, run this in PowerShell:

```powershell
scoop bucket add clickpulse https://github.com/endNone/ClickPulse; if ($?) { scoop install clickpulse/clickpulse }
```

Scoop downloads the complete x64 portable package, adds **ClickPulse** to the Start menu and exposes the `clickpulse` command. The program lives under Scoop's application directory; data remains in `%LOCALAPPDATA%\ClickPulse` or your chosen data folder. Quit an existing manual copy before starting this copy.

[Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/#download-section) must already be installed, or installed separately if ClickPulse reports it missing. The Scoop manifest does not override the WebView2 runtime used by other applications.

The executable is **unsigned**. Scoop does not establish publisher trust or bypass Smart App Control error 4551. See [Windows security prompts](install-windows.md#windows-security-prompts). Installation tests do not mean a protected PC will allow the App to launch.

Quit ClickPulse before updating or removing it:

```powershell
scoop update
scoop update clickpulse
# Or uninstall the program while retaining history:
scoop uninstall clickpulse
```

Prefer Scoop updates for a Scoop installation so its version directory and metadata stay consistent. If you change the installed program location, re-enable Launch at login from the new copy.

## Maintainer: keep the channels current

After publishing a new stable Release, download and validate its complete candidate, then update both manifests from those verified files:

```sh
python3 scripts/update_distribution.py --version X.Y.Z --packages /path/to/candidate/release
```

Review `Casks/clickpulse.rb` and `bucket/clickpulse.json`, verify the versioned downloads are public, and commit/push under the maintainer's personal identity. The script checks all three package hashes before changing either manifest and never publishes automatically. Package-manager versions follow the latest **published** stable release, which can differ from development source versions.

The **Package manager installation** workflow tests actual Homebrew installation/removal on Apple Silicon and Intel runners, plus Scoop installation, payload hashes, command, shortcut and removal on Windows x64. It does not disable system protection or claim Gatekeeper/Smart App Control approval.

Official Homebrew inclusion requires passing its [Gatekeeper requirements](https://docs.brew.sh/Acceptable-Casks). Broader Windows trust needs a [trusted code signature](https://learn.microsoft.com/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control). Signing and Apple notarization remain separate release work; package-manager distribution alone does not provide them.
