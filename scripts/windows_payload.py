"""Validate portable Windows payload boundaries before archiving."""
from pathlib import Path
import re


def payload_paths(bundle):
    paths = []
    for file in sorted(bundle.rglob('*')):
        if file.is_symlink():
            raise ValueError(f'Unexpected payload symlink: {file.name}')
        if not file.is_file():
            continue
        relative = file.relative_to(bundle)
        if any(not re.fullmatch(r'[A-Za-z0-9_.-]+', part) for part in relative.parts):
            raise ValueError(f'Unsafe portable path: {relative}')
        if file.suffix.lower() in {'.db', '.sqlite', '.sqlite3', '.log'} or file.name.endswith(('-wal', '-shm')):
            raise ValueError(f'Private data is not a portable payload: {relative}')
        if re.search(r'\.log\.\d+$', file.name, re.IGNORECASE):
            raise ValueError(f'Rotated logs are not a portable payload: {relative}')
        paths.append(relative)
    if Path('ClickPulse.exe') not in paths or Path('WebView2Loader.dll') not in paths:
        raise ValueError('Portable package requires the executable and adjacent WebView2 loader.')
    return paths
