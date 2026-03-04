use anyhow::{Result, anyhow};
use engine_sim::{
    DeterminismLevel, SimulationCommandFrame, SimulationHash, SimulationProfile, SimulationSeed,
    SimulationSessionId, SimulationTick,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub type WorldHash = SimulationHash;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointPolicy {
    pub interval_ticks: u64,
    pub retained_checkpoints: usize,
    pub hash_every_tick: bool,
}

impl Default for CheckpointPolicy {
    fn default() -> Self {
        Self {
            interval_ticks: 30,
            retained_checkpoints: 240,
            hash_every_tick: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayStoragePolicy {
    pub persist_archives: bool,
    pub compress_archives: bool,
}

impl Default for ReplayStoragePolicy {
    fn default() -> Self {
        Self {
            persist_archives: false,
            compress_archives: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub format_version: u32,
    pub profile: SimulationProfile,
    pub determinism: DeterminismLevel,
    pub session_id: SimulationSessionId,
    pub seed: SimulationSeed,
    pub tick_rate_hz: u16,
    pub codec_id: String,
    pub codec_version: u32,
}

impl ReplayHeader {
    pub const FORMAT_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCheckpointMeta {
    pub tick: SimulationTick,
    pub hash: WorldHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCheckpoint<S> {
    pub meta: ReplayCheckpointMeta,
    pub snapshot: S,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayJournalFrame<C> {
    pub tick: SimulationTick,
    pub commands: Vec<C>,
    pub post_hash: Option<WorldHash>,
}

impl<C> From<SimulationCommandFrame<C>> for ReplayJournalFrame<C> {
    fn from(value: SimulationCommandFrame<C>) -> Self {
        Self {
            tick: value.tick,
            commands: value.commands,
            post_hash: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayArchive<S, C> {
    pub header: ReplayHeader,
    pub checkpoints: Vec<ReplayCheckpoint<S>>,
    pub journal: Vec<ReplayJournalFrame<C>>,
}

impl<S, C> ReplayArchive<S, C>
where
    S: Serialize + DeserializeOwned + Clone,
    C: Serialize + DeserializeOwned + Clone,
{
    pub fn encode_compressed(&self) -> Result<Vec<u8>> {
        let bytes = postcard::to_allocvec(self)?;
        Ok(zstd::stream::encode_all(bytes.as_slice(), 3)?)
    }

    pub fn decode_compressed(bytes: &[u8]) -> Result<Self> {
        let decompressed = zstd::stream::decode_all(bytes)?;
        Ok(postcard::from_bytes(&decompressed)?)
    }
}

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
        tick.0 == 0 || tick.0 % self.checkpoint_policy.interval_ticks == 0
    }

    pub fn into_archive(self) -> ReplayArchive<S, C> {
        ReplayArchive {
            header: self.header,
            checkpoints: self.checkpoints.into_iter().collect(),
            journal: self.journal,
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayMismatch {
    MissingCheckpoint {
        target_tick: SimulationTick,
    },
    TickHashMismatch {
        tick: SimulationTick,
        expected: WorldHash,
        actual: WorldHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayValidationReport {
    pub mismatches: Vec<ReplayMismatch>,
}

impl ReplayValidationReport {
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_retains_checkpoint_ring() {
        let header = ReplayHeader {
            format_version: ReplayHeader::FORMAT_VERSION,
            profile: SimulationProfile::DedicatedAuthority,
            determinism: DeterminismLevel::Validated,
            session_id: SimulationSessionId(1),
            seed: SimulationSeed(7),
            tick_rate_hz: 60,
            codec_id: "test".to_string(),
            codec_version: 1,
        };
        let mut recorder = ReplayRecorder::<u32, u8>::new(
            header,
            CheckpointPolicy {
                interval_ticks: 5,
                retained_checkpoints: 2,
                hash_every_tick: true,
            },
            ReplayStoragePolicy::default(),
        );
        for tick in 0..3 {
            recorder.record_checkpoint(ReplayCheckpoint {
                meta: ReplayCheckpointMeta {
                    tick: SimulationTick(tick),
                    hash: SimulationHash([tick as u8; 32]),
                },
                snapshot: tick as u32,
            });
        }
        assert_eq!(recorder.checkpoint_count(), 2);
    }

    #[test]
    fn archive_round_trips_with_compression() {
        let archive = ReplayArchive {
            header: ReplayHeader {
                format_version: ReplayHeader::FORMAT_VERSION,
                profile: SimulationProfile::DedicatedAuthority,
                determinism: DeterminismLevel::Validated,
                session_id: SimulationSessionId(1),
                seed: SimulationSeed(3),
                tick_rate_hz: 60,
                codec_id: "scene".to_string(),
                codec_version: 1,
            },
            checkpoints: vec![ReplayCheckpoint {
                meta: ReplayCheckpointMeta {
                    tick: SimulationTick(0),
                    hash: SimulationHash([1; 32]),
                },
                snapshot: vec![1u8, 2, 3],
            }],
            journal: vec![ReplayJournalFrame {
                tick: SimulationTick(1),
                commands: vec![9u8],
                post_hash: Some(SimulationHash([2; 32])),
            }],
        };

        let bytes = archive.encode_compressed().expect("archive should encode");
        let decoded =
            ReplayArchive::<Vec<u8>, u8>::decode_compressed(&bytes).expect("archive should decode");
        assert_eq!(decoded, archive);
    }
}
