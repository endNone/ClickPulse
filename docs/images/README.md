# Screenshot assets

README.md uses English-interface screenshots, a Mini desktop demonstration and a Sport rank animation. Public project images must use English, regardless of the surrounding documentation language. Every showcase uses the default deep blue theme; other color themes remain an App feature described in text.

| Mini mode asset | Format |
| --- | --- |
| [Desktop demonstration](mini-desktop-en.webp) | Animated WebP used in the README |
| [GIF alternative](mini-desktop-en.gif) | The same desktop demonstration |
| [Static preview](mini-blue-en.jpg) | High-density view of the floating panel |

These are browser captures of the App's own HTML/CSS/JavaScript and unmodified bundled fonts, with generated data. They show interface content, without native OS window controls. Full pages render at twice the pixel density with a 1280-pixel logical width. The overview keeps the complete interface at 2560 pixels wide. Keyboard, treemap and history examples capture only their visible feature cards, excluding the sidebar, top toolbar, page heading and footer without changing the UI layout. The three feature images are displayed as separate full-width rows in the README, without comparison tables or table-heading rows. Mini renders at three times the pixel density: a 960 × 624 image of the unchanged 320 × 208 logical layout, available above. Text and vector icons are rendered at the target density, rather than enlarging a compressed image. They are not native Windows validation evidence.

The README's five SVG badges are local assets under `assets/badges/`: preview version (orange), GPL-3.0-or-later license (green), macOS target (violet), Windows target (blue) and Rust core (orange, with its artwork notice in `assets/badges/README.md`). The license badge uses compact text padding. Storage and network information remain in the privacy text instead of separate badges. Keep their facts consistent with installation status. The license badge links to the project LICENSE file and reflects the maintainer-approved GPL-3.0-or-later grant. Stars and a stable-release label require a published repository and release first.

## Reproduce the demo

From the project root:

```bash
node scripts/prepare-readme-preview.cjs
python3 -m http.server 8768 --bind 127.0.0.1 --directory dist/readme-preview
```

Open `http://127.0.0.1:8768/?locale=en` in a browser. The preview defaults to English. Query options:

| Option | Values |
| --- | --- |
| `locale` | `en` for all public screenshots; `zh-CN` only for local translation checks |
| `page` | `overview`, `history`, `keys`, `settings`, `mini` |
| `view` | `treemap` on the keys page; otherwise the physical layout |
| `scale` | `2` or `3` to render at a higher pixel density; omitted for ordinary browsing |

Capture after the `data-preview-ready="true"` attribute appears on `<html>` and fonts finish loading. For full pages use `?locale=en&page=overview&scale=2` (or another page), a 2560 × 1200 viewport, and a full-page capture. For mini use `?locale=en&page=mini&scale=3`, a 960 × 624 viewport, and a viewport capture. Without `scale`, mini requires a 320 × 208 viewport. Reset temporary viewport overrides afterwards. Save browser JPEG output directly with a `.jpg` suffix; do not regenerate UI elements or substitute fonts with an image tool.

For keyboard, treemap and history, find the bounding rectangle of the visible `.card` elements in the selected page. Use their combined bounds with 16 pixels of outer padding on each side for the browser screenshot's clip option. At the dimensions above, these examples start at x=464, y=318 and are 2052 pixels wide. This captures the actual feature content without surrounding navigation or duplicate page titles; do not hide controls in the App or stretch the cropped image. Keep overview and mini uncropped.

The Settings screen is not included in the public showcase.

## Mini mode desktop animation

