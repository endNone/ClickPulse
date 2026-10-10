"""Collect dependency license text from the resolved platform-specific Cargo graph."""
from pathlib import Path


def rust_notices(metadata, root):
    packages = {package['id']: package for package in metadata['packages']}
    nodes = {node['id']: node for node in metadata['resolve']['nodes']}
    pending = [metadata['resolve']['root']]
    active = set()
    while pending:
        identity = pending.pop()
        if identity not in active:
            active.add(identity)
            pending.extend(nodes[identity]['dependencies'])
    notices = ['Rust standard library: MIT OR Apache-2.0; https://github.com/rust-lang/rust', '']
    for identity in sorted(active, key=lambda item: (packages[item]['name'], packages[item]['version'])):
        package = packages[identity]
        if package['source'] is None:
            continue
        folder = Path(package['manifest_path']).parent
        notices.extend([f"{package['name']} {package['version']}", f"License: {package['license'] or 'See package license'}", f"Source: {package['repository'] or package['source']}"])
        files = sorted(file for file in folder.iterdir() if file.is_file() and file.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'UNLICENSE', 'COPYRIGHT')))
        if package.get('license_file'):
            license_file = folder/package['license_file']
            if license_file.is_file() and license_file not in files:
                files.append(license_file)
        for file in files:
            notices.extend(['', f'--- {file.name} ---', file.read_text(encoding='utf-8', errors='replace')])
        if not files and package['name'].startswith('webview2-'):
            notices.append((root/'assets/windows/WebView2-Rust-LICENSE.txt').read_text(encoding='utf-8'))
        notices.append('\n'+'='*72+'\n')
    return '\n'.join(notices)
