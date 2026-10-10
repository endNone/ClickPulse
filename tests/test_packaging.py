"""Installer deletion boundaries and shared build routing, without running an installer."""
import json
import struct
from pathlib import Path
import sys
import tempfile
import subprocess
import unittest
from unittest.mock import call, patch

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT/'scripts'))
from build_windows_installer import build_installer, payload_paths, uninstall_include
import build
import package_macos
import setup_windows_installer
from third_party import rust_notices
from windows_imports import imported_libraries, verify_static_crt


class PackagingTests(unittest.TestCase):
    def test_available_nsis_skips_downloads(self):
        with patch('setup_windows_installer.compiler_path', return_value=Path('makensis')), \
             patch('setup_windows_installer.subprocess.run') as run, \
             patch('setup_windows_installer.time.sleep') as sleep:
            self.assertEqual(setup_windows_installer.ensure_compiler(), Path('makensis'))
            run.assert_not_called()
            sleep.assert_not_called()

    def test_nsis_download_retries_failed_feed_and_false_success(self):
        missing = RuntimeError('NSIS compiler is missing')
        # The second attempt exits zero but still leaves no compiler; the third
        # must run rather than letting the package build fail in a later step.
        with patch('setup_windows_installer.compiler_path', side_effect=[missing, missing, missing, Path('makensis')]), \
             patch('setup_windows_installer.subprocess.run', side_effect=[subprocess.CompletedProcess([], 1), subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 0)]) as run, \
             patch('setup_windows_installer.time.sleep') as sleep:
            self.assertEqual(setup_windows_installer.ensure_compiler(), Path('makensis'))
            self.assertEqual(run.call_args_list, [call(['choco', 'install', 'nsis', '--no-progress', '-y', '--limit-output'], check=False, timeout=180)] * 3)
            self.assertEqual(sleep.call_args_list, [call(15), call(15)])

    def test_nsis_provisioning_fails_after_bounded_attempts(self):
        for result in [subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 1), OSError('choco unavailable'), subprocess.TimeoutExpired('choco', 180)]:
            with self.subTest(result=result), \
                 patch('setup_windows_installer.compiler_path', side_effect=RuntimeError('NSIS compiler is missing')), \
                 patch('setup_windows_installer.subprocess.run', side_effect=result if isinstance(result, (OSError, subprocess.TimeoutExpired)) else None, return_value=result) as run, \
                 patch('setup_windows_installer.time.sleep') as sleep:
                with self.assertRaisesRegex(RuntimeError, 'failed after 3 attempts'):
                    setup_windows_installer.ensure_compiler()
                self.assertEqual(run.call_count, 3)
                self.assertEqual(sleep.call_count, 2)

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
                'installer.nsi': (ROOT/'assets/windows/installer.nsi').read_bytes(),
                'assets/windows/NSIS-LICENSE.txt': (ROOT/'assets/windows/NSIS-LICENSE.txt').read_bytes(),
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

    def test_uninstall_enumerates_only_payload_and_removes_only_empty_directories(self):
        with tempfile.TemporaryDirectory() as temporary:
            bundle = self.bundle(Path(temporary))
            include = uninstall_include(bundle)
            self.assertIn('Delete "$INSTDIR\\ClickPulse.exe"', include)
            self.assertIn('RMDir "$INSTDIR\\Licenses"', include)
            self.assertNotIn('/r', include)
            self.assertNotIn('LOCALAPPDATA', include)
            self.assertNotIn('events.sqlite3', include)

    def test_private_files_and_injected_paths_cannot_become_installer_payload(self):
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

    def test_windows_build_defaults_to_installer_and_portable_option_does_not_require_it(self):
        for portable in [False, True]:
            argv = ['build.py', '--platform', 'windows', '--target', 'x86_64-pc-windows-gnu']
            if portable: argv.append('--portable-only')
            with patch.object(sys, 'argv', argv), patch('build.subprocess.run') as run:
                build.main()
                command = run.call_args.args[0]
                self.assertEqual('--installer' in command, not portable)
                self.assertIn('x86_64-pc-windows-gnu', command)

    def test_installer_version_has_four_dot_separated_parts_and_publishes_after_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            bundle = self.bundle(folder/'payload')
            output = folder/'published.exe'
            output.write_bytes(b'previous package')

            def compile_success(command, **kwargs):
                self.assertTrue(any(flag.endswith('NUMERIC_VERSION=0.1.0.44') for flag in command))
                self.assertEqual(output.read_bytes(), b'previous package')
                (folder/'ClickPulse-Setup.exe').write_bytes(b'new package')

            with patch('build_windows_installer.subprocess.run', side_effect=compile_success):
                sha = build_installer(ROOT, bundle, '0.1.0', 44, output, compiler=Path('makensis'))
            self.assertEqual(output.read_bytes(), b'new package')
            self.assertIn(sha, output.with_suffix('.exe.sha256').read_text(encoding='utf-8'))

    def test_failed_installer_compilation_keeps_previous_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            bundle = self.bundle(folder/'payload')
            output = folder/'published.exe'
            output.write_bytes(b'previous package')
            with patch('build_windows_installer.subprocess.run', side_effect=subprocess.CalledProcessError(1, 'makensis')):
                with self.assertRaises(subprocess.CalledProcessError):
                    build_installer(ROOT, bundle, '0.1.0', 44, output, compiler=Path('makensis'))
            self.assertEqual(output.read_bytes(), b'previous package')
            self.assertFalse(output.with_suffix('.exe.sha256').exists())

    def test_legacy_window_detection_covers_both_catalogs_and_installer_is_per_user(self):
        script = (ROOT/'assets/windows/installer.nsi').read_text(encoding='utf-8')
        messages = json.loads((ROOT/'ui/locales.json').read_text(encoding='utf-8'))
        for locale in ['en', 'zh-CN']:
            for key in ['native.full_title', 'native.mini_title']:
                self.assertIn(messages[locale][key], script)
        self.assertIn('RequestExecutionLevel user', script)
        self.assertNotIn('WriteRegStr HKLM', script)
        self.assertNotIn('RMDir /r', script)
        self.assertNotIn('Delete "$LOCALAPPDATA', script)
        self.assertIn('Local\\ClickPulse.Setup', script)
        self.assertIn('Call un.EnsureStopped', script)


if __name__ == '__main__':
    unittest.main()
