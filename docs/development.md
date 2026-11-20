# Development

## Prerequisites

Use Rust stable and Python 3.11+ with the checked-in `Cargo.lock`. Node.js 24 is needed for interface tests and screenshot preparation, not for running ready-built packages. No npm dependencies are required.

macOS packaging needs Xcode Command Line Tools and the system `ditto`, `sips`, `iconutil`, `codesign` and `hdiutil`. Mac packages target Apple Silicon (arm64) or Intel (x64), both on macOS 12+.

Native Windows builds need the Rust MSVC toolchain, Visual Studio C++ Build Tools and Windows SDK. Windows packaging produces the portable ZIP.

Linux builds need `build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev libxdo-dev libdbus-1-dev`. Build native ARM64 or x64 packages on Ubuntu 22.04 to retain the older glibc baseline, then test the same package on Ubuntu 24.04. When using a VM shared checkout, keep `CARGO_TARGET_DIR` on the guest’s own filesystem. The Linux packager supports Python 3.10, as shipped by Ubuntu 22.04.

Notifications use the pure Rust `zbus` backend, but the window library still needs `libdbus-1-dev`. Linux checks also install `notification-daemon`, `xvfb` and `dbus-x11` to verify two notification deliveries on an isolated desktop session; these are test dependencies, not additional build requirements.

## Common build entry point

On macOS:

```sh
python3 scripts/build.py
```

On Windows:

```powershell
python scripts/build.py
```

These commands only generate artifacts in `dist/`. They do not install or start ClickPulse, change permissions, register login startup, migrate databases or clean user history.

| Platform | Output |
| --- | --- |
| macOS arm64 | `ClickPulse.app`, `ClickPulse-X.Y.Z-macos-arm64.dmg`, matching ZIP, SHA256 files and manifest |
| macOS Intel x64 | `ClickPulse-macos-x64.app`, `ClickPulse-X.Y.Z-macos-x64.dmg`, matching ZIP, SHA256 files and manifest |
| Windows | `ClickPulse-X.Y.Z-windows-x64.zip`, SHA256 file and manifest |
| Ubuntu ARM64 / x64 | `ClickPulse-X.Y.Z-linux-arm64.zip` / `ClickPulse-X.Y.Z-linux-x64.zip`, SHA256 file and manifest |

Mac builds default to the host architecture. To cross-build Intel packages on an Apple Silicon Mac:

```sh
rustup target add x86_64-apple-darwin
python3 scripts/build.py --target x86_64-apple-darwin
```

Use `--target aarch64-apple-darwin` explicitly for Apple Silicon. The separate Intel output preserves the existing arm64 App; both archives contain `ClickPulse.app`.

Public Mac packages from 0.3.2 use the fixed self-signed release certificate recorded in `assets/macos-signing.json`. First migration from an ad-hoc build requires a one-time Input Monitoring grant. Apple does not notarize these packages. The Windows executable is unsigned, including in the portable ZIP. Windows MSVC packages statically link the C runtime and reject external Visual C++ imports. Package checks do not replace desktop testing or establish publisher trust.

Cross-build Windows on macOS after installing MinGW-w64 and the Rust target:

```sh
rustup target add x86_64-pc-windows-gnu
python3 scripts/build.py --platform windows
```

The existing `package-macos.sh` and `package-windows.ps1` wrappers use this same entry point. `package_windows.py` remains available for lower-level portable builds.

Release builds use thin LTO, one codegen unit and stripped symbols. Normal packaging temporarily retains line-table debug information, saves the matching dSYM (macOS), PDB (MSVC) or separate debug file (Linux/MinGW) under `dist/symbols/`, and strips only the staged executable. Cargo’s original binary is left intact. CI retains these symbols as a separate artifact; they are not included in download packages. Keep them with the corresponding build for crash diagnosis. Symbol manifests identify the executable before signing; package manifests record the final signed files. `--skip-build` packages the supplied binary as-is and does not generate or guarantee debug symbols.

