#!/usr/bin/env python3
"""One source-build entry point; creates packages without installing or launching."""
import argparse
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=['auto', 'macos', 'windows'], default='auto')
    parser.add_argument('--target', help='Rust target for the selected platform; macOS supports aarch64-apple-darwin and x86_64-apple-darwin')
    parser.add_argument('--portable-only', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--skip-build', action='store_true', help='Package an already built matching release binary')
    args = parser.parse_args()
    platform = args.platform
    if platform == 'auto':
        platform = {'darwin': 'macos', 'win32': 'windows'}.get(sys.platform)
    if platform is None:
        parser.error('Desktop packages require macOS or Windows; choose --platform windows for a configured cross-build.')
    root = Path(__file__).resolve().parent.parent
    if platform == 'macos':
        if args.portable_only:
            parser.error('--portable-only applies to Windows packages.')
        command = [sys.executable, str(root/'scripts/package_macos.py')]
        if args.target:
            if args.target not in {'aarch64-apple-darwin', 'x86_64-apple-darwin'}:
                parser.error('Select a macOS Rust target for --platform macos.')
            command.extend(['--target', args.target])
    else:
        target = args.target or ('x86_64-pc-windows-msvc' if sys.platform == 'win32' else 'x86_64-pc-windows-gnu')
        if target not in {'x86_64-pc-windows-msvc', 'x86_64-pc-windows-gnu'}:
            parser.error('Select a Windows Rust target for --platform windows.')
        command = [sys.executable, str(root/'scripts/package_windows.py'), '--target', target]
    if args.skip_build:
        command.append('--skip-build')
    try:
        subprocess.run(command, cwd=root, check=True)
    except (OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'Build failed: {error}\nSee docs/development.md.\n')


if __name__ == '__main__':
    main()
