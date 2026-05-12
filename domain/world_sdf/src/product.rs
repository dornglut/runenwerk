use ::product::{
    ProductAuthorityClass, ProductConsumerClass, ProductDescriptorCore, ProductFamily,
    ProductFreshness, ProductIdentity, ProductKind, ProductLineage, ProductQueryPolicy,
    ProductRebuildPolicy, ProductResidency, ProductRetentionPolicy, ProductScaleBand, ProductScope,
};
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

    pub fn product_core(&self) -> ProductDescriptorCore {
        let mut descriptor = ProductDescriptorCore::new(
            ProductIdentity::new(self.product_id.0),
            self.product_family(),
            ProductKind::new(self.kind.product_kind_name()),
            self.product_scope(),
            self.product_scale_band(),
            self.product_lineage(),
        );
        descriptor.freshness = self.product_freshness();
        descriptor.residency = self.product_residency();
        descriptor.consumer_class = self.product_consumer_class();
        descriptor.authority_class = self.product_authority_class();
        descriptor.retention_policy = self.product_retention_policy();
        descriptor.rebuild_policy = self.product_rebuild_policy();
        descriptor.query_policy = self.product_query_policy();
        descriptor
    }

    fn product_family(&self) -> ProductFamily {
        match self.kind {
            FieldProductKind::OccupancySupport => ProductFamily::Collision,
            FieldProductKind::BrickmapDebug => ProductFamily::Diagnostics,
            _ => ProductFamily::SurfaceSdf,
        }
    }

    fn product_scope(&self) -> ProductScope {
        ProductScope::field(
            self.scope.chunk_ids.iter().map(format_chunk_id),
            self.scope.region_ids.iter().map(format_region_id),
        )
    }

    fn product_scale_band(&self) -> ProductScaleBand {
        match self.scale_band.as_str() {
            "near" => ProductScaleBand::Near,
            "mid" => ProductScaleBand::Mid,
            "far" => ProductScaleBand::Far,
            "summary" => ProductScaleBand::Summary,
            "collision_strict_query" => ProductScaleBand::CollisionStrictQuery,
            "offline" => ProductScaleBand::Offline,
            "preview" => ProductScaleBand::Preview,
            _ if self.kind.is_strict_query_product() => ProductScaleBand::CollisionStrictQuery,
            _ => ProductScaleBand::FamilySpecific,
        }
    }

    fn product_lineage(&self) -> ProductLineage {
        let mut lineage =
            ProductLineage::new(self.lineage.producer.as_str(), self.lineage.source_revision)
                .with_source_revision(self.lineage.source_revision.to_string());
        if let Some(source_asset_id) = self.lineage.source_asset_id {
            lineage = lineage.with_source_key(format!("asset:{source_asset_id}"));
        }
        for payload_ref in &self.payload_refs {
            lineage = lineage.with_source_key(format!(
                "world_sdf_chunk:{}:{}",
                format_chunk_id(&payload_ref.chunk_id),
                payload_ref.chunk_revision.0
            ));
        }
        lineage
    }

    fn product_freshness(&self) -> ProductFreshness {
        match self.freshness {
            FieldProductFreshness::Current => ProductFreshness::Current,
            FieldProductFreshness::PotentiallyStale => ProductFreshness::PotentiallyStale,
            FieldProductFreshness::Rejected => ProductFreshness::Retired,
        }
    }

    fn product_residency(&self) -> ProductResidency {
        if self.kind.is_world_sdf_payload_product() {
            if self.payload_refs.is_empty() {
                ProductResidency::Missing
            } else {
                ProductResidency::Resident
            }
        } else {
            ProductResidency::NotApplicable
        }
    }

    fn product_consumer_class(&self) -> ProductConsumerClass {
        match self.consumer_class {
            FieldProductConsumerClass::EditorPreview => ProductConsumerClass::Editor,
            FieldProductConsumerClass::RuntimeRead => ProductConsumerClass::RuntimeRead,
            FieldProductConsumerClass::CollisionQuery => ProductConsumerClass::CollisionQuery,
            FieldProductConsumerClass::Diagnostics => ProductConsumerClass::Diagnostics,
        }
    }

    fn product_authority_class(&self) -> ProductAuthorityClass {
        match self.kind {
            FieldProductKind::BrickmapDebug => ProductAuthorityClass::DiagnosticOnly,
            _ if self.consumer_class == FieldProductConsumerClass::EditorPreview => {
                ProductAuthorityClass::DeterministicDerived
            }
            _ => ProductAuthorityClass::DeterministicDerived,
        }
    }

    fn product_retention_policy(&self) -> ProductRetentionPolicy {
        match self.retention_policy {
            FieldProductRetentionPolicy::RetainWhileReferenced => {
                ProductRetentionPolicy::RetainWhileReferenced
            }
            FieldProductRetentionPolicy::RetainUntilProjectClose => {
                ProductRetentionPolicy::SessionLocal
            }
            FieldProductRetentionPolicy::RebuildOnDemand => ProductRetentionPolicy::RebuildOnDemand,
        }
    }

    fn product_rebuild_policy(&self) -> ProductRebuildPolicy {
        match self.rebuild_policy.as_str() {
            "immediate" => ProductRebuildPolicy::Immediate,
            "lazy" => ProductRebuildPolicy::Lazy,
            "idle" => ProductRebuildPolicy::Idle,
            "manual" => ProductRebuildPolicy::Manual,
            "offline" => ProductRebuildPolicy::Offline,
            "never" => ProductRebuildPolicy::Never,
            _ => ProductRebuildPolicy::Budgeted,
        }
    }

    fn product_query_policy(&self) -> ProductQueryPolicy {
        if self.kind.is_strict_query_product()
            || self.consumer_class == FieldProductConsumerClass::CollisionQuery
        {
            ProductQueryPolicy::StrictCurrentOnly
        } else if self.kind == FieldProductKind::BrickmapDebug
            || self.consumer_class == FieldProductConsumerClass::Diagnostics
        {
            ProductQueryPolicy::DiagnosticOnly
        } else {
            ProductQueryPolicy::VisualFallbackAllowed
        }
    }
}

impl FieldProductKind {
    pub fn product_kind_name(self) -> &'static str {
        match self {
            Self::ScalarDistance => "scalar_distance",
            Self::VectorGradient => "vector_gradient",
            Self::MaterialChannel => "material_channel",
            Self::OccupancySupport => "occupancy_support",
            Self::WorldSdfChunkPages => "world_sdf_chunk_pages",
            Self::BrickmapDebug => "brickmap_debug",
        }
    }

    pub fn is_world_sdf_payload_product(self) -> bool {
        matches!(self, Self::WorldSdfChunkPages | Self::BrickmapDebug)
    }

    pub fn is_strict_query_product(self) -> bool {
        matches!(self, Self::WorldSdfChunkPages | Self::OccupancySupport)
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

fn format_chunk_id(chunk_id: &ChunkId) -> String {
    format!(
        "world:{}:chunk:{},{},{}",
        chunk_id.world_id.0, chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z
    )
}

fn format_region_id(region_id: &RegionId) -> String {
    format!(
        "world:{}:region:{},{},{}",
        region_id.world_id.0, region_id.coord.x, region_id.coord.y, region_id.coord.z
    )
}
