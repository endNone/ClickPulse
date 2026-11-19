//! Cleanup receipts name exact files and hashes, never a prefix-only recursive delete.
use super::*;

#[derive(Serialize, Deserialize)]
struct Receipt {
    installation: Installation,
    installed_hash: String,
    backup: BTreeMap<String, Option<String>>,
    staged: BTreeMap<String, Option<String>>,
}
const RECEIPT: &str = "cleanup.json";
fn nonce(directory: &Path) -> Result<&str> {
    let value = directory
        .file_name()
        .and_then(|v| v.to_str())
        .context("Missing cleanup nonce")?;
    ensure!(
        value.starts_with("pending-")
            && value.len() > 8
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "Invalid cleanup nonce"
    );
    Ok(value)
}
fn backup_path(install: &Installation, directory: &Path) -> Result<PathBuf> {
    let root = if install.target.mac() {
        install.root.parent().context("Missing App parent")?
    } else {
        &install.root
    };
    Ok(root.join(format!(".ClickPulse-before-{}", nonce(directory)?)))
}
// Windows enforces the byte-range lock even against other handles in this process.
// The staging lock is created empty. Verify that invariant without reading it, and
// keep the receipt compatible with older versions that hashed the empty file.
fn verify_empty_lock(path: &Path) -> Result<()> {
    let metadata = crate::removal::plain(path)?;
    ensure!(
        metadata.is_file() && metadata.len() == 0,
        "Invalid update lock"
    );
    Ok(())
}

fn inventory(root: &Path, skip_receipt: bool) -> Result<BTreeMap<String, Option<String>>> {
    let mut result = BTreeMap::new();
    for path in crate::removal::profile_files(root)? {
        if path == root {
            continue;
        }
        let name = path
            .strip_prefix(root)?
            .to_str()
            .context("Invalid cleanup filename")?
            .replace('\\', "/");
        safe_relative(&name)?;
        if skip_receipt && name == RECEIPT {
            continue;
        }
        ensure!(
            result.len() < MAX_FILES * 3 + 256,
            "Too many cleanup entries"
        );
        let hash = if skip_receipt && name == "running.lock" {
            verify_empty_lock(&path)?;
            Some(hash_bytes(&[]))
        } else if path.is_file() {
            Some(hash_file(&path)?)
        } else {
            None
        };
        result.insert(name, hash);
    }
    Ok(result)
}
pub(super) fn record(
    directory: &Path,
    installation: &Installation,
    backup: Option<&Path>,
) -> Result<()> {
    let expected = backup_path(installation, directory)?;
    ensure!(
        backup.is_none_or(|p| p == expected),
        "Unexpected backup location"
    );
    let receipt = Receipt {
        installation: installation.clone(),
        installed_hash: hash_file(&installation.executable())?,
        backup: match backup {
            Some(path) => inventory(path, false)?,
            None => BTreeMap::new(),
        },
        staged: inventory(directory, true)?,
    };
    let mut file = tempfile::NamedTempFile::new_in(directory)?;
    file.write_all(&serde_json::to_vec(&receipt)?)?;
    file.as_file().sync_all()?;
    file.persist(directory.join(RECEIPT))?;
    Ok(())
}
fn purge(
    root: &Path,
    expected: &BTreeMap<String, Option<String>>,
    receipt: bool,
    lock: Option<File>,
) -> Result<()> {
    if !root.try_exists()? {
        return Ok(());
    }
    let actual = inventory(root, receipt)?;
    // A previous attempt may already have removed unlocked files. Newly added,
    // replaced or linked files stop cleanup before any further deletion.
    ensure!(
        actual
            .iter()
            .all(|(name, hash)| expected.get(name) == Some(hash)),
        "Cleanup tree changed: {}",
        root.display()
    );
    let remove_lock = receipt && actual.contains_key("running.lock");
    ensure!(
        !remove_lock || lock.is_some(),
        "Cleanup did not acquire the update lock"
    );
    // Use the verified inventory itself, children before parents. A fresh tree
    // walk here could accidentally include a file added after preflight.
    for (name, hash) in actual.iter().rev() {
        if receipt && name == "running.lock" {
            continue; // Keep the lock in place until every other entry is gone.
        }
        let relative = safe_relative(name)?;
        let mut path = root.to_owned();
        crate::removal::plain(&path)?;
        for component in relative.components() {
            path.push(component);
            crate::removal::plain(&path)?;
        }
        if let Some(hash) = hash {
            ensure!(hash_file(&path)? == *hash, "Cleanup file changed");
            fs::remove_file(&path)?;
        } else {
            fs::remove_dir(&path)?;
        }
    }
    if remove_lock {
        crate::removal::plain(root)?;
        let path = root.join("running.lock");
        verify_empty_lock(&path)?;
        fs::remove_file(path)?;
    }
    // Windows completes deletion of the lock file after the last handle closes.
    drop(lock);
    finish_purge(
        root,
        receipt,
        if cfg!(windows) { 50 } else { 1 },
        || fs::remove_dir(root),
        std::thread::sleep,
    )
}

