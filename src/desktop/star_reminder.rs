use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

const DAY: u64 = 24 * 60 * 60;
const WEEK: u64 = 7 * DAY;
const CHECKPOINT: u64 = 15 * 60;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Progress {
    Running { seconds: u64 },
    Shown,
}

/// Only the instance holding the normal user's profile lock owns this file.
/// Keep it with the profile, independent of the app version and statistics DB.
pub(super) struct StarReminder {
    path: PathBuf,
    started: Instant,
    previous: u64,
    saved: u64,
    export_due: bool,
}

impl StarReminder {
    pub fn load(profile: &Path, now: Instant) -> Result<Option<Self>> {
        let path = profile.join("profile").join("star-reminder.json");
        let previous = match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice(&bytes)? {
                Progress::Shown => return Ok(None),
                Progress::Running { seconds } => seconds.min(WEEK),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error.into()),
        };
        Ok(Some(Self {
            path,
            started: now,
            previous,
            saved: previous,
            export_due: false,
        }))
    }

    fn seconds(&self, now: Instant) -> u64 {
        self.previous
            .saturating_add(now.saturating_duration_since(self.started).as_secs())
            .min(WEEK)
    }

    pub fn on_export(&mut self, now: Instant, completed: bool) {
        self.export_due |= completed && self.seconds(now) >= 2 * DAY;
    }

    pub fn due(&self, now: Instant) -> bool {
        self.export_due || self.seconds(now) >= WEEK
    }

    pub fn checkpoint(&mut self, now: Instant, exiting: bool) -> Result<()> {
        let seconds = self.seconds(now);
        if seconds > self.saved
            && (exiting || seconds - self.saved >= CHECKPOINT || seconds == WEEK)
        {
            self.write(Progress::Running { seconds })?;
            self.saved = seconds;
        }
        Ok(())
    }

    pub fn mark_shown(&self) -> Result<()> {
        self.write(Progress::Shown)
    }

    fn write(&self, progress: Progress) -> Result<()> {
        let parent = self.path.parent().expect("profile file has a parent");
        fs::create_dir_all(parent)?;
        let staged = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer(staged.as_file(), &progress)?;
        staged.as_file().sync_all()?;
        staged.persist(&self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn export_after_two_days_or_one_week_without_export() {
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        for via_export in [true, false] {
            let profile = tempfile::tempdir().unwrap();
            let mut reminder = StarReminder::load(profile.path(), start).unwrap().unwrap();
            reminder.on_export(at(2 * DAY - 1), true);
            assert!(!reminder.due(at(2 * DAY)));
            reminder.on_export(at(2 * DAY), false);
            assert!(!reminder.due(at(2 * DAY)));
            assert_eq!(
                reminder.seconds(start.checked_sub(Duration::from_secs(1)).unwrap_or(start)),
                0
            );
            if via_export {
                reminder.on_export(at(2 * DAY), true);
                reminder.on_export(at(2 * DAY), false);
                assert!(reminder.due(at(2 * DAY)));
            } else {
                assert!(!reminder.due(at(WEEK - 1)));
                assert!(reminder.due(at(WEEK)));
            }
            reminder.mark_shown().unwrap();
            assert!(
                StarReminder::load(profile.path(), at(WEEK))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn runtime_survives_restarts_without_counting_closed_time() {
        let profile = tempfile::tempdir().unwrap();
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut first = StarReminder::load(profile.path(), start).unwrap().unwrap();
        first.checkpoint(at(CHECKPOINT - 1), false).unwrap();
        assert!(!first.path.exists());
        first.checkpoint(at(DAY), false).unwrap();
        first.checkpoint(at(DAY + 40), true).unwrap();
        let mut next = StarReminder::load(profile.path(), at(30 * DAY))
            .unwrap()
            .unwrap();
        next.on_export(at(30 * DAY), true);
        assert!(!next.due(at(30 * DAY)));
        next.on_export(at(31 * DAY - 40), true);
        assert!(next.due(at(31 * DAY - 40)));
    }
}
