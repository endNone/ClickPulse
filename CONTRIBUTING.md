# Contributing

ClickPulse is an early preview. Start with [Development](docs/development.md) and the [Roadmap](docs/roadmap.md). Code contributions use [GPL-3.0-or-later](LICENSE); third-party resources retain their licenses.

## Reports

Use the issue templates to include app and OS versions, language, window mode, reproduction steps, expected behavior and actual results. Use demo or redacted screenshots without real databases, raw logs or typed content.

For capture problems, include the access status and whether the app was reopened. For Windows startup problems, include whether WebView2 Runtime and the adjacent Loader DLL are present. Report security-sensitive issues through the private process described in [Security](SECURITY.md).

## Changes

- Keep each change focused on one concrete behavior.
- Keep Chinese and English message keys aligned, including platform-specific text.
- Verify with temporary data; never delete real history to work around a failure.
- Preserve historical totals without adding overlapping summaries twice.
- Keep background saving independent of visible-window and mini-mode state.
- Do not add typed-text or pointer-position capture, or default online services.
- Preserve font and other third-party notices.

Run the checks in the development guide and verify native behavior appropriate to the change. Cross-compilation is not a native test. Describe the concrete problem, final behavior, validation and remaining limits in the pull request.
