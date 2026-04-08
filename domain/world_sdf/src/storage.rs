use serde::{Deserialize, Serialize};
use spatial::{ChunkId, RegionId};
use std::collections::BTreeMap;
use world_ops::{ChunkGeneration, ChunkRevision, OperationId};

pub const SDF_BRICK_EDGE_SAMPLES: usize = 8;
pub const SDF_PAGE_EDGE_BRICKS: usize = 4;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SdfPageCoord3 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdfBrickMetadata {
    pub min_distance: i16,
    pub max_distance: i16,
    pub occupancy_mask: u8,
    pub material_channel_mask: u16,
    pub last_touched_op_id: OperationId,
    pub surface_band_present: bool,
    pub compression_scheme: u8,
}

impl Default for SdfBrickMetadata {
    fn default() -> Self {
        Self {
            min_distance: 0,
            max_distance: 0,
            occupancy_mask: 0,
            material_channel_mask: 0,
            last_touched_op_id: OperationId::default(),
            surface_band_present: false,
            compression_scheme: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SdfBrickSamples {
    pub distances: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SdfBrickRecord {
    pub metadata: SdfBrickMetadata,
    pub samples: SdfBrickSamples,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SdfPageRecord {
    pub page_generation: u64,
    pub bricks: BTreeMap<[u8; 3], SdfBrickRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SdfChunkPayload {
    pub chunk_id: ChunkId,
    pub chunk_revision: ChunkRevision,
    pub chunk_generation: ChunkGeneration,
    pub page_table: BTreeMap<SdfPageCoord3, SdfPageRecord>,
    pub hierarchy_revision: u64,
    pub checksum: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RegionSdfSummary {
    pub min_distance: i16,
    pub max_distance: i16,
    pub occupied_chunk_count: u32,
    pub surface_chunk_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SdfChunkStore {
    pub chunks: BTreeMap<ChunkId, SdfChunkPayload>,
    pub region_summaries: BTreeMap<RegionId, RegionSdfSummary>,
}

#[cfg(test)]
mod tests {
    use super::SdfChunkPayload;
    use spatial::{ChunkCoord3, ChunkId, WorldId};

    #[test]
    fn payload_roundtrip_postcard() {
        let payload = SdfChunkPayload {
            chunk_id: ChunkId::new(WorldId(1), ChunkCoord3 { x: 2, y: 3, z: 4 }),
            checksum: 77,
            ..SdfChunkPayload::default()
        };
        let bytes = postcard::to_allocvec(&payload).expect("serialize payload");
        let decoded = postcard::from_bytes::<SdfChunkPayload>(&bytes).expect("decode payload");
        assert_eq!(decoded.chunk_id, payload.chunk_id);
        assert_eq!(decoded.checksum, payload.checksum);
    }
}
