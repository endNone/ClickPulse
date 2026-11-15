# Background WebView lifetime — approved for 0.3.1

Keep capture, SQLite storage, tray controls and the update controller alive without a
rendering view. The user approved this lifecycle change for 0.3.1 on 2026-11-15, alongside
the font/image optimization. No promise of a 20–40 MiB background total is made before native measurements.

## Ownership and transitions

`desktop.rs` remains the owner of `Runtime`, the reusable snapshot worker, preferences,
tray, update controller, data jobs and pending notices. The WebView becomes an optional presentation object; a lightweight native window
handle remains to preserve tray activation and platform geometry; none of those services may be owned by its callbacks.

| State | Trigger | Next action |
| --- | --- | --- |
| No view | Background launch | Start recorder/tray only; do not assemble HTML or create a WebView/context. |
| No view | Open or Mini | Create the WebView, restore geometry, then wait for the page's ready acknowledgement. |
| Visible full or Mini | Hide/minimize | Stop page work, mark the renderer invisible, start a five-minute timer. |
| Hidden | Open/Mini before expiry | Resume the existing view and cancel the timer. |
| Hidden for five minutes | No active dialog, data job, or installation | Release WebView and its context; retain application services. |
| Hidden with a protected operation | Timer expires | Defer disposal until the operation finishes; recording continues. |
| Visible Mini | Any elapsed time | Keep the view alive. |

Use a view-generation number to reject callbacks and snapshot replies from a discarded
view. Backend job completion is independent of that number and must always be handled.
Multiple open requests during creation share one pending creation; they must not create
a second window. Close during creation cancels presentation without stopping recording.

## Restoring the interface

Keep a small Rust-owned session state: page, Settings section, History interval,
trend interval, keyboard layout choice and scroll position. Update it through typed UI messages;
never keep an entire DOM, rendered HTML, history snapshot or input text in that state.

After `Ready`, restore language/theme/window mode and the session state, request a fresh
snapshot, then publish current download/install progress and pending completion/error
notices. Pending notices remain in the existing Rust-owned controller until a visible full
view can present them. Keyboard
selections, focus traps and popup state should be cleared instead of restored.

Downloads may continue without a view because the update controller already owns them.
Installation remains an explicit user action. Imports and folder moves keep their
existing worker/transaction and shutdown barriers; the conservative first implementation
defers view destruction while they or native dialogs are active.

## Platform handling

- Windows: make the renderer invisible before `TrySuspend`; resume before publishing
  UI state. An in-flight suspension callback needs a generation/visibility check so a
  rapid reopen cannot leave a visible page suspended.
- `TrySuspend` changes the memory target to Low. Also set Low/Normal at the visibility
  transition, so unsupported or rejected suspension still uses the reduced target.
- Wry 0.55.1's Windows `InnerWebView::drop` already calls `controller.Close()`. Release
  all WebView/controller/context clones and observe the associated process family;
  do not assume calling `Close` alone synchronously removes every browser process.
- macOS/Linux: releasing the view may still leave engine helper processes temporarily
  cached by the OS. Measure the application's process group, not only Rust RSS.
- Create and dispose WebViews only on the event-loop thread. Retain platform activation,
  tray reopen, DPI/monitor geometry and Mini sizing rules.

## Acceptance before implementation can ship

1. Compare the same four memory scenarios and ten-minute hidden interval against the
   font-only build; report main process and renderer group separately.
2. A background launch starts no view; daily update checks and capture still work.
3. Reopen full/Mini repeatedly, including rapid hide/reopen during suspension/creation;
   preserve geometry and never show duplicate windows or stale selections.
4. Destroy while idle, then restore all eight languages and the selected page/section.
5. Finish an update download, import, and folder move while hidden; preserve progress,
   notices, counts and history on reopen and save-and-quit.
6. Verify native Windows renderer processes actually leave after disposal; verify Mac
   and Linux independently and record any engine-retained memory.
7. Report cold-reopen latency and peak memory. Keep a rollback path if reopening fails.

## References

- [WebView2 suspend/resume](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_3)
- [WebView2 memory target](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_19)
- [WebView2 controller lifetime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controller)
- Wry 0.55.1 source: `src/webview2/mod.rs`, `InnerWebView::drop` and memory-usage methods.