On Linux, run `python3 scripts/build.py`. A shared Git worktree whose `.git` path only resolves on the host can use `python3 scripts/package_linux.py --source-revision FULL_COMMIT_SHA`; supply the actual revision from the host. Linux package generation is local; it does not add assets to an existing public release. See [Linux validation](linux-testing.md) for the distinction between unit, native-input and desktop checks.

## Checks and isolated previews

```sh
cargo run --locked -- --demo
cargo fmt --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
node scripts/test-ui.cjs
python3 -m unittest discover -s tests -p 'test_*.py'
```

`node scripts/test-ui.cjs` discovers every `*.test.cjs` recursively under `tests/`, including the insights and shortcut-guide suites. Discovery runs in Node, so the same command works without shell wildcard expansion on macOS, Windows and Linux. Use `node scripts/test-ui.cjs --list` to inspect the discovered files.

On Windows, the WinGet symbolic-link test needs link-creation permission. Without it, only that test returns early with a `SKIPPED` diagnostic (`cargo test -- --nocapture` shows it). Other failures remain fatal. `CLICKPULSE_REQUIRE_SYMLINK_TEST=1` requires this coverage; the Windows checks set it automatically.

Demo mode does not open a statistics database or capture input. Its metrics, daily history, hourly totals and keyboard counts use one consistent generated dataset. Use synthetic data for documentation screenshots and temporary directories for storage tests. Rust storage unit tests do not start native input listeners or write to the normal App log. Public tests are self-contained; legacy tests that required a separate personal Python/dashboard checkout are excluded.

Full windows request a snapshot every 2 seconds; Mini requests counts only every 500 ms. Queries use a reusable worker without waiting on the interface thread, with at most one snapshot in flight. Additional requests are coalesced; slow completed results are still delivered. Pause/Resume is acknowledged independently of those statistics. Exclusive app writers cache calendar totals by timezone and apply committed input deltas, including late input. Imports invalidate the cache; reopening, relocation and timezone changes rebuild it. Hour and rolling windows remain fresh bounded reads. Read-only external sources cannot see each write, so they cache only minutes older than two days and rescan that tail on every refresh. A late write or import before that point changes lifetime-minus-tail totals and rebuilds immediately; deleted old details are picked up by a rebuild at least every 10 minutes. Hidden windows check status every 5 seconds without requesting snapshots. Calendar projection starts from a non-calendar snapshot rather than querying local-day totals first. Preferences are saved only after a value changes; renderer visibility and readiness do not trigger disk writes. Timezone choices are cached and delivered once to each new renderer. Status pushes and live chart/calendar updates reuse unchanged values and nodes. Full windows still receive history payloads, so incremental history delivery remains a future optimization.

`.github/workflows/ci.yml` checks macOS/Windows code and Ubuntu 22.04/24.04 x64 code. The Linux matrix is configured here; a local ARM64 VM pass does not mean that the hosted x64 jobs have run. Packages reuses those checks before building CI artifacts. Release candidate runs the same pipeline for a version tag, verifies corresponding source and complete packages, then retains a candidate artifact. The maintainer creates and publishes the Release through the personal `endNone` account. See [Builds and releases](releases.md). The initial preview passes the GitHub checks and complete packaging/release workflow on Apple Silicon, Intel Mac and Windows x64.

For an interactive Mini Sport preview, run `node scripts/prepare-sport-preview.cjs` and serve `dist/sport-preview/` locally. The preview shows the production Warp Mini window using simulated click counts. Try C/B/A/S/SS, the Mini lightning switch, or the link to the actual Personalization settings. This does not open a database or start a capture listener.

To profile full snapshots with isolated synthetic data (365 days, temporary databases, no capture or installed App):

```sh
cargo test --locked --test performance -- --ignored --nocapture
```

Compare the same build profile and fixture before and after a change. These timings measure snapshot work, not total App memory or CPU. Linux input polling retains descriptor arrays until devices change and reuses its event buffer; physical input and hotplug acceptance still need the native test environment described in `linux-testing.md`.

## macOS titlebar compatibility

