"""Compile a per-user installer from the same allowlisted portable payload."""
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess


def compiler_path():
    found = shutil.which('makensis')
    if found:
        return Path(found)
    for variable in ['ProgramFiles(x86)', 'ProgramFiles']:
        base = os.environ.get(variable)
        if base and (path := Path(base)/'NSIS/makensis.exe').is_file():
            return path
    raise RuntimeError('NSIS is required for Setup.exe. Install NSIS from https://nsis.sourceforge.io/ or use --portable-only.')


def payload_paths(bundle):
    paths = []
    for file in sorted(bundle.rglob('*')):
        if file.is_symlink():
            raise ValueError(f'Unexpected payload symlink: {file.name}')
        if not file.is_file():
            continue
        relative = file.relative_to(bundle)
        if any(not re.fullmatch(r'[A-Za-z0-9_.-]+', part) for part in relative.parts):
            raise ValueError(f'Unsafe installer path: {relative}')
        if file.suffix.lower() in {'.db', '.sqlite', '.sqlite3', '.log'} or file.name.endswith(('-wal', '-shm')):
            raise ValueError(f'Private data is not an installation payload: {relative}')
        if re.search(r'\.log\.\d+$', file.name, re.IGNORECASE):
            raise ValueError(f'Rotated logs are not an installation payload: {relative}')
        paths.append(relative)
    if Path('ClickPulse.exe') not in paths or Path('WebView2Loader.dll') not in paths:
        raise ValueError('Installer requires the executable and adjacent WebView2 loader.')
    return paths


def uninstall_include(bundle):
    paths = payload_paths(bundle)
    lines = ['; Generated from explicit release files; never enumerate user statistics.']
    for relative in paths:
        lines.append('Delete "$INSTDIR\\'+str(relative).replace('/', '\\')+'"')
    directories = {parent for relative in paths for parent in relative.parents if parent != Path('.')}
    for relative in sorted(directories, key=lambda path: len(path.parts), reverse=True):
        # RMDir without /r removes only empty directories.
        lines.append('RMDir "$INSTDIR\\'+str(relative).replace('/', '\\')+'"')
    return '\n'.join(lines)+'\n'


def build_installer(root, bundle, version, internal_build, output, compiler=None):
    compiler = compiler or compiler_path()
    payload_paths(bundle)
    include = bundle.parent/'uninstall-files.nsh'
    include.write_text(uninstall_include(bundle), encoding='utf-8')
    numeric = '.'.join(version.split('.')+[str(internal_build)])
    staging_output = bundle.parent/'ClickPulse-Setup.exe'
    flags = [
        f'-DPAYLOAD={bundle}', f'-DOUTPUT={staging_output}', f'-DPUBLIC_VERSION={version}',
        f'-DFILE_VERSION={version}.{internal_build}', f'-DNUMERIC_VERSION={numeric}',
        f'-DICON_PATH={root}/assets/windows/ClickPulse.ico', f'-DUNINSTALL_FILES={include}',
    ]
    if os.name == 'nt':
        flags = ['/'+flag[1:] for flag in flags]
    subprocess.run([str(compiler), *flags, str(root/'assets/windows/installer.nsi')], cwd=root, check=True)
    os.replace(staging_output, output)
    sha = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix('.exe.sha256').write_text(sha+'  '+output.name+'\n')
    return sha
