//! WinGet ownership comes from its registration, not a directory name.
use std::path::{Path, PathBuf};

pub(crate) const WINGET_UPDATE: &str = "winget upgrade --id endNone.ClickPulse --exact";
const PACKAGE_ID: &str = "endNone.ClickPulse";

#[derive(Debug)]
struct Registration {
    package_id: String,
    location: PathBuf,
    link: PathBuf,
}

#[derive(Debug)]
pub(crate) struct WingetInstallation {
    pub link: Option<PathBuf>,
}

pub(crate) fn same_program(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => {
            left == right
                || (cfg!(windows)
                    && left
                        .to_str()
                        .zip(right.to_str())
                        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b)))
        }
        _ => false,
    }
}

fn registered_installation(
    executable: &Path,
    records: &[Registration],
) -> Option<WingetInstallation> {
    let executable = executable.canonicalize().ok()?;
    if !executable
        .file_name()?
        .to_str()?
        .eq_ignore_ascii_case("ClickPulse.exe")
    {
        return None;
    }
    records.iter().find_map(|record| {
        if !record.package_id.eq_ignore_ascii_case(PACKAGE_ID) || !record.location.is_absolute() {
            return None;
        }
        let root = record.location.canonicalize().ok()?;
        let relative = executable.strip_prefix(root).ok()?;
        // Our archive has one versioned directory. Do not claim unrelated
        // portable copies nested elsewhere under a custom installation root.
        let parts: Vec<_> = relative.components().collect();
        if parts.len() != 1 {
            let folder = parts.first()?.as_os_str().to_str()?;
            let version = folder
                .strip_prefix("ClickPulse-")?
                .strip_suffix("-windows-x64")?;
            if parts.len() != 2 || semver::Version::parse(version).is_err() {
                return None;
            }
        }
        Some(WingetInstallation {
            link: same_program(&record.link, &executable).then(|| record.link.clone()),
        })
    })
}

#[cfg(windows)]
pub(crate) fn winget_installation(executable: &Path) -> anyhow::Result<Option<WingetInstallation>> {
    Ok(registered_installation(
        executable,
        &registry::registrations()?,
    ))
}

#[cfg(windows)]
mod registry {
    use super::*;
    use anyhow::{Result, ensure};
    use std::{
        ffi::OsStr,
        os::windows::ffi::{OsStrExt, OsStringExt},
        ptr::{null, null_mut},
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
        System::Registry::*,
    };

    const UNINSTALL: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }
    struct Key(HKEY);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }

    fn string(key: HKEY, name: &str) -> Result<Option<Vec<u16>>> {
        let name = wide(name);
        let mut size = 0;
        let result = unsafe {
            RegGetValueW(
                key,
                null(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                null_mut(),
                null_mut(),
                &mut size,
            )
        };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        ensure!(
            result == ERROR_SUCCESS && size <= 32768 && size % 2 == 0,
            "Invalid WinGet registration value"
        );
        let mut value = vec![0u16; size as usize / 2];
        let result = unsafe {
            RegGetValueW(
                key,
                null(),
                name.as_ptr(),
                RRF_RT_REG_SZ,
                null_mut(),
                value.as_mut_ptr().cast(),
                &mut size,
            )
        };
        ensure!(
            result == ERROR_SUCCESS,
            "Read WinGet registration: Windows error {result}"
        );
        value.truncate(size as usize / 2);
        while value.last() == Some(&0) {
            value.pop();
        }
        ensure!(!value.contains(&0), "Invalid WinGet registration string");
        Ok(Some(value))
    }

    fn read(root: HKEY, path: &str, links: &Path) -> Result<Vec<Registration>> {
        let mut key = null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                root,
                wide(path).as_ptr(),
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut key,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(Vec::new());
        }
        ensure!(
            status == ERROR_SUCCESS,
            "Read installed packages: Windows error {status}"
        );
        let key = Key(key);
        let mut records = Vec::new();
        for index in 0..4096 {
            let mut name = [0u16; 256];
            let mut size = name.len() as u32;
            let status = unsafe {
                RegEnumKeyExW(
                    key.0,
                    index,
                    name.as_mut_ptr(),
                    &mut size,
                    null(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                )
            };
            if status == ERROR_NO_MORE_ITEMS {
                return Ok(records);
            }
            ensure!(
                status == ERROR_SUCCESS,
                "Read installed package name: Windows error {status}"
            );
            if !String::from_utf16_lossy(&name[..size as usize])
                .to_ascii_lowercase()
                .starts_with("endnone.clickpulse_")
            {
                continue;
            }
            let mut child = null_mut();
            let status =
                unsafe { RegOpenKeyExW(key.0, name.as_ptr(), 0, KEY_QUERY_VALUE, &mut child) };
            if status == ERROR_FILE_NOT_FOUND {
                continue;
            }
            ensure!(
                status == ERROR_SUCCESS,
                "Read ClickPulse registration: Windows error {status}"
            );
            let child = Key(child);
            if let (Some(id), Some(location)) = (
                string(child.0, "WinGetPackageIdentifier")?,
                string(child.0, "InstallLocation")?,
            ) {
                records.push(Registration {
                    package_id: String::from_utf16_lossy(&id),
                    location: std::ffi::OsString::from_wide(&location).into(),
                    link: links.join("clickpulse.exe"),
                });
            }
        }
        anyhow::bail!("Too many installed package registrations")
    }

    pub(super) fn registrations() -> Result<Vec<Registration>> {
        let mut records = Vec::new();
        for (root, variable, suffix) in [
            (HKEY_CURRENT_USER, "LOCALAPPDATA", "Microsoft/WinGet/Links"),
            (HKEY_LOCAL_MACHINE, "ProgramFiles", "WinGet/Links"),
        ] {
            if let Some(base) = std::env::var_os(variable)
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
            {
                records.extend(read(root, UNINSTALL, &base.join(suffix))?);
            }
        }
        Ok(records)
    }

    #[cfg(test)]
    pub(crate) struct TestRegistration {
        path: String,
    }
    #[cfg(test)]
    impl TestRegistration {
        pub(crate) fn new(location: &Path) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = format!(
                "{UNINSTALL}\\endNone.ClickPulse_ClickPulseTest-{}-{unique}",
                std::process::id()
            );
            let mut key = null_mut();
            assert_eq!(
                unsafe {
                    RegCreateKeyExW(
                        HKEY_CURRENT_USER,
                        wide(&path).as_ptr(),
                        0,
                        null(),
                        0,
                        KEY_SET_VALUE,
                        null(),
                        &mut key,
                        null_mut(),
                    )
                },
                ERROR_SUCCESS
            );
            let key = Key(key);
            for (name, value) in [
                ("WinGetPackageIdentifier", OsStr::new(PACKAGE_ID)),
                ("InstallLocation", location.as_os_str()),
            ] {
                let data: Vec<u16> = value.encode_wide().chain(Some(0)).collect();
                assert_eq!(
                    unsafe {
                        RegSetValueExW(
                            key.0,
                            wide(name).as_ptr(),
                            0,
                            REG_SZ,
                            data.as_ptr().cast(),
                            (data.len() * 2) as u32,
                        )
                    },
                    ERROR_SUCCESS
                );
            }
            Self { path }
        }
    }
    #[cfg(test)]
    impl Drop for TestRegistration {
        fn drop(&mut self) {
            unsafe {
                RegDeleteTreeW(HKEY_CURRENT_USER, wide(&self.path).as_ptr());
            }
        }
    }
}