The integrated titlebar with custom traffic-light alignment is enabled only on Apple Silicon with macOS 26 or later. macOS 12–25 and Intel Macs use the system’s default titlebar layout, including after leaving Mini mode. These environments have not received native visual validation for the custom layout; the fallback avoids moving their AppKit button containers. Fullscreen, resizing and theme changes on those systems still need a native smoke test before expanding custom-titlebar support.

## Paths and explicit migration

CLI help, statistics and success messages use English. The desktop app shares eight complete catalogs across native controls and the WebView: English, Simplified Chinese, Traditional Chinese, Japanese, Korean, French, German and Spanish. Keep locale matching and English fallback consistent in Rust and JavaScript, and include native bundle localization resources when adding a language.

Default desktop startup uses the platform's ClickPulse user-data directory and creates its database there. Replacing, moving or uninstalling program files does not move that data. A missing WebView2 runtime is detected before starting the collector. Startup and installation never automatically pick a legacy recorder, database or service.

```sh
cargo run --locked -- stats --json
cargo run --locked -- --source-db /path/to/existing.sqlite3 stats --json
cargo run --locked -- migrate --source /path/to/existing.sqlite3 --output /path/to/new.sqlite3
```

Open the desktop app once to initialize a fresh default database; CLI read operations do not create a missing one. `--source-db` is read-only. Migration requires a new destination and preserves the source; stop the old recorder yourself before migrating. It does not merge overlapping histories, overwrite the app's existing database or stop any service. Keep complete backups with SQLite backup or after normal shutdown so WAL records are included.

Advanced `--db` selects independent storage; explicit `--legacy-db`/`import` remain available for compatible legacy formats. Installed users keep using their existing App database and preferences. Settings identifies default, custom, read-only and demo storage. Only the default App database can manage login startup; independent sessions cannot change the default recorder's startup entry.

On Windows, the data root comes from an absolute `LOCALAPPDATA` path, never the current working directory. Missing, empty or relative values fail rather than creating data beside a source checkout. For isolated checks prefer `--demo` or an explicit temporary `--db` path. A process-local `LOCALAPPDATA` override also redirects preferences/logs/WebView data; label that as a test profile and never show its path as the installed app's default or persist the override to the user's environment.

Recovery journals from previously authorized handoffs retain their safety-recovery behavior.

The old macOS recorder handoff is a restricted compatibility path, not automatic discovery for new users. General migration uses the explicit source/output arguments above and does not alter launch agents or stop another recorder.

## Storage compatibility

Both writable database formats use `PRAGMA user_version`. Existing version-zero databases upgrade transactionally to version 1; a higher schema version is rejected before writes. Updates retain counts, complete history and cached historical key eras. Read-only source access does not upgrade the source.

App storage also records `key_encoding_version` and the legacy encoding boundary as an absolute minute bucket in `clickpulse_metadata`. Physical codes have their own namespace. Cached historical eras take precedence over date inference, and upgrades never reclassify them using the current timezone.

For an unversioned legacy source with no stored boundary, migration can explicitly specify the original encoding-change timestamp and UTC offset:

```sh
cargo run --locked -- migrate --source /path/to/legacy.sqlite3 --output /path/to/new.sqlite3 --legacy-key-cutoff '2026-06-05T12:46:00+08:00'
```

The timestamp is an example; use the recorder's actual change time and original offset. The override is only for sources without cached historical eras and is stored in the destination, never written to the source. Omitting it preserves the original recorder's compatibility fallback for June 5, 2026 at 12:46 in the migration machine's local timezone. An unversioned source cannot reveal its original timezone, so use an explicit offset when migrating it across timezones. New physical-only databases do not depend on that date.

## Version and publication

The latest published version is **0.3.1**; read the current internal build from `assets/Info.plist` and `assets/windows/app.rc` instead of a fixed documentation example. The app shows the public version only; `--version` and package manifests include the internal build for diagnostic reports. `build.rs` validates the Windows and macOS package versions and embeds that build in the executable. Use a new public version for each tagged release; never move a published tag or replace its packages with a different source revision. Internal build numbers identify unpublished revisions. Keep Cargo, macOS public version and Windows ProductVersion aligned. Verify signatures, versions, manifests, checksums, third-party licenses and absence of user data before distribution.

