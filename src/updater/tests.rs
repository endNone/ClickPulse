use super::*;

#[test]
fn update_reminders_require_a_newer_stable_version_and_clear_withdrawn_offers() {
    let mut notices = UpdateNotifications::default();
    for version in [
        crate::version::PUBLIC,
        "0.0.1",
        "invalid",
        "9.9.9-beta",
        "9.9.9+build",
    ] {
        assert!(!notices.observe(Some(version)), "{version}");
        assert_eq!(notices.pending(), None);
    }
    assert!(notices.observe(Some("9.9.9")));
    assert!(!notices.observe(Some("9.9.9")));
    assert!(notices.observe(None));
    assert_eq!(notices.pending(), None);
    notices.observe(Some("9.9.9"));
    notices.mark_shown();
    assert!(!notices.observe(Some("9.9.9")));
    // Pending versions from a previously installed build are revalidated.
    let old: UpdateNotifications = serde_json::from_value(serde_json::json!({
        "pending": crate::version::PUBLIC, "last_shown": null
    }))
    .unwrap();
    assert_eq!(old.pending(), None);
}

#[test]
fn daily_schedule_survives_restarts_failures_and_clock_changes() {
    let now = 1_794_310_000;
    assert!(daily_check_due(None, now));
    assert!(!daily_check_due(Some(now), now));
    assert!(!daily_check_due(Some(now), now + 86_399));
    assert!(daily_check_due(Some(now), now + 86_400));
    assert!(daily_check_due(Some(now), now + 7 * 86_400));
    // A backwards clock starts a fresh daily interval instead of suppressing
    // checks until a stale future timestamp is reached.
    assert!(daily_check_due(Some(now + 3600), now));
    assert!(daily_check_due(Some(i64::MIN), i64::MAX));
    // Persisting either a manual or automatic attempt has the same backoff,
    // regardless of whether the network result was success or failure.
    let saved = serde_json::to_string(&Some(now)).unwrap();
    let reloaded = serde_json::from_str(&saved).unwrap();
    assert!(!daily_check_due(reloaded, now + 60));
}
use zip::{ZipWriter, write::SimpleFileOptions};