`mini-desktop-en.webp` is a 19-second, 1350 × 850 looping demonstration, with a GIF alternative. A moderate 1.25× capture density renders the original 1080 × 680 desktop canvas with clearer text. The mini panel is displayed at 281.6 × 183.04 logical pixels (0.88× its unchanged 320 × 208 layout), reducing its previous 352 × 229 presentation by 1.25× as requested. WebP preserves full photographic colors at quality 90; it avoids the GIF palette while keeping loading practical. The desktop and notes document are fictional, with no decorative dock or imitation App icons; the floating panel embeds the App's unchanged mini HTML/CSS, fonts, icons and normal statistics renderer. The work session includes moving the floating panel, reviewing a checklist, focusing the editor, typing with pauses, correcting a typo with Backspace, saving with Cmd+S, and marking the draft complete. The pointer follows continuous paths, becomes a text cursor inside the editor, and hides during typing. This is a documentation example, not native OS capture validation.

The input event timeline is shared by the page and capture manifest. The four mouse presses include the panel drag and editor focus. All 46 physical key presses are counted, including Shift for the initial capital, four Backspace presses, the replacement letters and Cmd+S. Counts rise from 18,780 (14,862 keys / 3,918 clicks) to 18,830 (14,908 keys / 3,922 clicks); correcting the text never reduces the key count.

After preparing the preview and starting the server above, open `http://127.0.0.1:8768/mini-desktop.html?scale=1.25`. Wait for `data-preview-ready="true"`. At a 1350 × 850 viewport, save the initial viewport capture as `frame-000.jpg`. Press Arrow Right on the page and capture each subsequent frame through `frame-236.jpg`; Home resets the timeline. The frame output and `data-frame` attribute identify the current frame. The controls beneath the fixed desktop canvas are excluded from these viewport captures. Check that the saved image matches the requested dimensions and contains the complete panel; do not accept a narrower capture even if the DOM reports the requested viewport. Pointer anchors are converted from physical capture coordinates back to logical canvas coordinates. Reset any temporary browser viewport afterwards.

Encode the original frames with Pillow:

```bash
cp dist/readme-preview/mini-demo-manifest.json /path/to/frames/
python3 scripts/encode-mini-demo.py /path/to/frames docs/images/mini-desktop-en.webp
python3 scripts/encode-mini-demo.py /path/to/frames docs/images/mini-desktop-en.gif
```

The WebP encoder keeps full color at quality 90. The GIF alternative uses a shared 256-color palette. Both preserve the original captured dimensions. The manifest specifies 237 input frames at 80 ms each (18.96 seconds) and includes every simulated input event. The encoder checks the complete input sequence, total duration and infinite loop before atomically replacing either output. No live desktop, database, input listener or saved preferences are used.

The fixed random seed generates 191 calendar days, ending on 2026-10-08. Daily, weekly and lifetime counts are consistent; physical key counts sum to the keyboard total. The dataset exists only under the ignored `dist/readme-preview` folder. No database is read, no input listener runs and no live preferences are changed.

## Sport: C to SS on the desktop

The README embeds [sport-desktop-en.webp](sport-desktop-en.webp), a 3.6-second **C → B → A → S → SS** loop at 2.5× playback speed, with a [static SS preview](sport-desktop-ss-en.jpg). It uses the same mountain-lake scenery and fictional notes window as the Mini demonstration above. The Mini iframe renders the production HTML/CSS/fonts, speed meter and Warp effect in English/default blue. SS adds the red accent and particles without shake. No live screen, database, listener or saved preferences are used.

Capture at **2160 × 1360** pixels: a 1080 × 680 scene at 2× density. The original 320 × 208 Mini layout occupies 384 × 249.6 logical pixels in this scene, or about 768 × 499 captured pixels. Render the iframe at 3× density and reduce it within the scene so text and canvas remain crisp. This replaces the standalone 640 × 416 Sport showcase; the scenery source itself remains the original 1672 × 941 illustration, not a new high-resolution photograph.

`sport-demo-driver.js` advances the actual meter and canvas through 540 deterministic frames at 60 fps. The source simulation lasts 9 seconds. For the README, select 216 frames at 2.5× speed while keeping 60 fps output: output frame `i` uses source frame `round(i × 2.5)`. WebP delays alternate between 16 and 17 ms, producing a 3.6-second loop. This accelerates only the documentation playback; the App’s Sport timing is unchanged. Rates rise through 1, 3, 6, 10 and 14 combined presses/second. Every fourth input is a click; the other 54 inputs type the visible draft. Counts rise from 18,780 (14,862 keys / 3,918 clicks) to 18,851 (14,916 / 3,935). The loop restarts this fictional session after SS; real application totals never reset this way. Sport remains off by default in the App; the preview opts in explicitly.

