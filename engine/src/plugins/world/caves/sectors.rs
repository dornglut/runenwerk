use super::super::ids::ChunkId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    ecs::Resource,
)]
pub struct CaveSectorId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct CaveSectorSummary {
    pub sector_id: CaveSectorId,
    pub chunks: Vec<ChunkId>,
    pub portal_count: u16,
    pub has_sky_visibility: bool,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldCaveSectorResource {
    pub by_sector: BTreeMap<CaveSectorId, CaveSectorSummary>,
    pub by_chunk: BTreeMap<ChunkId, CaveSectorId>,
    pub visible_sectors: BTreeSet<CaveSectorId>,
    pub generation: u64,
}

impl WorldCaveSectorResource {
    pub fn mark_visible(&mut self, sector_id: CaveSectorId) {
        self.visible_sectors.insert(sector_id);
    }

    pub fn clear_visibility(&mut self) {
        self.visible_sectors.clear();
    }

    pub fn sector_for_chunk(&self, chunk_id: ChunkId) -> Option<CaveSectorId> {
        self.by_chunk.get(&chunk_id).copied()
    }
}
