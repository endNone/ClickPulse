<p align="center"><img src="assets/clickpulse-blue-cursor-v3.png" width="88" alt="ClickPulse icon"></p>

<h1 align="center">ClickPulse</h1>

<p align="center"><strong>Small app. Every click. Big insights.</strong></p>

<p align="center"><a href="#installation"><img src="assets/badges/preview.svg" alt="preview: v0.1.0"></a> <a href="docs/development.md"><img src="assets/badges/rust.svg" alt="Core: Rust"></a> <a href="LICENSE"><img src="assets/badges/license.svg" alt="license: GPL-3.0-or-later"></a> <a href="docs/install-macos.md"><img src="assets/badges/macos.svg" alt="macOS: 12+ · arm64/x64"></a> <a href="docs/install-windows.md"><img src="assets/badges/windows.svg" alt="Windows: 10/11 · x64"></a></p>

ClickPulse is an offline keyboard and mouse activity tracker for macOS and Windows. Explore daily trends, see your most-used keys, and keep today’s counts in a compact floating window.

[![ClickPulse overview](docs/images/overview-en.jpg)](docs/images/overview-en.jpg)

## Mini mode

[![Mini mode alongside everyday desktop work](docs/images/mini-desktop-en.webp)](docs/images/mini-desktop-en.webp)

Simulated desktop work with live demo counts.

## Installation

**0.1.0 Preview** — [Download packages](https://github.com/endNone/ClickPulse/releases/tag/v0.1.0). You can also [build from source](docs/development.md).

| System | Package | Guide |
| --- | --- | --- |
| macOS 12+ · Apple Silicon | **arm64** DMG or App ZIP | [macOS installation](docs/install-macos.md) |
| macOS 12+ · Intel | **x64** DMG or App ZIP | [macOS installation](docs/install-macos.md) |
| Windows 10/11 · x64 | Portable ZIP | [Windows installation](docs/install-windows.md) |

On macOS, move the complete **ClickPulse.app** to Applications, allow the [first launch](docs/install-macos.md#2-allow-the-first-launch), and enable **Input Monitoring**. On Windows, extract the complete portable ZIP and open **ClickPulse.exe**. Ready-built packages need no developer tools.

Confirm **Recording**, then press a few keys and click in another app to check that counts increase. Mac previews are ad-hoc signed without notarization; Windows previews are unsigned.

## Features

The download links above still point to public preview build 55. Sport mode, the rolling 24-hour chart, 30-day trends, manual time-zone selection, expanded language options and data management below are in development source and are not included in that published preview. Current development source also removes CSV export; the published preview retains its original features.

- **Activity overview:** 14/30/90-day trends, an hourly breakdown of the last 24 hours, seven-day and all-time totals, and an activity calendar.
- **Keyboard views:** physical-key heatmap and a treemap sized by usage.
- **History:** year filtering and complete daily records.
- **Mini mode:** floating counts with pin, pause and expand controls, plus optional Sport feedback. Launch at login is available in Settings.
- **Data management:** choose a save folder and merge compatible SQLite history with automatic backups.

Four themes and eight interface languages are available in Settings: English, Simplified Chinese, Traditional Chinese, Japanese, Korean, French, German and Spanish. The language follows the system by default. Statistics also follow the system time zone by default, with a saved override in **Settings → Application → Time zone**. Closing a window can keep recording; use **Quit ClickPulse** from the menu bar or tray to stop. See [Usage](docs/usage.md).

## Sport mode

Turn a burst of typing and clicking into a light runway. Inside Mini, Sport's **Warp** effect responds to your combined keyboard and mouse speed with light gates, particles and a live rank: **C → B → A → S → SS**. Red **SS** keeps the particles without shaking the window. Your activity totals stay visible.

Sport and its Mini switch are **off by default**. To try it:

1. Open **Settings → Personalization** and enable **Sport mode** to show the Mini switch.
2. Enter **Mini mode** using the top-right button, then click the lightning button to enable Sport.
3. Type or click in any app. Use the lightning button to turn Sport off; hiding its switch in Settings also disables it.

Ranks use combined presses per second (CPS): **C < 2 · B ≥ 2 · A ≥ 5 · S ≥ 8 · SS ≥ 12**. Holding a key counts once until released; moving or scrolling the mouse does not increase the rank. Animation stops when you leave Mini, and follows the system's reduced-motion preference. See [Sport in Mini mode](docs/usage.md#sport-in-mini-mode) for details.

## Screenshots

Demo data, English interface, default deep blue theme. Click an image to view the original.

### Keyboard heatmap

[![Keyboard heatmap](docs/images/keyboard-en.jpg)](docs/images/keyboard-en.jpg)

### Usage treemap

[![Usage treemap](docs/images/treemap-en.jpg)](docs/images/treemap-en.jpg)

### History

[![History](docs/images/history-en.jpg)](docs/images/history-en.jpg)

## Privacy

Statistics stay on your device, with no account, telemetry or cloud sync. ClickPulse records key/button identities, counts and timestamps; it does not record typed text, pointer positions or screen contents.

History is kept permanently. Updating or uninstalling the program retains your data. Use a complete SQLite backup to preserve history. See [Data and privacy](docs/data-and-privacy.md).

## Documentation

Read the [documentation](docs/README.md) for installation, everyday use, backups and development.

## Contributing

Bug reports and focused pull requests are welcome. See the [contribution guide](CONTRIBUTING.md).

## License

Licensed under [GPL-3.0-or-later](LICENSE).

Bundled fonts and artwork retain their own licenses. See the [font notices](assets/fonts/README.txt) and [artwork notices](assets/badges/README.md).