To reproduce:

```bash
node scripts/prepare-sport-preview.cjs
python3 -m http.server 8768 --bind 127.0.0.1 --directory dist/sport-preview
```

Open `http://127.0.0.1:8768/sport-desktop.html?scale=2` at a 2160 × 1360 viewport. Wait for `data-preview-ready="true"` and `data-frame="0"` on the outer `<html>`. Save the initial **full-page browser JPEG** as `frame-000.jpg`; press Arrow Right and wait for each outer `data-frame` before capturing through `frame-539.jpg`. Home resets the scene. The clock stays frozen between steps. Full-page capture is required because some browser viewport captures clip the right edge despite reporting the correct viewport. Verify every saved image is exactly 2160 × 1360 and contains the complete Mini panel. The capture driver controls are excluded from the scene. Reset the temporary viewport afterwards.

Copy `sport-desktop-manifest.json` from the preview to the frames folder, then prepare the 2.5× sequence without resizing:

```bash
python3 - /path/to/frames /path/to/frames-2.5x <<'PYTHON'
import json, shutil, sys
from pathlib import Path
source, output = map(Path, sys.argv[1:])
manifest = json.loads((source / "sport-desktop-manifest.json").read_text())
assert manifest["frame_count"] == 540 and manifest["duration_ms"] == 9000
output.mkdir(parents=True, exist_ok=True)
count = 216
for i in range(count):
    shutil.copyfile(source / f"frame-{round(i * 2.5):03}.jpg", output / f"frame-{i:03}.jpg")
manifest.update(frame_count=count, duration_ms=3600, fps=60,
    frame_durations_ms=[round((i + 1) * 1000 / 60) - round(i * 1000 / 60) for i in range(count)])
(output / "sport-desktop-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
PYTHON
python3 scripts/encode-mini-demo.py /path/to/frames-2.5x docs/images/sport-desktop-en.webp --manifest sport-desktop-manifest.json
cp /path/to/frames/frame-539.jpg docs/images/sport-desktop-ss-en.jpg
```

The encoder checks dimensions, total 3,600 ms duration and infinite looping, including when identical stationary frames coalesce. The output manifest stores all 216 frame delays explicitly. Keep the 16–17 ms delays instead of dividing them by 2.5, which can trigger minimum-delay clamping in image viewers. With `img2webp` available, encoding consumes the captures sequentially and minimizes repeated static scenery; Pillow remains the fallback. The high-frame-rate Sport animation uses WebP because GIF cannot represent its 16–17 ms delays reliably. Legacy 50/80 ms WebP and GIF manifests remain supported. This synthetic browser example is separate from native GUI or physical-input validation. The fixed-rank preview remains at `app.html?page=mini&locale=en&rank=SS`.

## Scenery provenance

`mini-desktop-scenery.png` is an original illustrative wallpaper generated with the built-in imagegen tool. The returned asset is 1672 × 941 pixels; no 4K output is claimed. The application UI is rendered separately from its production HTML/CSS and fonts. This is not a photograph of an identified location or a native desktop recording. The wallpaper is copied unchanged into the isolated preview.

Final generation prompt:

> Use case: photorealistic-natural. Asset type: a desktop wallpaper behind a compact productivity app demonstration. Primary request: a clear, realistic alpine lake landscape photograph. Snow-dusted mountain ridge in the distance, calm deep-blue lake, evergreen forest along the shoreline, soft natural morning light. Wide 4K landscape composition, 3840x2160, crisp natural textures and restrained colors. Photograph-like, quiet and believable, not fantasy or oversaturated. No interface, no icons, no text, no logos, no watermark. This is scenery only; all application UI will be rendered separately.
