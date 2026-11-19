//! Freedesktop integration; all startup configuration belongs to the current user.
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

pub fn xdg_dir(
    value: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
    fallback: &str,
) -> Result<PathBuf> {
    if let Some(value) = value.filter(|v| !v.is_empty()) {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(path);
        }
    }
    let home = PathBuf::from(home.context("HOME is unavailable")?);
    ensure!(home.is_absolute(), "HOME must be an absolute path");
    Ok(home.join(fallback))
}
fn startup_path() -> Result<PathBuf> {
    Ok(xdg_dir(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
        ".config",
    )?
    .join("autostart/app.clickpulse.desktop.desktop"))
}
const MARKER: &str = "X-ClickPulse-Managed=true";
fn desktop_entry(executable: &Path) -> Result<String> {
    let executable = executable
        .to_str()
        .context("Executable path must be UTF-8")?;
    ensure!(
        !executable.contains(['\n', '\r', '\0']),
        "Executable path contains a line break"
    );
    // Exec quoting, then desktop-entry string escaping. Percent signs are not field codes.
    let mut quoted = String::new();
    for ch in executable.chars() {
        match ch {
            '%' => quoted.push_str("%%"),
            '\\' | '"' | '`' | '$' => {
                quoted.push('\\');
                quoted.push(ch);
            }
            _ => quoted.push(ch),
        }
    }
    let escaped = quoted.replace('\\', "\\\\");
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=ClickPulse\nExec=\"{escaped}\" --background\nTerminal=false\n{MARKER}\n"
    ))
}
pub fn auto_start_enabled() -> bool {
    let (Ok(path), Ok(executable)) = (startup_path(), std::env::current_exe()) else {
        return false;
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|v| startup_executable(&v))
        .is_some_and(|saved| crate::platform::same_executable(&saved, &executable))
}

// Parse only our own quoted, argument-free executable followed by --background.
// Do not interpret arbitrary shell commands or third-party desktop entries.
fn startup_executable(contents: &str) -> Option<PathBuf> {
    if !contents.lines().any(|line| line == MARKER)
        || contents.lines().any(|line| line == "Hidden=true")
    {
        return None;
    }
    let exec = contents
        .lines()
        .find_map(|line| line.strip_prefix("Exec="))?;
    let quoted = exec.strip_prefix('"')?.strip_suffix("\" --background")?;
    let mut decoded = String::new();
    let mut chars = quoted.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => {
                if chars.next()? != '%' {
                    return None;
                }
                decoded.push('%');
            }
            '\\' => {
                if chars.next()? != '\\' {
                    return None;
                }
                let escaped = chars.next()?;
                if escaped == '\\' {
                    if chars.next()? != '\\' {
                        return None;
                    }
                } else if !matches!(escaped, '"' | '`' | '$') {
                    return None;
                }
                decoded.push(escaped);
            }
            '"' | '\n' | '\r' | '\0' => return None,
            _ => decoded.push(ch),
        }
    }
    let path = PathBuf::from(decoded);
    path.is_absolute().then_some(path)
}

pub fn repair_auto_start() -> Result<bool> {
    repair_entry(&startup_path()?, &std::env::current_exe()?)
}

fn repair_entry(path: &Path, executable: &Path) -> Result<bool> {
    if !path.is_file() || path.is_symlink() {
        return Ok(false);
    }
    let contents = std::fs::read_to_string(path)?;
    let Some(saved) = startup_executable(&contents) else {
        return Ok(false);
    };
    if saved.try_exists()? {
        return Ok(false);
    }
    write_entry(path, &desktop_entry(executable)?)?;
    Ok(true)
}

fn write_entry(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().context("Startup directory missing")?;
    std::fs::create_dir_all(parent)?;
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(content.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path)?;
    Ok(())
}
pub fn set_auto_start(enabled: bool) -> Result<()> {
    let path = startup_path()?;
    ensure!(!path.is_symlink(), "Startup entry is a symbolic link");
    if path.exists() {
        ensure!(
            std::fs::read_to_string(&path)?
                .lines()
                .any(|line| line == MARKER),
            "Existing startup entry is not managed by ClickPulse"
        );
    }
    if enabled {
        let content = desktop_entry(&std::env::current_exe()?)?;
        write_entry(&path, &content)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xdg_relative_or_empty_values_use_absolute_home() {
        let directory = tempfile::tempdir().unwrap();
        let home = directory.path().join("home");
        for value in [None, Some("".into()), Some("relative".into())] {
            assert_eq!(
                xdg_dir(value, Some(home.clone().into_os_string()), ".local/share").unwrap(),
                home.join(".local/share")
            );
        }
        assert!(xdg_dir(None, Some("relative".into()), ".config").is_err());
        let data = directory.path().join("data");
        assert_eq!(
            xdg_dir(Some(data.clone().into_os_string()), None, ".config").unwrap(),
            data
        );
    }
    #[test]
    fn startup_escapes_exec_metacharacters_and_rejects_newlines() {
        let temp = tempfile::tempdir().unwrap();
        let quoted_path = temp.path().join("a b/%u$foo\"/clickpulse");
        let entry = desktop_entry(&quoted_path).unwrap();
        assert!(entry.contains("%%u\\\\$foo\\\\\""));
        assert!(!entry.contains("sh -c"));
        assert!(entry.contains(" --background\n"));
        assert!(desktop_entry(Path::new("/bad\nExec=evil")).is_err());
        for path in [quoted_path, temp.path().join("a\\b/ClickPulse")] {
            assert_eq!(
                startup_executable(&desktop_entry(&path).unwrap()),
                Some(path)
            );
        }
    }
    #[test]
    fn renamed_installation_repairs_only_a_missing_managed_target() {
        let temp = tempfile::tempdir().unwrap();
        let old = temp.path().join("old");
        std::fs::create_dir(&old).unwrap();
        std::fs::write(old.join("ClickPulse"), "binary").unwrap();
        let entry = temp.path().join("startup.desktop");
        std::fs::write(&entry, desktop_entry(&old.join("ClickPulse")).unwrap()).unwrap();
        let new = temp.path().join("new");
        assert!(!repair_entry(&entry, &new.join("ClickPulse")).unwrap());
        std::fs::rename(old, &new).unwrap();
        assert!(repair_entry(&entry, &new.join("ClickPulse")).unwrap());
        assert_eq!(
            startup_executable(&std::fs::read_to_string(&entry).unwrap()),
            Some(new.join("ClickPulse"))
        );
        assert!(
            !repair_entry(&temp.path().join("absent.desktop"), &new.join("ClickPulse")).unwrap()
        );
        std::fs::write(&entry, "[Desktop Entry]\nExec=/other/app\n").unwrap();
        assert!(!repair_entry(&entry, &new.join("ClickPulse")).unwrap());
    }
}
