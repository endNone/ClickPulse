"""Sign and verify ClickPulse using the pinned public release certificate."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import subprocess
import tempfile

IDENTIFIER = 'app.clickpulse.desktop'


def fingerprints(root):
    pin = json.loads((root/'assets/macos-signing.json').read_text())
    if (pin.get('identifier') != IDENTIFIER
            or not re.fullmatch(r'[0-9a-f]{40}', pin['sha1'])
            or not re.fullmatch(r'[0-9a-f]{64}', pin['sha256'])):
        raise ValueError('Invalid public Mac signing fingerprints.')
    return pin


def requirement(pin):
    return f'identifier "{IDENTIFIER}" and certificate leaf = H"{pin["sha1"]}"'


def verify_app(app, pin):
    subprocess.run(['codesign', '--verify', '--deep', '--strict', '-R', '='+requirement(pin), str(app)], check=True)
    detail = subprocess.run(['codesign', '-d', '-r-', str(app)], check=True,
                            capture_output=True, text=True)
    lines = [line for line in (detail.stdout+'\n'+detail.stderr).splitlines() if line.startswith('designated => ')]
    if lines != ['designated => '+requirement(pin)]:
        raise ValueError('Unexpected Mac designated requirement.')
    with tempfile.TemporaryDirectory() as tmp:
        prefix = Path(tmp)/'cert'
        subprocess.run(['codesign', '-d', '--extract-certificates='+str(prefix), str(app)],
                       check=True, capture_output=True)
        leaf = Path(str(prefix)+'0').read_bytes()
        if hashlib.sha256(leaf).hexdigest() != pin['sha256']:
            raise ValueError('Unexpected Mac signing certificate.')


def sign_app(app, root, env):
    identity = env.get('MACOS_SIGNING_IDENTITY')
    args = ['codesign', '--force', '--timestamp=none', '--identifier', IDENTIFIER]
    if not identity:
        if env.get('MACOS_REQUIRE_RELEASE_SIGNING') == '1':
            raise ValueError('Release builds require the Mac signing identity.')
        subprocess.run([*args, '--sign', '-', str(app)], check=True)
        subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
        return {'signing': 'ad-hoc'}
    pin = fingerprints(root)
    if env.get('MACOS_SIGNING_KEYCHAIN'):
        args += ['--keychain', env['MACOS_SIGNING_KEYCHAIN']]
    subprocess.run([*args, '--sign', identity, '-r', '=designated => '+requirement(pin), str(app)], check=True)
    verify_app(app, pin)
    return {'signing': 'self-signed', 'certificate_sha256': pin['sha256']}


def verify_packages(zip_path, dmg_path, name, manifest, pin):
    if manifest.get('signing') != 'self-signed' or manifest.get('certificate_sha256') != pin['sha256']:
        raise ValueError('A release requires the pinned self-signed Mac package.')
    with tempfile.TemporaryDirectory() as tmp:
        directory = Path(tmp)
        # release.verify_archive has already validated the ZIP paths and manifest.
        subprocess.run(['ditto', '-x', '-k', str(zip_path), str(directory/'zip')], check=True)
        verify_app(directory/'zip'/name/'ClickPulse.app', pin)
        mount = directory/'volume'
        mount.mkdir()
        subprocess.run(['hdiutil', 'attach', '-readonly', '-nobrowse', '-noautoopen',
                        '-mountpoint', str(mount), str(dmg_path)], check=True, stdout=subprocess.DEVNULL)
        try:
            app = mount/'ClickPulse.app'
            if app.is_symlink() or not app.is_dir():
                raise ValueError('Missing Mac application inside DMG.')
            verify_app(app, pin)
            if json.loads((mount/'build-manifest.json').read_text()) != manifest:
                raise ValueError('DMG manifest differs from the updater ZIP.')
            for relative, digest in manifest['files'].items():
                file = mount/relative
                if (not file.resolve().is_relative_to(mount.resolve()) or not file.is_file()
                        or hashlib.sha256(file.read_bytes()).hexdigest() != digest):
                    raise ValueError('DMG contents differ from the updater ZIP: '+relative)
        finally:
            subprocess.run(['hdiutil', 'detach', str(mount)], check=True, stdout=subprocess.DEVNULL)


def security(*args):
    # CalledProcessError would include the entire command, including passwords.
    result = subprocess.run(['security', *map(str, args)], capture_output=True)
    if result.returncode:
        raise RuntimeError(f'security {args[0]} failed ({result.returncode}).')
    return result.stdout.decode()


def import_identity(directory, env, pin):
    data = env.get('MACOS_SIGNING_P12_BASE64', '')
    password = env.get('MACOS_SIGNING_P12_PASSWORD', '')
    if not data or not password:
        raise ValueError('Both Mac release signing secrets are required.')
    directory.mkdir(mode=0o700)
    keychain = directory/'release.keychain-db'
    prior = re.findall(r'"([^"]+)"', security('list-keychains', '-d', 'user'))
    (directory/'prior.json').write_text(json.dumps(prior))
    keypass = secrets.token_hex(32)
    with tempfile.NamedTemporaryFile(prefix='identity-', suffix='.p12', dir=directory) as p12:
        p12.write(base64.b64decode(data, validate=True))
        p12.flush()
        security('create-keychain', '-p', keypass, keychain)
        security('set-keychain-settings', '-lut', '3600', keychain)
        security('unlock-keychain', '-p', keypass, keychain)
        security('import', p12.name, '-k', keychain, '-P', password, '-T', '/usr/bin/codesign')
        security('set-key-partition-list', '-S', 'apple-tool:,apple:', '-s', '-k', keypass, keychain)
        security('list-keychains', '-d', 'user', '-s', keychain, *prior)
    with Path(env['GITHUB_ENV']).open('a') as f:
        f.write(f'MACOS_SIGNING_KEYCHAIN={keychain}\nMACOS_SIGNING_IDENTITY={pin["sha1"]}\n')


def cleanup(directory):
    keychain = directory/'release.keychain-db'
    previous = directory/'prior.json'
    try:
        if previous.is_file():
            security('list-keychains', '-d', 'user', '-s', *json.loads(previous.read_text()))
    finally:
        if keychain.exists():
            security('delete-keychain', keychain)
        previous.unlink(missing_ok=True)
        if directory.exists():
            # The runner may have interrupted import before the temp file closed.
            for p12 in directory.glob('identity-*.p12'):
                p12.unlink()
            directory.rmdir()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['import','cleanup'])
    args = parser.parse_args()
    directory = Path(os.environ['RUNNER_TEMP'])/'clickpulse-release-signing'
    if args.action == 'import':
        import_identity(directory, os.environ, fingerprints(Path(__file__).resolve().parent.parent))
    else:
        cleanup(directory)
