use super::*;

#[test]
fn scoop_owned_version_and_current_directories_reject_in_app_replacement() {
    let temp = tempfile::tempdir().unwrap();
    for directory in [
        "scoop/apps/clickpulse/0.2.1",
        "SCOOP/APPS/ClickPulse/current",
    ] {
        let root = temp.path().join(directory);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("ClickPulse.exe"), b"original program").unwrap();
        fs::write(root.join("WebView2Loader.dll"), b"loader").unwrap();
        let install = Installation {
            root: root.canonicalize().unwrap(),
            target: Target::WindowsX64,
        };
        assert!(is_scoop_installation(&install.root));
        assert!(
            install
                .check()
                .unwrap_err()
                .to_string()
                .contains(SCOOP_UPDATE)
        );
        assert_eq!(
            fs::read(root.join("ClickPulse.exe")).unwrap(),
            b"original program"
        );
    }
}

#[test]
fn custom_scoop_roots_are_identified_by_receipts_without_blocking_portable_copies() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("custom-tools/apps/clickpulse/0.2.1");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ClickPulse.exe"), b"program").unwrap();
    fs::write(root.join("WebView2Loader.dll"), b"loader").unwrap();
    let install = Installation {
        root: root.canonicalize().unwrap(),
        target: Target::WindowsX64,
    };
    assert!(install.check().is_ok());
    fs::write(
        root.join("install.json"),
        br#"{"architecture":"64bit","bucket":"extras"}"#,
    )
    .unwrap();
    fs::write(
        root.join("manifest.json"),
        br#"{"homepage":"https://github.com/endNone/ClickPulse"}"#,
    )
    .unwrap();
    assert!(
        install
            .check()
            .unwrap_err()
            .to_string()
            .contains(SCOOP_UPDATE)
    );
    // Scoop's current path resolves to the same version directory, including
    // when the custom root itself contains no literal "scoop" component.
    let current = root.parent().unwrap().join("current");
    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args(["/D", "/C", "mklink", "/J"])
            // PathBuf joins can retain '/' inside a relative component;
            // cmd's built-in mklink interprets those as option prefixes.
            .arg(current.to_string_lossy().replace('/', "\\"))
            .arg(root.to_string_lossy().replace('/', "\\"))
            .status()
            .unwrap();
        assert!(status.success());
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&root, &current).unwrap();
    assert!(is_scoop_installation(&current.canonicalize().unwrap()));
    fs::write(root.join("manifest.json"), b"invalid receipt").unwrap();
    assert!(!is_scoop_installation(&install.root));
    for directory in [
        "scoop-downloads/ClickPulse",
        "scoop/apps/another-app/0.2.1",
        "Downloads/scoop/ClickPulse",
    ] {
        assert!(!is_scoop_installation(&temp.path().join(directory)));
    }
}

