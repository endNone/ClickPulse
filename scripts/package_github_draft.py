"""Prepare source archives for a repository draft or release; never publish."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
import tomllib
import zipfile

ROOTS = [
    ".gitattributes", ".gitignore", "Cargo.toml", "Cargo.lock", "build.rs",
    "README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md",
    "LICENSE",
]
FOLDERS = [".github", "assets", "docs", "scripts", "src", "tests", "ui"]
EXCLUDED = {".DS_Store", "__pycache__", ".Codex", ".codex", ".git", "target", "dist", "trace.md"}


def source_files(root):
    root = root.resolve()
    files = [root / file for file in ROOTS]
    for folder in FOLDERS:
        files.extend(file for file in (root / folder).rglob("*") if file.is_file())
    selected = []
    for file in sorted(set(files)):
        relative = file.relative_to(root)
        if any(part in EXCLUDED for part in relative.parts):
            continue
        if file.name.startswith(".env") or file.suffix.lower() in {".log", ".pyc", ".db", ".sqlite", ".sqlite3"}:
            continue
        if re.search(r'\.log\.\d+$', file.name, re.IGNORECASE):
            continue
        if file.name.endswith(("-wal", "-shm")):
            continue
        if file.is_symlink() or not file.resolve().is_relative_to(root):
            raise ValueError(f"Unexpected source symlink: {relative}")
        selected.append(file)
    return selected


def package_source(root, release=False, tag=None, revision=None):
    root = root.resolve()
    package = tomllib.loads((root / "Cargo.toml").read_text(encoding='utf-8'))["package"]
    version = package["version"]
    if release and (tag != "v"+version or not revision or not re.fullmatch(r'[0-9a-f]{40}', revision)):
        raise ValueError('Release source requires a matching version tag and full source revision.')
    name = f"ClickPulse-{version}-source" + ("" if release else "-draft")
    selected = source_files(root)
    manifest = {
        "version": version,
        "code_license": package["license"],
        "files": {
            file.relative_to(root).as_posix(): hashlib.sha256(file.read_bytes()).hexdigest()
            for file in selected
        },
    }
    if release:
        manifest.update(tag=tag, revision=revision)
    else:
        manifest.update(draft=True, published=False, code_license_pending=False)
    manifest_name = "source-manifest.json" if release else "draft-manifest.json"
    dist = root / "dist"
    dist.mkdir(exist_ok=True)
    archive = dist / (name + ".zip")
    with tempfile.NamedTemporaryFile(dir=dist, prefix='.source-', suffix='.zip', delete=False) as temporary:
        temporary_path = Path(temporary.name)
    try:
        with zipfile.ZipFile(temporary_path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as output:
            for file in selected:
                output.write(file, f"{name}/{file.relative_to(root).as_posix()}")
            output.writestr(f"{name}/{manifest_name}", json.dumps(manifest, indent=2) + "\n")
        os.replace(temporary_path, archive)
    finally:
        temporary_path.unlink(missing_ok=True)
    sha = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix(".zip.sha256").write_text(f"{sha}  {archive.name}\n")
    return archive, len(selected), sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true', help='Prepare corresponding source for a release candidate')
    parser.add_argument('--tag')
    parser.add_argument('--revision')
    args = parser.parse_args()
    if not args.release and (args.tag or args.revision):
        parser.error('--tag and --revision apply to --release archives.')
    try:
        archive, count, sha = package_source(Path(__file__).resolve().parent.parent, args.release, args.tag, args.revision)
    except (ValueError, OSError, KeyError) as error:
        parser.exit(1, f'Source preparation failed: {error}\n')
    print(f"Source archive: {archive}")
    print(f"Files: {count}; SHA256: {sha}")


if __name__ == '__main__':
    main()
