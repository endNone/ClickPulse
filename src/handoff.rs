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
        std::env::var_os("HOME").context("无法确定用户目录")?,
    ))
}
fn agent(label: &str) -> Result<PathBuf> {
    Ok(home()?.join(format!("Library/LaunchAgents/{label}.plist")))
}
fn domain() -> Result<String> {
    let out = Command::new("/usr/bin/id").arg("-u").output()?;
    ensure!(out.status.success(), "无法确定登录用户");
    Ok(format!("gui/{}", String::from_utf8(out.stdout)?.trim()))
}
fn launchctl(args: &[&str]) -> Result<()> {
    let out = Command::new("/bin/launchctl").args(args).output()?;
    ensure!(
        out.status.success(),
        "更新自动启动失败：{}",
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
        .context("请从打包的 ClickPulse.app 操作")?;
    ensure!(
        app.extension().is_some_and(|e| e == "app"),
        "请从打包的 ClickPulse.app 操作"
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
    file.try_lock().context("App 正在交接记录，请稍后再打开")?;
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
            "还有未恢复的交接状态，请重新打开 App"
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
            eprintln!("原脚本恢复失败：{e:#}");
        }
    }
}
fn restore_script(pending: &Pending) -> Result<()> {
    if pending.parked.exists() {
        ensure!(
            !pending.original.exists(),
            "原脚本路径已被其他文件占用，已保留暂存文件"
        );
        std::fs::hard_link(&pending.parked, &pending.original)?;
        std::fs::remove_file(&pending.parked)?;
    }
    ensure!(pending.original.is_file(), "原脚本未能恢复");
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
        "交接恢复路径无效，原文件未改动"
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
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            ensure!(
                contents.contains(APP_LABEL),
                "自动启动配置名称冲突，请检查原文件"
            );
        }
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
        ensure!(checked.status.success(), "自动启动配置校验失败");
        let _ = launchctl(&["bootout", &service]);
        std::fs::rename(temp, &path)?;
        launchctl(&["enable", &service])?;
        launchctl(&[
            "bootstrap",
            &domain()?,
            path.to_str().context("应用路径无效")?,
        ])?;
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

/// The old agent is restored on every failure before the new runtime starts.
pub fn take_over(source: &Path) -> Result<PathBuf> {
    ensure!(
        crate::capture::has_permission(),
        "App 尚未获得有效输入监控权限，原记录器未停用"
    );
    let output = data_path()?;
    ensure!(!output.exists(), "App 数据库已经存在，不能再次接管");
    let old = agent(OLD_LABEL)?;
    ensure!(
        old.is_file(),
        "找不到此前确认的自动启动配置，原记录器未改动"
    );
    let args = Command::new("/usr/bin/plutil")
        .args(["-extract", "ProgramArguments", "json", "-o", "-"])
        .arg(&old)
        .output()?;
    let programs: Vec<String> =
        serde_json::from_slice(&args.stdout).context("无法读取原记录器启动路径")?;
    let script = source
        .parent()
        .context("原数据库路径无效")?
        .join("log.py")
        .canonicalize()?;
    ensure!(
        programs.len() == 2 && Path::new(&programs[1]).canonicalize()? == script,
        "启动配置与已确认的数据来源不一致，原记录器未改动"
    );
    let service = format!("{}/{}", domain()?, OLD_LABEL);
    let state = Command::new("/bin/launchctl")
        .args(["print", &service])
        .output()?;
    ensure!(
        state.status.success(),
        "原记录器未由已确认的启动服务运行，接管已取消"
    );
    let text = String::from_utf8(state.stdout)?;
    let pid = text
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pid = ")
                .and_then(|s| s.parse::<u32>().ok())
        })
        .context("无法确认原记录器进程，接管已取消")?;
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
        ensure!(signal.success(), "无法正常停止原记录器");
        let began = Instant::now();
        while Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .output()?
            .status
            .success()
        {
            if began.elapsed() > Duration::from_secs(10) {
                bail!("原记录器尚未保存完毕，接管已取消");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        launchctl(&["bootout", &service])?;
        let state = Command::new("/bin/launchctl")
            .args(["print", &service])
            .output()?;
        ensure!(!state.status.success(), "原记录服务仍未停止，接管已取消");
        ensure!(
            crate::capture::has_permission(),
            "App 的输入监控权限发生变化，已恢复原记录器"
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
        old.to_str().context("原配置路径无效")?,
    ]);
    launchctl(&["kickstart", &service])?;
    Ok(())
}
