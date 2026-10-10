#!/usr/bin/env python3
"""Build Apple Silicon or Intel App/ZIP/DMG packages without touching live data."""
import argparse
import datetime
import hashlib
import json
import os
import platform
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile
import tomllib
from third_party import rust_notices
from release import source_revision

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {'aarch64-apple-darwin': ('arm64', 'Apple Silicon'),
           'x86_64-apple-darwin': ('x64', 'Intel')}


def mac_target(requested=None, machine=None):
    if requested:
        if requested not in TARGETS:
            raise ValueError('Choose a supported macOS Rust target.')
        return requested
    machine = machine or platform.machine()
    if machine in {'arm64', 'aarch64'}:
        return 'aarch64-apple-darwin'
    if machine in {'x86_64', 'AMD64'}:
        return 'x86_64-apple-darwin'
    raise ValueError('Unknown Mac architecture; select --target explicitly.')


def run(command, **kwargs):
    subprocess.run(command, cwd=ROOT, check=True, **kwargs)


def checksum(file):
    sha = hashlib.sha256(file.read_bytes()).hexdigest()
    file.with_suffix(file.suffix+'.sha256').write_text(sha+'  '+file.name+'\n')
    return sha


def create_zip(stage, output):
    # Extended attributes would create undeclared AppleDouble files in the ZIP.
    run(['ditto', '-c', '-k', '--norsrc', '--noextattr', '--noqtn', '--noacl',
         '--keepParent', str(stage), str(output)])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--target', choices=list(TARGETS), help='Defaults to the build host architecture')
    args = parser.parse_args()
    if sys.platform != 'darwin':
        parser.error('macOS packaging requires macOS, Xcode Command Line Tools and Python 3.11+.')
    target = mac_target(args.target)
    arch, chip = TARGETS[target]
    for tool in ['cargo', 'ditto', 'codesign', 'hdiutil', 'sips', 'iconutil', 'lipo']:
        if shutil.which(tool) is None:
            parser.error(f'Missing {tool}; see docs/development.md.')
    version = tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    info = plistlib.loads((ROOT/'assets/Info.plist').read_bytes())
    if info['CFBundleShortVersionString'] != version:
        parser.error('Cargo and macOS public versions must match.')
    environment = os.environ.copy()
    environment.setdefault('MACOSX_DEPLOYMENT_TARGET', '12.0')
    if not args.skip_build:
        run(['cargo', 'build', '--release', '--locked', '--target', target], env=environment)
    binary = ROOT/'target'/target/'release/clickpulse'
    if not binary.is_file():
        parser.error(f'Missing release binary: {binary}')
    expected_arch = 'arm64' if arch == 'arm64' else 'x86_64'
    if subprocess.check_output(['lipo', '-archs', str(binary)], text=True).split() != [expected_arch]:
        parser.error('The release binary architecture does not match the selected Mac target.')
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--filter-platform', target], cwd=ROOT))
    dist = ROOT/'dist'; dist.mkdir(exist_ok=True)
    name = f'ClickPulse-{version}-macos-{arch}'
    with tempfile.TemporaryDirectory(prefix='.macos-package-', dir=dist) as temporary:
        temp = Path(temporary); stage = temp/name; stage.mkdir()
        app = stage/'ClickPulse.app'; contents = app/'Contents'
        resources = contents/'Resources'; licenses = resources/'Licenses'
        (contents/'MacOS').mkdir(parents=True); licenses.mkdir(parents=True)
        shutil.copy2(binary, contents/'MacOS/clickpulse')
        shutil.copy2(ROOT/'assets/Info.plist', contents/'Info.plist')
        shutil.copy2(ROOT/'LICENSE', resources/'LICENSE.txt')
        for file in sorted((ROOT/'assets/fonts').glob('*-OFL.txt')):
            shutil.copy2(file, licenses/file.name)
        (resources/'THIRD-PARTY-NOTICES.txt').write_text('ClickPulse third-party notices\n\nFonts: Inter, Noto Sans SC and Noto Sans Mono. Full SIL OFL licenses are in Licenses/.\n\n'+rust_notices(metadata, ROOT), encoding='utf-8')
        shutil.copytree(ROOT/'assets/localizations', resources, dirs_exist_ok=True)
        run([sys.executable, 'scripts/make_icon.py', str(resources/'ClickPulseLogo.icns')])
        run(['codesign', '--force', '--sign', '-', '--identifier', 'app.clickpulse.desktop', str(app)])
        run(['codesign', '--verify', '--deep', '--strict', str(app)])
        (stage/'Install Guide.txt').write_text(f'''ClickPulse {version} / macOS 12+ / {chip} ({arch})

Code license: GNU GPL version 3 or later (GPL-3.0-or-later).
Full license: Contents/Resources/LICENSE.txt inside the App. Third-party assets retain their licenses.

Drag the complete ClickPulse.app to Applications, or your own ~/Applications folder.
Open the installed App; if blocked, add the per-App Open Anyway exception in macOS settings.
Enable ClickPulse under Privacy & Security > Input Monitoring, then Quit & Reopen.
Confirm Recording and increasing counts.

Quit the old App before replacing it in the same location for an update.
This is an ad-hoc preview without Apple notarization; updated builds may need a new permission grant.
Do not delete statistics to fix permissions.
History and preferences remain in ~/Library/Application Support/ClickPulse when the App is replaced or removed.

Full installation guide: docs/install-macos.md in the source repository.
''', encoding='utf-8')
        files = {str(file.relative_to(stage)): hashlib.sha256(file.read_bytes()).hexdigest()
                 for file in sorted(stage.rglob('*')) if file.is_file()}
        manifest = {'version': version, 'internal_build': int(info['CFBundleVersion']), 'target': target,
                    'source_revision': source_revision(ROOT),
                    'signing': 'ad-hoc', 'notarized': False, 'code_license': 'GPL-3.0-or-later', 'files': files}
        (stage/'build-manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
        create_zip(stage, temp/(name+'.zip'))
        (stage/'Applications').symlink_to('/Applications', target_is_directory=True)
        run(['hdiutil', 'create', '-quiet', '-volname', 'ClickPulse', '-srcfolder', str(stage),
             '-format', 'UDZO', str(temp/(name+'.dmg'))])
        # Retain any earlier generated App rather than modifying an installed copy.
        destination = dist/('ClickPulse.app' if arch == 'arm64' else 'ClickPulse-macos-x64.app')
        if destination.exists():
            retired = dist/'retired'; retired.mkdir(exist_ok=True)
            stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
            os.replace(destination, retired/f'ClickPulse-before-package-{stamp}.app')
        os.replace(app, destination)
        for suffix in ['.zip', '.dmg']:
            output = dist/(name+suffix); os.replace(temp/(name+suffix), output)
            print(f'{output.name}: {checksum(output)}')
        (dist/(name+'.manifest.json')).write_text(json.dumps(manifest, indent=2)+'\n')
    print('Packages created. No installed App, permissions, login entries or user statistics were changed.')


if __name__ == '__main__':
    main()
