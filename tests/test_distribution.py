"""Distribution updates must fail before changing installable package metadata."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('update_distribution', ROOT/'scripts/update_distribution.py')
distribution = importlib.util.module_from_spec(spec)
spec.loader.exec_module(distribution)


class DistributionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for folder in ['Casks', 'bucket']:
            shutil.copytree(ROOT/folder, self.root/folder)
        self.packages = self.root/'release'
        self.packages.mkdir()
        sums = []
        for platform, ext in [('macos-arm64', 'dmg'), ('macos-x64', 'dmg'), ('windows-x64', 'zip')]:
            name = f'ClickPulse-9.8.7-{platform}.{ext}'
            payload = platform.encode()
            (self.packages/name).write_bytes(payload)
            sums.append(hashlib.sha256(payload).hexdigest()+'  '+name)
        (self.packages/'SHA256SUMS.txt').write_text('\n'.join(sums)+'\n')
        self.before = {p: p.read_bytes() for p in [self.root/'Casks/clickpulse.rb', self.root/'bucket/clickpulse.json']}

    def test_release_update_changes_both_architectures_and_windows_layout(self):
        hashes = distribution.update(self.root, self.packages, '9.8.7')
        cask = (self.root/'Casks/clickpulse.rb').read_text()
        self.assertIn('version "9.8.7"', cask)
        self.assertIn(hashes['macos-arm64'], cask)
        self.assertIn(hashes['macos-x64'], cask)
        scoop = json.loads((self.root/'bucket/clickpulse.json').read_text())
        self.assertEqual(scoop['version'], '9.8.7')
        arch = scoop['architecture']['64bit']
        self.assertEqual(arch['extract_dir'], 'ClickPulse-9.8.7-windows-x64')
        self.assertTrue(arch['url'].endswith('/v9.8.7/ClickPulse-9.8.7-windows-x64.zip'))
        self.assertEqual(arch['hash'], hashes['windows-x64'])
        original = json.loads(self.before[self.root/'bucket/clickpulse.json'])
        self.assertEqual(scoop['autoupdate'], original['autoupdate'])
        template = scoop['autoupdate']['architecture']['64bit']
        for field in ['url', 'extract_dir']:
            self.assertEqual(template[field].replace('$version', '9.8.7'), arch[field])

    def test_corrupt_package_changes_neither_manifest(self):
        (self.packages/'ClickPulse-9.8.7-windows-x64.zip').write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError, 'Checksum mismatch'):
            distribution.update(self.root, self.packages, '9.8.7')
        for path, contents in self.before.items():
            self.assertEqual(path.read_bytes(), contents)

    def test_prerelease_and_duplicate_hashes_are_rejected(self):
        with self.assertRaises(ValueError):
            distribution.update(self.root, self.packages, '9.8.7-beta')
        sums = self.packages/'SHA256SUMS.txt'
        sums.write_text(sums.read_text()*2)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            distribution.update(self.root, self.packages, '9.8.7')
        for path, contents in self.before.items():
            self.assertEqual(path.read_bytes(), contents)
