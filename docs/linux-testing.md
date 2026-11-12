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

## Local validation record

Ubuntu 22.04.5 ARM64, GNOME Wayland, UTM on Apple Silicon: Rust unit/CLI checks and the isolated native evdev test passed on 2026-11-12. The actual desktop rendered live counts, daily history, Linux keyboard heatmap and Mini; save-and-quit returned to the desktop. Final portable-package desktop checks and the same-artifact Ubuntu 24.04 run are tracked in the task evidence before acceptance. No x64 desktop or public Linux release is claimed by this record.
