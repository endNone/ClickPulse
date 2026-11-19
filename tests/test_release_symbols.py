"""Debug artifacts must match the staged executable without mutating Cargo output."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'scripts'))
from release_symbols import build_release, stage_binary, verify_release_symbols, record_packaged_binary


class ReleaseSymbolsTests(unittest.TestCase):
    def test_release_gate_requires_all_targets_and_exact_symbol_and_binary_hashes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); symbols=root/'symbols'; packages=root/'packages'; packages.mkdir()
            targets=[('aarch64-apple-darwin','macos-arm64','ClickPulse.app/Contents/MacOS/clickpulse'),('x86_64-apple-darwin','macos-x64','ClickPulse.app/Contents/MacOS/clickpulse'),('x86_64-pc-windows-msvc','windows-x64','ClickPulse.exe'),('x86_64-unknown-linux-gnu','linux-x64','clickpulse'),('aarch64-unknown-linux-gnu','linux-arm64','clickpulse')]
            digest=hashlib.sha256(b'binary').hexdigest()
            for target, platform, binary in targets:
                folder=symbols/target; folder.mkdir(parents=True)
                (folder/'debug').write_bytes(b'debug')
                (folder/'manifest.json').write_text(json.dumps({'version':'1.0.0','target':target,'packaged_binary_sha256':digest,'files':{'debug':hashlib.sha256(b'debug').hexdigest()}}))
                (packages/f'ClickPulse-1.0.0-{platform}.manifest.json').write_text(json.dumps({'version':'1.0.0','target':target,'files':{binary:digest}}))
            verify_release_symbols(symbols,packages,'1.0.0')
            file=symbols/targets[0][0]/'debug';file.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'file changed'):verify_release_symbols(symbols,packages,'1.0.0')
            file.write_bytes(b'debug')
            staged=root/'signed';staged.write_bytes(b'signed binary')
            record_packaged_binary(file.parent/'manifest.json',staged)
            with self.assertRaisesRegex(ValueError, 'packaged executable'):verify_release_symbols(symbols,packages,'1.0.0')
            (file.parent/'manifest.json').unlink()
            with self.assertRaisesRegex(ValueError, 'Missing platform'):verify_release_symbols(symbols,packages,'1.0.0')

    def test_build_retains_debug_info_and_caller_environment(self):
        for target, split in [('aarch64-apple-darwin', 'packed'),
                              ('x86_64-pc-windows-msvc', 'packed'),
                              ('aarch64-unknown-linux-gnu', 'off')]:
            environment = {'PATH': 'test-path', 'CARGO_PROFILE_RELEASE_STRIP': 'symbols'}
            with patch('release_symbols.subprocess.run') as run, patch('release_symbols.shutil.which', return_value='/tools/tool'):
                build_release(Path('source'), target, environment)
            args, kwargs = run.call_args
            self.assertEqual(args[0][-2:], ['--target', target])
            self.assertEqual(kwargs['env']['PATH'], 'test-path')
            self.assertEqual(kwargs['env']['CARGO_PROFILE_RELEASE_DEBUG'], 'line-tables-only')
            self.assertEqual(kwargs['env']['CARGO_PROFILE_RELEASE_STRIP'], 'none')
            self.assertEqual(kwargs['env']['CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO'], split)
            self.assertEqual(environment['CARGO_PROFILE_RELEASE_STRIP'], 'symbols')

    def test_missing_symbol_tools_fail_before_build_or_staging(self):
        with patch('release_symbols.shutil.which', return_value=None), patch('release_symbols.subprocess.run') as run:
            with self.assertRaisesRegex(RuntimeError, 'Missing debug-symbol tools'):
                build_release(Path('source'), 'aarch64-apple-darwin')
            run.assert_not_called()

    def test_skip_build_preserves_explicit_binary_without_claiming_symbols(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root/'clickpulse'; destination = root/'staged'
            source.write_bytes(b'explicit binary')
            stage_binary(source, destination, root, 'aarch64-apple-darwin', '0.3.1', False)
            self.assertEqual(source.read_bytes(), destination.read_bytes())
            self.assertFalse((root/'dist/symbols').exists())

    def test_msvc_requires_matching_pdb_and_keeps_it_outside_payload(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root/'clickpulse.exe'; destination = root/'staged.exe'
            source.write_bytes(b'PE with CodeView identity')
            with self.assertRaises(FileNotFoundError):
                stage_binary(source, destination, root, 'x86_64-pc-windows-msvc', '0.3.1', True)
            source.with_suffix('.pdb').write_bytes(b'matching pdb')
            stage_binary(source, destination, root, 'x86_64-pc-windows-msvc', '0.3.1', True)
            manifest = next((root/'dist/symbols').rglob('manifest.json'))
            data = json.loads(manifest.read_text())
            self.assertEqual(data['files']['clickpulse.pdb'], hashlib.sha256(b'matching pdb').hexdigest())
            self.assertEqual(source.read_bytes(), destination.read_bytes())
            self.assertFalse(destination.with_suffix('.pdb').exists())

    @unittest.skipUnless(sys.platform in ('darwin', 'linux') and shutil.which('cc'), 'native C toolchain required')
    def test_native_symbols_survive_stripping_and_cargo_binary_is_unchanged(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); source = root/'clickpulse'; destination = root/'staged'
            code = root/'main.c'; code.write_text('int helper(int x) { return x + 1; }\nint main(void) { return helper(0) - 1; }\n')
            subprocess.run(['cc', '-g', '-O0', str(code), '-o', str(source)], check=True)
            before = source.read_bytes()
            target = 'aarch64-apple-darwin' if sys.platform == 'darwin' else 'aarch64-unknown-linux-gnu'
            stage_binary(source, destination, root, target, '0.3.1', True)
            self.assertEqual(source.read_bytes(), before)
            subprocess.run([str(destination)], check=True)
            manifest = next((root/'dist/symbols').rglob('manifest.json'))
            data = json.loads(manifest.read_text())
            self.assertEqual(data['unstripped_sha256'], hashlib.sha256(before).hexdigest())
            self.assertEqual(data['stripped_binary_sha256_before_signing'], hashlib.sha256(destination.read_bytes()).hexdigest())
            if sys.platform == 'darwin':
                symbols = manifest.parent/'clickpulse.dSYM'
                binary_uuid = subprocess.check_output(['dwarfdump', '--uuid', str(destination)], text=True).split()[1]
                self.assertIn(binary_uuid, subprocess.check_output(['dwarfdump', '--uuid', str(symbols)], text=True))
                self.assertIn('helper', subprocess.check_output(['dwarfdump', '--debug-info', str(symbols)], text=True))
            else:
                debug_info = subprocess.check_output(['readelf', '--debug-dump=info', str(manifest.parent/'clickpulse.debug')], text=True)
                self.assertIn('helper', debug_info)
                self.assertLess(destination.stat().st_size, len(before))
