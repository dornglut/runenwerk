use serde::{Deserialize, Serialize};
use spatial::ChunkId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct CaveSectorId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaveSectorSummary {
    pub sector_id: CaveSectorId,
    pub chunks: Vec<ChunkId>,
    pub portal_count: u16,
    pub has_sky_visibility: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CaveSectorStore {
    pub by_sector: BTreeMap<CaveSectorId, CaveSectorSummary>,
    pub by_chunk: BTreeMap<ChunkId, CaveSectorId>,
    pub visible_sectors: BTreeSet<CaveSectorId>,
    pub generation: u64,
}

impl CaveSectorStore {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CavePortalEdge {
    pub a: CaveSectorId,
    pub b: CaveSectorId,
    pub bidirectional: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CavePortalGraph {
    pub portals: Vec<CavePortalEdge>,
}

impl CavePortalGraph {
    pub fn neighbors(&self, sector_id: CaveSectorId) -> Vec<CaveSectorId> {
        self.portals
            .iter()
            .filter_map(|edge| {
                if edge.a == sector_id {
                    Some(edge.b)
                } else if edge.bidirectional && edge.b == sector_id {
                    Some(edge.a)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaveLightVolumeScope {
    pub sector_id: CaveSectorId,
    pub local_center: [f32; 3],
    pub local_extents: [f32; 3],
    pub intensity_scale: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CaveLightingScope {
    pub scopes: BTreeMap<CaveSectorId, Vec<CaveLightVolumeScope>>,
}

impl CaveLightingScope {
    pub fn scopes_for_sector(&self, sector_id: CaveSectorId) -> &[CaveLightVolumeScope] {
        self.scopes
            .get(&sector_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
