use serde::{Deserialize, Serialize};

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