fn windows_fixture() -> (tempfile::TempDir, Installation, PathBuf, Manifest) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("installed");
    let payload = temp.path().join("new");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&payload).unwrap();
    let files = BTreeMap::from([
        ("ClickPulse.exe".into(), hash_bytes(b"new exe")),
        ("LICENSE.txt".into(), hash_bytes(b"new license")),
        ("WebView2Loader.dll".into(), hash_bytes(b"new loader")),
    ]);
    let m = Manifest {
        version: "1.2.3".into(),
        internal_build: 70,
        source_revision: "a".repeat(40),
        target: Target::WindowsX64.triple().into(),
        code_license: "GPL-3.0-or-later".into(),
        files,
    };
    for (name, old, new) in [
        ("ClickPulse.exe", "old exe", "new exe"),
        ("LICENSE.txt", "old license", "new license"),
        ("WebView2Loader.dll", "old loader", "new loader"),
    ] {
        fs::write(root.join(name), old).unwrap();
        fs::write(payload.join(name), new).unwrap();
    }
    fs::write(
        payload.join("build-manifest.json"),
        serde_json::to_vec(&m).unwrap(),
    )
    .unwrap();
    fs::write(root.join("events.sqlite3"), b"private history").unwrap();
    (
        temp,
        Installation {
            root: root.canonicalize().unwrap(),
            target: Target::WindowsX64,
        },
        payload,
        m,
    )
}
#[test]
fn windows_program_replacement_keeps_a_recovery_copy_and_unrelated_data() {
    let (_temp, install, payload, m) = windows_fixture();
    let backup = replace_windows(&install, &payload, &m, "fixture").unwrap();
    assert_eq!(
        fs::read(install.root.join("ClickPulse.exe")).unwrap(),
        b"new exe"
    );
    assert_eq!(fs::read(backup.join("ClickPulse.exe")).unwrap(), b"old exe");
    assert_eq!(
        fs::read(install.root.join("events.sqlite3")).unwrap(),
        b"private history"
    );
}
#[test]
fn partial_windows_failure_restores_every_moved_program_file() {
    let (_temp, install, payload, m) = windows_fixture();
    fs::remove_file(install.root.join("WebView2Loader.dll")).unwrap();
    fs::create_dir(install.root.join("WebView2Loader.dll")).unwrap();
    assert!(replace_windows(&install, &payload, &m, "failure").is_err());
    assert_eq!(
        fs::read(install.root.join("ClickPulse.exe")).unwrap(),
        b"old exe"
    );
    assert_eq!(
        fs::read(install.root.join("LICENSE.txt")).unwrap(),
        b"old license"
    );
    assert_eq!(
        fs::read(install.root.join("events.sqlite3")).unwrap(),
        b"private history"
    );
}
#[test]
fn mac_contents_replacement_preserves_outer_bundle_and_backup() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("ClickPulse.app");
    let payload = temp.path().join("payload");
    fs::create_dir_all(root.join("Contents")).unwrap();
    fs::write(root.join("Contents/program"), b"old").unwrap();
    fs::create_dir_all(payload.join("ClickPulse.app/Contents")).unwrap();
    fs::write(payload.join("ClickPulse.app/Contents/program"), b"new").unwrap();
    let install = Installation {
        root: root.canonicalize().unwrap(),
        target: Target::MacArm64,
    };
    #[cfg(unix)]
    let inode = {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(&root).unwrap().ino()
    };
    let backup = replace_mac(&install, &payload, "fixture").unwrap();
    assert_eq!(fs::read(root.join("Contents/program")).unwrap(), b"new");
    assert_eq!(fs::read(backup.join("program")).unwrap(), b"old");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(fs::metadata(root).unwrap().ino(), inode);
    }
}
#[test]
fn installation_waits_for_parent_release_and_explicit_successful_save() {
    let temp = tempfile::tempdir().unwrap();
    let lock = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(temp.path().join("running.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let other = File::options()
        .read(true)
        .write(true)
        .open(temp.path().join("running.lock"))
        .unwrap();
    let guard = ApplyGuard {
        directory: temp.path().to_path_buf(),
        manifest_hash: "a".repeat(64),
        _lock: lock,
    };
    assert!(other.try_lock().is_err());
    assert!(!temp.path().join("commit.ok").exists());
    guard.finish_shutdown(true, true).unwrap();
    assert!(other.try_lock().is_err());
    drop(guard);
    other.try_lock().unwrap();
    let commit: Commit =
        serde_json::from_slice(&fs::read(temp.path().join("commit.ok")).unwrap()).unwrap();
    assert_eq!(commit.manifest_hash, "a".repeat(64));
    assert!(commit.paused);
}
#[test]
fn save_failure_only_creates_an_abort_marker() {
    let temp = tempfile::tempdir().unwrap();
    let lock = File::create(temp.path().join("running.lock")).unwrap();
    let guard = ApplyGuard {
        directory: temp.path().into(),
        manifest_hash: "b".repeat(64),
        _lock: lock,
    };
    guard.finish_shutdown(false, true).unwrap();
    assert!(temp.path().join("abort").exists());
    assert!(!temp.path().join("commit.ok").exists());
}
#[cfg(unix)]
#[test]
fn symlinked_destinations_cannot_overwrite_external_data() {
    let (_temp, install, payload, m) = windows_fixture();
    let outside = install.root.parent().unwrap().join("outside");
    fs::write(&outside, b"untouched").unwrap();
    fs::remove_file(install.root.join("LICENSE.txt")).unwrap();
    std::os::unix::fs::symlink(&outside, install.root.join("LICENSE.txt")).unwrap();
    assert!(replace_windows(&install, &payload, &m, "links").is_err());
    assert_eq!(fs::read(outside).unwrap(), b"untouched");
    assert_eq!(
        fs::read(install.root.join("ClickPulse.exe")).unwrap(),
        b"old exe"
    );
}

#[cfg(unix)]
#[test]
fn payload_root_cannot_be_replaced_with_a_link() {
    let (temp, _install, payload, m) = windows_fixture();
    assert!(verify_payload(&payload, &m).is_ok());
    let link = temp.path().join("linked-payload");
    std::os::unix::fs::symlink(&payload, &link).unwrap();
    assert!(verify_payload(&link, &m).is_err());
}

// A real subprocess executes the helper. Linux also uses the real restart
// launcher with a tiny executable probe; other platforms substitute that last
// callback. Every profile, cache and installation path is disposable.
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
#[test]
fn helper_process_entry() {
    let Some(plan) = std::env::var_os("CLICKPULSE_UPDATER_TEST_PLAN") else {
        return;
    };
    #[cfg(target_os = "linux")]
    apply_update(Path::new(&plan)).unwrap();
    #[cfg(not(target_os = "linux"))]
    apply_update_with_restart(Path::new(&plan), |install, paused| {
        fs::write(
            install.root.parent().unwrap().join("restarted"),
            format!("paused={paused}"),
        )?;
        Ok(())
    })
    .unwrap();
}
#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
#[test]
fn helper_process_waits_for_saved_parent_and_preserves_profile_files() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    #[cfg(target_os = "macos")]
    let profile = home.join("Library/Application Support/ClickPulse");
    #[cfg(target_os = "windows")]
    let profile = home.join("AppData/Local/ClickPulse");
    #[cfg(target_os = "linux")]
    let profile = home.join(".local/share/ClickPulse");
    #[cfg(target_os = "macos")]
    let cache = home.join("Library/Caches/ClickPulse/updates");
    #[cfg(target_os = "windows")]
    let cache = profile.join("updates");
    #[cfg(target_os = "linux")]
    let cache = home.join(".cache/ClickPulse/updates");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&cache).unwrap();
    File::create(profile.join("profile.lock")).unwrap();
    {
        let db = rusqlite::Connection::open(profile.join("events.sqlite3")).unwrap();
        db.execute_batch("CREATE TABLE history (day TEXT, count INTEGER); INSERT INTO history VALUES ('2026-11-01', 7956);").unwrap();
    }
    let history_before = hash_file(&profile.join("events.sqlite3")).unwrap();
    fs::write(
        profile.join("presentation.json"),
        b"preferences must never change",
    )
    .unwrap();
    let stage = cache.join("pending-process");
    fs::create_dir(&stage).unwrap();
    let payload = stage.join("payload");
    fs::create_dir(&payload).unwrap();
    let target = Target::current().unwrap();
    let root = temp.path().join(if target.mac() {
        "ClickPulse.app"
    } else {
        "program"
    });
    fs::create_dir(&root).unwrap();
    let old_exe = if target.mac() {
        root.join("Contents/MacOS/clickpulse")
    } else if target.linux() {
        root.join("clickpulse")
    } else {
        root.join("ClickPulse.exe")
    };
    fs::create_dir_all(old_exe.parent().unwrap()).unwrap();
    fs::copy(std::env::current_exe().unwrap(), &old_exe).unwrap();
    let current = stable_version(crate::version::PUBLIC).unwrap();
    let next = format!("{}.0.0", current.major + 1);
    let mut files = BTreeMap::new();
    if target.mac() {
        let app = payload.join("ClickPulse.app");
        fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
        fs::copy(&old_exe, app.join("Contents/MacOS/clickpulse")).unwrap();
        let info = |version: &str| {
            format!(
                "<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleIdentifier</key><string>app.clickpulse.desktop</string><key>CFBundleExecutable</key><string>clickpulse</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>{version}</string><key>CFBundleVersion</key><string>70</string></dict></plist>"
            )
        };
        fs::write(
            root.join("Contents/Info.plist"),
            info(crate::version::PUBLIC),
        )
        .unwrap();
        fs::write(app.join("Contents/Info.plist"), info(&next)).unwrap();
        assert!(
            Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-"])
                .arg(&app)
                .output()
                .unwrap()
                .status
                .success()
        );
        fn collect(root: &Path, folder: &Path, out: &mut BTreeMap<String, String>) {
            for entry in fs::read_dir(folder).unwrap() {
                let p = entry.unwrap().path();
                if p.is_dir() {
                    collect(root, &p, out)
                } else {
                    out.insert(
                        p.strip_prefix(root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace('\\', "/"),
                        hash_file(&p).unwrap(),
                    );
                }
            }
        }
        collect(&payload, &app, &mut files);
    } else if target.linux() {
        // The installed probe writes its own launch arguments and version, so
        // this covers actual process creation without opening another App.
        let probe = temp.path().join("restart_probe.rs");
        fs::write(&probe, format!(r#"fn main() {{
            let exe = std::env::current_exe().unwrap();
            let paused = std::env::args().any(|arg| arg == "--paused");
            std::fs::write(exe.parent().unwrap().parent().unwrap().join("restarted"), format!("paused={{paused}}")).unwrap();
            println!("{next} (70)");
        }}"#)).unwrap();
        let result = Command::new("rustc")
            .arg(&probe)
            .arg("-o")
            .arg(payload.join("clickpulse"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        files.insert(
            "clickpulse".into(),
            hash_file(&payload.join("clickpulse")).unwrap(),
        );
    } else {
        fs::write(root.join("WebView2Loader.dll"), b"fixture loader").unwrap();
        let mut bytes = vec![0u8; 256];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&128u32.to_le_bytes());
        bytes[128..132].copy_from_slice(b"PE\0\0");
        bytes[132..134].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes.extend(
            format!("{next}.70")
                .encode_utf16()
                .flat_map(u16::to_le_bytes),
        );
        fs::write(payload.join("ClickPulse.exe"), bytes).unwrap();
        fs::write(payload.join("WebView2Loader.dll"), b"new loader").unwrap();
        for name in ["ClickPulse.exe", "WebView2Loader.dll"] {
            files.insert(name.into(), hash_file(&payload.join(name)).unwrap());
        }
    }
    let manifest = Manifest {
        version: next.clone(),
        internal_build: 70,
        source_revision: "a".repeat(40),
        target: target.triple().into(),
        code_license: "GPL-3.0-or-later".into(),
        files,
    };
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    fs::write(payload.join("build-manifest.json"), &manifest_bytes).unwrap();
    if target.linux() {
        let mut installed = manifest.clone();
        installed.version = crate::version::PUBLIC.into();
        installed
            .files
            .insert("clickpulse".into(), hash_file(&old_exe).unwrap());
        fs::write(
            root.join("build-manifest.json"),
            serde_json::to_vec(&installed).unwrap(),
        )
        .unwrap();
    }
    let helper = stage.join(if target != Target::WindowsX64 {
        "update-helper"
    } else {
        "update-helper.exe"
    });
    fs::copy(&old_exe, &helper).unwrap();
    let previous_hash = hash_file(&old_exe).unwrap();
    let manifest_hash = hash_bytes(&manifest_bytes);
    let plan = Plan {
        format: 1,
        installation: Installation {
            root: root.canonicalize().unwrap(),
            target,
        },
        previous_hash: previous_hash.clone(),
        manifest_hash: manifest_hash.clone(),
        version: next,
    };
    let plan_path = stage.join("apply.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let lock = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(stage.join("running.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let mut child = Command::new(helper)
        .args([
            "--exact",
            "updater::install::tests::helper_process_entry",
            "--nocapture",
        ])
        .env("CLICKPULSE_UPDATER_TEST_PLAN", &plan_path)
        .env("HOME", &home)
        .env("LOCALAPPDATA", home.join("AppData/Local"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(60);
    while !stage.join("helper.ready").exists() {
        assert!(Instant::now() < until, "helper validation timed out");
        assert!(
            child.try_wait().unwrap().is_none(),
            "helper exited before readiness"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(hash_file(&old_exe).unwrap(), previous_hash);
    assert!(!root.parent().unwrap().join("restarted").exists());
    let guard = ApplyGuard {
        directory: stage.clone(),
        manifest_hash,
        _lock: lock,
    };
    guard.finish_shutdown(true, true).unwrap();
    drop(guard);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= until {
            let _ = child.kill();
            panic!("helper handoff timed out");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_ne!(hash_file(&old_exe).unwrap(), previous_hash);
    while !root.parent().unwrap().join("restarted").exists() {
        assert!(Instant::now() < until, "updated program did not restart");
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        fs::read_to_string(root.parent().unwrap().join("restarted")).unwrap(),
        "paused=true"
    );
    assert_eq!(
        hash_file(&profile.join("events.sqlite3")).unwrap(),
        history_before
    );
    assert_eq!(
        fs::read(profile.join("presentation.json")).unwrap(),
        b"preferences must never change"
    );
}

#[test]
fn linux_elf_architecture_version_and_payload_scope_are_verified() {
    let temp = tempfile::tempdir().unwrap();
    for (target, machine) in [(Target::LinuxArm64, 183u16), (Target::LinuxX64, 62u16)] {
        let mut bytes = vec![0u8; 64];
        bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
        bytes[16..18].copy_from_slice(&3u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&machine.to_le_bytes());
        bytes.extend_from_slice(b"1.2.3 (78)");
        let binary = temp.path().join("clickpulse");
        fs::write(&binary, &bytes).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(validate_program(temp.path(), target, "1.2.3", 78).is_ok());
        assert!(validate_program(temp.path(), target, "1.2.4", 78).is_err());
        bytes[18..20].copy_from_slice(&0x8664u16.to_le_bytes());
        fs::write(&binary, bytes).unwrap();
        assert!(validate_program(temp.path(), target, "1.2.3", 78).is_err());
    }
    assert!(linux_payload("clickpulse"));
    for name in [
        "events.sqlite3",
        "prefs.json",
        "../../clickpulse",
        "WebView2Loader.dll",
    ] {
        assert!(!linux_payload(name));
    }
}

#[test]
fn linux_portable_replacement_preserves_data_and_refuses_foreign_manifest_paths() {
    let (temp, mut install, payload, mut manifest) = windows_fixture();
    install.target = Target::LinuxArm64;
    manifest.target = install.target.triple().into();
    manifest.files = BTreeMap::from([("clickpulse".into(), hash_bytes(b"new linux"))]);
    fs::write(install.root.join("clickpulse"), b"old linux").unwrap();
    fs::write(install.root.join("events.sqlite3"), b"history").unwrap();
    fs::write(payload.join("clickpulse"), b"new linux").unwrap();
    fs::write(
        install.root.join("build-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        payload.join("build-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let backup = replace_windows(&install, &payload, &manifest, "linux-test").unwrap();
    assert_eq!(fs::read(install.executable()).unwrap(), b"new linux");
    assert_eq!(fs::read(backup.join("clickpulse")).unwrap(), b"old linux");
    assert_eq!(
        fs::read(install.root.join("events.sqlite3")).unwrap(),
        b"history"
    );
    manifest
        .files
        .insert("events.sqlite3".into(), hash_bytes(b"history"));
    fs::write(
        install.root.join("build-manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(replace_windows(&install, &payload, &manifest, "reject-test").is_err());
    assert!(temp.path().is_dir());
}
