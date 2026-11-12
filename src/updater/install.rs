use super::*;
use std::{process::Command, time::Instant};

const SCOOP_UPDATE: &str = "scoop update clickpulse";

/// Resolve `current` junctions before inspecting ownership. Scoop permits custom
/// roots (including global installs), so a substring containing `scoop` is not
/// sufficient. Its version directory also carries two manager-owned receipts.
fn is_scoop_installation(root: &Path) -> bool {
    let named = |path: &Path, name: &str| {
        path.file_name()
            .is_some_and(|part| part.to_string_lossy().eq_ignore_ascii_case(name))
    };
    let Some(app) = root.parent() else {
        return false;
    };
    let Some(apps) = app.parent() else {
        return false;
    };
    if !named(app, "clickpulse") || !named(apps, "apps") {
        return false;
    }
    if apps.parent().is_some_and(|parent| named(parent, "scoop")) {
        return true;
    }
    let receipt = |name: &str| -> Option<serde_json::Value> {
        let bytes = read_limited(File::open(root.join(name)).ok()?, 65536).ok()?;
        serde_json::from_slice(&bytes).ok()
    };
    receipt("install.json").is_some_and(|value| value["architecture"] == "64bit")
        && receipt("manifest.json")
            .is_some_and(|value| value["homepage"] == "https://github.com/endNone/ClickPulse")
}

