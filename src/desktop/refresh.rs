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

#[cfg(test)]
mod tests {
    use super::*;
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
