# Background resource usage — 2026-11-12

## Measurement

Ubuntu 24.04 ARM64, Wayland, hidden desktop App with capture enabled. The baseline ran for 600.002 seconds and the final run for 600.002 seconds. Keyboard and mouse totals remained unchanged within both runs. The final package warmed up for at least 90 seconds before measurement.

| Metric | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| Regular-file write system calls | 1,136 | 40 | 96.5% |
| File synchronization calls | 284 | 0 | 100.0% |
| Application thread wakeups | 73,497 | 64,094 | 12.8% |
| Storage bytes attributed to the process family | 41,517,056 | 2,715,648 | 93.5% |
| Process-family CPU time (seconds) | 7.55 | 5.76 | 23.7% |

Writes count regular-file write/pwrite calls, not physical SSD operations. Wakeups count scheduler wakeup and new-thread wakeup events for ClickPulse and its WebKit children, not hardware CPU idle exits. Fork ancestry includes short-lived threads; all final threads were accounted for and no histogram entries were dropped. Global system totals were excluded. Both runs used the same process-tree and file-descriptor filtering.

This is one before/after observation in a virtual machine, not a general hardware performance guarantee. The second run followed a guest restart and used the retained profile. The host remained usable during both measurements. Existing Linux input polling remains at eight milliseconds, which accounts for most remaining wakeups.

Baseline binary SHA-256: `7deb16dfab969a55d86073460b3e3c1135aa2eed5d1b233ca5a55fcc56a78aba`. Final Linux binary SHA-256: `48b8a61cd73b4295233d6ce64f9cb55c6d769580b6b219948680ebb2ba564128`. The later physical Caps Lock correction affects macOS only.

## Changes

- Write the collector heartbeat every 60 seconds, including startup and a cleared heartbeat on normal exit. Read-only sources accept a heartbeat for up to 180 seconds.
- Use WAL with NORMAL synchronization for CollectorDb. SQLite preserves consistency, but abrupt power loss can lose recent commits that FULL would have synchronized.
- Suppress native script evaluation while hidden or minimized; refresh the hidden desktop loop every five seconds.
- Refresh the full snapshot every two seconds. Mini requests only today's counts and capture state, without history or keyboard distributions.
- Reuse one data worker across snapshot and storage jobs, and cache insert statements. Input still flushes in bounded batches.

## Validation

Mac Rust tests: 184 passed. Each Ubuntu ARM64 version: 179 Rust tests passed, plus an explicit native input integration check. Interface tests: 118 passed. Packaging tests: 33 passed. Warnings-denied checks passed on Mac and Ubuntu 22.04. Coverage includes time zones, daylight-saving boundaries, lightweight queries, heartbeat expiry, worker reuse, queued shutdown, Mini/full routing, transactional insert rollback and native input persistence.

The first post-change run received 57 keyboard events during simultaneous physical-key testing. It was excluded and replaced with the clean run above. Raw results and excluded evidence are retained locally; no contaminated run is reported as idle performance.
