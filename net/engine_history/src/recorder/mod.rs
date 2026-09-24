use crate::{
    CheckpointPolicy, ReplayArchive, ReplayCheckpoint, ReplayHeader, ReplayJournalFrame,
    ReplayStoragePolicy,
};
use engine_sim::SimulationTick;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRecorder<S, C> {
    header: ReplayHeader,
    checkpoint_policy: CheckpointPolicy,
    storage_policy: ReplayStoragePolicy,
    checkpoints: VecDeque<ReplayCheckpoint<S>>,
    journal: Vec<ReplayJournalFrame<C>>,
}

impl<S, C> ReplayRecorder<S, C>
where
    S: Clone,
    C: Clone,
{
    pub fn new(
        header: ReplayHeader,
        checkpoint_policy: CheckpointPolicy,
        storage_policy: ReplayStoragePolicy,
    ) -> Self {
        Self {
            header,
            checkpoint_policy,
            storage_policy,
            checkpoints: VecDeque::new(),
            journal: Vec::new(),
        }
    }

    pub fn header(&self) -> &ReplayHeader {
        &self.header
    }

    pub fn checkpoint_policy(&self) -> CheckpointPolicy {
        self.checkpoint_policy
    }

    pub fn storage_policy(&self) -> ReplayStoragePolicy {
        self.storage_policy
    }

    pub fn record_journal_frame(&mut self, frame: ReplayJournalFrame<C>) {
        self.journal.push(frame);
    }

    pub fn last_journal_frame_mut(&mut self) -> Option<&mut ReplayJournalFrame<C>> {
        self.journal.last_mut()
    }

    pub fn record_checkpoint(&mut self, checkpoint: ReplayCheckpoint<S>) {
        self.checkpoints.push_back(checkpoint);
        while self.checkpoints.len() > self.checkpoint_policy.retained_checkpoints {
            self.checkpoints.pop_front();
        }
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn recorded_frames(&self) -> usize {
        self.journal.len()
    }

    pub fn should_checkpoint(&self, tick: SimulationTick) -> bool {
        tick.0 == 0 || tick.0.is_multiple_of(self.checkpoint_policy.interval_ticks)
    }

    pub fn into_archive(self) -> ReplayArchive<S, C> {
        ReplayArchive {
            header: self.header,
            checkpoints: self.checkpoints.into_iter().collect(),
            journal: self.journal,
        }
    }
}