fn release(version: &str, target: Target) -> Release {
    let stem = format!("ClickPulse-{version}-{}", target.platform());
    Release {
        tag_name: format!("v{version}"),
        html_url: format!("{RELEASES}/tag/v{version}"),
        draft: false,
        prerelease: false,
        assets: [
            stem.clone() + ".zip",
            stem + ".manifest.json",
            "SHA256SUMS.txt".into(),
        ]
        .into_iter()
        .map(|name| Asset {
            browser_download_url: format!("{RELEASES}/download/v{version}/{name}"),
            name,
            size: 1,
            digest: None,
        })
        .collect(),
    }
}
fn candidate() -> Candidate {
    Candidate::from_release(
        release("1.2.3", Target::WindowsX64),
        "1.2.2",
        Target::WindowsX64,
    )
    .unwrap()
    .unwrap()
}
#[test]
fn cached_download_identity_includes_every_release_asset() {
    let original = candidate();
    let mut reordered = release("1.2.3", Target::WindowsX64);
    reordered.assets.reverse();
    assert_eq!(
        Some(original.clone()),
        Candidate::from_release(reordered, "1.2.2", Target::WindowsX64).unwrap()
    );
    for index in 0..3 {
        let mut replaced = release("1.2.3", Target::WindowsX64);
        replaced.assets[index].digest = Some(format!("sha256:{}", "a".repeat(64)));
        let replaced = Candidate::from_release(replaced, "1.2.2", Target::WindowsX64)
            .unwrap()
            .unwrap();
        assert_eq!(replaced.version, original.version);
        assert_ne!(replaced, original, "replaced asset {index}");
    }
}
fn executable(version: &str) -> Vec<u8> {
    let mut bytes = vec![0; 256];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[0x3c..0x40].copy_from_slice(&128u32.to_le_bytes());
    bytes[128..132].copy_from_slice(b"PE\0\0");
    bytes[132..134].copy_from_slice(&0x8664u16.to_le_bytes());
    bytes.extend(version.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}
fn manifest() -> (Manifest, BTreeMap<String, Vec<u8>>) {
    let files: BTreeMap<String, Vec<u8>> = BTreeMap::from([
        ("ClickPulse.exe".into(), executable("1.2.3.70")),
        ("WebView2Loader.dll".into(), b"fixture loader".to_vec()),
        ("LICENSE.txt".into(), b"fixture license".to_vec()),
    ]);
    (
        Manifest {
            version: "1.2.3".into(),
            internal_build: 70,
            source_revision: "a".repeat(40),
            target: Target::WindowsX64.triple().into(),
            code_license: "GPL-3.0-or-later".into(),
            files: files
                .iter()
                .map(|(n, b)| (n.clone(), hash_bytes(b)))
                .collect(),
        },
        files,
    )
}
fn archive(
    path: &Path,
    manifest: &Manifest,
    files: &BTreeMap<String, Vec<u8>>,
    extra: Option<(&str, &[u8])>,
    link: bool,
) {
    let mut zip = ZipWriter::new(File::create(path).unwrap());
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);
    let prefix = "ClickPulse-1.2.3-windows-x64/";
    for (name, bytes) in files {
        zip.start_file(format!("{prefix}{name}"), options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.start_file(format!("{prefix}build-manifest.json"), options)
        .unwrap();
    zip.write_all(
        serde_json::to_string_pretty(manifest)
            .unwrap()
            .replace('\n', "\r\n")
            .as_bytes(),
    )
    .unwrap();
    if let Some((name, bytes)) = extra {
        if link {
            zip.add_symlink(name, "../../outside", options).unwrap();
        } else {
            zip.start_file(name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
    }
    zip.finish().unwrap();
}
#[test]
fn stable_release_selection_is_semantic_and_platform_specific() {
    for target in [Target::MacArm64, Target::MacX64, Target::WindowsX64] {
        let c = Candidate::from_release(release("0.2.10", target), "0.2.9", target)
            .unwrap()
            .unwrap();
        assert_eq!(c.version, "0.2.10");
        assert!(c.archive.name.contains(target.platform()));
        assert!(
            Candidate::from_release(release("0.2.9", target), "0.2.10", target)
                .unwrap()
                .is_none()
        );
        assert!(
            Candidate::from_release(release("0.2.9", target), "0.2.9", target)
                .unwrap()
                .is_none()
        );
    }
}
#[test]
fn release_metadata_cannot_redirect_to_another_repo_or_unsafe_version() {
    let target = Target::WindowsX64;
    for mode in 0..8 {
        let mut r = release("1.2.3", target);
        match mode {
            0 => r.draft = true,
            1 => r.prerelease = true,
            2 => r.html_url = "https://example.com/release".into(),
            3 => {
                r.assets[0].browser_download_url =
                    "http://github.com/endNone/ClickPulse/file".into()
            }
            4 => r.assets[0].browser_download_url = "https://github.com/other/repo/file".into(),
            5 => r.assets.push(r.assets[0].clone()),
            6 => r.assets[0].size = MAX_ARCHIVE + 1,
            _ => r.assets[0].digest = Some("sha256:bad".into()),
        }
        assert!(
            Candidate::from_release(r, "1.2.2", target).is_err(),
            "mode {mode}"
        );
    }
    for version in ["01.2.3", "1.2.3-beta", "1.2.3+build", "../1.2.3", "1.2.3\n"] {
        assert!(stable_version(version).is_err(), "{version}");
    }
}
#[test]
fn paths_checksums_and_response_limits_reject_ambiguous_input() {
    for path in [
        "",
        "../file",
        "a/../file",
        "/file",
        "C:/file",
        "a\\file",
        "a//b",
        "a/./b",
        "nul.txt",
        "COM1",
        "LPT9.ini",
        "folder/file.",
        "folder/file ",
        "a\0b",
        "CONIN$",
        "conout$.txt",
        "COM¹.txt",
        "LPT²",
        "a\nb",
        "a?b",
    ] {
        assert!(safe_relative(path).is_err(), "{path:?}");
    }
    assert!(safe_relative("Licenses/font-OFL.txt").is_ok());
    assert!(parse_sums(&format!("{}  file.zip\n", "a".repeat(64))).is_ok());
    for text in [
        format!("{}  ../file\n", "a".repeat(64)),
        format!("{}  file\n{}  file\n", "a".repeat(64), "b".repeat(64)),
        "bad  file\n".into(),
    ] {
        assert!(parse_sums(&text).is_err());
    }
    assert!(read_limited(&b"12345"[..], 4).is_err());
    assert_eq!(read_limited(&b"1234"[..], 4).unwrap(), b"1234");
}
#[test]
fn complete_verified_package_extracts_only_its_declared_files() {
    let temp = tempfile::tempdir().unwrap();
    let (m, files) = manifest();
    let zip = temp.path().join("package.zip");
    archive(&zip, &m, &files, None, false);
    let out = temp.path().join("staged");
    fs::create_dir(&out).unwrap();
    let got = unpack(&zip, &out, &candidate(), &serde_json::to_vec(&m).unwrap()).unwrap();
    assert_eq!(got.version, "1.2.3");
    assert_eq!(
        fs::read(out.join("payload/build-manifest.json")).unwrap(),
        serde_json::to_vec(&m).unwrap()
    );
    for (name, bytes) in files {
        assert_eq!(fs::read(out.join("payload").join(name)).unwrap(), bytes);
    }
}
#[test]
fn package_tampering_and_missing_files_fail_before_installation() {
    for mode in 0..6 {
        let temp = tempfile::tempdir().unwrap();
        let (mut m, mut files) = manifest();
        let external;
        match mode {
            0 => {
                files.get_mut("LICENSE.txt").unwrap().push(0);
                external = serde_json::to_vec(&m).unwrap();
            }
            1 => {
                files.remove("LICENSE.txt");
                external = serde_json::to_vec(&m).unwrap();
            }
            2 => {
                external = serde_json::to_vec(&m).unwrap();
                m.internal_build = 71;
            }
            3 => {
                m.target = Target::MacArm64.triple().into();
                external = serde_json::to_vec(&m).unwrap();
            }
            4 => {
                m.source_revision = "bad".into();
                external = serde_json::to_vec(&m).unwrap();
            }
            _ => {
                m.internal_build = 71;
                external = serde_json::to_vec(&m).unwrap();
            }
        }
        let zip = temp.path().join("package.zip");
        archive(&zip, &m, &files, None, false);
        let out = temp.path().join("staged");
        fs::create_dir(&out).unwrap();
        assert!(
            unpack(&zip, &out, &candidate(), &external).is_err(),
            "mode {mode}"
        );
    }
}
#[test]
fn traversal_links_extra_files_and_case_collisions_never_escape_staging() {
    for (path, link) in [
        ("../escape", false),
        ("/absolute", false),
        ("ClickPulse-1.2.3-windows-x64/../../escape", false),
        ("ClickPulse-1.2.3-windows-x64/extra.txt", false),
        ("ClickPulse-1.2.3-windows-x64/clickpulse.exe", false),
        ("ClickPulse-1.2.3-windows-x64/link", true),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let sentinel = temp.path().join("outside");
        fs::write(&sentinel, b"unchanged").unwrap();
        let (m, files) = manifest();
        let zip = temp.path().join("package.zip");
        archive(&zip, &m, &files, Some((path, b"malicious")), link);
        let out = temp.path().join("staged");
        fs::create_dir(&out).unwrap();
        assert!(
            unpack(&zip, &out, &candidate(), &serde_json::to_vec(&m).unwrap()).is_err(),
            "{path}"
        );
        assert_eq!(fs::read(sentinel).unwrap(), b"unchanged");
    }
}
#[test]
fn user_data_paths_are_never_valid_windows_program_payloads() {
    let (mut m, _) = manifest();
    for path in [
        "events.sqlite3",
        "presentation.json",
        "storage.json",
        "logs/clickpulse.log",
        "../ClickPulse.exe",
    ] {
        m.files.insert(path.into(), "a".repeat(64));
        assert!(validate_manifest(&m, &candidate()).is_err(), "{path}");
        m.files.remove(path);
    }
}

#[test]
fn oversized_expansion_is_rejected_before_writing_a_large_file() {
    let temp = tempfile::tempdir().unwrap();
    let (m, files) = manifest();
    let path = temp.path().join("large.zip");
    archive(&path, &m, &files, None, false);
    let mut bytes = fs::read(&path).unwrap();
    let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
    let local = bytes.windows(4).position(|b| b == b"PK\x03\x04").unwrap();
    let size = ((MAX_EXPANDED + 1) as u32).to_le_bytes();
    bytes[central + 24..central + 28].copy_from_slice(&size);
    bytes[local + 22..local + 26].copy_from_slice(&size);
    fs::write(&path, bytes).unwrap();
    let out = temp.path().join("staged");
    fs::create_dir(&out).unwrap();
    let error = unpack(&path, &out, &candidate(), &serde_json::to_vec(&m).unwrap()).unwrap_err();
    assert!(error.to_string().contains("too large"), "{error:#}");
    assert!(!out.join("payload/ClickPulse.exe").exists());
}

#[test]
#[ignore = "Downloads and validates a public package in a disposable staging directory"]
fn published_release_downloads_and_verifies_without_installing() {
    for target in [Target::MacArm64, Target::MacX64, Target::WindowsX64] {
        let candidate = check("0.0.0", target).unwrap().unwrap();
        let staged = stage(&candidate, |_, _| {}).unwrap();
        assert_eq!(staged.manifest.version, candidate.version);
        assert!(
            staged
                .directory
                .path()
                .join("payload")
                .join(target.binary())
                .is_file()
        );
    }
}
