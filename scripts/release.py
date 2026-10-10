"""Validate versions and prepare a complete release candidate without publishing."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import re
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
VERSION = r'(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)'
REVISION = r'[0-9a-f]{40}'
LICENSE = 'GPL-3.0-or-later'


def source_revision(root):
    if not (root/'.git').exists():
        return None
    revision = subprocess.check_output(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root, text=True).strip()
    if not re.fullmatch(REVISION, revision):
        raise ValueError('Invalid source revision.')
    return revision


def release_plan(root, tag=None, require_tag=False):
    package = tomllib.loads((root/'Cargo.toml').read_text(encoding='utf-8'))['package']
    version = package['version']
    if not re.fullmatch(VERSION, version):
        raise ValueError('Release packages require a numeric major.minor.patch version.')
    if package.get('license') != LICENSE:
        raise ValueError('The release license must match GPL-3.0-or-later.')
    tag = tag or 'v'+version
    if tag != 'v'+version:
        raise ValueError('The version tag must exactly match Cargo.toml (v'+version+').')
    lock = tomllib.loads((root/'Cargo.lock').read_text(encoding='utf-8'))
    locked = [p['version'] for p in lock['package'] if p['name'] == package['name'] and 'source' not in p]
    info = plistlib.loads((root/'assets/Info.plist').read_bytes())
    resource = re.search(r'FILEVERSION\s+(\d+),(\d+),(\d+),(\d+)', (root/'assets/windows/app.rc').read_text(encoding='utf-8'))
    if locked != [version] or info['CFBundleShortVersionString'] != version or resource is None or '.'.join(resource.groups()[:3]) != version:
        raise ValueError('Cargo.lock, macOS and Windows public versions must match Cargo.toml.')
    build = int(info['CFBundleVersion'])
    if int(resource.group(4)) != build:
        raise ValueError('macOS and Windows internal build numbers must match.')
    result = {'version': version, 'tag': tag, 'internal_build': build}
    if require_tag:
        head = subprocess.check_output(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root, text=True).strip()
        tagged = subprocess.check_output(['git', 'rev-parse', '--verify', 'refs/tags/'+tag+'^{commit}'], cwd=root, text=True).strip()
        if not re.fullmatch(REVISION, head) or head != tagged:
            raise ValueError('The checked-out revision must be the existing version tag.')
        result['revision'] = head
    return result


def digest_stream(stream):
    sha = hashlib.sha256()
    for block in iter(lambda: stream.read(1024*1024), b''):
        sha.update(block)
    return sha.hexdigest()


def digest(path):
    with path.open('rb') as stream:
        return digest_stream(stream)


def safe_archive_path(name):
    path = PurePosixPath(name)
    if not name or path.is_absolute() or '\\' in name or ':' in name or '..' in path.parts:
        raise ValueError('Unsafe archive path: '+name)
    private = {'.git', '.Codex', '.codex', 'trace.md', 'target', 'dist', '__pycache__', 'retired', '.DS_Store', '__MACOSX'}
    if any(part in private or part.startswith(('.env', '._')) for part in path.parts):
        raise ValueError('Private or generated archive path: '+name)
    if path.suffix.lower() in {'.sqlite3', '.sqlite', '.db', '.log', '.pyc'} or path.name.endswith(('-wal', '-shm')):
        raise ValueError('User data cannot become a release asset: '+name)
    if re.search(r'\.log\.\d+$', path.name, re.IGNORECASE):
        raise ValueError('Rotated logs cannot become a release asset: '+name)
    return path


def verify_archive(path, prefix, manifest_name, plan, license_path, license_bytes):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise ValueError('Duplicate archive entries: '+path.name)
        for entry in archive.infolist():
            relative = safe_archive_path(entry.filename)
            if relative.parts[0] != prefix or (entry.external_attr >> 16) & 0o170000 == 0o120000:
                raise ValueError('Unexpected archive root or symlink: '+entry.filename)
        manifest = json.loads(archive.read(prefix+'/'+manifest_name))
        if manifest.get('version') != plan['version'] or manifest.get('code_license') != LICENSE:
            raise ValueError('Package version or license mismatch: '+path.name)
        files = manifest.get('files')
        if not isinstance(files, dict) or not files:
            raise ValueError('Empty package manifest: '+path.name)
        payload = {name[len(prefix)+1:] for name in names if not name.endswith('/')}
        if payload != set(files) | {manifest_name}:
            raise ValueError('Package manifest does not cover every file: '+path.name)
        for name, expected in files.items():
            safe_archive_path(name)
            if not isinstance(expected, str) or not re.fullmatch(r'[0-9a-f]{64}', expected):
                raise ValueError('Invalid manifest checksum: '+name)
            with archive.open(prefix+'/'+name) as stream:
                if digest_stream(stream) != expected:
                    raise ValueError('Package file checksum mismatch: '+name)
        if archive.read(prefix+'/'+license_path) != license_bytes:
            raise ValueError('The package must include the project GPL license: '+path.name)
        return manifest


def release_notes(version, repository, revision, private=False):
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('Repository must be a GitHub owner/name.')
    base = 'https://github.com/'+repository+'/blob/v'+version+'/docs/'
    name = 'ClickPulse-'+version
    downloads = f'https://github.com/{repository}/releases/download/v{version}/'
    artwork = f'https://github.com/{repository}/blob/v{version}/assets/downloads/'

    def button(image, label, asset):
        return f'[![{label}]({artwork}{image}.svg?raw=true)]({downloads}{asset})'

    windows = button('windows-portable-x64', 'Windows Portable x64', name+'-windows-x64.zip')
    mac_arm = button('macos-apple-silicon', 'DMG Apple Silicon', name+'-macos-arm64.dmg')
    mac_intel = button('macos-intel-x64', 'DMG Intel x64', name+'-macos-x64.dmg')
    access = '\nThis private preview requires a GitHub account with repository access.\n' if private else ''
    return f'''Local keyboard and mouse activity statistics for macOS and Windows.
{access}

## Downloads

| Platform | Download |
| --- | --- |
| Windows 10/11 | {windows} |
| macOS 12+ | {mac_arm} |
| | {mac_intel} |

Mac App ZIPs are also available in the release attachments.

### Release files

| File | Contents |
| --- | --- |
| [Source code]({downloads}{name}-source.zip) | Corresponding source and build scripts |
| [SHA256 checksums]({downloads}SHA256SUMS.txt) | Verify downloaded files |

Platform manifests in the attachments list individual package files and their hashes.

### Documentation

| Guide | Covers |
| --- | --- |
| [macOS installation]({base}install-macos.md) | First launch, Input Monitoring and updates |
| [Windows installation]({base}install-windows.md) | Portable ZIP, WebView2 and updates |
| [Changelog](https://github.com/{repository}/blob/v{version}/CHANGELOG.md) | Changes included in this release |

## Preview notes

- The Mac App is ad-hoc signed without Apple notarization. Grant Input Monitoring after installation.
- Windows is distributed as a portable ZIP and requires Microsoft Edge WebView2 Runtime. No Setup installer is offered.
- The Windows executable is unsigned. Windows 11 Smart App Control may block it; portable packaging does not change publisher trust. See [Microsoft's guidance](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions).
- Apple Silicon, Intel Mac and Windows x64 packages passed automated tests and packaging on their native GitHub runners.
- Physical special-key input, login startup, multi-display DPI and Intel desktop coverage are tracked in the [validation checklist]({base}windows-testing.md) and roadmap. Automated checks do not replace those hardware tests.
- Program updates and uninstalling retain the separate user-data directory.
- Code: GPL-3.0-or-later. Packages include the license and third-party notices.

Source revision: `{revision}`
'''


def prepare_release(root, packages, output, tag, revision, repository, private=False):
    plan = release_plan(root, tag)
    if not re.fullmatch(REVISION, revision):
        raise ValueError('A full 40-character source revision is required.')
    notes = release_notes(plan['version'], repository, revision, private)
    if output.exists() or output.is_symlink():
        raise ValueError('Release output already exists; choose a new staging directory.')
    version = plan['version']
    mac_targets = [(f'ClickPulse-{version}-macos-arm64', 'aarch64-apple-darwin'),
                   (f'ClickPulse-{version}-macos-x64', 'x86_64-apple-darwin')]
    win = f'ClickPulse-{version}-windows-x64'
    source = f'ClickPulse-{version}-source'
    asset_names = [name+suffix for name,_ in mac_targets for suffix in ['.dmg','.zip']]
    asset_names += [win+'.zip', source+'.zip']
    for name in asset_names:
        file = packages/name
        if file.is_symlink() or not file.is_file() or not file.stat().st_size:
            raise ValueError('Missing or empty release asset: '+name)
        expected = digest(file)+'  '+name
        if (packages/(name+'.sha256')).read_text(encoding='utf-8').strip() != expected:
            raise ValueError('Release asset checksum mismatch: '+name)
    license_bytes = (root/'LICENSE').read_bytes()
    manifests = []
    for mac,target in mac_targets:
        manifest = verify_archive(packages/(mac+'.zip'), mac, 'build-manifest.json', plan, 'ClickPulse.app/Contents/Resources/LICENSE.txt', license_bytes)
        if json.loads((packages/(mac+'.manifest.json')).read_text(encoding='utf-8')) != manifest:
            raise ValueError('Mac external manifest does not match the App ZIP.')
        manifests.append((mac, manifest, {target}))
    win_manifest = verify_archive(packages/(win+'.zip'), win, 'build-manifest.json', plan, 'LICENSE.txt', license_bytes)
    source_manifest = verify_archive(packages/(source+'.zip'), source, 'source-manifest.json', plan, 'LICENSE', license_bytes)
    if source_manifest.get('revision') != revision or source_manifest.get('tag') != tag:
        raise ValueError('Corresponding source provenance does not match the version tag.')
    manifests.append((win, win_manifest, {'x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu'}))
    for _,manifest,targets in manifests:
        if manifest.get('source_revision') != revision:
            raise ValueError('Binary source provenance does not match the corresponding source.')
        if manifest.get('internal_build') != plan['internal_build'] or manifest.get('target') not in targets:
            raise ValueError('Package build number or architecture mismatch.')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.release-', dir=output.parent) as temporary:
        stage = Path(temporary)/'release'
        stage.mkdir()
        for name in asset_names:
            shutil.copyfile(packages/name, stage/name)
        for name,manifest,_ in manifests:
            (stage/(name+'.manifest.json')).write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
        sums = [digest(file)+'  '+file.name for file in sorted(stage.iterdir())]
        (stage/'SHA256SUMS.txt').write_text('\n'.join(sums)+'\n', encoding='utf-8')
        os.replace(stage, output)
    return notes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    plan = sub.add_parser('plan')
    plan.add_argument('--tag')
    plan.add_argument('--require-tag', action='store_true')
    plan.add_argument('--github-output', type=Path)
    prepare = sub.add_parser('prepare')
    prepare.add_argument('--tag', required=True)
    prepare.add_argument('--revision', required=True)
    prepare.add_argument('--repository', required=True)
    prepare.add_argument('--private', action='store_true', help='Note that downloads require repository access')
    prepare.add_argument('--packages', type=Path, required=True)
    prepare.add_argument('--output', type=Path, required=True)
    prepare.add_argument('--notes', type=Path, required=True)
    prepare.add_argument('--github-summary', type=Path)
    args = parser.parse_args()
    try:
        if args.command == 'plan':
            result = release_plan(ROOT, args.tag, args.require_tag)
            if args.github_output:
                with args.github_output.open('a', encoding='utf-8') as stream:
                    for key, value in result.items():
                        stream.write(f'{key}={value}\n')
            print(json.dumps(result))
        else:
            notes = prepare_release(ROOT, args.packages, args.output, args.tag, args.revision, args.repository, args.private)
            args.notes.write_text(notes, encoding='utf-8')
            if args.github_summary:
                with args.github_summary.open('a', encoding='utf-8') as stream:
                    stream.write('## Verified release candidate\n\nAll '+str(len(list(args.output.iterdir())))+' assets were validated. The Release workflow creates an unpublished draft for maintainer review.\n\n'+notes)
            print('Verified release candidate: '+str(args.output))
    except (ValueError, OSError, KeyError, zipfile.BadZipFile, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Release preparation failed: {error}\n')


if __name__ == '__main__':
    main()
