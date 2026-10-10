"""Encode browser-captured mini desktop frames as looping WebP or GIF.

Requires Pillow. Captures must match mini-demo-manifest.json dimensions,
with that manifest alongside them, without resizing.
"""
import argparse
import json
import os
from pathlib import Path
import tempfile

from PIL import Image


def encode(directory, output):
    manifest = json.loads((directory / "mini-demo-manifest.json").read_text(encoding='utf-8'))
    frame_count, frame_ms = manifest["frame_count"], manifest["frame_ms"]
    dimensions = manifest["width"], manifest["height"]
    if not 1 <= frame_count <= 1000 or frame_ms < 20 or frame_ms % 10:
        raise ValueError("Invalid animation timing")
    if dimensions not in {(1080, 680), (1350, 850)}:
        raise ValueError("Invalid desktop capture dimensions")
    if output.suffix.lower() not in {".gif", ".webp"}:
        raise ValueError("Output must be GIF or WebP")
    paths = [directory / f"frame-{index:03}.jpg" for index in range(frame_count)]
    frames = []
    for path in paths:
        with Image.open(path) as image:
            if image.size != dimensions:
                raise ValueError(f"Unexpected capture dimensions: {path}: {image.size}")
            frames.append(image.convert("RGB"))
    if output.suffix.lower() == ".webp":
        # Keep full photographic colors at a moderate size, without a GIF palette.
        encoded_frames = frames
        options = {"lossless": False, "quality": 90, "method": 4}
    else:
        encoded_frames = gif_palette(frames, dimensions)
        options = {"optimize": True, "disposal": 1,
                   "comment": b"ClickPulse mini mode. Simulated desktop and synthetic demo data."}
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=output.parent, suffix=output.suffix, delete=False) as staging:
        temporary = Path(staging.name)
    try:
        encoded_frames[0].save(
            temporary, save_all=True, append_images=encoded_frames[1:],
            duration=frame_ms, loop=0, **options,
        )
        with Image.open(temporary) as result:
            duration = 0
            for index in range(result.n_frames):
                result.seek(index)
                result.load()
                duration += result.info["duration"]
            if result.size != dimensions or duration != frame_count * frame_ms or duration != manifest["duration_ms"] or result.info.get("loop") != 0:
                raise ValueError("Unexpected animation dimensions, duration or loop metadata")
            print(f"{output}: {result.size}, {result.n_frames} encoded frames, {duration} ms, {temporary.stat().st_size:,} bytes")
        os.replace(temporary, output)
    finally:
        temporary.unlink(missing_ok=True)


def gif_palette(frames, dimensions):
    # One palette for the whole sequence keeps stationary text/background stable.
    width, height = dimensions
    samples = Image.new("RGB", (width, height * 3))
    frame_count = len(frames)
    for row, index in enumerate((0, frame_count // 2, frame_count - 1)):
        samples.paste(frames[index], (0, row * height))
    palette = samples.quantize(colors=240, method=Image.Quantize.MEDIANCUT)
    # Preserve tiny window/control accents that occupy too few pixels to win
    # a palette slot against the large background and mini panel surfaces.
    accents = ("ff665b", "ffbf42", "43c67b", "2563eb", "285cab", "ffffff", "102543", "dfe7f2", "263448")
    colors = palette.getpalette()[:240 * 3]
    colors.extend(channel for color in accents for channel in bytes.fromhex(color))
    colors.extend([0] * (768 - len(colors)))
    palette.putpalette(colors)
    return [frame.quantize(palette=palette, dither=Image.Dither.NONE) for frame in frames]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("frames", type=Path)
    parser.add_argument("output", type=Path, nargs="?", default=Path("docs/images/mini-desktop-en.webp"))
    args = parser.parse_args()
    encode(args.frames, args.output)


if __name__ == "__main__":
    main()
