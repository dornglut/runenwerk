use crate::WorldHash;
use engine_sim::{
    DeterminismLevel, SimulationProfile, SimulationSeed, SimulationSessionId, SimulationTick,
};
use serde::{Deserialize, Serialize};

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
