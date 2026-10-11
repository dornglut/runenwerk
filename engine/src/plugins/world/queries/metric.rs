use product::ProductIdentity;
use runen_spatial::GridPartitionConfig;
use std::fmt;
use world_ops::BuildGeneration;
use world_sdf::{
    FieldProductKind, SdfChunkPayload, WorldSdfMetricError, WorldSdfMetricPayloadRef,
    WorldSdfMetricProductCandidate, WorldSdfMetricSample, WorldSdfPayloadRef,
    ratify_world_sdf_metric_product_candidate, sample_world_sdf_metric_distance,
    validate_world_sdf_metric_payload,
};

use super::super::adapters::resources::SdfChunkStoreResource;
use super::super::build::integration::{
    WorldRuntimeSdfMetricCapabilityCatalogResource, WorldRuntimeSdfProductCatalogResource,
};
use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};

/// A borrowed, revision-qualified view of an existing *approximate* metric
/// WorldSDF payload. It does not certify exact distance or analytic provenance.
#[derive(Debug)]
pub struct WorldSdfCurrentMetricRead<'a> {
    payload: &'a SdfChunkPayload,
    metric_ref: WorldSdfMetricPayloadRef,
}

impl WorldSdfCurrentMetricRead<'_> {
    pub fn payload(&self) -> &SdfChunkPayload {
        self.payload
    }

    pub const fn metric_ref(&self) -> WorldSdfMetricPayloadRef {
        self.metric_ref
    }

    /// Uses the existing validated metric sampler. This currently revalidates
    /// the page on every sample; callers must not treat it as a cheap sphere cast.
    pub fn sample(
        &self,
        partition: &GridPartitionConfig,
        world_position_meters: [f64; 3],
    ) -> Result<WorldSdfMetricSample, WorldSdfMetricError> {
        sample_world_sdf_metric_distance(
            &self.metric_ref,
            self.payload,
            partition,
            world_position_meters,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfCurrentMetricReadError {
    MissingProduct,
    WrongProductKind,
    InvalidProductScope,
    MissingMetricCapability,
    MetricReferenceMismatch,
    MissingCpuPayload,
    PayloadReferenceMismatch,
    MissingRuntimeRecord,
    RuntimeNotReady,
    RuntimeVersionMismatch,
    InvalidMetricProduct,
    InvalidMetricPayload(WorldSdfMetricError),
}

impl fmt::Display for WorldSdfCurrentMetricReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingProduct => "World SDF product is not in the catalog",
            Self::WrongProductKind => "World SDF product is not a chunk-page product",
            Self::InvalidProductScope => "World SDF product has no unique chunk scope and ref",
            Self::MissingMetricCapability => "World SDF product has no metric capability",
            Self::MetricReferenceMismatch => "metric capability and product ref disagree",
            Self::MissingCpuPayload => "World SDF chunk has no integrated CPU payload",
            Self::PayloadReferenceMismatch => "CPU chunk and product payload ref disagree",
            Self::MissingRuntimeRecord => "World SDF chunk has no runtime record",
            Self::RuntimeNotReady => "World SDF runtime chunk is pending, dirty, or not ready",
            Self::RuntimeVersionMismatch => "World SDF runtime version disagrees with CPU payload",
            Self::InvalidMetricProduct => "World SDF metric product ratification rejected",
            Self::InvalidMetricPayload(_) => "World SDF metric payload validation rejected",
        })
    }
}

impl std::error::Error for WorldSdfCurrentMetricReadError {}

/// Qualify the *current resident CPU metric bytes* for one selected product.
/// This is not exact-distance, contact, continuity, or source-identity proof.
/// Borrowing the store prevents the returned payload from outliving its read.
pub fn qualify_current_world_sdf_metric_read<'a>(
    product_id: ProductIdentity,
    products: &WorldRuntimeSdfProductCatalogResource,
    metrics: &WorldRuntimeSdfMetricCapabilityCatalogResource,
    chunks: &WorldChunkRuntimeMapResource,
    store: &'a SdfChunkStoreResource,
) -> Result<WorldSdfCurrentMetricRead<'a>, WorldSdfCurrentMetricReadError> {
    let descriptor = products
        .product(product_id)
        .ok_or(WorldSdfCurrentMetricReadError::MissingProduct)?;
    if descriptor.kind != FieldProductKind::WorldSdfChunkPages
        || descriptor.product_core().identity != product_id
    {
        return Err(WorldSdfCurrentMetricReadError::WrongProductKind);
    }
    if descriptor.payload_refs.len() != 1
        || descriptor.scope.chunk_ids.len() != 1
        || !descriptor.scope.region_ids.is_empty()
        || !descriptor
            .scope
            .chunk_ids
            .contains(&descriptor.payload_refs[0].chunk_id)
    {
        return Err(WorldSdfCurrentMetricReadError::InvalidProductScope);
    }

    let payload_ref = descriptor.payload_refs[0];
    let metric_ref = *metrics
        .capability(product_id)
        .ok_or(WorldSdfCurrentMetricReadError::MissingMetricCapability)?;
    if metric_ref.payload_ref != payload_ref {
        return Err(WorldSdfCurrentMetricReadError::MetricReferenceMismatch);
    }

    let payload = store
        .chunks
        .get(&payload_ref.chunk_id)
        .ok_or(WorldSdfCurrentMetricReadError::MissingCpuPayload)?;
    if WorldSdfPayloadRef::from(payload) != payload_ref {
        return Err(WorldSdfCurrentMetricReadError::PayloadReferenceMismatch);
    }

    let record = chunks
        .by_chunk_id
        .get(&payload_ref.chunk_id)
        .ok_or(WorldSdfCurrentMetricReadError::MissingRuntimeRecord)?;
    if record.pending_build_generation.is_some()
        || !record.dirty_reasons.is_empty()
        || !matches!(
            record.lifecycle,
            ChunkLifecycleState::Ready | ChunkLifecycleState::Resident
        )
    {
        return Err(WorldSdfCurrentMetricReadError::RuntimeNotReady);
    }
    if record.chunk_id != payload.chunk_id
        || record.chunk_revision != payload.chunk_revision
        || record.chunk_generation != payload.chunk_generation
        || record.build_generation != BuildGeneration(payload.chunk_generation.0)
    {
        return Err(WorldSdfCurrentMetricReadError::RuntimeVersionMismatch);
    }

    if ratify_world_sdf_metric_product_candidate(&WorldSdfMetricProductCandidate::new(
        descriptor.clone(),
        metric_ref,
    ))
    .has_blocking_issues()
    {
        return Err(WorldSdfCurrentMetricReadError::InvalidMetricProduct);
    }
    validate_world_sdf_metric_payload(&metric_ref, payload)
        .map_err(WorldSdfCurrentMetricReadError::InvalidMetricPayload)?;

    Ok(WorldSdfCurrentMetricRead {
        payload,
        metric_ref,
    })
}
