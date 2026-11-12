#!/usr/bin/env python3
"""Package the native Linux build with an explicit, independently verified payload."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import tempfile
import zipfile
from third_party import rust_notices


def validate_elf(data, target):
    expected = {'aarch64-unknown-linux-gnu': 183, 'x86_64-unknown-linux-gnu': 62}[target]
    if len(data) < 64 or data[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', data, 18)[0] != expected or struct.unpack_from('<H', data, 16)[0] not in (2, 3):
        raise ValueError('Not a matching Linux 64-bit executable')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', choices=['aarch64-unknown-linux-gnu', 'x86_64-unknown-linux-gnu'])
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--source-revision', help='Exact 40-character source revision for a shared checkout')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    os.chdir(root)
    target = args.target or next(line.split(': ', 1)[1] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    if target not in {'aarch64-unknown-linux-gnu','x86_64-unknown-linux-gnu'}:
        parser.error('Build with a Linux ARM64 or x64 toolchain')
    if not args.skip_build:
        subprocess.run(['cargo','build','--release','--locked','--target',target],check=True)
    metadata = json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',target]))
    package = next(p for p in metadata['packages'] if p['id'] == metadata['resolve']['root'])
    version = package['version']
    internal_build = int(re.search(r'FILEVERSION\s+\d+,\d+,\d+,(\d+)',(root/'assets/windows/app.rc').read_text()).group(1))
    revision = args.source_revision or subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    if not re.fullmatch(r'[0-9a-f]{40}',revision):
        parser.error('A full source revision is required')
    executable = Path(metadata['target_directory'])/target/'release/clickpulse'
    validate_elf(executable.read_bytes(),target)
    arch = 'arm64' if target.startswith('aarch64') else 'x64'
    name = f'ClickPulse-{version}-linux-{arch}'
    dist = root/'dist'; dist.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.linux-package-',dir=dist) as temporary:
        bundle = Path(temporary)/name; bundle.mkdir()
        shutil.copy2(executable,bundle/'clickpulse'); (bundle/'clickpulse').chmod(0o755)
        shutil.copy2(root/'LICENSE',bundle/'LICENSE.txt')
        shutil.copy2(root/'assets/clickpulse-mark-blue-transparent-v1-1024.png',bundle/'clickpulse.png')
        licenses = bundle/'Licenses'; licenses.mkdir()
        for license_file in (root/'assets/fonts').glob('*-OFL.txt'):
            shutil.copy2(license_file,licenses/license_file.name)
        (bundle/'THIRD-PARTY-NOTICES.txt').write_text(rust_notices(metadata,root),encoding='utf-8')
        (bundle/'README.txt').write_text((root/'docs/install-linux.md').read_text(),encoding='utf-8')
        (bundle/'clickpulse.desktop').write_text('[Desktop Entry]\nType=Application\nName=ClickPulse\nComment=Local keyboard and mouse activity statistics\nExec=clickpulse\nIcon=clickpulse\nTerminal=false\nCategories=Utility;\n',encoding='utf-8')
        files = {f.relative_to(bundle).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(bundle.rglob('*')) if f.is_file()}
        manifest = {'version':version,'internal_build':internal_build,'target':target,'source_revision':revision,'code_license':'GPL-3.0-or-later','files':files,'native_linux_tested':False}
        (bundle/'build-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        archive = Path(temporary)/(name+'.zip')
        with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as output:
            for f in sorted(bundle.rglob('*')):
                if f.is_file(): output.write(f,f.relative_to(Path(temporary)))
        os.replace(archive,dist/(name+'.zip'))
        shutil.copy2(bundle/'build-manifest.json',dist/(name+'.manifest.json'))
    archive = dist/(name+'.zip')
    archive.with_suffix('.zip.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n')
    print(archive)

if __name__ == '__main__':
    main()
