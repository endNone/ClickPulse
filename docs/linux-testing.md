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

## Update follow-up — 2026-11-12

Ubuntu 24.04 ARM64 passed 23 updater checks and 9 daily-update controller checks. The installed App's manual check also returned “up to date”, confirmed by the user. The public release is still 0.2.1 and has no Linux download.

The helper-process test now runs on Linux. It waits for the parent to finish saving, replaces a disposable program, then uses the real restart launcher to start a small compiled Linux probe. The probe confirms the preserved pause argument; a seeded SQLite database and preference file remain byte-for-byte unchanged. macOS updater regression checks also passed (23).

This establishes local replacement and process restart, not a complete GitHub-to-App Linux upgrade. That still needs a newer Linux release package. Daily checks use controlled timestamps rather than a 24-hour waiting test. The user's installed program, history and preferences were unchanged by this follow-up.

## 0.3.0 local candidate validation — 2026-11-12

The isolated `Release030` worktree combines the Linux changes and the local documentation draft. The local review retained a file-by-file source snapshot before the release preparation commit; base revision alone did not identify those working changes.

The Ubuntu 22.04 ARM64 package was installed on both Ubuntu versions. Each old 0.2.1 updater replaced the actual program, restarted it as the normal desktop user, and retained every existing statistics row/count. Both databases passed integrity checks and every installed package file matched its recorded hash. This uses a locally staged real package, not a published GitHub download. Manual online checks and the daily schedule have separate earlier coverage.

Both normal-user desktops showed the new 0.3.0 interface. The user confirmed Mini/full round trips on 22.04. On 24.04, a pink/brown highlight appeared after expanding Mini and disappeared after clicking. The saved theme remained Blue. A focused fix clears the old text selection only when switching modes, preserving copy selections during ordinary live refreshes. The user then confirmed two round trips, including selecting a Mini count before expanding. No speculative native transparency change remains. Final selection behavior is covered across all three platforms by the interface regression suite; the native selection retest was on 24.04.

Native isolated input checks passed on both guests: repeated key events are ignored, pause/resume works, device reconnect works, and saved history reopens correctly. The temporary input-device exclusion rules were removed afterward. The 24.04 test executable was copied onto its local disk before running; an earlier attempt to execute it from the shared folder failed before test output.

Automated checks: macOS 175 Rust tests; Ubuntu 22.04 ARM64 and emulated Ubuntu 22.04 x64 each 173 Rust tests; 116 JavaScript checks; 33 Python packaging checks. Opt-in network and stress checks are not included. Warnings-denied Rust checks passed on macOS and ARM64 Ubuntu. macOS ARM64/Intel and Linux ARM64/x64 packages were rebuilt after the selection fix. x64 Linux also passed dependency resolution and command-line startup inside its Ubuntu 22.04 container; no x64 desktop or physical Intel Mac validation is claimed.

Windows acceptance is assigned to the user. Final committed-source packaging, Windows results, distribution-list updates, and an online download of the newly published release remain release steps. No 24-hour soak, physical multi-display/fractional scaling, suspend/resume, or X11 acceptance is claimed.

## Final code validation — 2026-11-12

Both Ubuntu ARM64 versions passed 179 Rust checks after the background and cached-insert changes. Warnings-denied checks passed on Ubuntu 22.04, where the compatible ARM64 package was built. The explicit native input check passed again on 22.04 (5.64 seconds) and 24.04 (4.90 seconds): four physical-device test keys, one click, repeat filtering, pause/resume, reconnect and persisted history. The test device was isolated from desktop input and its temporary rule was removed.

The final Linux package replaced the program on both guests after normal exit, with separate backups of the previous program and SQLite profile. The data passed integrity checks. Ubuntu 24.04 then ran that package in its normal Wayland session with its window hidden for the [resource comparison](validation/resource-usage-2026-11-12.md). The subsequent Caps Lock correction is macOS-only.

This extends the native desktop and update checks above; it does not add unperformed x64 desktop, physical Intel Mac or Windows hardware coverage.