fn retryable_removal(error: &std::io::Error) -> bool {
    // A helper still in DELETE_PENDING can make an empty directory report 145.
    matches!(error.raw_os_error(), Some(5 | 32 | 145))
}

fn finish_purge(
    root: &Path,
    receipt: bool,
    attempts: usize,
    mut remove_root: impl FnMut() -> std::io::Result<()>,
    mut wait: impl FnMut(Duration),
) -> Result<()> {
    let saved_receipt = if receipt {
        let path = root.join(RECEIPT);
        let bytes = read_limited(File::open(&path)?, 2 * 1024 * 1024)?;
        fs::remove_file(path)?;
        Some(bytes)
    } else {
        None
    };
    let mut result = remove_root();
    for _ in 1..attempts {
        if !result.as_ref().is_err_and(retryable_removal) {
            break;
        }
        wait(Duration::from_millis(200));
        result = remove_root();
    }
    if let Err(error) = result {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Ok(());
        }
        // Retain authorization for a later attempt. Never replace an existing
        // receipt or follow a substituted link after the preflight check.
        if let Some(bytes) = saved_receipt {
            ensure!(
                crate::removal::plain(root)?.is_dir(),
                "Cleanup root changed"
            );
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(RECEIPT))?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        // The local grace period is exhausted; do not multiply it by the
        // outer file retry loop. The restored receipt enables a later launch.
        anyhow::bail!(
            "Final update cleanup deferred for {}: {error}",
            root.display()
        );
    }
    Ok(())
}
fn clean(directory: &Path, installation: &Installation) -> Result<()> {
    // A newly reopened App must not delete an updater's active staging tree.
    let lock_path = directory.join("running.lock");
    let active_guard = if lock_path.try_exists()? {
        crate::removal::plain(&lock_path)?;
        let file = File::options().read(true).write(true).open(lock_path)?;
        file.try_lock()?;
        Some(file)
    } else {
        None
    };
    nonce(directory)?;
    let receipt: Receipt = serde_json::from_slice(&read_limited(
        File::open(directory.join(RECEIPT))?,
        2 * 1024 * 1024,
    )?)?;
    ensure!(
        receipt.installation.root == installation.root
            && receipt.installation.target == installation.target
            && receipt.installed_hash == hash_file(&installation.executable())?,
        "Cleanup belongs to a different installation"
    );
    if !receipt.backup.is_empty() {
        purge(
            &backup_path(installation, directory)?,
            &receipt.backup,
            false,
            None,
        )?;
    }
    purge(directory, &receipt.staged, true, active_guard)
}

