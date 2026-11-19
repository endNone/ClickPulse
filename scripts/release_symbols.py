"""Keep matching debug information outside user packages before stripping copies."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def require_symbol_tools(target, environment=None):
    prefix = 'x86_64-w64-mingw32-' if target.endswith('windows-gnu') else ''
    tools = ['dsymutil', 'strip'] if 'apple' in target else [] if target.endswith('msvc') else [prefix+'objcopy', prefix+'strip']
    search_path = (os.environ if environment is None else environment).get('PATH')
    missing = [name for name in tools if shutil.which(name, path=search_path) is None]
    if missing:
        raise RuntimeError('Missing debug-symbol tools: '+', '.join(missing)+'. Install the target toolchain before packaging.')


def build_release(root, target, environment=None):
    require_symbol_tools(target, environment)
    env = dict(os.environ if environment is None else environment)
    # Keep the linker's original output intact in Cargo's cache. Strip only the
    # staged copy, otherwise --skip-build/repeated packaging loses the symbols.
    env.update(CARGO_PROFILE_RELEASE_DEBUG='line-tables-only',
               CARGO_PROFILE_RELEASE_STRIP='none',
               CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO='packed' if 'apple' in target or 'msvc' in target else 'off')
    subprocess.run(['cargo', 'build', '--release', '--locked', '--target', target],
                   cwd=root, env=env, check=True)


def stage_binary(source, destination, root, target, version, built):
    source, destination, root = Path(source), Path(destination), Path(root)
    if built:
        require_symbol_tools(target)
    shutil.copy2(source, destination)
    if not built:
        # --skip-build keeps its explicit contract: package the given binary.
        # Debug-symbol coverage is guaranteed only by a normal packaging build.
        return
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    directory = root/'dist'/'symbols'/f'ClickPulse-{version}-{target}'/digest[:16]
    directory.mkdir(parents=True, exist_ok=True)
    if 'apple' in target:
        symbols = directory/'clickpulse.dSYM'
        generated = source.with_suffix('.dSYM')
        if generated.is_dir():
            shutil.copytree(generated, symbols, dirs_exist_ok=True)
        else:
            subprocess.run(['dsymutil', str(source), '-o', str(symbols)], check=True)
        subprocess.run(['strip', '-S', '-x', str(destination)], check=True)
    elif target.endswith('msvc'):
        # MSVC stores debug information in the PDB, not in executable sections.
        # Keep its CodeView identity in the PE so this exact PDB can be located.
        symbols = directory/'clickpulse.pdb'
        shutil.copy2(source.with_suffix('.pdb'), symbols)
    else:
        prefix = 'x86_64-w64-mingw32-' if target.endswith('windows-gnu') else ''
        symbols = directory/(source.name+'.debug')
        subprocess.run([prefix+'objcopy', '--only-keep-debug', str(source), str(symbols)], check=True)
        subprocess.run([prefix+'strip', '--strip-all', str(destination)], check=True)
        subprocess.run([prefix+'objcopy', '--add-gnu-debuglink='+str(symbols), str(destination)], check=True)
    files = {p.relative_to(directory).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in sorted(directory.rglob('*')) if p.is_file() and p.name != 'manifest.json'}
    if not files:
        raise ValueError('No separate debug symbols were generated')
    manifest = directory/'manifest.json'
    manifest.write_text(json.dumps({
        'target': target, 'version': version, 'unstripped_sha256': digest,
        'stripped_binary_sha256_before_signing': hashlib.sha256(destination.read_bytes()).hexdigest(),
        'packaged_binary_sha256': hashlib.sha256(destination.read_bytes()).hexdigest(),
        'files': files,
    }, indent=2)+'\n')
    return manifest


def record_packaged_binary(manifest, binary):
    if manifest is None:  # explicit --skip-build has no debug artifact
        return
    path = Path(manifest)
    data = json.loads(path.read_text(encoding='utf-8'))
    data['packaged_binary_sha256'] = hashlib.sha256(Path(binary).read_bytes()).hexdigest()
    path.write_text(json.dumps(data, indent=2)+'\n', encoding='utf-8')


def verify_release_symbols(symbols, packages, version):
    # Resolve the caller's root once: macOS /var is itself a system symlink.
    # Reject symlinks within the artifact, not trusted ancestors of its root.
    symbols = Path(symbols).resolve(strict=True)
    targets = {
        'aarch64-apple-darwin': ('macos-arm64', 'ClickPulse.app/Contents/MacOS/clickpulse'),
        'x86_64-apple-darwin': ('macos-x64', 'ClickPulse.app/Contents/MacOS/clickpulse'),
        'x86_64-pc-windows-msvc': ('windows-x64', 'ClickPulse.exe'),
        'x86_64-unknown-linux-gnu': ('linux-x64', 'clickpulse'),
        'aarch64-unknown-linux-gnu': ('linux-arm64', 'clickpulse'),
    }
    found = set()
    for path in symbols.rglob('manifest.json'):
        manifest = json.loads(path.read_text(encoding='utf-8'))
        target = manifest.get('target')
        if target not in targets or target in found or manifest.get('version') != version:
            raise ValueError('Unexpected or duplicate debug-symbol target/version')
        found.add(target)
        files = manifest.get('files', {})
        if not files: raise ValueError('Empty debug-symbol artifact')
        for name, expected in files.items():
            relative = Path(name)
            if relative.is_absolute() or '..' in relative.parts or '\\' in name:
                raise ValueError('Unsafe symbol path')
            file = path.parent/relative
            artifact_paths = [file, *file.parents]
            if any(p.is_symlink() for p in artifact_paths if p.is_relative_to(symbols)) or hashlib.sha256(file.read_bytes()).hexdigest() != expected:
                raise ValueError('Debug-symbol file changed')
        platform, binary = targets[target]
        package = json.loads((Path(packages)/f'ClickPulse-{version}-{platform}.manifest.json').read_text(encoding='utf-8'))
        if package.get('target') != target or package.get('version') != version or package['files'].get(binary) != manifest.get('packaged_binary_sha256'):
            raise ValueError('Debug symbols do not match the packaged executable')
    if found != set(targets): raise ValueError('Missing platform debug symbols')


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='Verify separate symbols against all five release packages')
    parser.add_argument('--symbols', type=Path, required=True)
    parser.add_argument('--packages', type=Path, required=True)
    parser.add_argument('--version', required=True)
    args = parser.parse_args()
    verify_release_symbols(args.symbols, args.packages, args.version)
