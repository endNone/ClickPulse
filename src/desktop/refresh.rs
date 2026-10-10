//! Coalesce timer ticks without starving delivery of a slow completed snapshot.
#[derive(Default)]
pub(super) struct SnapshotGate {
    generation: u64,
    pending: Option<u64>,
    rerun: bool,
}

impl SnapshotGate {
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