pub(super) fn record_cancelled(directory: &Path, plan: &Plan) -> Result<()> {
    ensure!(
        !backup_path(&plan.installation, directory)?.try_exists()?,
        "A recovery backup still exists"
    );
    ensure!(
        hash_file(&plan.installation.executable())? == plan.previous_hash,
        "Previous installation was not fully restored"
    );
    record(directory, &plan.installation, None)
}
/// Older helpers have no receipt. Require a committed plan, verified staged
/// package, installed version >= that plan, and the exact previous binary hash.
fn adopt_legacy(directory: &Path, installation: &Installation) -> Result<()> {
    let plan: Plan = serde_json::from_slice(&read_limited(
        File::open(directory.join("apply.json"))?,
        65536,
    )?)?;
    ensure!(
        plan.format == 1
            && plan.installation.root == installation.root
            && plan.installation.target == installation.target
            && stable_version(crate::version::PUBLIC)? >= stable_version(&plan.version)?,
        "Unfinished or foreign update"
    );
    let commit: Commit = serde_json::from_slice(&read_limited(
        File::open(directory.join("commit.ok"))?,
        4096,
    )?)?;
    ensure!(
        commit.manifest_hash == plan.manifest_hash,
        "Update not committed"
    );
    let payload = directory.join("payload");
    let bytes = read_limited(
        File::open(payload.join("build-manifest.json"))?,
        1024 * 1024,
    )?;
    ensure!(
        hash_bytes(&bytes) == plan.manifest_hash,
        "Staging identity changed"
    );
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    verify_payload(&payload, &manifest)?;
    let backup = backup_path(installation, directory)?;
    let binary = backup.join(if installation.target.mac() {
        "MacOS/clickpulse"
    } else if installation.target.linux() {
        "clickpulse"
    } else {
        "ClickPulse.exe"
    });
    ensure!(
        hash_file(&binary)? == plan.previous_hash,
        "Backup identity changed"
    );
    let entries = inventory(&backup, false)?;
    if installation.target.mac() {
        let info = fs::read_to_string(backup.join("Info.plist"))?;
        ensure!(
            plist_value(&info, "CFBundleIdentifier") == Some("app.clickpulse.desktop"),
            "Foreign backup App"
        );
        ensure!(
            entries.iter().all(|(name, hash)| hash.is_none()
                || name == "Info.plist"
                || name == "MacOS/clickpulse"
                || name.starts_with("Resources/")
                || name.starts_with("_CodeSignature/")),
            "Unknown files in backup App"
        );
    } else {
        ensure!(
            entries.iter().all(|(name, hash)| if hash.is_none() {
                name == "Licenses"
            } else {
                name == "build-manifest.json"
                    || if installation.target.linux() {
                        linux_payload(name)
                    } else {
                        windows_payload(name)
                    }
            }),
            "Unknown files in portable backup"
        );
        let old: Manifest = serde_json::from_slice(&read_limited(
            File::open(backup.join("build-manifest.json"))?,
            1024 * 1024,
        )?)?;
        ensure!(
            old.files
                .iter()
                .all(|(name, hash)| entries.get(name) == Some(&Some(hash.clone()))),
            "Previous package changed"
        );
    }
    // Prefix, identity and complete file inventory are checked before a receipt
    // authorizes removal. No scan of arbitrary folders beside the application.
    record(directory, installation, Some(&backup))
}
pub(super) fn start() {
    // Cleanup can hash sizable files and Windows may still be exiting its helper.
    // Keep that work off the UI thread and leave modified files for manual review.
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("clickpulse-update-cleanup".into())
            .spawn(|| {
                let Ok(install) = installation() else {
                    return;
                };
                let Ok(base) = cache_dir().and_then(|p| Ok(p.canonicalize()?)) else {
                    return;
                };
                let Ok(entries) = fs::read_dir(&base) else {
                    return;
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if nonce(&path).is_err()
                        || !entry
                            .file_type()
                            .is_ok_and(|t| t.is_dir() && !t.is_symlink())
                    {
                        continue;
                    }
                    if !path.join(RECEIPT).exists() && adopt_legacy(&path, &install).is_err() {
                        continue;
                    }
                    for attempt in 0..50 {
                        match clean(&path, &install) {
                            Ok(()) => break,
                            Err(error) => {
                                let locked = cfg!(windows)
                                    && error
                                        .downcast_ref::<std::io::Error>()
                                        .is_some_and(retryable_removal);
                                if locked && attempt < 49 {
                                    std::thread::sleep(Duration::from_millis(200));
                                } else {
                                    eprintln!(
                                        "Update cleanup deferred for {}: {error:#}",
                                        path.display()
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            });
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unfinished_update_cleans_staging_only_after_helper_releases_it() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("installed");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("ClickPulse.exe"), "old binary").unwrap();
        let install = Installation {
            root,
            target: Target::WindowsX64,
        };
        let stage = temp.path().join("pending-cancelled");
        fs::create_dir(&stage).unwrap();
        let lock = File::options()
            .create_new(true)
            .read(true)
            .write(true)
            .open(stage.join("running.lock"))
            .unwrap();
        lock.try_lock().unwrap();
        fs::write(stage.join("payload"), "download").unwrap();
        record(&stage, &install, None).unwrap();
        assert!(clean(&stage, &install).is_err());
        assert!(stage.join("payload").is_file());
        drop(lock);
        fs::write(stage.join("running.lock"), b"not an updater lock").unwrap();
        assert!(clean(&stage, &install).is_err());
        assert!(stage.join("payload").is_file());
        fs::write(stage.join("running.lock"), b"").unwrap();
        clean(&stage, &install).unwrap();
        assert!(!stage.exists());
        assert_eq!(fs::read(install.executable()).unwrap(), b"old binary");
    }

    #[test]
    fn final_directory_removal_retries_delete_pending_locally() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("pending-test");
        fs::create_dir(&root).unwrap();
        fs::write(root.join(RECEIPT), "receipt").unwrap();
        let mut attempts = 0;
        let mut waits = 0;
        finish_purge(
            &root,
            true,
            4,
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(std::io::Error::from_raw_os_error(145))
                } else {
                    fs::remove_dir(&root)
                }
            },
            |_| waits += 1,
        )
        .unwrap();
        assert_eq!((attempts, waits), (3, 2));
        assert!(!root.exists());
    }

    #[test]
    fn exhausted_final_removal_preserves_receipt_and_unknown_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("pending-test");
        fs::create_dir(&root).unwrap();
        fs::write(root.join(RECEIPT), "receipt").unwrap();
        let mut attempts = 0;
        assert!(
            finish_purge(
                &root,
                true,
                3,
                || {
                    attempts += 1;
                    fs::write(root.join("unrelated"), "keep")?;
                    Err(std::io::Error::from_raw_os_error(145))
                },
                |_| {}
            )
            .is_err()
        );
        assert_eq!(attempts, 3);
        assert_eq!(fs::read(root.join(RECEIPT)).unwrap(), b"receipt");
        assert!(purge(&root, &BTreeMap::new(), true, None).is_err());
        assert_eq!(fs::read(root.join("unrelated")).unwrap(), b"keep");
        fs::remove_file(root.join("unrelated")).unwrap();
        purge(&root, &BTreeMap::new(), true, None).unwrap();
        assert!(!root.exists());
    }
    #[test]
    fn receipt_cleanup_removes_only_completed_update_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("installed");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("ClickPulse.exe"), "new binary").unwrap();
        fs::write(root.join("events.sqlite3"), "history stays").unwrap();
        let install = Installation {
            root,
            target: Target::WindowsX64,
        };
        let stage = temp.path().join("pending-test");
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("update-helper.exe"), "old binary").unwrap();
        let backup = backup_path(&install, &stage).unwrap();
        fs::create_dir(&backup).unwrap();
        fs::write(backup.join("ClickPulse.exe"), "old binary").unwrap();
        record(&stage, &install, Some(&backup)).unwrap();
        let other = Installation {
            root: temp.path().to_owned(),
            target: Target::WindowsX64,
        };
        assert!(clean(&stage, &other).is_err());
        assert!(backup.exists());
        clean(&stage, &install).unwrap();
        assert!(!stage.exists());
        assert!(!backup.exists());
        assert_eq!(
            fs::read(install.root.join("events.sqlite3")).unwrap(),
            b"history stays"
        );
        assert_eq!(fs::read(install.executable()).unwrap(), b"new binary");
    }
    #[test]
    fn cleanup_preserves_changed_or_extra_files_and_retries_partial_removal() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("pending-test");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("helper"), "owned").unwrap();
        let expected = inventory(&root, false).unwrap();
        fs::write(root.join("notes"), "unrelated").unwrap();
        assert!(purge(&root, &expected, false, None).is_err());
        assert!(root.join("helper").exists());
        fs::remove_file(root.join("notes")).unwrap();
        fs::write(root.join("helper"), "changed").unwrap();
        assert!(purge(&root, &expected, false, None).is_err());
        fs::remove_file(root.join("helper")).unwrap();
        purge(&root, &expected, false, None).unwrap();
        assert!(!root.exists());
    }
    #[cfg(unix)]
    #[test]
    fn cleanup_never_follows_a_link() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("pending-test");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("helper"), "owned").unwrap();
        let expected = inventory(&root, false).unwrap();
        fs::remove_file(root.join("helper")).unwrap();
        std::os::unix::fs::symlink(temp.path(), root.join("helper")).unwrap();
        assert!(purge(&root, &expected, false, None).is_err());
        assert!(root.exists());
    }
}
