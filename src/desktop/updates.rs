use clickpulse::updater::{self, ApplyGuard, Candidate, Installation, Target};
use serde::Serialize;
use std::{
    cell::Cell,
    process::Command,
    time::{Duration, Instant},
};
use tao::event_loop::EventLoopProxy;

#[derive(Debug)]
pub(super) enum Event {
    Checked {
        id: u64,
        result: Result<Option<Box<Candidate>>, String>,
    },
    Progress {
        id: u64,
        received: u64,
        total: u64,
    },
    Preparing {
        id: u64,
    },
    Prepared {
        id: u64,
        result: Result<ApplyGuard, String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_checks_require_normal_enabled_idle_sessions() {
        // Excluded sessions must not read or consume the installed app's notice.
        let mut controller = Controller::new(false);
        assert!(!controller.automatic_due(true, None, 100_000));
        controller.automatic_allowed = true;
        assert!(!controller.automatic_due(false, None, 100_000));
        for status in [
            "checking",
            "downloading",
            "verifying",
            "preparing",
            "installing",
        ] {
            controller.view.status = status;
            assert!(!controller.automatic_due(true, None, 100_000), "{status}");
        }
        for status in ["idle", "current", "available", "error"] {
            controller.view.status = status;
            assert!(controller.automatic_due(true, Some(1), 100_000), "{status}");
            assert!(!controller.automatic_due(true, Some(99_999), 100_000));
            assert!(controller.guard.is_none());
        }
    }
}
#[derive(Serialize)]
struct View {
    current: &'static str,
    status: &'static str,
    version: Option<String>,
    can_install: bool,
    received: u64,
    total: u64,
}
pub(super) struct Controller {
    view: View,
    candidate: Option<Candidate>,
    installation: Result<Installation, String>,
    automatic_allowed: bool,
    id: u64,
    pub guard: Option<ApplyGuard>,
    notice: Option<updater::Notice>,
}
impl Controller {
    pub fn new(allowed: bool) -> Self {
        Self {
            view: View {
                current: clickpulse::version::PUBLIC,
                status: "idle",
                version: None,
                can_install: false,
                received: 0,
                total: 0,
            },
            candidate: None,
            automatic_allowed: allowed,
            installation: if allowed {
                updater::installation().map_err(|e| e.to_string())
            } else {
                Err("Update from the normal installed application".into())
            },
            id: 0,
            guard: None,
            notice: if allowed {
                updater::take_notice()
            } else {
                None
            },
        }
    }
    pub fn installing(&self) -> bool {
        matches!(
            self.view.status,
            "downloading" | "verifying" | "preparing" | "installing"
        )
    }
    fn busy(&self) -> bool {
        self.installing() || self.view.status == "checking"
    }
    pub fn automatic_due(&self, enabled: bool, last_attempt: Option<i64>, now: i64) -> bool {
        self.automatic_allowed
            && enabled
            && !self.busy()
            && updater::daily_check_due(last_attempt, now)
    }
    pub fn publish(&mut self, webview: &wry::WebView) {
        if let Ok(state) = serde_json::to_string(&self.view) {
            let _ = webview.evaluate_script(&format!("window.receiveUpdateState?.({state});"));
        }
        if let Some(notice) = self.notice.take()
            && let Ok(notice) = serde_json::to_string(&notice)
        {
            let _ = webview.evaluate_script(&format!("window.receiveUpdateNotice?.({notice});"));
        }
    }
    pub fn check(&mut self, proxy: EventLoopProxy<super::UiEvent>) -> bool {
        if self.busy() {
            return false;
        }
        self.id += 1;
        let id = self.id;
        self.candidate = None;
        self.view.status = "checking";
        self.view.version = None;
        self.view.received = 0;
        self.view.total = 0;
        if let Err(error) = std::thread::Builder::new()
            .name("clickpulse-update-check".into())
            .spawn(move || {
                let result = Target::current()
                    .and_then(|target| updater::check(clickpulse::version::PUBLIC, target))
                    .map(|candidate| candidate.map(Box::new))
                    .map_err(|e| format!("{e:#}"));
                let _ = proxy.send_event(super::UiEvent::Update(Event::Checked { id, result }));
            })
        {
            eprintln!("Update check failed: {error}");
            self.view.status = "error";
        }
        true
    }
    pub fn install(&mut self, proxy: EventLoopProxy<super::UiEvent>) -> anyhow::Result<()> {
        anyhow::ensure!(!self.busy(), "Update already in progress");
        let candidate = self
            .candidate
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Check for an update first"))?;
        let installation = self.installation.clone().map_err(anyhow::Error::msg)?;
        self.id += 1;
        let id = self.id;
        self.view.status = "downloading";
        let result = std::thread::Builder::new()
            .name("clickpulse-update-download".into())
            .spawn(move || {
                let last = Cell::new(Instant::now() - Duration::from_secs(1));
                let result = updater::stage(&candidate, |received, total| {
                    if last.get().elapsed() >= Duration::from_millis(100) || received == total {
                        last.set(Instant::now());
                        let _ = proxy.send_event(super::UiEvent::Update(Event::Progress {
                            id,
                            received,
                            total,
                        }));
                    }
                })
                .and_then(|staged| {
                    let _ = proxy.send_event(super::UiEvent::Update(Event::Preparing { id }));
                    updater::launch_installer(staged, installation)
                })
                .map_err(|e| format!("{e:#}"));
                let _ = proxy.send_event(super::UiEvent::Update(Event::Prepared { id, result }));
            });
        if let Err(error) = result {
            self.view.status = "error";
            return Err(error.into());
        }
        Ok(())
    }
    /// A Ready helper is held until normal runtime shutdown commits every event.
    pub fn receive(&mut self, event: Event) -> bool {
        match event {
            Event::Checked { id, result } if id == self.id => match result {
                Ok(Some(candidate)) => {
                    let candidate = *candidate;
                    self.view.version = Some(candidate.version.clone());
                    self.view.status = "available";
                    self.view.can_install = self.installation.is_ok();
                    self.candidate = Some(candidate);
                }
                Ok(None) => self.view.status = "current",
                Err(error) => {
                    eprintln!("Update check failed: {error}");
                    self.view.status = "error";
                }
            },
            Event::Progress {
                id,
                received,
                total,
            } if id == self.id => {
                self.view.received = received;
                self.view.total = total;
                self.view.status = if received == total {
                    "verifying"
                } else {
                    "downloading"
                };
            }
            Event::Preparing { id } if id == self.id => self.view.status = "preparing",
            Event::Prepared { id, result } if id == self.id => match result {
                Ok(guard) => {
                    self.guard = Some(guard);
                    self.view.status = "installing";
                    return true;
                }
                Err(error) => {
                    eprintln!("Update preparation failed: {error}");
                    self.view.status = "error";
                }
            },
            _ => {}
        }
        false
    }
    pub fn open_release(&self) -> anyhow::Result<()> {
        let url = self
            .candidate
            .as_ref()
            .map(|c| c.release_url.as_str())
            .unwrap_or(updater::RELEASES);
        #[cfg(target_os = "macos")]
        Command::new("/usr/bin/open").arg(url).spawn()?;
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            Command::new("rundll32.exe")
                .arg("url.dll,FileProtocolHandler")
                .arg(url)
                .creation_flags(0x08000000)
                .spawn()?;
        }
        Ok(())
    }
}
