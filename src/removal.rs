//! Explicit removal plans never traverse links or delete unrelated program files.
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Friendly paths also work with Windows PowerShell 5.1 / .NET Framework,
/// whose path provider does not accept Rust's canonical verbatim prefix.
#[cfg(any(windows, test))]
fn readable_windows_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else if let Some(drive) = text.strip_prefix(r"\\?\")
        && drive.as_bytes().get(1..3) == Some(b":\\")
        && drive.as_bytes()[0].is_ascii_alphabetic()
    {
        drive.to_owned()
    } else {
        text.into_owned()
    }
}

pub(crate) fn plain(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "Refusing linked removal path: {}",
        path.display()
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "Refusing reparse point: {}",
            path.display()
        );
    }
    Ok(metadata)
}

/// Preflight the complete tree before deleting any part of an owned profile.
pub fn profile_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    if !root.try_exists()? {
        return Ok(result);
    }
    ensure!(
        root.is_absolute() && root.parent().is_some(),
        "Invalid profile directory"
    );
    fn visit(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let meta = plain(path)?;
        if meta.is_dir() {
            for item in fs::read_dir(path)? {
                visit(&item?.path(), files)?;
            }
        } else {
            ensure!(meta.is_file(), "Unsupported data file");
        }
        files.push(path.to_owned());
        Ok(())
    }
    visit(root, &mut result)?;
    Ok(result)
}

/// Keep only the user's history, settings and profile; clear disposable data.
/// The held lock is handled by the caller after all other profile work.
pub fn profile_cleanup_files(root: &Path, clear: bool) -> Result<Vec<PathBuf>> {
    if clear {
        return profile_files(root);
    }
    if !root.try_exists()? {
        return Ok(Vec::new());
    }
    ensure!(
        root.is_absolute() && plain(root)?.is_dir(),
        "Invalid profile directory"
    );
    let mut files = Vec::new();
    for item in fs::read_dir(root)? {
        let path = item?.path();
        // Only known disposable caches/locks are removed. Unknown files may be
        // recovery backups or a user-selected database directory.
        if !matches!(
            path.file_name().and_then(|n| n.to_str()),
            Some(
                "webview"
                    | "updates"
                    | "logs"
                    | "events.lock"
                    | "events.sqlite3.lock"
                    | "source-view.lock"
            )
        ) {
            continue;
        }
        files.extend(
            profile_files(&path).with_context(|| format!("Data remains at {}", path.display()))?,
        );
    }
    Ok(files)
}

fn missing(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<std::io::Error>()
        .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
}
fn delete_one(path: &Path) -> Result<()> {
    let result = (|| {
        if plain(path)?.is_dir() {
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    })();
    match result {
        Err(error) if missing(&error) => Ok(()),
        other => other,
    }
}
fn retry_cache_removal(
    allow_retry: bool,
    mut remove: impl FnMut() -> Result<()>,
    mut elapsed: impl FnMut() -> std::time::Duration,
    mut wait: impl FnMut(std::time::Duration),
) -> Result<()> {
    use std::time::Duration;
    loop {
        let error = match remove() {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };
        let retryable = error
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| matches!(error.raw_os_error(), Some(5 | 32)));
        if !allow_retry || !retryable {
            return Err(error);
        }
        let remaining = Duration::from_secs(10).saturating_sub(elapsed());
        if remaining.is_zero() {
            return Err(error);
        }
        wait(remaining.min(Duration::from_millis(200)));
    }
}

