"""Release completeness, provenance and corruption checks using isolated fixtures."""
import hashlib
import json
from pathlib import Path
import plistlib
import re
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT/'scripts'))
import package_github_draft
import package_macos
import release

REVISION = 'a'*40


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)/'source'
        (self.root/'assets/windows').mkdir(parents=True)
        (self.root/'Cargo.toml').write_text('[package]\nname="clickpulse"\nversion="0.1.0"\nlicense="GPL-3.0-or-later"\n')
        (self.root/'Cargo.lock').write_text('version=4\n[[package]]\nname="clickpulse"\nversion="0.1.0"\n')
        (self.root/'assets/Info.plist').write_bytes(plistlib.dumps({'CFBundleShortVersionString':'0.1.0', 'CFBundleVersion':'45'}))
        (self.root/'assets/windows/app.rc').write_text('FILEVERSION 0,1,0,45\n')
        (self.root/'LICENSE').write_bytes(b'GPL license fixture\n')
        (self.root/'.gitattributes').write_bytes((ROOT/'.gitattributes').read_bytes())
        self.packages = Path(self.temporary.name)/'packages'
        self.packages.mkdir()
        self.output = Path(self.temporary.name)/'release'

    def checksum(self, file):
        file.with_suffix(file.suffix+'.sha256').write_text(hashlib.sha256(file.read_bytes()).hexdigest()+'  '+file.name+'\n')

    def archive(self, prefix, payload, manifest_name='build-manifest.json', **extra):
        manifest = {'version':'0.1.0', 'code_license':'GPL-3.0-or-later',
                    'files':{name:hashlib.sha256(data).hexdigest() for name,data in payload.items()}, **extra}
        file = self.packages/(prefix+'.zip')
        with zipfile.ZipFile(file, 'w') as archive:
            for name,data in payload.items():
                archive.writestr(prefix+'/'+name, data)
            archive.writestr(prefix+'/'+manifest_name, json.dumps(manifest))
        self.checksum(file)
        return manifest

    def candidate(self):
        mac = 'ClickPulse-0.1.0-macos-arm64'
        win = 'ClickPulse-0.1.0-windows-x64'
        license_bytes = (self.root/'LICENSE').read_bytes()
        manifest = self.archive(mac, {'ClickPulse.app/Contents/MacOS/clickpulse':b'app',
                                     'ClickPulse.app/Contents/Resources/LICENSE.txt':license_bytes},
                                internal_build=45, target='aarch64-apple-darwin', source_revision=REVISION)
        (self.packages/(mac+'.manifest.json')).write_text(json.dumps(manifest))
        intel = 'ClickPulse-0.1.0-macos-x64'
        manifest = self.archive(intel, {'ClickPulse.app/Contents/MacOS/clickpulse':b'intel app',
                                       'ClickPulse.app/Contents/Resources/LICENSE.txt':license_bytes},
                                internal_build=45, target='x86_64-apple-darwin', source_revision=REVISION)
        (self.packages/(intel+'.manifest.json')).write_text(json.dumps(manifest))
        self.archive(win, {'ClickPulse.exe':b'MZ executable', 'LICENSE.txt':license_bytes},
                     internal_build=45, target='x86_64-pc-windows-msvc', source_revision=REVISION)
        self.archive('ClickPulse-0.1.0-source', {'LICENSE':license_bytes, 'Cargo.toml':(self.root/'Cargo.toml').read_bytes()},
                     manifest_name='source-manifest.json', tag='v0.1.0', revision=REVISION)
        for name in [mac+'.dmg', intel+'.dmg', win+'-Setup.exe']:
            file = self.packages/name
            file.write_bytes(b'binary fixture')
            self.checksum(file)

    def prepare(self):
        return release.prepare_release(self.root, self.packages, self.output, 'v0.1.0', REVISION, 'example/ClickPulse')

    def rewrite_zip(self, name, change):
        file = self.packages/name
        with zipfile.ZipFile(file) as archive:
            entries = {entry:archive.read(entry) for entry in archive.namelist()}
        change(entries)
        with zipfile.ZipFile(file, 'w') as archive:
            for entry,data in entries.items():
                archive.writestr(entry, data)
        self.checksum(file)

    def test_tag_rejects_other_versions_and_shell_text(self):
        for tag in ['v0.2.0', '0.1.0', 'v0.1.0\nversion=9', 'v0.1.0;echo unsafe']:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.release_plan(self.root, tag)

    def test_rejects_mismatched_platform_version_lock_and_internal_build(self):
        for path,text in [('assets/windows/app.rc','FILEVERSION 0,2,0,45'),
                          ('assets/windows/app.rc','FILEVERSION 0,1,0,46'),
                          ('Cargo.lock','version=4\n[[package]]\nname="clickpulse"\nversion="0.2.0"\n')]:
            file = self.root/path
            original = file.read_bytes()
            try:
                file.write_text(text)
                with self.subTest(path=path,text=text), self.assertRaises(ValueError):
                    release.release_plan(self.root)
            finally:
                file.write_bytes(original)

    def test_checks_existing_annotated_tag_against_checked_out_commit(self):
        config = ['git', '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
                  '-c', 'commit.gpgsign=false', '-c', 'tag.gpgsign=false', '-c', 'core.hooksPath='+str(self.root/'no-hooks')]
        def git(*args):
            return subprocess.check_output(config+list(args), cwd=self.root, text=True, stderr=subprocess.STDOUT).strip()
        git('init', '--quiet', '--initial-branch=main')
        git('add', '.')
        git('commit', '--quiet', '-m', 'Fixture')
        git('tag', '-a', 'v0.1.0', '-m', 'Fixture version')
        plan = release.release_plan(self.root, 'v0.1.0', require_tag=True)
        self.assertEqual(plan['revision'], git('rev-parse', 'HEAD'))
        (self.root/'README.md').write_text('another revision')
        git('add', '.')
        git('commit', '--quiet', '-m', 'Changed after tag')
        with self.assertRaises(ValueError):
            release.release_plan(self.root, 'v0.1.0', require_tag=True)

    def test_complete_candidate_has_ten_assets_and_excludes_old_setup_and_unrelated_files(self):
        self.candidate()
        (self.packages/'events.sqlite3').write_bytes(b'private')
        (self.packages/'unrelated.exe').write_bytes(b'unrelated')
        notes = self.prepare()
        files = {file.name for file in self.output.iterdir()}
        self.assertEqual(len(files), 10)
        self.assertNotIn('ClickPulse-0.1.0-windows-x64-Setup.exe', files)
        self.assertNotIn('events.sqlite3', files)
        self.assertNotIn('unrelated.exe', files)
        for line in (self.output/'SHA256SUMS.txt').read_text(encoding='utf-8').splitlines():
            expected,name = line.split('  ')
            self.assertEqual(expected, hashlib.sha256((self.output/name).read_bytes()).hexdigest())
        self.assertIn('/blob/v0.1.0/docs/install-windows.md', notes)
        self.assertNotRegex(notes, r'[\u3400-\u9fff]')
        self.assertIn(REVISION, notes)
        self.assertIn('macos-x64.dmg', notes)

        # Every displayed download must exist in the assembled candidate.
        linked_assets = re.findall(r'\]\(https://github.com/example/ClickPulse/releases/download/v0\.1\.0/([^()]+)\)', notes)
        self.assertEqual(len(linked_assets), 5)
        self.assertTrue(set(linked_assets).issubset(files))
        self.assertEqual(set(linked_assets), {
            'ClickPulse-0.1.0-windows-x64.zip',
            'ClickPulse-0.1.0-macos-arm64.dmg',
            'ClickPulse-0.1.0-macos-x64.dmg',
            'ClickPulse-0.1.0-source.zip',
            'SHA256SUMS.txt',
        })

    def test_download_buttons_pin_images_and_assets_to_the_requested_release(self):
        notes = release.release_notes('1.2.3', 'fixture/RenamedApp', REVISION)
        buttons = re.findall(r'\[!\[([^\]]+)\]\(([^)]+)\)\]\(([^)]+)\)', notes)
        expected = {
            'Windows Portable x64': ('windows-portable-x64', 'windows-x64.zip'),
            'DMG Apple Silicon': ('macos-apple-silicon', 'macos-arm64.dmg'),
            'DMG Intel x64': ('macos-intel-x64', 'macos-x64.dmg'),
        }
        self.assertEqual(len(buttons), len(expected))
        for label, image, target in buttons:
            artwork, asset = expected[label]
            self.assertEqual(image, f'https://github.com/fixture/RenamedApp/blob/v1.2.3/assets/downloads/{artwork}.svg?raw=true')
            self.assertEqual(target, f'https://github.com/fixture/RenamedApp/releases/download/v1.2.3/ClickPulse-1.2.3-{asset}')
            self.assertTrue((ROOT/'assets/downloads'/f'{artwork}.svg').is_file())
        self.assertNotIn('/main/', notes)
        self.assertNotIn('/latest/', notes)
        self.assertNotIn('ClickPulse-0.1.0-', notes)

    def test_private_release_explains_repository_access(self):
        private = release.release_notes('0.1.0', 'fixture/ClickPulse', REVISION, private=True)
        public = release.release_notes('0.1.0', 'fixture/ClickPulse', REVISION)
        self.assertIn('requires a GitHub account with repository access', private)
        self.assertNotIn('requires a GitHub account with repository access', public)

    def test_missing_or_wrong_architecture_intel_package_prevents_a_release(self):
        self.candidate()
        zip_path = self.packages/'ClickPulse-0.1.0-macos-x64.zip'
        old = zip_path.read_bytes();zip_path.unlink()
        with self.assertRaisesRegex(ValueError, 'Missing'):
            self.prepare()
        self.assertFalse(self.output.exists())
        zip_path.write_bytes(old)
        name = 'ClickPulse-0.1.0-macos-x64/build-manifest.json'
        def change(entries):
            manifest = json.loads(entries[name]);manifest['target']='aarch64-apple-darwin'
            entries[name]=json.dumps(manifest).encode()
            (self.packages/'ClickPulse-0.1.0-macos-x64.manifest.json').write_text(json.dumps(manifest))
        self.rewrite_zip(zip_path.name, change)
        with self.assertRaisesRegex(ValueError, 'architecture'):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_missing_platform_or_wrong_outer_checksum_cannot_create_partial_output(self):
        self.candidate()
        file = self.packages/'ClickPulse-0.1.0-windows-x64.zip'
        data = file.read_bytes()
        file.unlink()
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())
        file.write_bytes(data+b'corruption')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_inner_file_corruption_is_detected_even_with_a_recomputed_zip_checksum(self):
        self.candidate()
        self.rewrite_zip('ClickPulse-0.1.0-windows-x64.zip', lambda entries:entries.update({'ClickPulse-0.1.0-windows-x64/ClickPulse.exe':b'corrupted'}))
        with self.assertRaisesRegex(ValueError, 'file checksum'):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_unlisted_file_in_package_is_rejected(self):
        self.candidate()
        self.rewrite_zip('ClickPulse-0.1.0-windows-x64.zip', lambda entries:entries.update({'ClickPulse-0.1.0-windows-x64/unlisted.txt':b'not in manifest'}))
        with self.assertRaisesRegex(ValueError, 'every file'):
            self.prepare()

    def test_wrong_source_revision_is_rejected(self):
        self.candidate()
        name = 'ClickPulse-0.1.0-source/source-manifest.json'
        def change(entries):
            manifest = json.loads(entries[name]);manifest['revision'] = 'b'*40
            entries[name] = json.dumps(manifest).encode()
        self.rewrite_zip('ClickPulse-0.1.0-source.zip', change)
        with self.assertRaisesRegex(ValueError, 'provenance'):
            self.prepare()

    def test_gpl_text_and_private_database_are_checked_beyond_the_manifest_hashes(self):
        for payload in [{'ClickPulse.exe':b'app','LICENSE.txt':b'wrong license'},
                        {'ClickPulse.exe':b'app','LICENSE.txt':(self.root/'LICENSE').read_bytes(),'events.sqlite3':b'private'}]:
            self.candidate()
            self.archive('ClickPulse-0.1.0-windows-x64', payload, internal_build=45, target='x86_64-pc-windows-msvc', source_revision=REVISION)
            with self.subTest(payload=list(payload)), self.assertRaises(ValueError):
                self.prepare()
            self.assertFalse(self.output.exists())

    def test_older_binary_with_same_public_version_cannot_be_paired_with_new_source(self):
        self.candidate()
        name = 'ClickPulse-0.1.0-windows-x64/build-manifest.json'
        def change(entries):
            manifest = json.loads(entries[name]);manifest['source_revision'] = 'b'*40
            entries[name] = json.dumps(manifest).encode()
        self.rewrite_zip('ClickPulse-0.1.0-windows-x64.zip', change)
        with self.assertRaisesRegex(ValueError, 'Binary source provenance'):
            self.prepare()
        self.assertFalse(self.output.exists())

    def test_existing_release_candidate_is_never_overwritten(self):
        self.candidate()
        self.output.mkdir()
        previous = self.output/'previous.txt';previous.write_bytes(b'keep')
        with self.assertRaisesRegex(ValueError, 'already exists'):
            self.prepare()
        self.assertEqual(previous.read_bytes(), b'keep')

    def test_unsafe_and_metadata_archive_paths_are_rejected(self):
        for name in ['../outside','/absolute','C:\\outside','root/file:stream','root/._hidden','__MACOSX/file', 'root/clickpulse.log.1', 'root/clickpulse.LOG.3']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                release.safe_archive_path(name)

    @unittest.skipUnless(sys.platform == 'darwin', 'Uses the macOS archive tool')
    def test_mac_zip_omits_extended_metadata_and_preserves_manifest_payload(self):
        self.candidate()
        prefix = 'ClickPulse-0.1.0-macos-arm64'
        with zipfile.ZipFile(self.packages/(prefix+'.zip')) as archive:
            archive.extractall(self.packages/'stage')
        stage = self.packages/'stage'/prefix
        file = stage/'ClickPulse.app/Contents/MacOS/clickpulse'
        subprocess.run(['xattr','-w','com.clickpulse.fixture','synthetic metadata',str(file)], check=True)
        output = self.packages/'clean.zip'
        package_macos.create_zip(stage, output)
        manifest = release.verify_archive(output, prefix, 'build-manifest.json', release.release_plan(self.root),
                                          'ClickPulse.app/Contents/Resources/LICENSE.txt', (self.root/'LICENSE').read_bytes())
        self.assertEqual(len(manifest['files']), 2)

    def test_source_modes_include_license_and_exclude_history_environment_and_private_notes(self):
        for name in ['.gitignore','build.rs','README.md','CHANGELOG.md','CONTRIBUTING.md','SECURITY.md']:
            (self.root/name).write_text('public fixture')
        (self.root/'docs').mkdir()
        for name in ['events.sqlite3','events.sqlite3-wal','capture.log','capture.log.1','capture.LOG.3','.env','trace.md']:
            (self.root/'docs'/name).write_bytes(b'private')
        (self.root/'.Codex').mkdir();(self.root/'.Codex/secret.md').write_bytes(b'private')
        (self.root/'.cargo').mkdir()
        (self.root/'.cargo/credentials.toml').write_text('private fixture')
        for mode in [False, True]:
            archive,_,_ = package_github_draft.package_source(self.root, mode, 'v0.1.0' if mode else None, REVISION if mode else None)
            with zipfile.ZipFile(archive) as file:
                names = file.namelist()
                self.assertTrue(any(name.endswith('/LICENSE') for name in names))
                self.assertTrue(any(name.endswith('/.gitattributes') for name in names))
                self.assertFalse(any('credentials.toml' in name for name in names))
                self.assertFalse(any('capture.log' in name.lower() or '.Codex' in name or name.endswith('trace.md') or 'events.sqlite3' in name or '/.env' in name for name in names))
                manifest = json.loads(file.read(next(name for name in names if name.endswith('-manifest.json'))))
                if mode:
                    self.assertEqual(manifest['revision'], REVISION)
                else:
                    self.assertTrue(manifest['draft'])


if __name__ == '__main__':
    unittest.main()