#[cfg(all(test, windows))]
pub(crate) use registry::TestRegistration;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn winget_ownership_requires_matching_registration_and_location() {
        let temp = tempfile::tempdir().unwrap();
        let location = temp.path().join("custom apps");
        let exe = location.join("ClickPulse-0.3.1-windows-x64/ClickPulse.exe");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"app").unwrap();
        let mut records = [Registration {
            package_id: PACKAGE_ID.into(),
            location,
            link: temp.path().join("Links/clickpulse.exe"),
        }];
        assert!(registered_installation(&exe, &records).is_some());
        assert!(
            registered_installation(&exe, &records)
                .unwrap()
                .link
                .is_none()
        );
        records[0].package_id = "another.ClickPulse".into();
        assert!(registered_installation(&exe, &records).is_none());
        records[0].package_id = PACKAGE_ID.into();
        for name in [
            "WinGet/Packages/endNone.ClickPulse/ClickPulse.exe",
            "custom apps/backup/ClickPulse-0.3.1-windows-x64/ClickPulse.exe",
        ] {
            let copy = temp.path().join(name);
            fs::create_dir_all(copy.parent().unwrap()).unwrap();
            fs::copy(&exe, &copy).unwrap();
            assert!(registered_installation(&copy, &records).is_none());
        }
        assert!(!same_program(
            &temp.path().join("missing-a"),
            &temp.path().join("missing-b")
        ));
    }

    #[test]
    fn stable_entry_must_resolve_to_this_program_and_survives_version_changes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("packages");
        fs::create_dir(&root).unwrap();
        let link = temp.path().join("clickpulse.exe");
        let record = [Registration {
            package_id: PACKAGE_ID.into(),
            location: root.clone(),
            link: link.clone(),
        }];
        for version in ["0.3.1", "0.3.2"] {
            let exe = root.join(format!("ClickPulse-{version}-windows-x64/ClickPulse.exe"));
            fs::create_dir_all(exe.parent().unwrap()).unwrap();
            fs::write(&exe, b"app").unwrap();
            if link.exists() {
                fs::remove_file(&link).unwrap();
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(&exe, &link).unwrap();
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&exe, &link).unwrap();
            assert_eq!(
                registered_installation(&exe, &record).unwrap().link,
                Some(link.clone())
            );
        }
        let other = temp.path().join("other.exe");
        fs::write(&other, b"other").unwrap();
        assert!(!same_program(&link, &other));
        assert_eq!(
            WINGET_UPDATE,
            "winget upgrade --id endNone.ClickPulse --exact"
        );
    }
}
