"""Reject wrong-platform binaries before assembling Linux packages."""
import struct
import sys
from pathlib import Path
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parent.parent/'scripts'))
import package_linux
import build

class LinuxPackagingTests(unittest.TestCase):
    def test_elf_architecture_class_endianness_and_type(self):
        for target,machine in [('aarch64-unknown-linux-gnu',183),('x86_64-unknown-linux-gnu',62)]:
            data=bytearray(64);data[:6]=b'\x7fELF\x02\x01';struct.pack_into('<HH',data,16,3,machine)
            package_linux.validate_elf(data,target)
            for offset,value in [(0,0),(4,1),(5,2),(16,1),(18,0)]:
                bad=data.copy();bad[offset]=value
                with self.assertRaises(ValueError):package_linux.validate_elf(bad,target)
            with self.assertRaises(ValueError):package_linux.validate_elf(data[:20],target)
    def test_linux_build_uses_native_packager_and_preserves_explicit_target(self):
        with patch.object(sys,'argv',['build.py','--platform','linux','--target','aarch64-unknown-linux-gnu','--skip-build']),patch('build.subprocess.run') as run:
            build.main();command=run.call_args.args[0]
            self.assertEqual(Path(command[1]), Path(package_linux.__file__).resolve())
            self.assertIn('aarch64-unknown-linux-gnu',command);self.assertIn('--skip-build',command)
