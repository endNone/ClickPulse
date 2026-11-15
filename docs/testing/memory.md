# Memory measurement

Keep the published baseline and candidate on the same machine, with identical data,
languages, window dimensions and idle intervals. Do not compare absolute numbers
between operating systems: their memory accounting differs.

## Interface comparison

`scripts/measure_memory.py prepare SOURCE DISPOSABLE_COPY` creates an instrumented
copy that uses synthetic data, default preferences, native tray actions and an isolated
renderer cache. It never records input or modifies the installed app. Build that copy
with `cargo build --release --locked --manifest-path DISPOSABLE_COPY/Cargo.toml`.
Use separate target directories for different source copies, or run `cargo clean
--release -p clickpulse --manifest-path DISPOSABLE_COPY/Cargo.toml` before switching.
Preserved timestamps can otherwise reuse a different same-version executable.
Keep binary hashes as well as source revisions.

Run the executable with `scripts/measure_memory.py run BINARY OUTPUT --label REVISION`
on Linux or Windows; Windows requires `psutil==7.2.2`. On macOS use
`scripts/measure_macos_memory.py BINARY OUTPUT --label REVISION`, which launches a
separate application coalition. Every output directory must be new.

The states are unopened background, full English, full Chinese, Mini, hidden for 610
seconds, and reopened. Each visible state settles for 35 seconds. Take five samples at
one-second intervals and report the median. Preserve process-level samples and source
revisions. The probe also checks decoded brand images, loaded font faces, view disposal,
and the new page's ready acknowledgement after recreation. The final report records
normal quit separately from an unexpected process exit. macOS also samples main-process
CPU time, converting `libproc` Mach ticks with the host's timebase.

Report the main process, the rendering process family, and their combined measured
footprint. Include helper processes even if they become reparented:

- macOS: physical footprint from `libproc`, with raw `footprint` reports as corroboration.
- Linux: proportional resident size from `/proc/PID/smaps_rollup`.
- Windows: private working set; also preserve resident size and private committed bytes.

A hosted Windows Server run is not a physical Windows 10/11 desktop test. Record the
Linux graphics configuration; software rendering can materially increase WebKit costs.
A page-ready message alone does not verify images or fonts. The UI workload omits the
recorder and must be identified as such in every results table.

## Recorder comparison

Use `scripts/measure_runtime_memory.py prepare SOURCE DISPOSABLE_COPY`, build its
`memory_runtime` binary, and run it with the same script's `run BINARY OUTPUT --label
REVISION` command. This uses the real capture and storage code, but starts with an empty
isolated database and redirects its status log to the output directory. It records only
the counts normally collected by ClickPulse and never changes input permissions.

After 20 seconds of startup, collect ten one-second samples. The report includes native
capture status: a denied or unavailable listener must not be described as active capture.
Do not add this standalone process measurement to the separate UI measurement and call
the sum an observed installed-app total.

The manual `Memory comparison` workflow accepts an exact 40-character candidate revision
and either the `interface` or `recorder` scope. It measures both versions on the same
Windows host and retains the raw reports as artifacts. Publication is a separate action.