The updater checks daily by default in normal desktop sessions, with a persisted 24-hour attempt timestamp and a Settings opt-out; manual checks remain available at any time. Demo, read-only source, setup-pending and independent sessions do not run scheduled checks. A failed attempt is also recorded to prevent startup retry loops; the running app then retries at most three times (5 min, 30 min, 2 h, not persisted) before waiting for the next daily window. Background failures are logged and return to current-version copy (or retain an existing update offer); manual check and installation failures remain visible. Normal-session check and installation-preparation failures are written to the application log with UTC time, request source and operation. Diagnostics and periodic statistics share serialized rotation (10 MiB per file, three archives); log write failures do not change the update result or stop recording. Excluded sessions keep update diagnostics on stderr only. Only normal sessions persist the toggle or attempt time, because other sessions share the preferences file. Successful automatic checks queue a localized eight-second in-app reminder for a newer stable version; a manual check records its result as already announced. The pending version and highest version already shown survive restarts; delivery waits for the full, focused window, so hidden, minimized and Mini views do not consume it. A successful check with no update clears a pending offer. Starting with 0.3.1, a successful check automatically downloads and verifies an update for a writable ordinary installation. The controller retains the temporary staged package in a ready state; only an explicit install request starts the helper. A same-version recheck retains it, a withdrawn or changed offer discards it, and exiting before installation cleans the completed temporary staging. Downloading does not block normal quit. Managed installations never stage packages. The updater uses the fixed public GitHub repository, semantic stable versions and the matching ZIP/manifest/checksums. Downloads are bounded and validated before a helper runs. The helper waits for a successful normal shutdown and the profile lock, revalidates the staged payload and backs up program files before replacement. It never opens the statistics database. Source/demo/independent sessions cannot download or install themselves. The network regression is opt-in: `cargo test --lib updater::tests::published_release_downloads_and_verifies_without_installing -- --ignored`; all ordinary tests use disposable local fixtures.

Project code is licensed under [GPL-3.0-or-later](../LICENSE). Source archives and both platforms' packages include the full license; fonts and other third-party components retain their own licenses. Mac workflows use ad-hoc signatures for ordinary builds and the pinned release identity for public packages. Windows packages are unsigned. Hardware-specific validation is tracked in the Windows checklist and roadmap. See the [roadmap](roadmap.md). Local scripts do not create repositories or publish Releases; the tag workflow retains verified artifacts for personal-account publication.


Linux message dialogs use the existing GTK runtime, including startup recovery before a window exists; `zenity` is not required. Folder selection still uses rfd's portal backend. Native confirmation checks run under Xvfb in an isolated environment.

After a committed update, the restarted App cleans exact recorded staging/backup files in a background worker. Checksums, installation identity and a complete no-link inventory gate deletion; changed or unrecognized files stay in place. Failed replacements retain recovery material. Windows cleanup tolerates the helper remaining open briefly after launch.

Database locks append `.lock` to the full filename. A shared legacy lock also excludes older ClickPulse recorders while allowing separate new databases with the same stem. Settings use synced temporary files and atomic persistence.

### Mac release signing

The release workflow passes only `MACOS_SIGNING_P12_BASE64` and `MACOS_SIGNING_P12_PASSWORD` to the reusable packaging workflow. The two macOS runners import the protected PKCS#12 into temporary keychains and remove them after packaging. A missing secret aborts the release. Local and manually dispatched preview builds default to ad-hoc signing. To sign a release candidate locally, set `MACOS_SIGNING_IDENTITY` to the pinned certificate fingerprint and, if needed, `MACOS_SIGNING_KEYCHAIN` to its unlocked keychain. The separate local development identity stays on its owner’s Mac.

Release preparation runs on macOS and verifies both the updater ZIP and the app mounted from each DMG. It checks the signature, exact designated requirement and SHA-256 fingerprint of the embedded leaf certificate against the repository pin. Packages use no timestamp server. Back up the encrypted release identity and its password separately. Replacing the certificate changes the designated requirement and may require another Input Monitoring grant.
