"""Portable payload boundaries and shared build routing, without running the app."""
import json
import struct
from pathlib import Path
import sys
import tempfile
import subprocess
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT/'scripts'))
from windows_payload import payload_paths
import build
import package_macos
from third_party import rust_notices
from windows_imports import imported_libraries, verify_static_crt

class PackagingTests(unittest.TestCase):

    def test_windows_package_rejects_external_crt_missing_on_a_clean_machine(self):
        def executable(library):
            data = bytearray(0x600)
            data[:2] = b'MZ'
            struct.pack_into('<I', data, 0x3c, 0x80)
            data[0x80:0x84] = b'PE\0\0'
            struct.pack_into('<HH', data, 0x84, 0x8664, 1)
            struct.pack_into('<H', data, 0x94, 240)
            struct.pack_into('<H', data, 0x98, 0x20b)
            struct.pack_into('<II', data, 0x98+120, 0x1000, 40)
            struct.pack_into('<IIII', data, 0x98+240+8, 0x400, 0x1000, 0x400, 0x200)
            struct.pack_into('<IIIII', data, 0x200, 0, 0, 0, 0x1100, 0)
            name = library.encode('ascii')+b'\0'
            data[0x300:0x300+len(name)] = name
            return data

        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'ClickPulse.exe'
            path.write_bytes(executable('KERNEL32.dll'))
            self.assertEqual(verify_static_crt(path), ['kernel32.dll'])
            for library in ['VCRUNTIME140.dll', 'VCRUNTIME140_1.dll', 'MSVCP140.dll']:
                path.write_bytes(executable(library))
                with self.subTest(library=library), self.assertRaisesRegex(ValueError, 'unbundled'):
                    verify_static_crt(path)
            for data in [b'not PE', executable('KERNEL32.dll')[:0x308]]:
                with self.assertRaises(ValueError):
                    imported_libraries(data)

    def test_autocrlf_checkout_preserves_license_source_and_binary_bytes(self):
        # Exercise Git itself: the Windows checkout used CRLF while the release
        # assembler compared its LF license, despite every platform test passing.
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)/'source'
            source.mkdir()
            payload = {
                '.gitattributes': (ROOT/'.gitattributes').read_bytes(),
                'LICENSE': (ROOT/'LICENSE').read_bytes(),
                'scripts/package_windows.py': (ROOT/'scripts/package_windows.py').read_bytes(),
                'assets/windows/WebView2-Rust-LICENSE.txt': (ROOT/'assets/windows/WebView2-Rust-LICENSE.txt').read_bytes(),
                'font.ttf': b'synthetic font\r\nbytes\n',
                'image.png': b'synthetic image\r\nbytes\n',
            }
            for name, data in payload.items():
                (source/name).parent.mkdir(parents=True, exist_ok=True)
                (source/name).write_bytes(data)
            git = ['git', '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
                   '-c', 'commit.gpgsign=false', '-c', 'core.autocrlf=false',
                   '-c', 'core.hooksPath='+str(source/'no-hooks')]
            subprocess.run(git+['init', '--quiet', '--initial-branch=main'], cwd=source, check=True)
            subprocess.run(git+['add', '.'], cwd=source, check=True)
            subprocess.run(git+['commit', '--quiet', '-m', 'Fixture'], cwd=source, check=True)
            checkout = Path(temporary)/'windows-checkout'
            subprocess.run(['git', '-c', 'core.autocrlf=true', 'clone', '--quiet', '--no-local',
                            str(source), str(checkout)], check=True)
            for name, data in payload.items():
                with self.subTest(name=name):
                    self.assertEqual((checkout/name).read_bytes(), data)

    def test_dependency_notice_preserves_utf8_authors_and_license_text(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            dependency = root/'dependency'
            dependency.mkdir()
            text = 'Copyright François — 世界\nMIT license fixture\n'
            (dependency/'LICENSE').write_text(text, encoding='utf-8')
            metadata = {
                'packages': [
                    {'id':'root', 'name':'clickpulse', 'version':'0.1.0', 'source':None},
                    {'id':'dependency', 'name':'fixture', 'version':'1.0.0', 'source':'registry+fixture',
                     'manifest_path':str(dependency/'Cargo.toml'), 'license':'MIT', 'repository':None},
                ],
                'resolve': {'root':'root', 'nodes':[
                    {'id':'root', 'dependencies':['dependency']},
                    {'id':'dependency', 'dependencies':[]},
                ]},
            }
            self.assertIn(text, rust_notices(metadata, root))

    def test_mac_build_selects_host_architecture_or_an_explicit_cross_target(self):
        self.assertEqual(package_macos.mac_target(machine='arm64'), 'aarch64-apple-darwin')
        self.assertEqual(package_macos.mac_target(machine='x86_64'), 'x86_64-apple-darwin')
        self.assertEqual(package_macos.mac_target('x86_64-apple-darwin', 'arm64'), 'x86_64-apple-darwin')
        with self.assertRaises(ValueError):
            package_macos.mac_target('x86_64-pc-windows-msvc')
        argv=['build.py','--platform','macos','--target','x86_64-apple-darwin','--skip-build']
        with patch.object(sys,'argv',argv), patch('build.subprocess.run') as run:
            build.main()
            command=run.call_args.args[0]
            self.assertIn('x86_64-apple-darwin', command)
            self.assertIn('--skip-build', command)
            self.assertNotIn('--installer', command)

    def bundle(self, path):
        for name in ['ClickPulse.exe', 'WebView2Loader.dll', 'README.txt', 'Licenses/Inter-OFL.txt']:
            file = path/name; file.parent.mkdir(parents=True, exist_ok=True); file.write_bytes(b'test fixture')
        return path

    def test_private_files_and_injected_paths_cannot_become_portable_payload(self):
        for name in ['events.sqlite3', 'events.sqlite', 'database.db', 'capture.log', 'capture.log.1', 'capture.LOG.3', 'events.sqlite3-wal', '$unsafe.txt']:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                bundle = self.bundle(Path(temporary)); (bundle/name).write_bytes(b'private')
                with self.assertRaises(ValueError):
                    payload_paths(bundle)

    @unittest.skipIf(sys.platform == 'win32', 'Windows symlink creation can require elevated developer settings')
    def test_payload_does_not_follow_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            bundle = self.bundle(Path(temporary)); (bundle/'outside').symlink_to('/tmp')
            with self.assertRaises(ValueError):
                payload_paths(bundle)

    def test_windows_build_produces_only_portable_zip(self):
        for option in [None, '--portable-only']:
            argv = ['build.py', '--platform', 'windows', '--target', 'x86_64-pc-windows-gnu']
            if option: argv.append(option)
            with patch.object(sys, 'argv', argv), patch('build.subprocess.run') as run:
                build.main()
                command = run.call_args.args[0]
                self.assertNotIn('--installer', command)
                self.assertIn('x86_64-pc-windows-gnu', command)

    def test_removed_installer_option_cannot_start_a_build(self):
        for script in ['build.py', 'package_windows.py']:
            result = subprocess.run([sys.executable, str(ROOT/'scripts'/script), '--installer'], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('unrecognized arguments: --installer', result.stderr)

if __name__ == '__main__':
    unittest.main()
