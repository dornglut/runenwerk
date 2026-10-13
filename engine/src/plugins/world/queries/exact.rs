use super::super::adapters::resources::{PartitionConfigResource, SdfChunkStoreResource};
use super::super::build::integration::WorldRuntimeSdfProductCatalogResource;
use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};
use product::ProductIdentity;
use super::super::chunks::lifecycle::WorldChunkRuntimeRecord;
use runen_spatial::GridPartitionConfig;
use world_ops::BuildGeneration;
use world_sdf::{
    FieldProductConsumerClass, FieldProductFreshness, FieldProductKind, SdfChunkPayload,
    WorldSdfExactFormationError, WorldSdfOpenTopRectInteriorSource, WorldSdfPayloadRef,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldSdfExactQueryBounds {
    min: [f64; 3],
    max: [f64; 3],
}
impl WorldSdfExactQueryBounds {
    pub fn try_new(min: [f64; 3], max: [f64; 3]) -> Result<Self, WorldSdfExactReadError> {
        if !(0..3).all(|i| min[i].is_finite() && max[i].is_finite() && min[i] <= max[i]) {
            return Err(WorldSdfExactReadError::InvalidBounds);
        }
        Ok(Self { min, max })
    }
    pub fn min(self) -> [f64; 3] {
        self.min
    }
    pub fn max(self) -> [f64; 3] {
        self.max
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfExactReadError {
    InvalidBounds,
    MissingProduct,
    UnsupportedExactSource,
    ProductSourceMismatch,
    MissingCpuPayload,
    MissingRuntimeRecord,
    RuntimeNotReady,
    VersionMismatch,
    PartitionMismatch,
    IncompleteCoverage,
    InvalidFormation(WorldSdfExactFormationError),
}
impl std::fmt::Display for WorldSdfExactReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WorldSDF current exact-source read rejected: {self:?}")
    }
}
impl std::error::Error for WorldSdfExactReadError {}

/// Borrows committed CPU geometry. The caller may evaluate only inside this
/// previously verified scope, never a global source callback.
#[derive(Debug)]
pub struct WorldSdfCurrentExactSourceRead<'a> {
    source: WorldSdfOpenTopRectInteriorSource,
    bounds: WorldSdfExactQueryBounds,
    _cpu: &'a SdfChunkPayload,
    _partition: &'a GridPartitionConfig,
    _record: &'a WorldChunkRuntimeRecord,
    _descriptor: &'a world_sdf::FieldProductDescriptor,
}
impl WorldSdfCurrentExactSourceRead<'_> {
    pub const fn bounds(&self) -> WorldSdfExactQueryBounds {
        self.bounds
    }
    pub fn sample_exact_distance_meters(
        &self,
        position: [f64; 3],
    ) -> Result<f64, WorldSdfExactReadError> {
        if !(0..3).all(|i| {
            position[i].is_finite()
                && position[i] >= self.bounds.min[i]
                && position[i] <= self.bounds.max[i]
        }) {
            return Err(WorldSdfExactReadError::IncompleteCoverage);
        }
        Ok(self.source.signed_distance_meters(position))
    }
}

/// Revalidates current public-mutable World CPU data; this is not a sweep,
/// contact-time estimate, GPU-readiness proof or generic SDF capability.
pub fn qualify_current_world_sdf_exact_source_read<'a>(
    product_id: ProductIdentity,
    required_bounds: WorldSdfExactQueryBounds,
    partition: &'a PartitionConfigResource,
    products: &'a WorldRuntimeSdfProductCatalogResource,
    chunks: &'a WorldChunkRuntimeMapResource,
    store: &'a SdfChunkStoreResource,
) -> Result<WorldSdfCurrentExactSourceRead<'a>, WorldSdfExactReadError> {
    use WorldSdfExactReadError as E;
    let descriptor = products.product(product_id).ok_or(E::MissingProduct)?;
    if descriptor.kind != FieldProductKind::WorldSdfChunkPages
        || descriptor.freshness != FieldProductFreshness::Current
        || !matches!(
            descriptor.consumer_class,
            FieldProductConsumerClass::RuntimeRead | FieldProductConsumerClass::CollisionQuery
        )
        || descriptor.product_core().identity != product_id
        || descriptor.payload_refs.len() != 1
    {
        return Err(E::ProductSourceMismatch);
    }
    let reference = descriptor.payload_refs[0];
    if descriptor.scope.chunk_ids.len() != 1
        || !descriptor.scope.region_ids.is_empty()
        || !descriptor.scope.chunk_ids.contains(&reference.chunk_id)
    {
        return Err(E::ProductSourceMismatch);
    }
    let record = chunks
        .by_chunk_id
        .get(&reference.chunk_id)
        .ok_or(E::MissingRuntimeRecord)?;
    let committed = record
        .exact_source
        .as_ref()
        .ok_or(E::UnsupportedExactSource)?;
    if committed.eligible_products.get(&product_id) != Some(descriptor) {
        return Err(E::ProductSourceMismatch);
    }
    if record.pending_build_generation.is_some()
        || !record.dirty_reasons.is_empty()
        || !matches!(
            record.lifecycle,
            ChunkLifecycleState::Ready | ChunkLifecycleState::Resident
        )
    {
        return Err(E::RuntimeNotReady);
    }
    let payload = store
        .chunks
        .get(&reference.chunk_id)
        .ok_or(E::MissingCpuPayload)?;
    if WorldSdfPayloadRef::from(payload) != reference
        || record.chunk_id != payload.chunk_id
        || record.chunk_revision != payload.chunk_revision
        || record.chunk_generation != payload.chunk_generation
        || record.build_generation != BuildGeneration(payload.chunk_generation.0)
    {
        return Err(E::VersionMismatch);
    }
    if committed.evidence.partition() != &**partition {
        return Err(E::PartitionMismatch);
    }
    let region_id = partition.region_id_from_chunk_id(reference.chunk_id);
    let summary = *store
        .region_summaries
        .get(&region_id)
        .ok_or(E::InvalidFormation(
            WorldSdfExactFormationError::InvalidRegionSummary,
        ))?;
    committed
        .evidence
        .revalidate(descriptor, payload, summary, partition)
        .map_err(E::InvalidFormation)?;

    let origin = partition
        .chunk_origin_world_position(reference.chunk_id.world_id, reference.chunk_id.coord)
        .map_err(|_| E::PartitionMismatch)?
        .meters();
    let edge = partition.chunk_edge_meters();
    if !(0..3).all(|i| {
        let upper = origin[i] + edge;
        upper.is_finite() && required_bounds.min[i] >= origin[i] && required_bounds.max[i] < upper
    }) {
        return Err(E::IncompleteCoverage);
    }
    Ok(WorldSdfCurrentExactSourceRead {
        source: committed.evidence.source(),
        bounds: required_bounds,
        _cpu: payload,
        _partition: partition,
        _record: record,
        _descriptor: descriptor,
    })
}
