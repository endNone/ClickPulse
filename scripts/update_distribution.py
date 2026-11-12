"""Update first-party package manifests from a locally verified release directory.

Run only after the corresponding stable release is publicly downloadable. This
command changes local manifests; it never publishes, commits or disables security.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent


def update(root, packages, version):
    if not re.fullmatch(r'(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)', version):
        raise ValueError('Use a stable major.minor.patch version.')
    sums = {}
    for line in (packages / 'SHA256SUMS.txt').read_text(encoding='utf-8').splitlines():
        match = re.fullmatch(r'([a-f0-9]{64})  ([A-Za-z0-9._-]+)', line)
        if not match or match[2] in sums:
            raise ValueError('Invalid or duplicate checksum entry.')
        sums[match[2]] = match[1]
    hashes = {}
    for platform, extension in [('macos-arm64', 'dmg'), ('macos-x64', 'dmg'), ('windows-x64', 'zip')]:
        name = f'ClickPulse-{version}-{platform}.{extension}'
        digest = hashlib.sha256((packages / name).read_bytes()).hexdigest()
        if sums.get(name) != digest:
            raise ValueError('Checksum mismatch: ' + name)
        hashes[platform] = digest

    # Complete validation before writing either manifest.
    cask_path = root / 'Casks/clickpulse.rb'
    cask = cask_path.read_text(encoding='utf-8')
    for pattern, replacement in [
        (r'(?m)^  version "[^"]+"$', f'  version "{version}"'),
        (r'(?m)^(  sha256 arm:\s+)"[a-f0-9]+"', r'\g<1>"' + hashes['macos-arm64'] + '"'),
        (r'(?m)^(         intel:\s+)"[a-f0-9]+"', r'\g<1>"' + hashes['macos-x64'] + '"'),
    ]:
        cask, count = re.subn(pattern, replacement, cask)
        if count != 1:
            raise ValueError('Unexpected Cask structure; review manually.')
    scoop_path = root / 'bucket/clickpulse.json'
    scoop = json.loads(scoop_path.read_text(encoding='utf-8'))
    scoop['version'] = version
    scoop['architecture']['64bit'] = {
        'url': f'https://github.com/endNone/ClickPulse/releases/download/v{version}/ClickPulse-{version}-windows-x64.zip',
        'hash': hashes['windows-x64'],
        'extract_dir': f'ClickPulse-{version}-windows-x64',
    }
    cask_path.write_text(cask, encoding='utf-8')
    scoop_path.write_text(json.dumps(scoop, indent=4) + '\n', encoding='utf-8')
    return hashes


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--version', required=True)
    parser.add_argument('--packages', required=True, type=Path)
    args = parser.parse_args()
    hashes = update(ROOT, args.packages, args.version)
    print(f'Updated Cask and Scoop manifests for {args.version}; verified {len(hashes)} package checksums.')
