# Background resource usage — 2026-11-12

## Measurement

Ubuntu 24.04 ARM64, Wayland, hidden desktop App with capture enabled. The baseline ran for 600.002 seconds; input totals stayed unchanged throughout.

| Metric | Before | After |
| --- | ---: | --- |
| Regular-file write system calls | 1,136 | Pending manual validation |
| File synchronization calls | 284 | Pending manual validation |
| Application thread wakeups | 73,497 | Pending manual validation |
| Storage bytes attributed to the process family | 41,517,056 | Pending manual validation |
| Process-family CPU time | 7.55 seconds | Pending manual validation |

Writes count regular-file write/pwrite calls, not physical SSD operations. Wakeups count scheduler wakeup and new-thread wakeup events for ClickPulse and its WebKit children, not hardware CPU idle exits. Fork ancestry includes short-lived threads; no histogram entries were dropped. Global system totals were excluded. Both measurements must use the same process-tree and file-descriptor filtering.

Baseline binary SHA-256: `7deb16dfab969a55d86073460b3e3c1135aa2eed5d1b233ca5a55fcc56a78aba`.

## Changes

- Write the collector heartbeat every 60 seconds, including startup and a cleared heartbeat on normal exit. Read-only sources accept a heartbeat for up to 180 seconds.
- Use WAL with NORMAL synchronization for CollectorDb. SQLite preserves consistency, but abrupt power loss can lose recent commits that FULL would have synchronized.
- Suppress native script evaluation while hidden or minimized; refresh the hidden desktop loop every five seconds.
- Refresh the full snapshot every two seconds. Mini requests only today's counts and capture state, without history or keyboard distributions.
- Reuse a single data worker across snapshot and storage jobs. Input still flushes in bounded batches.

Existing Linux input polling remains at eight milliseconds, so a large reduction in total wakeups is not assumed.

## Validation status

Mac Rust tests: 180 passed. Ubuntu ARM64 Rust tests: 178 passed. Interface tests: 118 passed. Mac Clippy passed. Tests cover time zones, daylight-saving boundaries, lightweight queries, heartbeat expiry, worker reuse and shutdown, and Mini/full snapshot routing.

The maintainer requested release before the second ten-minute measurement and subsequent physical checks. No post-change performance improvement is claimed. Repeat the same idle measurement after a 60-second warm-up, then manually verify Mini/full transitions, pause/resume, background recording and normal exit.
