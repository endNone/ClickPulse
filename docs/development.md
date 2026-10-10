# Development

## Prerequisites

Use Rust stable and Python 3.11+ with the checked-in `Cargo.lock`. Node.js 24 is needed for interface tests and screenshot preparation, not for running ready-built packages. No npm dependencies are required.

macOS packaging needs Xcode Command Line Tools and the system `ditto`, `sips`, `iconutil`, `codesign` and `hdiutil`. Mac packages target Apple Silicon (arm64) or Intel (x64), both on macOS 12+.

Native Windows builds need the Rust MSVC toolchain, Visual Studio C++ Build Tools and Windows SDK. Windows packaging produces the portable ZIP.

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
| macOS arm64 | `ClickPulse.app`, `ClickPulse-0.1.0-macos-arm64.dmg`, matching ZIP, SHA256 files and manifest |
| macOS Intel x64 | `ClickPulse-macos-x64.app`, `ClickPulse-0.1.0-macos-x64.dmg`, matching ZIP, SHA256 files and manifest |
| Windows | `ClickPulse-0.1.0-windows-x64.zip`, SHA256 file and manifest |

Mac builds default to the host architecture. To cross-build Intel packages on an Apple Silicon Mac:

```sh
rustup target add x86_64-apple-darwin
python3 scripts/build.py --target x86_64-apple-darwin
```

Use `--target aarch64-apple-darwin` explicitly for Apple Silicon. The separate Intel output preserves the existing arm64 App; both archives contain `ClickPulse.app`.

The Mac App is ad-hoc signed, not notarized. The Windows executable is unsigned, including in the portable ZIP. Windows MSVC packages statically link the C runtime and reject external Visual C++ imports. Package checks do not replace desktop testing or establish publisher trust.

Cross-build Windows on macOS after installing MinGW-w64 and the Rust target:

```sh
rustup target add x86_64-pc-windows-gnu
python3 scripts/build.py --platform windows
```

The existing `package-macos.sh` and `package-windows.ps1` wrappers use this same entry point. `package_windows.py` remains available for lower-level portable builds.

## Checks and isolated previews

```sh
cargo run --locked -- --demo
cargo fmt --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
node --test tests/history.test.cjs tests/i18n.test.cjs tests/keyboard.test.cjs tests/rendering.test.cjs tests/dropdown.test.cjs tests/themes.test.cjs
python3 -m unittest discover -s tests -p 'test_*.py'
```

Demo mode does not open a statistics database or capture input. Its metrics, daily history, hourly totals and keyboard counts use one consistent generated dataset. Use synthetic data for documentation screenshots and temporary directories for storage tests. Rust storage unit tests do not start native input listeners or write to the normal App log. Public tests are self-contained; legacy tests that required a separate personal Python/dashboard checkout are excluded.

Desktop refresh queues one snapshot at a time without waiting on the interface thread. The storage worker reads and serializes the result, then wakes the event loop. Additional refreshes are coalesced; slow completed results are still delivered. Hidden windows do not request full snapshots. This removes the synchronous interface wait; full-history payloads are still sent while the window is visible, so incremental history delivery remains a future optimization.

`.github/workflows/ci.yml` checks macOS/Windows code. Packages reuses those checks before building CI artifacts. Release draft runs the same pipeline for a version tag, verifies corresponding source and complete packages, then creates an unpublished preview draft. See [Builds and releases](releases.md). The initial private preview has passed the GitHub checks and complete packaging/release workflow on Apple Silicon, Intel Mac and Windows x64.

## Paths and explicit migration

CLI help, statistics and success messages use English. The desktop app retains English and Simplified Chinese interfaces.

Default desktop startup uses the platform's ClickPulse user-data directory and creates its database there. Replacing, moving or uninstalling program files does not move that data. A missing WebView2 runtime is detected before starting the collector. Startup and installation never automatically pick a legacy recorder, database or service.

```sh
cargo run --locked -- stats --json
cargo run --locked -- export --output /path/to/new-activity.csv
cargo run --locked -- --source-db /path/to/existing.sqlite3 stats --json
cargo run --locked -- migrate --source /path/to/existing.sqlite3 --output /path/to/new.sqlite3
```

Open the desktop app once to initialize a fresh default database; CLI read operations do not create a missing one. `--source-db` is read-only. Migration requires a new destination and preserves the source; stop the old recorder yourself before migrating. It does not merge overlapping histories, overwrite the app's existing database or stop any service. Keep complete backups with SQLite backup or after normal shutdown so WAL records are included.

Advanced `--db` selects independent storage; explicit `--legacy-db`/`import` remain available for compatible legacy formats. Installed users keep using their existing App database and preferences.

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

Project metadata identifies the **0.1.0** private preview (internal build 52). Use a new public version for each tagged release; never move a published tag or replace its packages with a different source revision. Internal build numbers identify unpublished revisions. Keep Cargo, macOS public version and Windows ProductVersion aligned. Verify signatures, versions, manifests, checksums, third-party licenses and absence of user data before distribution.

Project code is licensed under [GPL-3.0-or-later](../LICENSE). Source archives and both platforms' packages include the full license; fonts and other third-party components retain their own licenses. Signing, native Windows validation and public release remain pending. See the [roadmap](roadmap.md). Local scripts do not create repositories or publish Releases; the tag workflow creates an unpublished draft for manual review.
