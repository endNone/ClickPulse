"""Build and package a Windows x64 portable release using only explicit assets."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import zipfile
from windows_payload import payload_paths
from third_party import rust_notices, asset_notices
from release import source_revision
from windows_imports import verify_static_crt
from release_symbols import build_release, stage_binary

parser = argparse.ArgumentParser()
parser.add_argument('--target', default='x86_64-pc-windows-gnu', choices=['x86_64-pc-windows-gnu', 'x86_64-pc-windows-msvc'])
parser.add_argument('--skip-build', action='store_true')
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
os.chdir(root)
environment = os.environ.copy()
if args.target.endswith('gnu'):
    environment.setdefault('CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER', 'x86_64-w64-mingw32-gcc')
else:
    # --target keeps build-host proc macros separate from the static app CRT.
    environment.setdefault('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS', '-C target-feature=+crt-static')
if not args.skip_build:
    build_release(root, args.target, environment)
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--filter-platform', args.target], env=environment))
packages = {p['id']: p for p in metadata['packages']}
version = tomllib.loads((root/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
resource_version = re.search(r'FILEVERSION\s+(\d+),(\d+),(\d+),(\d+)', (root/'assets/windows/app.rc').read_text(encoding='utf-8'))
if resource_version is None or '.'.join(resource_version.groups()[:3]) != version:
    raise SystemExit('Cargo and Windows resource public versions must match.')
internal_build = int(resource_version.group(4))
name = f'ClickPulse-{version}-windows-x64'
dist = root/'dist'
dist.mkdir(exist_ok=True)
exe = Path(metadata['target_directory'])/args.target/'release/clickpulse.exe'
if not exe.is_file():
    raise SystemExit(f'Missing built executable: {exe}')
if args.target.endswith('msvc'):
    verify_static_crt(exe)
with tempfile.TemporaryDirectory(prefix='.windows-package-', dir=dist) as temporary:
    temp = Path(temporary)
    bundle = temp/name
    bundle.mkdir()
    stage_binary(exe, bundle/'ClickPulse.exe', root, args.target, version, not args.skip_build)
    shutil.copy2(root/'LICENSE', bundle/'LICENSE.txt')
    loader = next(p for p in packages.values() if p['name'] == 'webview2-com-sys')
    loader_dir = Path(loader['manifest_path']).parent
    shutil.copy2(loader_dir/'x64/WebView2Loader.dll', bundle/'WebView2Loader.dll')
    licenses = bundle/'Licenses'
    licenses.mkdir()
    if (root/'TRADEMARKS.md').is_file(): shutil.copy2(root/'TRADEMARKS.md', licenses/'TRADEMARKS.txt')
    for file in [*(root/'assets/fonts').glob('*-OFL.txt'), *(root/'assets/windows').glob('*LICENSE*.txt'), *(root/'assets/windows').glob('*NOTICE*.txt')]:
        shutil.copy2(file, licenses/file.name)
    notices = [asset_notices(root), 'WebView2Loader.dll: Microsoft WebView2 SDK (version resolved by Cargo.lock / webview2-com-sys). SDK license/notice and webview2-rs MIT license are in Licenses/.', '', rust_notices(metadata, root)]
    (bundle/'THIRD-PARTY-NOTICES.txt').write_text('\n'.join(notices), encoding='utf-8')
    (bundle/'README.txt').write_text(f'''ClickPulse {version} — Windows x64

Code license: GNU GPL version 3 or later (GPL-3.0-or-later).
Full license: LICENSE.txt. Third-party assets retain their licenses.

Extract the entire folder to a fixed location such as %LOCALAPPDATA%\\Programs\\ClickPulse.
Keep WebView2Loader.dll beside the EXE and open ClickPulse.exe. No Setup installer is required.
The program is unsigned. Windows 11 Smart App Control may block it; ZIP packaging does not bypass this policy.
Windows 10/11 x64 requires Microsoft Edge WebView2 Runtime:
https://go.microsoft.com/fwlink/p/?LinkId=2124703

Quit the old version from the tray before updating. Updates retain history and preferences.
To uninstall, use Settings > Application > Uninstall and data, or run ClickPulse.exe uninstall.
Choose whether to keep local data for reinstalling or remove it after confirmation.
Databases outside the data folder are kept at the displayed location. Direct program removal keeps all user data.
First launch uses the standard app data folder without automatically selecting old developer databases.
The app follows the system language; English, Simplified Chinese, Traditional Chinese, Japanese, Korean, German, French and Spanish are available in Settings.
Choose Blue, Teal, Violet or Graphite; the full and mini views share the saved theme.

Closing offers Minimize to tray or Save and quit, with an optional remembered choice.
Change Close behavior in Settings. Double-click the tray icon to reopen; right-click to quit.
Quitting saves pending data, stops capture and closes the renderer.
Enable launch at login in Settings, then keep the application folder in a fixed location.

Held keys count once until released. Mouse counts track button presses.
No typed text or pointer coordinates are stored.
Statistics and preferences: %LOCALAPPDATA%\\ClickPulse; logs are in its logs folder.
Font licenses and third-party notices: Licenses/ and THIRD-PARTY-NOTICES.txt.
Full installation guide: docs/install-windows.md in the source repository.

Package integrity checks do not replace native Windows desktop/input testing.
''', encoding='utf-8')
    files = {path.as_posix(): hashlib.sha256((bundle/path).read_bytes()).hexdigest() for path in payload_paths(bundle)}
    (bundle/'build-manifest.json').write_text(json.dumps({'version':version,'internal_build':internal_build,'target':args.target,'source_revision':source_revision(root),'code_license':'GPL-3.0-or-later','files':files,'native_windows_tested':False}, indent=2)+'\n')
    external_manifest = (bundle/'build-manifest.json').read_bytes()
    archive = temp/(name+'.zip')
    with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as output:
        for file in sorted(bundle.rglob('*')):
            if file.is_file():
                output.write(file,file.relative_to(temp))
    destination = dist/(name+'.zip')
    os.replace(archive,destination)
    (dist/(name+'.manifest.json')).write_bytes(external_manifest)
sha = hashlib.sha256(destination.read_bytes()).hexdigest()
destination.with_suffix('.zip.sha256').write_text(sha+'  '+destination.name+'\n')
print(f'Windows release: {destination}\nSHA256: {sha}')
