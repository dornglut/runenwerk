use crate::{ReplayArchive, ReplayCheckpoint, ReplayJournalFrame};
use anyhow::{Result, anyhow};
use engine_sim::SimulationTick;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayController<S, C> {
    archive: Option<ReplayArchive<S, C>>,
}

impl<S, C> Default for ReplayController<S, C> {
    fn default() -> Self {
        Self { archive: None }
    }
}

impl<S, C> ReplayController<S, C>
where
    S: Clone,
    C: Clone,
{
    pub fn load(&mut self, archive: ReplayArchive<S, C>) {
        self.archive = Some(archive);
    }

    pub fn clear(&mut self) {
        self.archive = None;
    }

    pub fn archive(&self) -> Option<&ReplayArchive<S, C>> {
        self.archive.as_ref()
    }

    pub fn archive_cloned(&self) -> Option<ReplayArchive<S, C>> {
        self.archive.clone()
    }

    pub fn checkpoint_for_tick(&self, target_tick: SimulationTick) -> Option<&ReplayCheckpoint<S>> {
        self.archive.as_ref().and_then(|archive| {
            archive
                .checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.meta.tick.0 <= target_tick.0)
                .max_by_key(|checkpoint| checkpoint.meta.tick.0)
        })
    }

    pub fn frames_between(
        &self,
        start_exclusive: SimulationTick,
        target_tick: SimulationTick,
    ) -> Result<Vec<ReplayJournalFrame<C>>> {
        let archive = self
            .archive
            .as_ref()
            .ok_or_else(|| anyhow!("no replay archive is loaded"))?;
        Ok(archive
            .journal
            .iter()
            .filter(|frame| frame.tick.0 > start_exclusive.0 && frame.tick.0 <= target_tick.0)
            .cloned()
            .collect())
    }
}
