//! User-initiated updates from this project's public releases. No startup polling.
use anyhow::{Context, Result, bail, ensure};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

mod install;
pub use install::{
    ApplyGuard, Installation, Notice, apply_update, installation, launch_installer, take_notice,
};

pub const RELEASES: &str = "https://github.com/endNone/ClickPulse/releases";
const API: &str = "https://api.github.com/repos/endNone/ClickPulse/releases/latest";
const MAX_ARCHIVE: u64 = 128 * 1024 * 1024;
const MAX_EXPANDED: u64 = 512 * 1024 * 1024;
const MAX_FILES: usize = 2048;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Target {
    MacArm64,
    MacX64,
    WindowsX64,
}
impl Target {
    pub fn current() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => Ok(Self::MacArm64),
            ("macos", "x86_64") => Ok(Self::MacX64),
            ("windows", "x86_64") => Ok(Self::WindowsX64),
            _ => bail!("This platform has no compatible update package"),
        }
    }
    pub fn platform(self) -> &'static str {
        match self {
            Self::MacArm64 => "macos-arm64",
            Self::MacX64 => "macos-x64",
            Self::WindowsX64 => "windows-x64",
        }
    }
    fn triple(self) -> &'static str {
        match self {
            Self::MacArm64 => "aarch64-apple-darwin",
            Self::MacX64 => "x86_64-apple-darwin",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }
    fn mac(self) -> bool {
        self != Self::WindowsX64
    }
    fn binary(self) -> &'static str {
        if self.mac() {
            "ClickPulse.app/Contents/MacOS/clickpulse"
        } else {
            "ClickPulse.exe"
        }
    }
}

fn stable_version(value: &str) -> Result<Version> {
    let parsed = Version::parse(value)?;
    ensure!(
        parsed.pre.is_empty() && parsed.build.is_empty(),
        "Only stable numeric versions are supported"
    );
    Ok(parsed)
}

#[derive(Debug, Clone, Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Candidate {
    pub version: String,
    pub release_url: String,
    target: Target,
    archive: Asset,
    manifest: Asset,
    sums: Asset,
}
impl Candidate {
    fn from_release(release: Release, current: &str, target: Target) -> Result<Option<Self>> {
        ensure!(
            !release.draft && !release.prerelease,
            "Release is not public and stable"
        );
        let version = release
            .tag_name
            .strip_prefix('v')
            .context("Invalid release tag")?;
        let latest = stable_version(version)?;
        ensure!(
            release.html_url == format!("{RELEASES}/tag/v{version}"),
            "Unexpected release repository"
        );
        if latest <= stable_version(current)? {
            return Ok(None);
        }
        let stem = format!("ClickPulse-{version}-{}", target.platform());
        let find = |name: &str, limit: u64| -> Result<Asset> {
            let matches: Vec<_> = release.assets.iter().filter(|a| a.name == name).collect();
            ensure!(
                matches.len() == 1,
                "Missing or duplicate update asset: {name}"
            );
            let asset = matches[0];
            ensure!(
                asset.size > 0 && asset.size <= limit,
                "Invalid update asset size"
            );
            ensure!(
                asset.browser_download_url == format!("{RELEASES}/download/v{version}/{name}"),
                "Unexpected update URL"
            );
            if let Some(digest) = &asset.digest {
                ensure!(
                    digest.strip_prefix("sha256:").is_some_and(valid_hash),
                    "Invalid asset digest"
                );
            }
            Ok(asset.clone())
        };
        Ok(Some(Self {
            version: version.to_owned(),
            release_url: release.html_url.clone(),
            target,
            archive: find(&(stem.clone() + ".zip"), MAX_ARCHIVE)?,
            manifest: find(&(stem + ".manifest.json"), 1024 * 1024)?,
            sums: find("SHA256SUMS.txt", 65536)?,
        }))
    }
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(5)
        .timeout_global(Some(timeout))
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_per_read(Some(Duration::from_secs(30)))
        .user_agent(format!("ClickPulse/{}", crate::version::PUBLIC))
        .build()
        .into()
}
fn read_limited(reader: impl Read, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Update response exceeds its size limit"
    );
    Ok(bytes)
}
fn fetch_small(client: &ureq::Agent, asset: &Asset) -> Result<Vec<u8>> {
    let mut response = client.get(&asset.browser_download_url).call()?;
    let bytes = read_limited(response.body_mut().as_reader(), asset.size)?;
    ensure!(
        bytes.len() as u64 == asset.size,
        "Incomplete update metadata"
    );
    if let Some(hash) = &asset.digest {
        ensure!(
            hash == &format!("sha256:{}", hash_bytes(&bytes)),
            "Metadata checksum mismatch"
        );
    }
    Ok(bytes)
}
pub fn check(current: &str, target: Target) -> Result<Option<Candidate>> {
    let client = agent(Duration::from_secs(30));
    let mut response = client
        .get(API)
        .header("Accept", "application/vnd.github+json")
        .call()?;
    let bytes = read_limited(response.body_mut().as_reader(), 1024 * 1024)?;
    Candidate::from_release(serde_json::from_slice(&bytes)?, current, target)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub version: String,
    pub internal_build: u64,
    pub source_revision: String,
    pub target: String,
    pub code_license: String,
    pub files: BTreeMap<String, String>,
}
pub struct StagedUpdate {
    pub(crate) directory: tempfile::TempDir,
    pub(crate) manifest: Manifest,
    pub(crate) manifest_hash: String,
    pub(crate) target: Target,
}

