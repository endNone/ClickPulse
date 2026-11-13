# Submit ClickPulse 0.3.1 to WinGet

Status: preparation only. Publish 0.3.1 before opening the first submission to
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).

## Prepare the published package

Generate the manifests on Windows with [WinGet Create](https://github.com/microsoft/winget-create):

```powershell
wingetcreate new https://github.com/endNone/ClickPulse/releases/download/v0.3.1/ClickPulse-0.3.1-windows-x64.zip
```

Use `endNone.ClickPulse`, version `0.3.1`, publisher `Wuhan Zhang`, and license
`GPL-3.0-or-later`. The homepage is `https://github.com/endNone/ClickPulse`.
Set the release notes URL to `https://github.com/endNone/ClickPulse/releases/tag/v0.3.1`.

Review these three files under `manifests/e/endNone/ClickPulse/0.3.1/`:

- `endNone.ClickPulse.yaml`
- `endNone.ClickPulse.installer.yaml`
- `endNone.ClickPulse.locale.en-US.yaml`

The installer uses this package layout:

```yaml
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
- RelativeFilePath: ClickPulse-0.3.1-windows-x64\ClickPulse.exe
  PortableCommandAlias: clickpulse
Dependencies:
  PackageDependencies:
  - PackageIdentifier: Microsoft.EdgeWebView2Runtime
```

Use architecture `x64` and the published ZIP URL above. Set `ReleaseDate` to the
actual publication date. Compare `InstallerSha256` with that release's
`SHA256SUMS.txt`; never reuse the 0.3.0 value or a private test package's value.

## Validate and submit

```powershell
winget validate --manifest manifests/e/endNone/ClickPulse/0.3.1
```

In an isolated Windows test account, install the local manifest and verify:

- Open ClickPulse using its `clickpulse` entry.
- See the WinGet upgrade command in Settings.
- Enable login startup and inspect its fixed WinGet entry.
- Retain history after updating and reopening.
- Preserve disabled login startup.

Fork `microsoft/winget-pkgs`, commit the three files under the maintainer's
personal identity, and open a pull request. The maintainer completes any CLA
request. Resolve the repository checks before merging.

The fixed entry is used only when it resolves to this installation. WinGet can
fall back to adding the package folder to PATH when it cannot create a link;
ClickPulse then uses its current path and repairs a missing old path on the next
launch. See Microsoft's [portable installation behavior](https://github.com/microsoft/winget-cli/blob/v1.29.380/src/AppInstallerCLICore/PortableInstaller.cpp)
and [manifest schema](https://github.com/microsoft/winget-cli/tree/v1.29.380/schemas/JSON/manifests/v1.12.0).

Homebrew and Scoop continue to track the published stable release. Update those
sources after publication using the [distribution workflow](package-managers.md#maintainer-keep-the-channels-current).