/// UI guidance is independent of whether the installation is writable.
pub fn package_update_command() -> Option<&'static str> {
    if std::env::consts::OS != "windows" {
        return None;
    }
    let exe = std::env::current_exe().ok()?.canonicalize().ok()?;
    is_scoop_installation(exe.parent()?).then_some(SCOOP_UPDATE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub root: PathBuf,
    pub target: Target,
}
impl Installation {
    fn executable(&self) -> PathBuf {
        self.root.join(if self.target.mac() {
            "Contents/MacOS/clickpulse"
        } else {
            "ClickPulse.exe"
        })
    }
    fn check(&self) -> Result<()> {
        ensure!(
            self.root.is_absolute() && self.root.canonicalize()? == self.root,
            "Invalid installation path"
        );
        ensure!(
            self.target != Target::WindowsX64 || !is_scoop_installation(&self.root),
            "This installation is managed by Scoop; run {SCOOP_UPDATE}"
        );
        ensure!(
            self.executable().is_file(),
            "Installed executable is missing"
        );
        if self.target.mac() {
            ensure!(
                self.root.extension().is_some_and(|e| e == "app"),
                "Use an installed Mac App"
            );
            let info = fs::read_to_string(self.root.join("Contents/Info.plist"))?;
            ensure!(
                plist_value(&info, "CFBundleIdentifier") == Some("app.clickpulse.desktop"),
                "Wrong application identity"
            );
        } else {
            ensure!(
                self.root.join("WebView2Loader.dll").is_file(),
                "Use the complete portable application folder"
            );
        }
        Ok(())
    }
}
pub fn installation() -> Result<Installation> {
    let exe = std::env::current_exe()?.canonicalize()?;
    let target = Target::current()?;
    let root = if target.mac() {
        ensure!(
            exe.file_name().is_some_and(|v| v == "clickpulse")
                && exe
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|v| v == "MacOS"),
            "Updates require an installed App"
        );
        exe.parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .context("Missing App bundle")?
            .to_path_buf()
    } else {
        ensure!(
            exe.file_name()
                .is_some_and(|v| v.to_string_lossy().eq_ignore_ascii_case("ClickPulse.exe")),
            "Updates require the portable App"
        );
        exe.parent()
            .context("Missing program folder")?
            .to_path_buf()
    };
    let value = Installation { root, target };
    value.check()?;
    let parent = if target.mac() {
        value.root.parent().context("Missing App directory")?
    } else {
        &value.root
    };
    let probe = tempfile::Builder::new()
        .prefix(".clickpulse-update-check-")
        .tempfile_in(parent)
        .context("Application folder is not writable; download and install the update manually")?;
    drop(probe);
    Ok(value)
}
fn plist_value<'a>(xml: &'a str, key: &str) -> Option<&'a str> {
    xml.split_once(&format!("<key>{key}</key>"))?
        .1
        .trim_start()
        .strip_prefix("<string>")?
        .split_once("</string>")
        .map(|v| v.0)
}
pub(super) fn validate_program(
    payload: &Path,
    target: Target,
    version: &str,
    build: u64,
) -> Result<()> {
    let executable = payload.join(target.binary());
    let bytes = fs::read(&executable)?;
    if target.mac() {
        let bundle = payload.join("ClickPulse.app");
        let info = fs::read_to_string(bundle.join("Contents/Info.plist"))?;
        ensure!(
            plist_value(&info, "CFBundleIdentifier") == Some("app.clickpulse.desktop")
                && plist_value(&info, "CFBundleShortVersionString") == Some(version)
                && plist_value(&info, "CFBundleVersion")
                    .is_some_and(|n| n.parse::<u64>().ok() == Some(build)),
            "Mac App identity/version mismatch"
        );
        ensure!(
            bytes.len() >= 8 && u32::from_le_bytes(bytes[..4].try_into()?) == 0xfeedfacf,
            "Invalid Mac executable"
        );
        let cpu = u32::from_le_bytes(bytes[4..8].try_into()?);
        ensure!(
            cpu == if target == Target::MacArm64 {
                0x0100000c
            } else {
                0x01000007
            },
            "Mac executable architecture mismatch"
        );
        #[cfg(target_os = "macos")]
        ensure!(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(bundle)
                .output()?
                .status
                .success(),
            "Mac App signature is invalid"
        );
    } else {
        ensure!(
            bytes.len() >= 64 && &bytes[..2] == b"MZ",
            "Invalid Windows executable"
        );
        let offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into()?) as usize;
        ensure!(
            offset.checked_add(6).is_some_and(|end| end <= bytes.len())
                && &bytes[offset..offset + 4] == b"PE\0\0"
                && u16::from_le_bytes(bytes[offset + 4..offset + 6].try_into()?) == 0x8664,
            "Windows executable architecture mismatch"
        );
        let wide: Vec<_> = format!("{version}.{build}")
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        ensure!(
            bytes.windows(wide.len()).any(|w| w == wide),
            "Windows executable version mismatch"
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Plan {
    format: u8,
    installation: Installation,
    previous_hash: String,
    manifest_hash: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct Commit {
    manifest_hash: String,
    paused: bool,
}

/// Kept alive by the desktop event loop. The helper waits until this process
/// releases the lock, and installs only after shutdown explicitly committed.
#[derive(Debug)]
pub struct ApplyGuard {
    directory: PathBuf,
    manifest_hash: String,
    _lock: File,
}
impl ApplyGuard {
    pub fn finish_shutdown(&self, saved: bool, paused: bool) -> Result<()> {
        let mut file =
            File::create_new(
                self.directory
                    .join(if saved { "commit.ok" } else { "abort" }),
            )?;
        file.write_all(&serde_json::to_vec(&Commit {
            manifest_hash: self.manifest_hash.clone(),
            paused,
        })?)?;
        file.sync_all()?;
        Ok(())
    }
}
pub fn launch_installer(update: StagedUpdate, installation: Installation) -> Result<ApplyGuard> {
    installation.check()?;
    ensure!(
        installation.target == update.target,
        "Update target changed"
    );
    ensure!(
        stable_version(&update.manifest.version)? > stable_version(crate::version::PUBLIC)?,
        "Refusing to install the same or an older release"
    );
    let current = std::env::current_exe()?.canonicalize()?;
    ensure!(
        current == installation.executable().canonicalize()?,
        "Update must run from this installation"
    );
    let previous_hash = hash_file(&current)?;
    let helper = update.directory.path().join(if update.target.mac() {
        "update-helper"
    } else {
        "update-helper.exe"
    });
    fs::copy(&current, &helper)?;
    ensure!(
        hash_file(&helper)? == previous_hash,
        "Could not stage updater helper"
    );
    if !update.target.mac() {
        fs::copy(
            installation.root.join("WebView2Loader.dll"),
            update.directory.path().join("WebView2Loader.dll"),
        )?;
    }
    let lock = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(update.directory.path().join("running.lock"))?;
    lock.try_lock()?;
    let plan = Plan {
        format: 1,
        installation,
        previous_hash,
        manifest_hash: update.manifest_hash.clone(),
        version: update.manifest.version.clone(),
    };
    let path = update.directory.path().join("apply.json");
    let mut file = File::create_new(&path)?;
    file.write_all(&serde_json::to_vec(&plan)?)?;
    file.sync_all()?;
    let mut command = Command::new(&helper);
    command.arg("--apply-update").arg(&path);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().context("Unable to start update helper")?;
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        if fs::read_to_string(update.directory.path().join("helper.ready"))
            .is_ok_and(|v| v == plan.manifest_hash)
        {
            break;
        }
        if child.try_wait()?.is_some() {
            bail!("Update helper could not validate the staged installation");
        }
        if Instant::now() >= until {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Update helper did not become ready");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(ApplyGuard {
        directory: update.directory.keep(),
        manifest_hash: plan.manifest_hash,
        _lock: lock,
    })
}

fn no_links(root: &Path, relative: &Path) -> Result<()> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(meta) => ensure!(
                !meta.file_type().is_symlink(),
                "Update path contains a symbolic link"
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn verify_payload(payload: &Path, manifest: &Manifest) -> Result<()> {
    let root = fs::symlink_metadata(payload)?;
    ensure!(
        root.is_dir() && !root.file_type().is_symlink(),
        "Staged payload is not a regular directory"
    );
    fn walk(root: &Path, folder: &Path, found: &mut BTreeSet<String>) -> Result<()> {
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            ensure!(!kind.is_symlink(), "Staged package contains a link");
            if kind.is_dir() {
                walk(root, &entry.path(), found)?;
            } else {
                ensure!(kind.is_file(), "Staged package contains a special file");
                let name = entry
                    .path()
                    .strip_prefix(root)?
                    .to_str()
                    .context("Invalid staged filename")?
                    .replace('\\', "/");
                safe_relative(&name)?;
                found.insert(name);
            }
        }
        Ok(())
    }
    let mut found = BTreeSet::new();
    walk(payload, payload, &mut found)?;
    let mut expected: BTreeSet<_> = manifest.files.keys().cloned().collect();
    expected.insert("build-manifest.json".into());
    ensure!(found == expected, "Staged package file list changed");
    for (name, hash) in &manifest.files {
        let relative = safe_relative(name)?;
        no_links(payload, &relative)?;
        ensure!(
            hash_file(&payload.join(relative))? == *hash,
            "Staged file changed"
        );
    }
    Ok(())
}
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(from)?.file_type().is_symlink(),
        "Update source cannot be a link"
    );
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        } else {
            ensure!(kind.is_file(), "Special files cannot be installed");
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn replace_mac(install: &Installation, payload: &Path, nonce: &str) -> Result<PathBuf> {
    let parent = install.root.parent().context("App directory unavailable")?;
    let prepared = parent.join(format!(".ClickPulse-new-{nonce}"));
    let backup = parent.join(format!(".ClickPulse-before-{nonce}"));
    ensure!(
        !prepared.exists() && !backup.exists(),
        "Update recovery path already exists"
    );
    copy_tree(&payload.join("ClickPulse.app/Contents"), &prepared)?;
    let contents = install.root.join("Contents");
    no_links(&install.root, Path::new("Contents"))?;
    if let Err(error) = fs::rename(&contents, &backup) {
        let _ = fs::remove_dir_all(&prepared);
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&prepared, &contents) {
        fs::rename(&backup, &contents).context("Restore previous App Contents")?;
        let _ = fs::remove_dir_all(prepared);
        return Err(error.into());
    }
    Ok(backup)
}
fn replace_windows(
    install: &Installation,
    payload: &Path,
    manifest: &Manifest,
    nonce: &str,
) -> Result<PathBuf> {
    let backup = install.root.join(format!(".ClickPulse-before-{nonce}"));
    let prepared = install.root.join(format!(".ClickPulse-new-{nonce}"));
    fs::create_dir(&backup)?;
    fs::create_dir(&prepared)?;
    let mut files: BTreeSet<_> = manifest.files.keys().cloned().collect();
    files.insert("build-manifest.json".into());
    let old = install.root.join("build-manifest.json");
    if old.is_file() {
        let bytes = read_limited(File::open(old)?, 1024 * 1024)?;
        let old: Manifest = serde_json::from_slice(&bytes)?;
        for name in old.files.keys() {
            safe_relative(name)?;
            ensure!(
                windows_payload(name),
                "Previous manifest contains a non-program path"
            );
            files.insert(name.clone());
        }
    }
    for name in &files {
        let relative = safe_relative(name)?;
        no_links(&install.root, &relative)?;
        let src = payload.join(&relative);
        if src.is_file() {
            let dest = prepared.join(&relative);
            fs::create_dir_all(dest.parent().context("Missing staging directory")?)?;
            fs::copy(src, dest)?;
        }
    }
    let mut moved = Vec::new();
    let mut installed = Vec::new();
    let result = (|| -> Result<()> {
        for name in &files {
            let relative = safe_relative(name)?;
            let dest = install.root.join(&relative);
            if dest.exists() {
                ensure!(dest.is_file(), "A program path is occupied by a directory");
                let saved = backup.join(&relative);
                fs::create_dir_all(saved.parent().context("Missing backup directory")?)?;
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    match fs::rename(&dest, &saved) {
                        Ok(()) => break,
                        Err(e) if Instant::now() < deadline => {
                            let _ = e;
                            std::thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                moved.push(relative.clone());
            }
            let src = prepared.join(&relative);
            if src.is_file() {
                fs::create_dir_all(dest.parent().context("Missing install directory")?)?;
                fs::rename(src, dest)?;
                installed.push(relative);
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        for name in installed.iter().rev() {
            fs::remove_file(install.root.join(name))?;
        }
        for name in moved.iter().rev() {
            fs::rename(backup.join(name), install.root.join(name))
                .context("Restore previous program file")?;
        }
        let _ = fs::remove_dir_all(prepared);
        return Err(error);
    }
    let _ = fs::remove_dir_all(prepared);
    Ok(backup)
}
#[derive(Serialize, Deserialize)]
pub struct Notice {
    pub success: bool,
    pub version: String,
}
fn write_notice(success: bool, version: &str) -> Result<()> {
    let base = cache_dir()?;
    fs::create_dir_all(&base)?;
    let mut file = tempfile::NamedTempFile::new_in(&base)?;
    file.write_all(&serde_json::to_vec(&Notice {
        success,
        version: version.to_owned(),
    })?)?;
    file.as_file().sync_all()?;
    file.persist(base.join("last-update.json"))?;
    Ok(())
}
pub fn take_notice() -> Option<Notice> {
    let path = cache_dir().ok()?.join("last-update.json");
    let bytes = read_limited(File::open(&path).ok()?, 4096).ok()?;
    let notice: Notice = serde_json::from_slice(&bytes).ok()?;
    stable_version(&notice.version).ok()?;
    fs::remove_file(path).ok()?;
    Some(notice)
}
fn restart(install: &Installation, paused: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("/usr/bin/open");
        command.arg(&install.root);
        if paused {
            command.args(["--args", "--paused"]);
        }
        command.spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new(install.executable());
        command.current_dir(&install.root);
        if paused {
            command.arg("--paused");
        }
        command.spawn()?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (install, paused);
        bail!("Restart unsupported on this platform");
    }
    #[allow(unreachable_code)]
    Ok(())
}
pub fn apply_update(path: &Path) -> Result<()> {
    apply_update_with_restart(path, restart)
}
fn apply_update_with_restart(
    path: &Path,
    relaunch: impl FnOnce(&Installation, bool) -> Result<()>,
) -> Result<()> {
    let exe = std::env::current_exe()?.canonicalize()?;
    let path = path.canonicalize()?;
    let directory = path.parent().context("Missing update directory")?;
    ensure!(
        path.file_name().is_some_and(|v| v == "apply.json") && exe.parent() == Some(directory),
        "Helper must run beside its update plan"
    );
    ensure!(
        exe.file_name()
            .is_some_and(|v| v == "update-helper" || v == "update-helper.exe"),
        "This option is only for the staged updater helper"
    );
    ensure!(
        directory.parent() == Some(cache_dir()?.canonicalize()?.as_path()),
        "Helper is outside the private update cache"
    );
    let nonce = directory
        .file_name()
        .context("Missing update nonce")?
        .to_string_lossy();
    ensure!(
        nonce.starts_with("pending-")
            && nonce
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "Invalid update directory"
    );
    let plan: Plan = serde_json::from_slice(&read_limited(File::open(&path)?, 65536)?)?;
    ensure!(
        plan.format == 1 && hash_file(&exe)? == plan.previous_hash,
        "Updater helper identity mismatch"
    );
    ensure!(
        stable_version(&plan.version)? > stable_version(crate::version::PUBLIC)?,
        "Refusing to downgrade the installed program"
    );
    ensure!(
        plan.installation.target == Target::current()?,
        "Helper architecture does not match the update target"
    );
    plan.installation.check()?;
    let payload = directory.join("payload");
    let bytes = read_limited(
        File::open(payload.join("build-manifest.json"))?,
        1024 * 1024,
    )?;
    ensure!(
        hash_bytes(&bytes) == plan.manifest_hash,
        "Staged manifest changed"
    );
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    ensure!(
        manifest.version == plan.version && manifest.target == plan.installation.target.triple(),
        "Staged target changed"
    );
    verify_payload(&payload, &manifest)?;
    validate_program(
        &payload,
        plan.installation.target,
        &plan.version,
        manifest.internal_build,
    )?;
    let mut ready = File::create_new(directory.join("helper.ready"))?;
    ready.write_all(plan.manifest_hash.as_bytes())?;
    ready.sync_all()?;
    let lock = File::options()
        .read(true)
        .write(true)
        .open(directory.join("running.lock"))?;
    let until = Instant::now() + Duration::from_secs(120);
    loop {
        match lock.try_lock() {
            Ok(()) => break,
            Err(_) if Instant::now() < until => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(e.into()),
        }
    }
    let commit = File::open(directory.join("commit.ok"))
        .ok()
        .and_then(|f| read_limited(f, 4096).ok())
        .and_then(|b| serde_json::from_slice::<Commit>(&b).ok())
        .filter(|c| c.manifest_hash == plan.manifest_hash);
    let committed = commit.is_some();
    let aborted = File::open(directory.join("abort"))
        .ok()
        .and_then(|f| read_limited(f, 4096).ok())
        .and_then(|b| serde_json::from_slice::<Commit>(&b).ok())
        .filter(|c| c.manifest_hash == plan.manifest_hash);
    if !committed && aborted.is_none() {
        return Ok(());
    }
    let paused = commit
        .as_ref()
        .or(aborted.as_ref())
        .is_some_and(|c| c.paused);
    let result = (|| -> Result<PathBuf> {
        // Block another normal App launch while its program files are being swapped.
        // The existing profile is only locked, never opened as a database or rewritten.
        let profile_lock = File::options()
            .read(true)
            .write(true)
            .open(crate::platform::data_dir()?.join("profile.lock"))?;
        profile_lock
            .try_lock()
            .context("ClickPulse reopened before the update could start")?;
        ensure!(
            committed,
            "Update cancelled because the application did not finish saving"
        );
        ensure!(
            hash_file(&plan.installation.executable())? == plan.previous_hash,
            "Installed program changed while updating"
        );
        verify_payload(&payload, &manifest)?;
        let backup = if plan.installation.target.mac() {
            replace_mac(&plan.installation, &payload, &nonce)?
        } else {
            replace_windows(&plan.installation, &payload, &manifest, &nonce)?
        };
        drop(profile_lock);
        Ok(backup)
    })();
    let _ = write_notice(result.is_ok(), &plan.version);
    relaunch(&plan.installation, paused)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests;
