# Builds and releases

GitHub Actions checks and packages ClickPulse on hosted Mac, Windows and Ubuntu machines. The release workflow produces a **verified candidate artifact** for maintainer review. Wuhan Zhang (`endNone`) creates and publishes Releases with the personal GitHub account; automation never creates a bot-authored Release. The [0.3.1 release](https://github.com/endNone/ClickPulse/releases/tag/v0.3.1) uses internal build **80**. Source, binaries and checksums identify the same tagged revision. Check the running app's complete version and package manifest when testing a fix. Every candidate runs checks and packaging on hosted Apple Silicon, Intel Mac, Windows x64 and Ubuntu runners. Hardware-specific validation is recorded separately in the Windows checklist and roadmap.

## Release download layout

Start with two short release highlights, followed by an underlined **Full release notes & downloads** link to that version's page. Keep this introduction concise so the download entry appears early in the Feed summary. Detailed changes follow the download table.

| Platform | Download |
| --- | --- |
| <img src="../assets/downloads/platform-windows.svg" width="18" height="18" align="absmiddle" alt=""> Windows 10/11 | [<ins>Portable x64</ins>](install-windows.md) |
| <img src="../assets/downloads/platform-apple.svg" width="18" height="18" align="absmiddle" alt=""> macOS 12+ | [<ins>Apple Silicon DMG</ins>](install-macos.md) · [<ins>Intel DMG</ins>](install-macos.md) |
| <img src="../assets/downloads/platform-ubuntu.svg" width="18" height="18" align="absmiddle" alt=""> Ubuntu 22.04 / 24.04 | [<ins>x64 ZIP</ins>](install-linux.md) · [<ins>ARM64 ZIP</ins>](install-linux.md) |

[<ins>Source code</ins>](../README.md) · [<ins>Build from source</ins>](development.md)

Each platform uses one row with an 18-pixel icon aligned vertically beside its name. Underlined text links keep package names and download targets available when GitHub Feed removes images. Preview links here open the guides; published Releases link directly to their own attachments. Mac App ZIPs remain available in the release attachments.

Icons use the version tag, or an explicit immutable `artwork_revision` for a presentation-only correction. Record that revision with the reviewed release notes. Keep existing release assets and tags unchanged when updating the layout. The actual Feed determines how much of a release summary appears; a local Markdown preview does not verify that truncation.

## Workflows

| Workflow | Trigger | Result |
| --- | --- | --- |
| **Checks** | Push to `main`, pull request, manual run, or another workflow | Rust tests, formatting, Clippy, interface tests, packaging/release tests and release build |
| **Packages** | Manual run, or Release candidate | Checks and all five platform builds run in parallel; packages are saved as Actions artifacts |
| **Release candidate** | Push a `vX.Y.Z` tag, or manually select an existing tag | Validate versions, run Packages, verify all assets, assemble corresponding source and retain a candidate artifact |

Checks is reused by the other workflows. Packaging starts alongside checks, but release assembly waits for both to succeed. Failed or cancelled checks prevent a verified release candidate even if package artifacts are available. Every release stage uses the exact tagged commit. All workflow jobs have read-only repository access. The built-in GitHub token is used only for read access, with no personal token stored in Actions. Signing certificates are not configured.

Actions artifacts are retained for **14 days**; Release attachments are separate. Current targets are Apple Silicon, Intel Mac, Windows x64, Ubuntu x64 and Ubuntu ARM64. Linux packages are built on Ubuntu 22.04 for compatibility with 22.04 and 24.04. No Windows ARM package is generated.

## Prepare and publish a release

1. Push the reviewed source, including `.github/workflows/`, to `main` when ready. Select a new public version for each release before tagging.
2. Confirm **Checks** succeeds. **Packages → Run workflow** rehearses packaging without creating a Release.
3. Confirm `Cargo.toml`, `Cargo.lock`, `assets/Info.plist`, `assets/windows/app.rc` and `assets/windows/ClickPulse.manifest` agree on the public version. Both internal build numbers must also match.
4. Create and push a new `vX.Y.Z` tag at the reviewed commit, replacing `X.Y.Z` with the version in project metadata. `v0.1.0` identifies the first public preview; never move or reuse a published tag.

5. Follow **Actions → Release candidate** through version resolution, checks, packaging and verification. Download `Release-candidate-X.Y.Z` before its 14-day retention period ends. Manual runs also require an existing version tag.
6. Verify the candidate checksums, source revision and current remote tag. Review release notes, downloads and limitations, then create a draft with the personal account as described below. Publish only after confirming every attachment uploaded. Record the [Windows validation checklist](windows-testing.md), Intel Mac desktop coverage, signing status and the [security-reporting channel](../SECURITY.md) alongside the release. Do not describe unperformed hardware checks as passed.

A version tag starts the candidate workflow. Use manual Packages for ordinary test builds. Existing releases, including drafts, are never edited or overwritten by this workflow.

## Publish under the maintainer’s personal identity

Future maintainer commits use `Wuhan Zhang <122805853+endNone@users.noreply.github.com>`. Use the authenticated `endNone` GitHub CLI session to **create** the Release, not an Actions token. Editing a bot-created draft does not change its author. Existing published releases and tags are retained unchanged.

After downloading and verifying the candidate, substitute the reviewed tag, revision and candidate path below. Run from the repository and review the final release notes before publication:

```sh
release_tag=vX.Y.Z
source_revision=REPLACE_WITH_VERIFIED_40_CHARACTER_REVISION
candidate=/path/to/candidate
test "$(gh api user --jq .login)" = endNone &&
test "$(gh api "repos/endNone/ClickPulse/commits/$release_tag" --jq .sha)" = "$source_revision" &&
gh release create "$release_tag" "$candidate"/release/* --repo endNone/ClickPulse --verify-tag --draft --title "ClickPulse ${release_tag#v}" --notes-file "$candidate/release-notes.md"
```

Verify the draft's author is `endNone`, all fourteen expected assets and hashes match the candidate, and the notes contain download links, changes, upgrade instructions and known limitations. Then publish the reviewed stable version:

```sh
gh release edit "$release_tag" --repo endNone/ClickPulse --draft=false --prerelease=false --latest
```

Do not delete/recreate a published release to change attribution. Do not store personal authentication tokens in source or workflow files. Public downloads, version tags and old release checksums must be checked after publishing. Finally update the [Homebrew and Scoop manifests](package-managers.md#maintainer-keep-the-channels-current) for the newly published version.

## Download assets

The 0.3.1 release package set contains fourteen attachments. Windows releases contain the portable ZIP; the former Setup download has been withdrawn. Versions in filenames come from project metadata; the workflow verifies the complete set before retaining the candidate.

| Asset | Purpose |
| --- | --- |
| `ClickPulse-0.3.1-macos-arm64.dmg` | Mac drag-to-install package |
| `ClickPulse-0.3.1-macos-arm64.zip` | Complete Apple Silicon App ZIP |
| `ClickPulse-0.3.1-macos-x64.dmg` | Intel Mac drag-to-install package |
| `ClickPulse-0.3.1-macos-x64.zip` | Complete Intel App ZIP |
| `ClickPulse-0.3.1-windows-x64.zip` | Windows portable folder |
| `ClickPulse-0.3.1-linux-arm64.zip` | Ubuntu ARM64 portable folder |
| `ClickPulse-0.3.1-linux-x64.zip` | Ubuntu x64 portable folder |
| `ClickPulse-0.3.1-source.zip` | Corresponding source, licenses, build scripts and documentation |
| `ClickPulse-0.3.1-macos-arm64.manifest.json` | Apple Silicon package hashes and source revision |
| `ClickPulse-0.3.1-macos-x64.manifest.json` | Intel package hashes and source revision |
| `ClickPulse-0.3.1-windows-x64.manifest.json` | Windows package file hashes and source revision |
| `ClickPulse-0.3.1-linux-arm64.manifest.json` | Ubuntu ARM64 package hashes and source revision |
| `ClickPulse-0.3.1-linux-x64.manifest.json` | Ubuntu x64 package hashes and source revision |
| `SHA256SUMS.txt` | SHA256 values for the thirteen other attachments |

The current packages are available in the [0.3.1 Release](https://github.com/endNone/ClickPulse/releases/tag/v0.3.1). Installation is covered by the [Mac](install-macos.md), [Windows](install-windows.md) and [Ubuntu](install-linux.md) guides.

## Validation and failures

Preparation stops on missing assets, mismatched versions/build numbers, incorrect outer or package-file checksums, unlisted files, missing GPL text, or different binary/source revisions. Database files, logs, environment files and private notes cannot become payloads. Only declared assets are copied; unrelated files in `dist/` are excluded.

The source ZIP and all five platform manifests must record the tagged commit. Packages made outside a Git checkout lack verified provenance and remain local test packages; rebuild from the tagged source for release.

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

The Mac preview remains ad-hoc signed without notarization; Windows packages remain unsigned. Installation notes explain manual authorization. Daily and manual checks are included. From 0.3.1, ordinary installations download updates automatically and wait for confirmation in Settings before installation and restart.
