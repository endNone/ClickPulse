# Builds and releases

GitHub Actions checks and packages ClickPulse on hosted Mac and Windows machines. The release workflow creates an **unpublished preview draft** for maintainer review. Publishing is a separate manual step. Current source metadata uses **0.1.0**, internal build **54**; the [0.1.0 private preview download](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0) can contain an earlier build until a new release is published. Check the running app's complete version and the package manifest when testing a fix. Each candidate runs the complete workflow on hosted Apple Silicon, Intel Mac and Windows x64 runners before publication. Downloads require repository access; Windows/Intel desktop and physical-input testing remain pending before public release.

## Release download layout

| Platform | Download |
| --- | --- |
| Windows 10/11 | [![Windows Portable x64](../assets/downloads/windows-portable-x64.svg)](install-windows.md) |
| macOS 12+ | [![DMG Apple Silicon](../assets/downloads/macos-apple-silicon.svg)](install-macos.md) |
| | [![DMG Intel x64](../assets/downloads/macos-intel-x64.svg)](install-macos.md) |

This is the layout used by the release-note generator. The main download table contains three consistent buttons, with macOS grouped under one platform label. Mac App ZIPs remain available in the release attachments. Preview links open installation guides; a published Release links directly to that version’s attachments. Source code and checksums use a separate file table. Installation guides and the changelog each have their own documentation row. Badge images also use the version tag, with no external badge service.

## Workflows

| Workflow | Trigger | Result |
| --- | --- | --- |
| **Checks** | Push to `main`, pull request, manual run, or another workflow | Rust tests, formatting, Clippy, interface tests, packaging/release tests and release build |
| **Packages** | Manual run, or Release draft | Checks first, then Mac arm64/Intel x64 and Windows x64 packages saved as Actions artifacts |
| **Release draft** | Push a `vX.Y.Z` tag, or manually select an existing tag | Validate versions, run Packages, verify all assets, assemble corresponding source and create a preview draft |

Checks is reused by the other workflows. Every release stage uses the exact tagged commit. Only the final draft job receives repository write permission; build jobs have read-only access. The built-in GitHub token is used, with no personal token or signing certificate required for the unsigned preview.

Actions artifacts are retained for **14 days**; Release attachments are separate. Current targets are Apple Silicon, Intel Mac and Windows x64. No Windows ARM or Linux package is generated.

## Prepare a preview

1. Push the reviewed source, including `.github/workflows/`, to `main` when ready. Select a new public version for each release before tagging.
2. Confirm **Checks** succeeds. **Packages → Run workflow** rehearses packaging without creating a Release.
3. Confirm `Cargo.toml`, `Cargo.lock`, `assets/Info.plist` and `assets/windows/app.rc` agree on the public version. Both internal build numbers must also match.
4. Create and push a new `vX.Y.Z` tag at the reviewed commit, replacing `X.Y.Z` with the version in project metadata. `v0.1.0` already identifies the published private preview; never move or reuse a published tag.

5. Follow **Actions → Release draft** through version resolution, checks, packaging, verification and draft creation. Manual runs also require an existing version tag.
6. Review the draft under **Releases**, including downloads, release notes and limitations, then publish the private preview manually when ready. Before public distribution, finish the [Windows validation checklist](windows-testing.md), Intel Mac desktop tests, signing review and the [security-reporting channel](../SECURITY.md).

A version tag starts the draft workflow. Use manual Packages for ordinary test builds. Existing releases, including drafts, are never edited or overwritten by this workflow.

## Download assets

The 0.1.0 private preview package set contains ten attachments. Windows releases contain the portable ZIP; the former Setup download has been withdrawn. Versions in filenames come from project metadata; the workflow verifies the complete set before creating a draft.

| Asset | Purpose |
| --- | --- |
| `ClickPulse-0.1.0-macos-arm64.dmg` | Mac drag-to-install package |
| `ClickPulse-0.1.0-macos-arm64.zip` | Complete Apple Silicon App ZIP |
| `ClickPulse-0.1.0-macos-x64.dmg` | Intel Mac drag-to-install package |
| `ClickPulse-0.1.0-macos-x64.zip` | Complete Intel App ZIP |
| `ClickPulse-0.1.0-windows-x64.zip` | Windows portable folder |
| `ClickPulse-0.1.0-source.zip` | Corresponding source, licenses, build scripts and documentation |
| `ClickPulse-0.1.0-macos-arm64.manifest.json` | Apple Silicon package hashes and source revision |
| `ClickPulse-0.1.0-macos-x64.manifest.json` | Intel package hashes and source revision |
| `ClickPulse-0.1.0-windows-x64.manifest.json` | Windows package file hashes and source revision |
| `SHA256SUMS.txt` | SHA256 values for the nine other attachments |

The current packages are available in the [private preview Release](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0). Repository access is required. Installation is covered by the [Mac](install-macos.md) and [Windows](install-windows.md) guides.

## Validation and failures

Preparation stops on missing assets, mismatched versions/build numbers, incorrect outer or package-file checksums, unlisted files, missing GPL text, or different binary/source revisions. Database files, logs, environment files and private notes cannot become payloads. Only declared assets are copied; unrelated files in `dist/` are excluded.

The source ZIP and all three platform manifests must record the tagged commit. Packages made outside a Git checkout lack verified provenance and remain local test packages; rebuild from the tagged source for release.

| Failure | Action |
| --- | --- |
| Version/tag mismatch | Fix metadata before creating the release tag |
| Checks or packaging failed | Inspect the failed job; retry transient failures, fix source failures before preparing another candidate |
| Asset or revision mismatch | Use workflow packages from the same tagged commit; do not mix old local packages |
| Tag changed during the run | Prepare a consistent candidate; never move a published release tag |
| Release already exists | Review it manually. Remove only a failed unpublished draft yourself before repeating the rehearsal |
| Draft created but uploads incomplete | Inspect the validated Actions candidate; remove the incomplete unpublished draft before rerunning |

Windows builds produce the portable ZIP. Release assembly uses an explicit asset list and excludes Setup executables even if an older local build left one in the input directory.

Build and integrity results do not certify window appearance, DPI, physical input, login startup, upgrades or actual process exit. Those still need native checks. No workflow operates a personal Windows machine or reads local statistics.

The Mac preview remains ad-hoc signed without notarization; Windows packages remain unsigned. Installation notes explain manual authorization. Signing and application update checks can be added later.
