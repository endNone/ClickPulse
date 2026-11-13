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
    startup_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .is_some_and(|v| {
            v.lines().any(|line| line == MARKER) && !v.lines().any(|line| line == "Hidden=true")
        })
}
pub fn set_auto_start(enabled: bool) -> Result<()> {
    let path = startup_path()?;
    if path.exists() {
        ensure!(!path.is_symlink(), "Startup entry is a symbolic link");
        ensure!(
            std::fs::read_to_string(&path)?
                .lines()
                .any(|line| line == MARKER),
            "Existing startup entry is not managed by ClickPulse"
        );
    }
    if enabled {
        let content = desktop_entry(&std::env::current_exe()?)?;
        let parent = path.parent().context("Startup directory missing")?;
        std::fs::create_dir_all(parent)?;
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(content.as_bytes())?;
        file.as_file().sync_all()?;
        file.persist(path)?;
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
        let entry = desktop_entry(Path::new("/home/test/a b/%u$foo\"/clickpulse")).unwrap();
        assert!(entry.contains("%%u\\\\$foo\\\\\""));
        assert!(!entry.contains("sh -c"));
        assert!(entry.contains(" --background\n"));
        assert!(desktop_entry(Path::new("/bad\nExec=evil")).is_err());
    }
}