fn delete_profile_with(
    profile: &Path,
    files: &[PathBuf],
    mut remove: impl FnMut(&Path, bool) -> Result<()>,
) -> Result<()> {
    for path in files {
        ensure!(
            path.starts_with(profile),
            "Data removal escaped the profile"
        );
        if let Err(error) = plain(path)
            && !missing(&error)
        {
            return Err(error).with_context(|| format!("Data remains at {}", path.display()));
        }
    }
    let mut failures = Vec::new();
    for path in files {
        let cache = path
            .strip_prefix(profile)?
            .components()
            .next()
            .is_some_and(|part| part.as_os_str() == "webview" || part.as_os_str() == "updates");
        if let Err(error) = remove(path, cache) {
            failures.push(format!("{}: {error:#}", path.display()));
        }
    }
    ensure!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

pub fn delete_profile_files(profile: &Path, files: &[PathBuf]) -> Result<()> {
    // Share a ten-second grace period across locked cache files, starting at
    // the first sharing/access error, not while deleting unrelated history.
    let started = std::cell::Cell::new(None);
    delete_profile_with(profile, files, |path, retry| {
        retry_cache_removal(
            retry,
            || delete_one(path),
            || {
                let now = std::time::Instant::now();
                let first = started.get().unwrap_or_else(|| {
                    started.set(Some(now));
                    now
                });
                now.duration_since(first)
            },
            std::thread::sleep,
        )
    })
}

/// Any data-phase failure leaves every program file available for another try.
pub fn data_before_program(
    data: impl FnOnce() -> Result<()>,
    program: impl FnOnce() -> Result<()>,
) -> Result<()> {
    data()?;
    program()
}

/// Only files listed by the shipped package may be removed. Changed and unknown
/// files stay in place; a malformed receipt aborts the operation.
pub fn program_files(root: &Path) -> Result<Vec<PathBuf>> {
    let current = package_files(root)?;
    let mut backups = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !name.starts_with(".ClickPulse-before-pending-")
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        {
            continue;
        }
        ensure!(plain(&path)?.is_dir(), "Invalid update backup directory");
        if fs::read_dir(&path)?.next().is_none() {
            // A fully rolled-back update may leave an empty owned directory.
            // remove_dir will still refuse any file added after this check.
            backups.push(path);
            continue;
        }
        let files = package_files(&path)
            .with_context(|| format!("Update backup remains at {}", path.display()))?;
        // Refuse an unrecognized file in a purported backup before uninstalling
        // the current program. Do not recursively remove arbitrary directories.
        let tree = profile_files(&path)?;
        ensure!(
            tree.iter()
                .all(|p| p == &path || p == &path.join("Licenses") || files.contains(p)),
            "Unrecognized files in backup {}",
            path.display()
        );
        backups.extend(files);
        if path.join("Licenses").exists() {
            backups.push(path.join("Licenses"));
        }
        backups.push(path);
    }
    backups.extend(current);
    Ok(backups)
}

fn package_files(root: &Path) -> Result<Vec<PathBuf>> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    ensure!(plain(root)?.is_dir(), "Invalid application directory");
    let manifest = root.join("build-manifest.json");
    ensure!(
        plain(&manifest)?.is_file() && fs::metadata(&manifest)?.len() <= 1024 * 1024,
        "Invalid package receipt"
    );
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&manifest)?)?;
    let files = value["files"]
        .as_object()
        .context("Missing package files")?;
    ensure!(
        files.contains_key("ClickPulse.exe") && files.contains_key("WebView2Loader.dll"),
        "Not a ClickPulse Windows package"
    );
    let mut result = Vec::new();
    for (name, expected) in files {
        let allowed = matches!(
            name.as_str(),
            "ClickPulse.exe"
                | "WebView2Loader.dll"
                | "README.txt"
                | "LICENSE.txt"
                | "THIRD-PARTY-NOTICES.txt"
        ) || name.strip_prefix("Licenses/").is_some_and(|leaf| {
            !leaf.is_empty()
                && leaf.ends_with(".txt")
                && leaf
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        });
        ensure!(allowed, "Unexpected package path: {name}");
        let path = root.join(name);
        if !path.try_exists()? {
            continue;
        }
        if name.starts_with("Licenses/") {
            ensure!(
                plain(&root.join("Licenses"))?.is_dir(),
                "Invalid license directory"
            );
        }
        ensure!(plain(&path)?.is_file(), "Invalid program file");
        let mut hash = Sha256::new();
        let mut input = fs::File::open(&path)?;
        let mut buffer = [0u8; 65536];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        ensure!(
            expected.as_str() == Some(format!("{:x}", hash.finalize()).as_str()),
            "Program file changed; no files removed: {name}"
        );
        result.push(path);
    }
    // Keep the executable and receipt available until the other files succeed.
    result.sort_by_key(|p| p.file_name().is_some_and(|n| n == "ClickPulse.exe"));
    result.push(manifest);
    Ok(result)
}

