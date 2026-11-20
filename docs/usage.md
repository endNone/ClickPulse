# Usage

**English** · [简体中文](zh-CN/usage.md)

## Pages at a glance

| Page | What it shows |
| --- | --- |
| **Overview** | Today / Last 24h, 7-day and all-time totals, rolling windows, 14/30/90-day trends, activity mix, hourly chart and past-year calendar |
| **History** | Daily activity chart, hourly scatter plot with draggable time-range brush, year filter and paginated daily records |
| **Heatmap** | Keyboard heatmap (**Default** or **With numpad**) or **Treemap**, plus cumulative **Mouse** button usage below |
| **Shortcuts** | Cumulative shortcut counts, top-8 ranking, searchable list and a link to the online cross-OS [Shortcut Atlas](https://endnone.github.io/ClickPulse/) |
| **Report** | Daily (24-hour) or weekly (2–31 day) report cards with floating card-deck style switcher, PNG export, clipboard copy and AI prompt recipes |
| **Mini / Sport** | Compact **320 × 208** floating counter window with optional speed rank (C/B/A/S/SS) and Warp animation |
| **Settings** | Theme, font size, language, counter period, time zone, recording access, login startup, updates and data management |

Toggle the sidebar with its top-bar button or **⌘B** (**Ctrl+B** on Windows/Linux). Your sidebar state and Heatmap layout preference are remembered across restarts.

## Heatmap, Shortcuts and Report

- **Heatmap**: Held keys count once until released; mouse counts track button presses (not movement or scrolling). Switch between **Default** and **With numpad**, or switch to **Treemap** to compare block areas and counts. Hovering the Mouse diagram highlights the matching button row and vice versa.
- **Shortcuts**: Tracks modifier combinations recorded on the current OS (excluding Shift-only combos in charts/rankings). On macOS and Windows, letters, digits and supported punctuation follow your active keyboard layout (AZERTY, QWERTZ, Dvorak, etc.); Linux uses physical US positions. Click **Shortcut Atlas** in the card header to browse the interactive cross-OS reference in your browser (or see [shortcuts.md](shortcuts.md)).
- **Report**:
  - **Daily** shows the 24 clock hours of the selected date.
  - **Weekly** covers **2 to 31 days** (the last 7 by default) and shows one bar per date. The busiest hour is a single dated hour. The busiest time slot is the clock hour with the highest daily average across all selected dates, including days with no recorded input.
  - The chart keeps each hour or date in its original position. It labels a peak when the counts vary and leaves zero-count slots empty.
  - Switch card styles via the floating deck, then **Save PNG** or **Copy image** (macOS/Windows).
  - Open **AI prompt** below the export buttons to copy an English brief for an English poster. It asks the AI to choose a few distinctive facts and leave quiet intervals unlabeled. **Regenerate** changes the style and highlights.

## Mini mode and Sport

Enter **Mini mode** from the top-right button. Drag the brand area to move it; pin, hide or expand from the top bar, and pause/resume at the bottom.

Enable **Sport mode** in **Settings → Personalization** to show the lightning button in Mini. Sport grades combined keyboard + mouse speed over a rolling 1-second window (**C** < 2, **B** ≥ 2, **A** ≥ 5, **S** ≥ 8, **SS** ≥ 12 CPS) with the **Warp** animation. Animation stops automatically when hidden, expanded or when system reduced-motion is enabled.

## Counter period, time zone and personalization

- **Counter period** (**Settings → Application**): Choose **Calendar day** (from midnight in the selected time zone) or **Last 24 hours** (rolling 24-hour window) for Overview and Mini.
- **Time zone**: Follows the system zone by default, or pick any IANA zone (type a city name to search). Changing zones reprojects read-time views without rewriting stored timestamps.
- **Appearance**: Choose from 4 themes (**Blue**, **Teal**, **Violet**, **Graphite**), 6 font sizes (80%–150%) and 8 languages.

## Background recording and updates

- **macOS**: Closing the main window or hiding Mini keeps recording from the menu bar and hides the running Dock icon. Choose **Quit ClickPulse** or press **⌘Q** to save and exit.
- **Windows / Linux**: Closing the window can minimize to the tray or save and quit (configurable in **Settings → Close behavior**).
- **Updates**: In **Settings → Application → App updates**, daily checks download newer packages automatically in the background and install only when you click **Update & restart**. Scoop and WinGet installations show their package-manager upgrade command instead. Starting with **0.3.2**, public macOS releases use a fixed self-signed certificate; upgrading from 0.3.1 or earlier requires refreshing Input Monitoring once (see [macOS installation](install-macos.md#4-update-and-refresh-access)).

## Data locations, backups and import

| Platform | Statistics and preferences | Diagnostic log |
| --- | --- | --- |
| macOS | `~/Library/Application Support/ClickPulse/` | `~/Library/Logs/clickpulse.log` |
| Windows | `%LOCALAPPDATA%\ClickPulse\` | `logs\clickpulse.log` inside the data folder |
| Linux | `$XDG_DATA_HOME/ClickPulse/` (`~/.local/share/ClickPulse/`) | `logs/clickpulse.log` inside the data folder |

- **Save folder** (**Settings → Data management → Change folder**): Copies and verifies `events.sqlite3` before switching while keeping the old copy.
- **Import database**: Merges another ClickPulse `events.sqlite3` without doubling repeated imports, automatically saving a backup in `backups/` first (retaining up to 5 recent automatic backups).
- **Recovery & uninstall**: Missing or damaged databases trigger a safe recovery dialog instead of silently overwriting data. On Windows, **Settings → Application → Uninstall and data** lets you explicitly keep or delete local history.

For privacy guarantees, retention rules and literary comparison details, see [Data and privacy](data-and-privacy.md). For CLI read-only viewing (`--source-db`) and explicit migration (`migrate`), see [Development](development.md#paths-and-explicit-migration).
