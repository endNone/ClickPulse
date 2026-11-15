//! Keep save failures visible even when database snapshots cannot be read.
use super::{VisibleWebView, tray_icon};
use clickpulse::{localization::Locale, runtime::CaptureStatus};
use std::sync::atomic::Ordering;
use tray_icon::{Icon, TrayIcon};

#[derive(Default)]
pub(super) struct StorageAlert {
    warning: bool,
    notified: bool,
}

impl StorageAlert {
    pub fn update(
        &mut self,
        tray: &TrayIcon,
        view: &VisibleWebView,
        locale: Locale,
        capture: &CaptureStatus,
    ) {
        let warning = capture.storage_error.is_some();
        if warning != self.warning {
            let icon = if warning { warning_icon() } else { tray_icon() };
            if let Err(error) = icon.and_then(|icon| Ok(tray.set_icon(Some(icon))?)) {
                eprintln!("Cannot update recording status icon: {error:#}");
            }
            self.warning = warning;
        }
        if let Ok(json) = serde_json::to_string(capture) {
            let _ = view.evaluate_script(&format!("window.receiveCapture?.({json});"));
        }
        if !warning {
            self.notified = false;
        } else if capture.storage_blocked.load(Ordering::Relaxed) && !self.notified {
            self.notified = true;
            let body = locale.text("error.storage_waiting");
            // A notification service may be unavailable or disabled by the user.
            // The persistent tray warning and in-app message remain regardless.
            if let Err(error) = std::thread::Builder::new()
                .name("clickpulse-save-notice".into())
                .spawn(move || {
                    if let Err(error) = notify(&body) {
                        eprintln!("Cannot show recording notification: {error}");
                    }
                })
            {
                eprintln!("Cannot start recording notification: {error}");
            }
        }
    }
}

fn warning_icon() -> anyhow::Result<Icon> {
    const SIZE: usize = 44;
    let mut pixels = vec![0; SIZE * SIZE * 4];
    // A monochrome exclamation mark remains visible in macOS template mode.
    for y in 4..40 {
        for x in 18..26 {
            if !(29..34).contains(&y) {
                pixels[(y * SIZE + x) * 4..(y * SIZE + x) * 4 + 4]
                    .copy_from_slice(&[232, 150, 35, 255]);
            }
        }
    }
    Ok(Icon::from_rgba(pixels, SIZE as u32, SIZE as u32)?)
}

fn notify(body: &str) -> anyhow::Result<()> {
    let mut notification = notify_rust::Notification::new();
    notification.summary("ClickPulse").body(body);
    #[cfg(target_os = "windows")]
    notification.app_id(clickpulse::platform::windows::notification_app_id()?);
    #[cfg(target_os = "macos")]
    notify_rust::set_application("app.clickpulse.desktop")?;
    #[cfg(target_os = "linux")]
    notification.appname("ClickPulse").icon("dialog-warning");
    notification.show()?;
    Ok(())
}
