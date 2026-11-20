//! Coalesce timer ticks without starving delivery of a slow completed snapshot.
pub(super) fn interval(visible: bool, compact: bool) -> std::time::Duration {
    std::time::Duration::from_millis(if !visible {
        5000
    } else if compact {
        500
    } else {
        2000
    })
}

#[derive(Default)]
pub(super) struct SnapshotGate {
    generation: u64,
    pending: Option<u64>,
    rerun: bool,
}

impl SnapshotGate {
    pub(super) fn invalidate(&mut self) {
        self.pending = None;
        self.rerun = false;
        self.generation = self.generation.wrapping_add(1);
    }
    pub(super) fn begin(&mut self) -> Option<u64> {
        if self.pending.is_some() {
            self.rerun = true;
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.pending = Some(self.generation);
        self.pending
    }

    pub(super) fn complete(&mut self, id: u64) -> Option<bool> {
        if self.pending != Some(id) {
            return None;
        }
        self.pending = None;
        Some(std::mem::take(&mut self.rerun))
    }
}

/// Only the latest range in the current renderer can consume its reply.
/// Keep a completed reply until a visible renderer accepts it, or is disposed.
#[derive(Default)]
pub(super) struct HistoryHoursDelivery {
    request: Option<(u64, u32)>,
    rows: Option<String>,
}
impl HistoryHoursDelivery {
    pub(super) fn begin(&mut self, generation: u64, id: u32) {
        self.request = Some((generation, id));
        self.rows = None;
    }
    pub(super) fn complete(&mut self, generation: u64, id: u32, rows: String) {
        if self.request == Some((generation, id)) {
            self.rows = Some(rows);
        }
    }
    pub(super) fn deliver(&mut self, ready: bool, send: impl FnOnce(u32, &str) -> bool) {
        if ready
            && let (Some((_, id)), Some(rows)) = (self.request, &self.rows)
            && send(id, rows)
        {
            *self = Self::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delayed_history_replies_follow_the_latest_request_and_resume_once() {
        let mut reply = HistoryHoursDelivery::default();
        reply.begin(1, 3);
        reply.complete(1, 3, "[]".into());
        reply.deliver(false, |_, _| panic!("hidden"));
        reply.deliver(true, |_, _| false);
        reply.deliver(true, |id, rows| {
            assert_eq!((id, rows), (3, "[]"));
            true
        });
        reply.deliver(true, |_, _| panic!("duplicate"));
        reply.begin(1, 4);
        reply.complete(1, 4, "old range".into());
        reply.begin(2, 4);
        reply.complete(2, 4, "null".into());
        reply.complete(1, 4, "old renderer".into());
        reply.deliver(true, |id, rows| {
            assert_eq!((id, rows), (4, "null"));
            true
        });
    }
    #[test]
    fn storage_changes_reject_old_snapshot_and_speed_completions() {
        let mut gate = SnapshotGate::default();
        let old = gate.begin().unwrap();
        gate.invalidate();
        let new = gate.begin().unwrap();
        assert_eq!(gate.complete(old), None);
        assert_eq!(gate.complete(new), Some(false));
    }
    #[test]
    fn slow_requests_coalesce_and_still_deliver_completed_results() {
        let mut gate = SnapshotGate::default();
        let id = gate.begin().unwrap();
        for _ in 0..100 {
            assert!(gate.begin().is_none());
        }
        // Completion is accepted even after many timer ticks. Only one new
        // request is needed, and the completed data can be displayed now.
        assert_eq!(gate.complete(id), Some(true));
        let next = gate.begin().unwrap();
        assert_eq!(gate.complete(id), None);
        assert_eq!(gate.complete(next), Some(false));
    }
}
