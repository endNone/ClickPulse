import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT/'scripts'))
import macos_signing as sign


class SigningTests(unittest.TestCase):
    def setUp(self):
        self.pin = sign.fingerprints(ROOT)
        self.app = Path('ClickPulse.app')

    def test_preview_without_identity_is_ad_hoc(self):
        with patch('macos_signing.subprocess.run') as run:
            self.assertEqual(sign.sign_app(self.app, ROOT, {}), {'signing': 'ad-hoc'})
            args = run.call_args_list[0].args[0]
            self.assertIn('--timestamp=none', args)
            self.assertEqual(args[args.index('--sign')+1], '-')

    def test_release_binds_to_leaf_and_verifies_before_reporting_signing(self):
        env = {'MACOS_SIGNING_IDENTITY': self.pin['sha1'], 'MACOS_SIGNING_KEYCHAIN': '/tmp/test'}
        with patch('macos_signing.subprocess.run') as run, patch.object(sign, 'verify_app') as verify:
            self.assertEqual(sign.sign_app(self.app, ROOT, env),
                             {'signing': 'self-signed','certificate_sha256': self.pin['sha256']})
            args = run.call_args.args[0]
            self.assertIn('--timestamp=none', args)
            self.assertEqual(args[args.index('--keychain')+1], '/tmp/test')
            self.assertEqual(args[args.index('--sign')+1], self.pin['sha1'])
            self.assertEqual(args[args.index('-r')+1], '=designated => '+sign.requirement(self.pin))
            verify.assert_called_once_with(self.app, self.pin)

    def test_missing_release_credentials_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(sign, 'security') as security:
            for env in ({},{'MACOS_SIGNING_P12_PASSWORD':'fixture'},{'MACOS_SIGNING_P12_BASE64':'AA=='}):
                with self.assertRaisesRegex(ValueError, 'secrets'):
                    sign.import_identity(Path(tmp)/'unused', env, self.pin)
            security.assert_not_called()
        with self.assertRaisesRegex(ValueError, 'identity'):
            sign.sign_app(self.app, ROOT, {'MACOS_REQUIRE_RELEASE_SIGNING':'1'})

    def test_verifier_requires_exact_dr_and_independent_sha256_of_embedded_leaf(self):
        cert = b'public fixture'
        pin = {**self.pin, 'sha256':hashlib.sha256(cert).hexdigest()}
        dr = sign.requirement(pin)
        for expression,leaf,expected in [(dr,cert,None), (dr+' or true',cert,'requirement'),
                                         ('identifier "app.clickpulse.desktop"',cert,'requirement'),
                                         (dr,b'wrong','certificate')]:
            def codesign(argv, **_):
                if '-r-' in argv:
                    return subprocess.CompletedProcess(argv,0,'','designated => '+expression+'\n')
                for arg in argv:
                    if arg.startswith('--extract-certificates='):
                        Path(arg.split('=',1)[1]+'0').write_bytes(leaf)
                return subprocess.CompletedProcess(argv,0)
            with self.subTest(dr=expression,leaf=leaf), patch.object(sign.subprocess,'run',side_effect=codesign):
                if expected:
                    with self.assertRaisesRegex(ValueError,expected):
                        sign.verify_app(self.app,pin)
                else:
                    sign.verify_app(self.app,pin)

    def test_verifier_rejects_ad_hoc_manifest(self):
        with self.assertRaisesRegex(ValueError,'self-signed'):
            sign.verify_packages(None,None,'fixture',{'signing':'ad-hoc'},self.pin)