pub fn delete_files(files: &[PathBuf]) -> Result<()> {
    // Check again before the first mutation. Never follow a late link replacement.
    for path in files {
        plain(path).with_context(|| format!("Unable to inspect {}", path.display()))?;
    }
    for path in files {
        delete_one(path).with_context(|| format!("File remains at {}", path.display()))?;
    }
    Ok(())
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{launch, run};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_paths_are_readable_without_changing_ordinary_or_device_paths() {
        for (input, expected) in [
            (r"\\?\C:\Users\测试\ClickPulse", r"C:\Users\测试\ClickPulse"),
            (
                r"\\?\UNC\server\share\ClickPulse",
                r"\\server\share\ClickPulse",
            ),
            (r"C:\ClickPulse", r"C:\ClickPulse"),
            (r"\\?\GLOBALROOT\Device", r"\\?\GLOBALROOT\Device"),
        ] {
            assert_eq!(readable_windows_path(Path::new(input)), expected);
        }
    }
    #[test]
    fn profile_removal_is_explicit_and_keeps_external_history() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("ClickPulse");
        fs::create_dir_all(root.join("webview/cache")).unwrap();
        let custom = temp.path().join("external.sqlite3");
        fs::write(&custom, b"history").unwrap();
        fs::write(
            root.join("storage.json"),
            custom.to_string_lossy().as_bytes(),
        )
        .unwrap();
        fs::write(root.join("webview/cache/data"), b"cache").unwrap();
        let plan = profile_files(&root).unwrap();
        assert!(root.join("storage.json").exists()); // Planning/cancel/keep do not delete.
        delete_files(&plan).unwrap();
        assert!(!root.exists());
        assert_eq!(fs::read(custom).unwrap(), b"history");
    }
    #[test]
    fn keeping_data_clears_caches_but_retains_history_settings_and_profile() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("ClickPulse");
        fs::create_dir_all(root.join("profile/nested")).unwrap();
        let kept = [
            "events.sqlite3",
            "events.sqlite3-wal",
            "events.sqlite3-shm",
            "storage.json",
            "presentation.json",
            "profile/nested/settings",
            "profile.lock",
            "events.sqlite3-journal",
            "events.sqlite3.damaged-20261119",
            "storage.json.corrupt-20261119",
            "presentation.corrupt-20261119.json",
            "custom/events.sqlite3",
            "custom/events.sqlite3-wal",
            "my-backup.sqlite3",
        ];
        fs::create_dir_all(root.join("custom")).unwrap();
        for name in kept {
            fs::write(root.join(name), name).unwrap();
        }
        for dir in ["webview", "updates", "logs"] {
            fs::create_dir_all(root.join(dir).join("nested")).unwrap();
            fs::write(root.join(dir).join("nested/file"), b"cache").unwrap();
        }
        fs::write(root.join("events.lock"), b"stale lock").unwrap();
        let plan = profile_cleanup_files(&root, false).unwrap();
        delete_profile_files(&root, &plan).unwrap();
        for name in kept {
            assert_eq!(fs::read(root.join(name)).unwrap(), name.as_bytes());
        }
        for name in ["webview", "updates", "logs", "events.lock"] {
            assert!(!root.join(name).exists());
        }
        assert!(root.is_dir());
    }

    #[test]
    fn cache_cleanup_retries_sharing_and_access_errors_with_a_bounded_delay() {
        use std::{cell::Cell, time::Duration};
        for code in [5, 32] {
            let elapsed = Cell::new(Duration::ZERO);
            let attempts = Cell::new(0);
            retry_cache_removal(
                true,
                || {
                    attempts.set(attempts.get() + 1);
                    if attempts.get() < 4 {
                        Err(std::io::Error::from_raw_os_error(code).into())
                    } else {
                        Ok(())
                    }
                },
                || elapsed.get(),
                |delay| {
                    assert_eq!(delay, Duration::from_millis(200));
                    elapsed.set(elapsed.get() + delay);
                },
            )
            .unwrap();
            assert_eq!(attempts.get(), 4);
            assert_eq!(elapsed.get(), Duration::from_millis(600));
        }
        let elapsed = Cell::new(Duration::ZERO);
        let attempts = Cell::new(0);
        assert!(
            retry_cache_removal(
                true,
                || {
                    attempts.set(attempts.get() + 1);
                    Err(std::io::Error::from_raw_os_error(32).into())
                },
                || elapsed.get(),
                |delay| elapsed.set(elapsed.get() + delay)
            )
            .is_err()
        );
        assert_eq!(elapsed.get(), Duration::from_secs(10));
        assert_eq!(attempts.get(), 51);
        for (cache, code) in [(false, 32), (false, 5), (true, 112)] {
            assert!(
                retry_cache_removal(
                    cache,
                    || Err(std::io::Error::from_raw_os_error(code).into()),
                    || Duration::ZERO,
                    |_| panic!("Do not retry unrelated errors")
                )
                .is_err()
            );
        }
    }

    #[test]
    fn failed_cache_cleanup_lists_paths_and_keeps_program_for_retry() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("ClickPulse");
        fs::create_dir_all(root.join("webview")).unwrap();
        fs::create_dir_all(root.join("updates")).unwrap();
        let files = [root.join("webview/held"), root.join("updates/held")];
        for file in &files {
            fs::write(file, b"held").unwrap();
        }
        let program = temp.path().join("ClickPulse.exe");
        fs::write(&program, b"program").unwrap();
        let error = data_before_program(
            || {
                delete_profile_with(&root, &files, |_, cache| {
                    assert!(cache);
                    Err(std::io::Error::from_raw_os_error(32).into())
                })
            },
            || {
                fs::remove_file(&program)?;
                Ok(())
            },
        )
        .unwrap_err();
        for file in &files {
            assert!(error.to_string().contains(&file.display().to_string()));
        }
        assert_eq!(fs::read(&program).unwrap(), b"program");
        data_before_program(
            || delete_profile_files(&root, &files),
            || {
                assert!(files.iter().all(|p| !p.exists()));
                fs::remove_file(&program)?;
                Ok(())
            },
        )
        .unwrap();
        assert!(!program.exists());
    }

    #[test]
    fn cleanup_tolerates_files_already_released_and_rejects_external_paths() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("ClickPulse");
        fs::create_dir_all(root.join("updates")).unwrap();
        let file = root.join("updates/stale");
        fs::write(&file, b"stale").unwrap();
        let plan = profile_cleanup_files(&root, false).unwrap();
        fs::remove_file(&file).unwrap();
        delete_profile_files(&root, &plan).unwrap();
        let outside = temp.path().join("outside");
        fs::write(&outside, b"keep").unwrap();
        assert!(delete_profile_files(&root, std::slice::from_ref(&outside)).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"keep");
    }

    #[cfg(unix)]
    #[test]
    fn profile_link_aborts_before_any_deletion() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("profile");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("keep"), b"history").unwrap();
        std::os::unix::fs::symlink(temp.path(), root.join("link")).unwrap();
        assert!(profile_files(&root).is_err());
        assert!(root.join("keep").exists());
    }
    #[test]
    fn uninstall_includes_verified_update_backups_before_current_program() {
        use sha2::{Digest, Sha256};
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let backup = root.join(".ClickPulse-before-pending-test");
        fs::create_dir(&backup).unwrap();
        for folder in [root, backup.as_path()] {
            let mut files = serde_json::Map::new();
            for name in ["ClickPulse.exe", "WebView2Loader.dll"] {
                fs::write(folder.join(name), name).unwrap();
                files.insert(
                    name.into(),
                    format!("{:x}", Sha256::digest(name.as_bytes())).into(),
                );
            }
            fs::write(
                folder.join("build-manifest.json"),
                serde_json::to_vec(&serde_json::json!({"files": files})).unwrap(),
            )
            .unwrap();
        }
        fs::write(root.join("personal.txt"), "keep").unwrap();
        fs::write(backup.join("personal.txt"), "keep too").unwrap();
        assert!(program_files(root).is_err());
        assert!(root.join("ClickPulse.exe").exists());
        fs::remove_file(backup.join("personal.txt")).unwrap();
        let empty = root.join(".ClickPulse-before-pending-rolled-back");
        fs::create_dir(&empty).unwrap();
        let plan = program_files(root).unwrap();
        assert!(plan.contains(&empty));
        assert!(
            plan.iter().position(|p| p == &backup).unwrap()
                < plan
                    .iter()
                    .position(|p| p == &root.join("ClickPulse.exe"))
                    .unwrap()
        );
        delete_files(&plan).unwrap();
        assert!(!backup.exists());
        assert!(!empty.exists());
        assert_eq!(fs::read(root.join("personal.txt")).unwrap(), b"keep");
    }
    #[test]
    fn program_plan_rejects_escape_and_preserves_unknown_files() {
        use sha2::{Digest, Sha256};
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let mut files = serde_json::Map::new();
        for name in ["ClickPulse.exe", "WebView2Loader.dll"] {
            fs::write(root.join(name), name).unwrap();
            files.insert(
                name.into(),
                format!("{:x}", Sha256::digest(name.as_bytes())).into(),
            );
        }
        fs::write(root.join("personal.txt"), b"keep").unwrap();
        let receipt = root.join("build-manifest.json");
        let write = |f: &serde_json::Map<String, serde_json::Value>| {
            fs::write(
                &receipt,
                serde_json::to_vec(&serde_json::json!({"files":f})).unwrap(),
            )
            .unwrap()
        };
        files.insert("../outside.txt".into(), "bad".into());
        write(&files);
        assert!(program_files(root).is_err());
        assert!(root.join("ClickPulse.exe").exists());
        files.remove("../outside.txt");
        write(&files);
        fs::write(root.join("ClickPulse.exe"), b"changed").unwrap();
        assert!(program_files(root).is_err());
        fs::write(root.join("ClickPulse.exe"), "ClickPulse.exe").unwrap();
        delete_files(&program_files(root).unwrap()).unwrap();
        assert!(root.join("personal.txt").exists());
    }
}
