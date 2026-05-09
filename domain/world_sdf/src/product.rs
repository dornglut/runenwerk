use serde::{Deserialize, Serialize};
use spatial::{ChunkId, RegionId};
use std::collections::BTreeSet;

use crate::SdfChunkPayload;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct FieldProductId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductKind {
    ScalarDistance,
    VectorGradient,
    MaterialChannel,
    OccupancySupport,
    WorldSdfChunkPages,
    BrickmapDebug,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FieldProductScope {
    pub chunk_ids: BTreeSet<ChunkId>,
    pub region_ids: BTreeSet<RegionId>,
}

impl FieldProductScope {
    pub fn from_chunks(chunk_ids: impl IntoIterator<Item = ChunkId>) -> Self {
        Self {
            chunk_ids: chunk_ids.into_iter().collect(),
            region_ids: BTreeSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chunk_ids.is_empty() && self.region_ids.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductFreshness {
    Current,
    PotentiallyStale,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductConsumerClass {
    EditorPreview,
    RuntimeRead,
    CollisionQuery,
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldProductRetentionPolicy {
    RetainWhileReferenced,
    RetainUntilProjectClose,
    RebuildOnDemand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProductLineage {
    pub source_asset_id: Option<u64>,
    pub source_revision: u64,
    pub producer: String,
}

impl FieldProductLineage {
    pub fn new(source_revision: u64, producer: impl Into<String>) -> Self {
        Self {
            source_asset_id: None,
            source_revision,
            producer: producer.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSdfPayloadRef {
    pub chunk_id: ChunkId,
    pub chunk_revision: world_ops::ChunkRevision,
    pub checksum: u64,
}

impl From<&SdfChunkPayload> for WorldSdfPayloadRef {
    fn from(payload: &SdfChunkPayload) -> Self {
        Self {
            chunk_id: payload.chunk_id,
            chunk_revision: payload.chunk_revision,
            checksum: payload.checksum,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProductDescriptor {
    pub product_id: FieldProductId,
    pub kind: FieldProductKind,
    pub scope: FieldProductScope,
    pub scale_band: String,
    pub lineage: FieldProductLineage,
    pub freshness: FieldProductFreshness,
    pub consumer_class: FieldProductConsumerClass,
    pub retention_policy: FieldProductRetentionPolicy,
    pub rebuild_policy: String,
    pub payload_refs: Vec<WorldSdfPayloadRef>,
}

impl FieldProductDescriptor {
    pub fn new(
        product_id: FieldProductId,
        kind: FieldProductKind,
        scope: FieldProductScope,
        lineage: FieldProductLineage,
    ) -> Self {
        Self {
            product_id,
            kind,
            scope,
            scale_band: "preview".to_string(),
            lineage,
            freshness: FieldProductFreshness::Current,
            consumer_class: FieldProductConsumerClass::EditorPreview,
            retention_policy: FieldProductRetentionPolicy::RetainWhileReferenced,
            rebuild_policy: "rebuild_on_source_revision_change".to_string(),
            payload_refs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldProductCandidate {
    pub descriptor: FieldProductDescriptor,
}

impl FieldProductCandidate {
    pub fn new(descriptor: FieldProductDescriptor) -> Self {
        Self { descriptor }
    }
}
