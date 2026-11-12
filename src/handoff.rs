//! Replace only the user's identified collector, after effective App permission.
use crate::collector::{CollectorDb, data_path};
use anyhow::{Context, Result, bail, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const OLD_LABEL: &str = "com.zhangwuhan.clickpulse";
const APP_LABEL: &str = "app.clickpulse.desktop.login";
fn home() -> Result<PathBuf> {
    Ok(PathBuf::from(
        std::env::var_os("HOME").context("Cannot determine the user home directory")?,
    ))
}
fn agent(label: &str) -> Result<PathBuf> {
    Ok(home()?.join(format!("Library/LaunchAgents/{label}.plist")))
}
fn domain() -> Result<String> {
    let out = Command::new("/usr/bin/id").arg("-u").output()?;
    ensure!(out.status.success(), "Cannot determine the logged-in user");
    Ok(format!("gui/{}", String::from_utf8(out.stdout)?.trim()))
}
fn launchctl(args: &[&str]) -> Result<()> {
    let out = Command::new("/bin/launchctl").args(args).output()?;
    ensure!(
        out.status.success(),
        "Failed to update login startup: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(())
}
fn app_path() -> Result<PathBuf> {
    let executable = std::env::current_exe()?.canonicalize()?;
    let app = executable
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .context("Run this operation from the packaged ClickPulse.app")?;
    ensure!(
        app.extension().is_some_and(|e| e == "app"),
        "Run this operation from the packaged ClickPulse.app"
    );
    Ok(app.to_path_buf())
}

fn handoff_lock() -> Result<std::fs::File> {
    let path = home()?.join("Library/Application Support/ClickPulse/handoff.lock");
    std::fs::create_dir_all(path.parent().unwrap())?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    file.try_lock()
        .context("Recording handoff is in progress; try opening the app again shortly")?;
    Ok(file)
}
fn pending_path() -> Result<PathBuf> {
    Ok(home()?.join("Library/Application Support/ClickPulse/pending-handoff.json"))
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Pending {
    original: PathBuf,
    parked: PathBuf,
}
struct ParkedScript {
    pending: Pending,
    _lock: std::fs::File,
}
impl ParkedScript {
    fn new(original: &Path, parked: &Path) -> Result<Self> {
        let lock = handoff_lock()?;
        ensure!(
            !parked.exists() && !pending_path()?.exists(),
            "An unfinished recording handoff needs recovery; reopen the app"
        );
        let pending = Pending {
            original: original.into(),
            parked: parked.into(),
        };
        use std::io::Write;
        let mut file = std::fs::File::create_new(pending_path()?)?;
        file.write_all(&serde_json::to_vec_pretty(&pending)?)?;
        file.sync_all()?;
        if let Err(error) = std::fs::rename(original, parked) {
            let _ = std::fs::remove_file(pending_path()?);
            return Err(error.into());
        }
        Ok(Self {
            pending,
            _lock: lock,
        })
    }
    fn restore(&mut self) -> Result<()> {
        restore_script(&self.pending)
    }
    fn finish(&self) -> Result<()> {
        std::fs::remove_file(pending_path()?)?;
        Ok(())
    }
}
impl Drop for ParkedScript {
    fn drop(&mut self) {
        if let Err(e) = self.restore() {
            eprintln!("Failed to restore the previous recorder script: {e:#}");
        }
    }
}
fn restore_script(pending: &Pending) -> Result<()> {
    if pending.parked.exists() {
        ensure!(
            !pending.original.exists(),
            "Another file occupies the previous script path; the staged file was retained"
        );
        std::fs::hard_link(&pending.parked, &pending.original)?;
        std::fs::remove_file(&pending.parked)?;
    }
    ensure!(
        pending.original.is_file(),
        "The previous recorder script could not be restored"
    );
    Ok(())
}

/// Recover an interrupted handoff on desktop startup. No arbitrary paths from
/// a JSON file are used: only the previously identified collector is accepted.
pub fn recover_pending() -> Result<()> {
    if !pending_path()?.exists() {
        return Ok(());
    }
    let _lock = handoff_lock()?;
    let pending: Pending = serde_json::from_slice(&std::fs::read(pending_path()?)?)?;
    let expected = home()?.join("code/individual/plugins/clickpulse/log.py");
    ensure!(
        pending.original == expected
            && pending.parked.parent() == expected.parent()
            && pending
                .parked
                .file_name()
                .is_some_and(|p| p.to_string_lossy().starts_with(".clickpulse-handoff-")),
        "Invalid handoff recovery path; the original file was not modified"
    );
    restore_script(&pending)?;
    let output = data_path()?;
    if !output.exists() {
        rollback(&output)?;
    } else {
        CollectorDb::open(&output)?;
        set_auto_start(true)?;
        let old = agent(OLD_LABEL)?;
        if old.exists() {
            let retired = home()?.join("Library/Application Support/ClickPulse/retired");
            std::fs::create_dir_all(&retired)?;
            std::fs::rename(&old, retired.join(format!("{OLD_LABEL}.plist")))?;
        }
    }
    std::fs::remove_file(pending_path()?)?;
    Ok(())
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
pub fn auto_start_enabled() -> bool {
    agent(APP_LABEL).is_ok_and(|p| p.is_file())
}

pub fn set_auto_start(enabled: bool) -> Result<()> {
    let path = agent(APP_LABEL)?;
    let service = format!("{}/{}", domain()?, APP_LABEL);
    if enabled {
        let previous = if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            ensure!(
                contents.contains(APP_LABEL),
                "Conflicting login startup configuration name; check the original file"
            );
            Some(contents)
        } else {
            None
        };
        let app = app_path()?;
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>{APP_LABEL}</string><key>ProgramArguments</key><array><string>/usr/bin/open</string><string>-g</string><string>-j</string><string>{}</string><string>--args</string><string>--background</string></array><key>RunAtLoad</key><true/><key>KeepAlive</key><false/><key>ProcessType</key><string>Interactive</string></dict></plist>\n",
            xml(&app.to_string_lossy())
        );
        std::fs::create_dir_all(path.parent().unwrap())?;
        let temp = path.with_extension("plist.tmp");
        std::fs::write(&temp, content)?;
        let checked = Command::new("/usr/bin/plutil")
            .arg("-lint")
            .arg(&temp)
            .output()?;
        ensure!(
            checked.status.success(),
            "Login startup configuration validation failed"
        );
        let _ = launchctl(&["bootout", &service]);
        std::fs::rename(temp, &path)?;
        activate_with_plist_rollback(&path, previous.as_deref(), || {
            launchctl(&["enable", &service])?;
            launchctl(&[
                "bootstrap",
                &domain()?,
                path.to_str().context("Invalid application path")?,
            ])
        })?;
    } else if path.exists() {
        let _ = launchctl(&["bootout", &service]);
        let retired = home()?.join("Library/Application Support/ClickPulse/retired");
        std::fs::create_dir_all(&retired)?;
        std::fs::rename(
            path,
            retired.join(format!(
                "{APP_LABEL}-{}.plist",
                chrono::Local::now().timestamp_millis()
            )),
        )?;
    }
    Ok(())
}

fn activate_with_plist_rollback(
    path: &Path,
    previous: Option<&str>,
    activate: impl FnOnce() -> Result<()>,
) -> Result<()> {
    activate().inspect_err(|_| {
        // Preserve the activation error even if restoring the file also fails.
        let _ = match previous {
            Some(contents) => std::fs::write(path, contents),
            None => std::fs::remove_file(path),
        };
    })
}

/// The old agent is restored on every failure before the new runtime starts.
pub fn take_over(source: &Path) -> Result<PathBuf> {
    ensure!(
        crate::capture::has_permission(),
        "The app lacks Input Monitoring permission; the previous recorder remains active"
    );
    let output = data_path()?;
    ensure!(
        !output.exists(),
        "The app database already exists; recording cannot be handed over again"
    );
    let old = agent(OLD_LABEL)?;
    ensure!(
        old.is_file(),
        "The confirmed login startup configuration is missing; the previous recorder was not modified"
    );
    let args = Command::new("/usr/bin/plutil")
        .args(["-extract", "ProgramArguments", "json", "-o", "-"])
        .arg(&old)
        .output()?;
    let programs: Vec<String> = serde_json::from_slice(&args.stdout)
        .context("Cannot read the previous recorder launch path")?;
    let script = source
        .parent()
        .context("Invalid source database path")?
        .join("log.py")
        .canonicalize()?;
    ensure!(
        programs.len() == 2 && Path::new(&programs[1]).canonicalize()? == script,
        "The startup configuration does not match the confirmed data source; the previous recorder was not modified"
    );
    let service = format!("{}/{}", domain()?, OLD_LABEL);
    let state = Command::new("/bin/launchctl")
        .args(["print", &service])
        .output()?;
    ensure!(
        state.status.success(),
        "The previous recorder is not running under the confirmed startup service; handoff was cancelled"
    );
    let text = String::from_utf8(state.stdout)?;
    let pid = text
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pid = ")
                .and_then(|s| s.parse::<u32>().ok())
        })
        .context("Cannot verify the previous recorder process; handoff was cancelled")?;
    // disable prevents future loading, but does not suppress a loaded KeepAlive
    // job. Temporarily park its script while the already-loaded Python process
    // drains; respawns cannot start a second collector. Restore the exact inode
    // after unloading the job, including on every error path.
    let parked_path = script.with_file_name(format!(
        ".clickpulse-handoff-{}.py",
        chrono::Local::now().timestamp_millis()
    ));
    let mut parked = ParkedScript::new(&script, &parked_path)?;
    let result = (|| {
        launchctl(&["disable", &service])?;
        let signal = Command::new("/bin/kill")
            .args(["-INT", &pid.to_string()])
            .status()?;
        ensure!(
            signal.success(),
            "Cannot stop the previous recorder normally"
        );
        let began = Instant::now();
        while Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .output()?
            .status
            .success()
        {
            if began.elapsed() > Duration::from_secs(10) {
                bail!("The previous recorder has not finished saving; handoff was cancelled");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        launchctl(&["bootout", &service])?;
        let state = Command::new("/bin/launchctl")
            .args(["print", &service])
            .output()?;
        ensure!(
            !state.status.success(),
            "The previous recording service is still running; handoff was cancelled"
        );
        ensure!(
            crate::capture::has_permission(),
            "Input Monitoring permission changed; the previous recorder was restored"
        );
        parked.restore()?;
        let baseline = CollectorDb::migrate(source, &output)?;
        set_auto_start(true)?;
        let retired = home()?.join("Library/Application Support/ClickPulse/retired");
        std::fs::create_dir_all(&retired)?;
        // A manifest gives future sessions the exact baseline and original paths.
        let manifest = serde_json::json!({"source":source,"destination":output,"baseline":baseline.lifetime,"days":baseline.history_days.len(),"completed_at":chrono::Local::now().to_rfc3339(),"original_agent":old});
        std::fs::write(
            retired.join("handoff.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        std::fs::rename(&old, retired.join(format!("{OLD_LABEL}.plist")))?;
        Ok(output.clone())
    })();
    parked.restore()?;
    if result.is_err() {
        rollback(&output)?;
    }
    parked.finish()?;
    result
}

pub fn rollback(output: &Path) -> Result<()> {
    let retired = home()?.join("Library/Application Support/ClickPulse/retired");
    std::fs::create_dir_all(&retired)?;
    if output.exists() {
        std::fs::rename(
            output,
            retired.join(format!(
                "failed-migration-{}.sqlite3",
                chrono::Local::now().timestamp_millis()
            )),
        )?;
    }
    let old = agent(OLD_LABEL)?;
    if !old.exists() && retired.join(format!("{OLD_LABEL}.plist")).exists() {
        std::fs::rename(retired.join(format!("{OLD_LABEL}.plist")), &old)?;
    }
    let _ = set_auto_start(false);
    let service = format!("{}/{}", domain()?, OLD_LABEL);
    launchctl(&["enable", &service])?;
    let _ = launchctl(&[
        "bootstrap",
        &domain()?,
        old.to_str()
            .context("Invalid original configuration path")?,
    ]);
    launchctl(&["kickstart", &service])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_auto_start_restores_previous_plist_or_removes_new_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("login.plist");
        for previous in [None, Some("previous plist contents")] {
            for failure in ["enable failed", "bootstrap failed"] {
                std::fs::write(&path, "new plist contents").unwrap();
                let error =
                    activate_with_plist_rollback(&path, previous, || bail!(failure)).unwrap_err();
                assert_eq!(error.to_string(), failure);
                match previous {
                    Some(contents) => assert_eq!(std::fs::read_to_string(&path).unwrap(), contents),
                    None => assert!(!path.exists()),
                }
            }
        }
    }

    #[test]
    fn successful_auto_start_keeps_new_plist() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("login.plist");
        std::fs::write(&path, "new plist contents").unwrap();
        activate_with_plist_rollback(&path, Some("previous plist contents"), || Ok(())).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "new plist contents");
    }
}
