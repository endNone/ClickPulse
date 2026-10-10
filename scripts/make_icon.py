"""Package the supplied app artwork into all standard macOS icon resolutions."""
import argparse
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument("output", type=Path)
parser.add_argument("--source", type=Path, default=Path(__file__).resolve().parent.parent / "assets/clickpulse-blue-cursor-v3.png")
args = parser.parse_args()
source = args.source.resolve(strict=True)
output = args.output.resolve()
output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix="clickpulse-icon-") as temp:
    iconset = Path(temp) / "ClickPulse.iconset"
    iconset.mkdir()
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            pixels = str(size * scale)
            suffix = "@2x" if scale == 2 else ""
            target = iconset / f"icon_{size}x{size}{suffix}.png"
            subprocess.run(["sips", "-z", pixels, pixels, str(source), "--out", str(target)], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(["iconutil", "-c", "icns", "--output", str(output), str(iconset)], check=True)
print(f"Application icon: {output}")