pub fn stage(candidate: &Candidate, progress: impl Fn(u64, u64)) -> Result<StagedUpdate> {
    let base = cache_dir()?;
    fs::create_dir_all(&base)?;
    let directory = tempfile::Builder::new()
        .prefix("pending-")
        .tempdir_in(base)?;
    let client = agent(Duration::from_secs(900));
    let sums = fetch_small(&client, &candidate.sums)?;
    let hashes = parse_sums(std::str::from_utf8(&sums)?)?;
    let external = fetch_small(&client, &candidate.manifest)?;
    ensure!(
        hashes.get(&candidate.manifest.name) == Some(&hash_bytes(&external)),
        "Manifest checksum mismatch"
    );
    let expected = hashes
        .get(&candidate.archive.name)
        .context("Missing package checksum")?;
    if let Some(digest) = &candidate.archive.digest {
        ensure!(
            digest == &format!("sha256:{expected}"),
            "Conflicting package checksums"
        );
    }
    let archive_path = directory.path().join("package.zip");
    let mut response = client.get(&candidate.archive.browser_download_url).call()?;
    let mut reader = response.body_mut().as_reader();
    let mut file = File::create_new(&archive_path)?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut chunk = [0u8; 65536];
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        ensure!(
            bytes <= candidate.archive.size,
            "Package exceeds declared size"
        );
        file.write_all(&chunk[..n])?;
        hash.update(&chunk[..n]);
        progress(bytes, candidate.archive.size);
    }
    file.sync_all()?;
    drop(file);
    ensure!(
        bytes == candidate.archive.size && format!("{:x}", hash.finalize()) == *expected,
        "Package checksum mismatch or incomplete download"
    );
    let manifest = unpack(&archive_path, directory.path(), candidate, &external)?;
    Ok(StagedUpdate {
        directory,
        manifest,
        manifest_hash: hash_bytes(&external),
        target: candidate.target,
    })
}

