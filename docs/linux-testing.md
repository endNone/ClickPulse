# Linux validation

The Linux port targets Ubuntu 22.04 and 24.04. Native desktop validation must be recorded for each OS and architecture; ARM64 in UTM does not establish x64 or physical-PC coverage. The x64 GitHub Actions matrix runs compilation and unit checks, not desktop or hardware acceptance.

## Repeatable checks

1. Build on Ubuntu 22.04 with the checked-in lockfile: `cargo test --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings`, then `python3 scripts/build.py`.
2. Verify the ZIP checksum and every file against `build-manifest.json`. Record the source revision, internal build and architecture. Run `ldd clickpulse` and require no missing libraries.
3. Install the complete folder as a normal desktop user and grant recording access as documented in [Ubuntu installation](install-linux.md). Test a logged-in desktop, not a root-launched app.
4. Check keyboard/mouse counts, held-key repeat suppression, pause/resume, device reconnect, history and heatmap, Mini/full transitions, close-to-tray, and save/quit/restart with retained history.
5. Check language/theme persistence, launch-at-login, manual and daily update settings. A current-version check does not prove newer-release installation. Package validation and rollback tests use disposable fixtures.
6. Copy the **same** 22.04-built ZIP to 24.04 and repeat. Retain its checksum in the results; do not rebuild on 24.04 to hide a baseline compatibility failure.

`tests/linux_capture.rs` is an opt-in native kernel integration test. It creates a uniquely named uinput device, sends known key/repeat/button events, pauses/resumes capture, reconnects a device, and reopens the temporary SQLite history. Ordinary unit tests do not emit system input. Run this integration test only in an isolated test VM with access to `/dev/uinput`. Before running it, install a temporary udev rule matching exactly `ATTRS{name}=="ClickPulse isolated capture test"` and assigning `ENV{LIBINPUT_IGNORE_DEVICE}="1"`, then reload udev rules. The test verifies this property before emitting any input. Set `CLICKPULSE_NATIVE_CAPTURE_TEST=1` and run `cargo test --test linux_capture -- --ignored --nocapture --test-threads=1`. Do not type or click in that VM during its exact-count assertions. Remove the temporary rule and reload udev afterward.

## Desktop limits

Wayland controls window placement and always-on-top behavior; requested geometry/pinning may be constrained by the compositor. GNOME's tray requires an enabled AppIndicator extension. The collector reads evdev without grabbing devices and uses the same capture path on X11 and Wayland. Desktop environments other than Ubuntu GNOME, physical special keyboards, fractional scaling/multiple monitors, and suspend/resume need separate validation.

## Local validation record — 2026-11-12

Tested sequentially on Apple Silicon UTM, as the normal desktop user in GNOME Wayland. Each VM has 4 CPUs and 4 GB RAM. Ubuntu 22.04.5 ARM64 (kernel 6.8.0-149) built the package; Ubuntu 24.04.5 ARM64 (kernel 7.0.0-43) installed that exact ZIP without rebuilding it.

- Version: 0.2.1, internal build 78 (local Linux candidate, not a public release).
- Code revision: `c6c46779cd99a2d03616d8ad89f367585af70add`.
- Archive: `ClickPulse-0.2.1-linux-arm64.zip`.
- SHA256: `4d862b9e498d491bbec57cbb4f4801dea56b9138167bfabccfb36eeb96844ced`.
- Both installations verified every manifest payload hash, executable version and `ldd` with no missing libraries.

| Check | Ubuntu 22.04 ARM64 | Ubuntu 24.04 ARM64 |
| --- | --- | --- |
| Normal-user Wayland launch and live keyboard/mouse counts | Pass | Pass |
| Pause stops counts; resume records again | Pass | Pass |
| Daily history and Linux keyboard heatmap | Pass | Pass |
| Mini/full, unmaximize, then re-maximize | Pass after GTK fix | Pass, same fixed package |
| Hide to tray and reopen | Pass | Pass |
| Chinese/theme switching | Pass | Pass |
| Manual update check and daily-check toggle | Pass | Pass |
| Save and quit | Pass | Pass |
| Reboot/login autostart and retained history/settings | Pass | Pass |
| Isolated native evdev integration | Pass (6.14 s) | Pass (5.74 s) |

The native test verifies 4 key presses and 1 mouse click, ignored repeat events, pause/resume, device reconnect and reopened SQLite history. Its temporary exact-name udev rule was removed afterward. Desktop checks separately establish capture under the ordinary user's `input` group.

Both guests were actually rebooted and logged in again: the managed desktop entry started ClickPulse as the normal user with `--background`, the tray reopened its window, and saved Chinese/Teal preferences and existing counts remained. On 24.04, the 58 pre-reboot events were retained and new input raised the displayed total to 63. Test preferences were then restored to Blue/System default, login autostart off, daily update checks on and Sport off. The 22.04 app was saved and quit; the 24.04 app was left open on Overview.

Ubuntu 22.04 automated checks passed: 140 library tests, 23 desktop tests and 8 CLI tests; 3 unrelated opt-in library tests remained ignored. The native test was run explicitly. Clippy with warnings denied and the final release build passed. Host regression checks passed 175 Rust tests, 113 JavaScript tests and 32 Python packaging tests. The last code-only change synchronizes GTK resize state before Tao's delayed maximization; it was compiled/linted and tested through real Mini/full/unmaximize/re-maximize cycles on both guests.

A current-version update response does not validate downloading/installing a newer public Linux release. Linux package/ELF validation and rollback use test fixtures, and the daily schedule is covered by clock/restart tests; no 24-hour soak is claimed. The build manifest's `native_linux_tested` field remains its build-time value; this acceptance record is separate evidence.

No x64 desktop test or remote Linux CI run is claimed. Physical special keys, suspend/resume, fractional scaling/multiple monitors, X11 and other desktop environments remain outside this validation. These VMs use software graphics fallback; successful rendering does not establish physical-GPU performance.
