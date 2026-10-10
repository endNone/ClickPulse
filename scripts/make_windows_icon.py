"""Generate a multi-resolution ICO from the supplied artwork on macOS."""
from pathlib import Path
import struct
import subprocess
import tempfile

root = Path(__file__).resolve().parent.parent
sizes = [16, 24, 32, 48, 64, 128, 256]
images = []
with tempfile.TemporaryDirectory(prefix="clickpulse-ico-") as folder:
    for size in sizes:
        output = Path(folder) / f"{size}.png"
        subprocess.run(["sips", "-z", str(size), str(size), str(root / "assets/clickpulse-blue-cursor-v3.png"), "--out", str(output)], check=True, stdout=subprocess.DEVNULL)
        images.append(output.read_bytes())
offset = 6 + 16 * len(images)
directory = []
for size, image in zip(sizes, images):
    directory.append(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(image), offset))
    offset += len(image)
output = root / "assets/windows/ClickPulse.ico"
output.write_bytes(struct.pack("<HHH", 0, 1, len(images)) + b"".join(directory) + b"".join(images))
print(output)