fn parse_sums(text: &str) -> Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();
    for line in text.lines() {
        let (hash, name) = line.split_once("  ").context("Invalid checksum line")?;
        ensure!(
            valid_hash(hash) && safe_relative(name)?.components().count() == 1,
            "Invalid checksum entry"
        );
        ensure!(
            hashes.insert(name.to_owned(), hash.to_owned()).is_none(),
            "Duplicate checksum entry"
        );
    }
    Ok(hashes)
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn windows_payload(name: &str) -> bool {
    matches!(
        name,
        "ClickPulse.exe"
            | "WebView2Loader.dll"
            | "README.txt"
            | "LICENSE.txt"
            | "THIRD-PARTY-NOTICES.txt"
    ) || name
        .strip_prefix("Licenses/")
        .is_some_and(|p| !p.contains('/') && p.ends_with(".txt"))
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn hash_file(path: &Path) -> Result<String> {
    let mut reader = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub(crate) fn safe_relative(name: &str) -> Result<PathBuf> {
    ensure!(
        !name.is_empty()
            && !name.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
            && !name.chars().any(char::is_control)
            && !name.starts_with('/'),
        "Unsafe update path"
    );
    for part in name.trim_end_matches('/').split('/') {
        ensure!(
            !part.is_empty() && part != "." && part != ".." && !part.ends_with(['.', ' ']),
            "Unsafe update path component"
        );
        let base = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        ensure!(
            !["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$", "CLOCK$"].contains(&base.as_str())
                && !["COM", "LPT"].iter().any(|prefix| {
                    base.strip_prefix(prefix).is_some_and(|suffix| {
                        matches!(
                            suffix,
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                        )
                    })
                }),
            "Reserved update filename"
        );
    }
    let path = PathBuf::from(name);
    ensure!(
        path.components().all(|c| matches!(c, Component::Normal(_))),
        "Unsafe update path"
    );
    Ok(path)
}
fn validate_manifest(manifest: &Manifest, candidate: &Candidate) -> Result<()> {
    ensure!(
        manifest.version == candidate.version && manifest.target == candidate.target.triple(),
        "Package version or architecture mismatch"
    );
    ensure!(
        manifest.code_license == "GPL-3.0-or-later"
            && manifest.internal_build > 0
            && manifest.source_revision.len() == 40
            && manifest
                .source_revision
                .bytes()
                .all(|b| b.is_ascii_hexdigit()),
        "Invalid package provenance"
    );
    ensure!(
        !manifest.files.is_empty()
            && manifest.files.len() <= MAX_FILES
            && manifest.files.contains_key(candidate.target.binary()),
        "Invalid package file list"
    );
    for (name, hash) in &manifest.files {
        safe_relative(name)?;
        ensure!(valid_hash(hash), "Invalid file checksum");
        if candidate.target.mac() {
            ensure!(
                name.starts_with("ClickPulse.app/") || name == "Install Guide.txt",
                "Unexpected Mac payload"
            );
        } else {
            ensure!(windows_payload(name), "Unexpected Windows program file");
        }
    }
    Ok(())
}
fn unpack(
    archive: &Path,
    directory: &Path,
    candidate: &Candidate,
    external: &[u8],
) -> Result<Manifest> {
    let manifest: Manifest = serde_json::from_slice(external)?;
    validate_manifest(&manifest, candidate)?;
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    ensure!(zip.len() <= MAX_FILES + 128, "Too many archive entries");
    let stem = format!(
        "ClickPulse-{}-{}/",
        candidate.version,
        candidate.target.platform()
    );
    let output = directory.join("payload");
    fs::create_dir(&output)?;
    let mut names = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut expanded = 0u64;
    for i in 0..zip.len() {
        let mut member = zip.by_index(i)?;
        let name = member.name()?.into_owned();
        safe_relative(&name)?;
        ensure!(
            names.insert(name.trim_end_matches('/').to_lowercase()),
            "Duplicate or case-colliding archive path"
        );
        let kind = member.unix_mode().unwrap_or(0) & 0o170000;
        ensure!(
            kind == 0 || kind == 0o100000 || kind == 0o040000,
            "Archive links and special files are not allowed"
        );
        let relative = name.strip_prefix(&stem).context("Wrong archive root")?;
        if member.is_dir() {
            if !relative.is_empty() {
                fs::create_dir_all(output.join(safe_relative(relative)?))?;
            }
            continue;
        }
        let path = safe_relative(relative)?;
        expanded = expanded
            .checked_add(member.size())
            .context("Archive size overflow")?;
        ensure!(expanded <= MAX_EXPANDED, "Expanded package is too large");
        let dest = output.join(&path);
        fs::create_dir_all(dest.parent().context("Missing file parent")?)?;
        if relative == "build-manifest.json" {
            let bytes = read_limited(&mut member, 1024 * 1024)?;
            ensure!(
                serde_json::from_slice::<serde_json::Value>(&bytes)?
                    == serde_json::from_slice::<serde_json::Value>(external)?,
                "Embedded manifest differs from verified manifest"
            );
            // Windows packages use CRLF; the release service writes its verified
            // external copy with LF. Preserve that authenticated copy for the
            // helper's exact-byte handoff after comparing the complete JSON.
            fs::write(&dest, external)?;
        } else {
            let wanted = manifest
                .files
                .get(relative)
                .context("Undeclared package file")?;
            let mut file = File::create_new(&dest)?;
            let written = std::io::copy(&mut member.by_ref().take(MAX_EXPANDED + 1), &mut file)?;
            ensure!(written == member.size(), "Incomplete extracted file");
            file.sync_all()?;
            ensure!(
                hash_file(&dest)? == *wanted,
                "Extracted file checksum mismatch"
            );
            seen.insert(relative.to_owned());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                &dest,
                fs::Permissions::from_mode(member.unix_mode().unwrap_or(0o644) & 0o777),
            )?;
        }
    }
    ensure!(
        seen == manifest.files.keys().cloned().collect(),
        "Package is missing declared files"
    );
    ensure!(
        output.join("build-manifest.json").is_file(),
        "Missing embedded manifest"
    );
    install::validate_program(
        &output,
        candidate.target,
        &manifest.version,
        manifest.internal_build,
    )?;
    Ok(manifest)
}
pub fn cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Ok(
            PathBuf::from(std::env::var_os("HOME").context("Home directory unavailable")?)
                .join("Library/Caches/ClickPulse/updates"),
        )
    }
    #[cfg(target_os = "windows")]
    {
        Ok(crate::platform::data_dir()?.join("updates"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(std::env::temp_dir().join("clickpulse-updates"))
    }
}

#[cfg(test)]
mod tests;
