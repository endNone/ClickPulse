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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CheckSource {
    Manual,
    Automatic,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(controller: &mut Controller, now: i64) {
        assert!(controller.begin_check(now, CheckSource::Automatic));
        controller.receive(Event::Checked {
            id: controller.id,
            result: Err("offline".into()),
        });
    }

    #[test]
    fn failed_checks_retry_a_bounded_number_of_times_then_wait_a_day() {
        // A check right after login or wake often runs before the network is
        // back. Retry in-process without changing the persisted daily window.
        let mut controller = Controller::new(false);
        controller.automatic_allowed = true;
        let start = 1_794_310_000;
        let mut now = start;
        for delay in RETRY_DELAYS {
            failed(&mut controller, now);
            assert_eq!(controller.view.status, "idle");
            assert!(!controller.automatic_due(true, Some(now), now + delay - 1));
            assert!(controller.automatic_due(true, Some(now), now + delay));
            // Turning the daily preference off also stops pending retries.
            assert!(!controller.automatic_due(false, Some(now), now + delay));
            now += delay;
        }
        failed(&mut controller, now);
        assert_eq!(controller.retry_at, None);
        assert!(!controller.automatic_due(true, Some(now), now + RETRY_DELAYS[2] * 4));
        assert!(controller.automatic_due(true, Some(now), now + 86_400));
        // A success clears the backoff; the next failure starts over.
        controller.view.status = "checking";
        controller.receive(Event::Checked {
            id: controller.id,
            result: Ok(None),
        });
        assert_eq!((controller.failures, controller.retry_at), (0, None));
        failed(&mut controller, now);
        assert_eq!(controller.retry_at, Some(now + RETRY_DELAYS[0]));
        // Excluded sessions never schedule automatic retries.
        let mut excluded = Controller::new(false);
        failed(&mut excluded, now);
        assert!(!excluded.automatic_due(true, Some(now), now + 86_400));
        assert!(!excluded.view.automatic);
    }

    #[test]
    fn a_failed_recheck_keeps_the_update_already_offered() {
        let mut controller = Controller::new(false);
        controller.automatic_allowed = true;
        controller.candidate = Some(Candidate::placeholder("9.9.9"));
        failed(&mut controller, 1);
        assert_eq!(controller.view.status, "available");
        assert_eq!(controller.view.version.as_deref(), Some("9.9.9"));
        assert!(controller.candidate.is_some());
        assert!(controller.begin_check(2, CheckSource::Manual));
        controller.receive(Event::Checked {
            id: controller.id,
            result: Err("offline".into()),
        });
        assert_eq!(controller.view.status, "error");
        assert!(controller.candidate.is_some());
        // A successful re-check that finds nothing newer drops the offer.
        controller.view.status = "checking";
        controller.receive(Event::Checked {
            id: controller.id,
            result: Ok(None),
        });
        assert_eq!(controller.view.status, "current");
        assert!(controller.candidate.is_none());
    }

    #[test]
    fn background_errors_are_quiet_but_manual_and_install_errors_remain_visible() {
        let mut controller = Controller::new(false);
        controller.automatic_allowed = true;
        failed(&mut controller, 100_000);
        assert_eq!(controller.view.status, "idle");
        assert_eq!(controller.retry_at, Some(100_300));
        assert!(controller.begin_check(100_010, CheckSource::Manual));
        assert!(!controller.begin_check(100_011, CheckSource::Automatic));
        assert_eq!(controller.check_source, CheckSource::Manual);
        controller.receive(Event::Checked {
            id: controller.id,
            result: Err("offline".into()),
        });
        assert_eq!(controller.view.status, "error");
        failed(&mut controller, 100_310);
        assert_eq!(controller.view.status, "idle");
        controller.receive(Event::Prepared {
            id: controller.id,
            result: Err("verification failed".into()),
        });
        assert_eq!(controller.view.status, "error");
        assert!(controller.guard.is_none());
    }

    #[test]
    fn a_new_daily_window_gets_a_fresh_bounded_retry_cycle() {
        let mut controller = Controller::new(false);
        let mut now = 100_000;
        for delay in RETRY_DELAYS {
            failed(&mut controller, now);
            now += delay;
        }
        failed(&mut controller, now);
        assert_eq!(controller.retry_at, None);
        failed(&mut controller, now + 86_400);
        assert_eq!(controller.retry_at, Some(now + 86_400 + 300));
    }

    #[test]
    fn only_current_successful_normal_checks_change_the_notification_queue() {
        let mut notices = updater::UpdateNotifications::default();
        let mut controller = Controller::new(false);
        controller.automatic_allowed = true;
        controller.id = 2;
        controller.receive(Event::Checked {
            id: 1,
            result: Ok(Some(Box::new(Candidate::placeholder("9.9.9")))),
        });
        assert!(!controller.sync_notifications(&mut notices));
        assert_eq!(notices.pending(), None);
        controller.receive(Event::Checked {
            id: 2,
            result: Ok(Some(Box::new(Candidate::placeholder("9.9.9")))),
        });
        assert!(controller.sync_notifications(&mut notices));
        assert_eq!(notices.pending(), Some("9.9.9"));
        failed(&mut controller, 100_000);
        assert!(!controller.sync_notifications(&mut notices));
        assert_eq!(notices.pending(), Some("9.9.9"));
        controller.receive(Event::Checked {
            id: controller.id,
            result: Ok(None),
        });
        assert!(controller.sync_notifications(&mut notices));
        assert_eq!(notices.pending(), None);
        let mut excluded = Controller::new(false);
        excluded.receive(Event::Checked {
            id: 0,
            result: Ok(Some(Box::new(Candidate::placeholder("10.0.0")))),
        });
        assert!(!excluded.sync_notifications(&mut notices));
        assert_eq!(notices.pending(), None);
    }

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
    /// False in demo, read-only source and independent sessions.
    automatic: bool,
    received: u64,
    total: u64,
}
/// In-process retries after a failed check. Restarts do not retry; they wait
/// for the persisted daily window so startup cannot become a request loop.
const RETRY_DELAYS: [i64; 3] = [5 * 60, 30 * 60, 2 * 60 * 60];
pub(super) struct Controller {
    view: View,
    candidate: Option<Candidate>,
    installation: Result<Installation, String>,
    automatic_allowed: bool,
    attempted_at: i64,
    check_source: CheckSource,
    failures: usize,
    retry_at: Option<i64>,
    checked_version: Option<Option<String>>,
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
                automatic: allowed,
                received: 0,
                total: 0,
            },
            candidate: None,
            automatic_allowed: allowed,
            attempted_at: 0,
            check_source: CheckSource::Manual,
            failures: 0,
            retry_at: None,
            checked_version: None,
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
    /// Only normal installed-app sessions schedule checks or persist update
    /// preferences; other sessions must not alter the installed app's state.
    pub fn automatic_allowed(&self) -> bool {
        self.automatic_allowed
    }
    pub fn sync_notifications(&mut self, notices: &mut updater::UpdateNotifications) -> bool {
        self.checked_version
            .take()
            .is_some_and(|version| self.automatic_allowed && notices.observe(version.as_deref()))
    }
    pub fn automatic_due(&self, enabled: bool, last_attempt: Option<i64>, now: i64) -> bool {
        self.automatic_allowed
            && enabled
            && !self.busy()
            && (updater::daily_check_due(last_attempt, now)
                || self.retry_at.is_some_and(|at| now >= at))
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
    fn begin_check(&mut self, now: i64, source: CheckSource) -> bool {
        if self.busy() {
            return false;
        }
        if source == CheckSource::Manual || updater::daily_check_due(Some(self.attempted_at), now) {
            self.failures = 0;
        }
        self.id += 1;
        self.attempted_at = now;
        self.check_source = source;
        self.retry_at = None;
        // Keep an offered update until a successful check replaces it, so a
        // failed re-check cannot hide an install the user has not chosen yet.
        self.view.status = "checking";
        self.view.version = None;
        self.view.received = 0;
        self.view.total = 0;
        true
    }
    pub fn check(
        &mut self,
        proxy: EventLoopProxy<super::UiEvent>,
        now: i64,
        source: CheckSource,
    ) -> bool {
        if !self.begin_check(now, source) {
            return false;
        }
        let id = self.id;
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
            eprintln!("{source:?} update check failed: {error}");
            self.check_failed();
        }
        true
    }
    fn check_failed(&mut self) {
        if self.check_source == CheckSource::Manual {
            self.view.status = "error";
        } else if let Some(candidate) = &self.candidate {
            self.view.version = Some(candidate.version.clone());
            self.view.status = "available";
        } else {
            self.view.status = "idle";
        }
        self.failures = self.failures.saturating_add(1);
        self.retry_at = RETRY_DELAYS
            .get(self.failures - 1)
            .map(|delay| self.attempted_at.saturating_add(*delay));
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
                    self.checked_version = Some(Some(candidate.version.clone()));
                    (self.failures, self.retry_at) = (0, None);
                    self.view.version = Some(candidate.version.clone());
                    self.view.status = "available";
                    self.view.can_install = self.installation.is_ok();
                    self.candidate = Some(candidate);
                }
                Ok(None) => {
                    self.checked_version = Some(None);
                    (self.failures, self.retry_at) = (0, None);
                    self.candidate = None;
                    self.view.status = "current";
                }
                Err(error) => {
                    eprintln!("{:?} update check failed: {error}", self.check_source);
                    self.check_failed();
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
