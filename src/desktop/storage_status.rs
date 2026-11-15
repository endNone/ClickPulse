//! Keep save failures visible even when database snapshots cannot be read.
use super::{VisibleWebView, tray_icon};
use clickpulse::{localization::Locale, runtime::CaptureStatus};
#[cfg(any(target_os = "macos", test))]
use std::sync::OnceLock;
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
        if self.should_notify(warning, capture.storage_blocked.load(Ordering::Relaxed)) {
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

    fn should_notify(&mut self, warning: bool, blocked: bool) -> bool {
        if !warning {
            self.notified = false;
        } else if blocked && !self.notified {
            self.notified = true;
            return true;
        }
        false
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
    #[cfg(target_os = "linux")]
    notification.appname("ClickPulse").icon("dialog-warning");

    let show = || {
        notification.show()?;
        Ok(())
    };
    #[cfg(target_os = "macos")]
    {
        // mac-notification-sys rejects every subsequent set_application call.
        // Cache initialization, not delivery: each new outage still gets a notice.
        static APPLICATION: OnceLock<Result<(), String>> = OnceLock::new();
        show_after_initialization(
            &APPLICATION,
            || {
                notify_rust::set_application("app.clickpulse.desktop")?;
                Ok(())
            },
            show,
        )
    }
    #[cfg(not(target_os = "macos"))]
    show()
}

#[cfg(any(target_os = "macos", test))]
fn show_after_initialization(
    application: &OnceLock<Result<(), String>>,
    initialize: impl FnOnce() -> anyhow::Result<()>,
    show: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    application
        .get_or_init(|| initialize().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(|error| anyhow::anyhow!("Cannot initialize notification application: {error}"))?;
    show()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn separate_storage_outages_each_show_a_notification() -> anyhow::Result<()> {
        let mut alert = StorageAlert::default();
        let application = OnceLock::new();
        let initializations = Cell::new(0);
        let deliveries = Cell::new(0);
        for _ in 0..2 {
            assert!(!alert.should_notify(true, false)); // Retrying, not paused yet.
            assert!(alert.should_notify(true, true));
            show_after_initialization(
                &application,
                || {
                    initializations.set(initializations.get() + 1);
                    anyhow::ensure!(initializations.get() == 1, "AlreadySet");
                    Ok(())
                },
                || {
                    deliveries.set(deliveries.get() + 1);
                    Ok(())
                },
            )?;
            assert!(!alert.should_notify(true, true)); // No repeat within an outage.
            assert!(!alert.should_notify(false, false)); // Saving recovered.
        }
        assert_eq!(initializations.get(), 1);
        assert_eq!(deliveries.get(), 2);
        Ok(())
    }

    #[test]
    fn initialization_failure_remains_visible_without_attempting_delivery() {
        let application = OnceLock::new();
        for _ in 0..2 {
            let result = show_after_initialization(
                &application,
                || anyhow::bail!("CouldNotSet"),
                || panic!("Cannot deliver with an uninitialized application"),
            );
            assert!(result.unwrap_err().to_string().contains("CouldNotSet"));
        }
    }

    #[test]
    fn delivery_failure_does_not_repeat_application_initialization() -> anyhow::Result<()> {
        let application = OnceLock::new();
        assert!(
            show_after_initialization(
                &application,
                || Ok(()),
                || anyhow::bail!("Notification service unavailable"),
            )
            .is_err()
        );
        show_after_initialization(
            &application,
            || panic!("Application was already initialized"),
            || Ok(()),
        )?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires an isolated desktop notification service; run by Linux CI"]
    fn linux_notification_service_accepts_repeated_alerts() -> anyhow::Result<()> {
        // Exercise the production zbus transport without starting capture or a database.
        notify("ClickPulse notification test: first storage pause")?;
        notify("ClickPulse notification test: storage paused again after recovery")?;
        Ok(())
    }
}
