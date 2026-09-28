use super::super::adapters::resources::{
    PartitionConfigResource, RegionInvalidationJournalResource, SdfChunkStoreResource,
};
use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::plugin::{WorldAuthorityState, WorldRuntimeState};
use super::jobs::WorldBuildStaleness;
use crate::runtime::{Res, ResMut};
use product::ProductIdentity;
use runen_spatial::ChunkId;
use std::collections::{BTreeMap, VecDeque};
use world_ops::{BuildGeneration, ChunkGeneration, ChunkRevision};
use world_sdf::{
    FieldProductCandidate, FieldProductDescriptor, FieldProductKind, RegionSdfSummary,
    SdfChunkPayload, ratify_field_product_candidate,
};

#[derive(Debug, Clone, runen_ecs::Resource)]
pub struct WorldCompletedBuildOutput {
    pub chunk_id: ChunkId,
    pub target_chunk_revision: ChunkRevision,
    pub target_build_generation: BuildGeneration,
    pub staleness: WorldBuildStaleness,
    pub chunk_payload: SdfChunkPayload,
    pub region_summary: RegionSdfSummary,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WorldCompletedBuildQueueResource {
    pub outputs: VecDeque<WorldCompletedBuildOutput>,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WorldRuntimeSdfProductCatalogResource {
    products: BTreeMap<ProductIdentity, FieldProductDescriptor>,
}

impl WorldRuntimeSdfProductCatalogResource {
    pub fn products(&self) -> &BTreeMap<ProductIdentity, FieldProductDescriptor> {
        &self.products
    }

    pub fn product(&self, product_id: ProductIdentity) -> Option<&FieldProductDescriptor> {
        self.products.get(&product_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfRuntimePayloadPackageError {
    UnsupportedProductKind { kind: FieldProductKind },
    PayloadCount { count: usize },
    PayloadRefCount { count: usize },
    ScopeMismatch,
    PayloadRefMismatch,
    ProductRatificationRejected { issue_count: usize },
}

impl std::fmt::Display for WorldSdfRuntimePayloadPackageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedProductKind { kind } => {
                write!(formatter, "unsupported runtime SDF product kind: {kind:?}")
            }
            Self::PayloadCount { count } => write!(
                formatter,
                "runtime SDF product must contain exactly one payload, found {count}"
            ),
            Self::PayloadRefCount { count } => write!(
                formatter,
                "runtime SDF product descriptor must contain exactly one payload ref, found {count}"
            ),
            Self::ScopeMismatch => formatter.write_str(
                "runtime SDF product scope must identify exactly the packaged chunk",
            ),
            Self::PayloadRefMismatch => formatter.write_str(
                "runtime SDF product payload ref must match packaged chunk, revision, and checksum",
            ),
            Self::ProductRatificationRejected { issue_count } => write!(
                formatter,
                "runtime SDF product descriptor failed ratification with {issue_count} issue(s)"
            ),
        }
    }
}

impl std::error::Error for WorldSdfRuntimePayloadPackageError {}

#[derive(Debug, Clone)]
pub struct WorldSdfRuntimePayloadPackage {
    pub descriptor: FieldProductDescriptor,
    pub payloads: Vec<SdfChunkPayload>,
    pub region_summary: RegionSdfSummary,
}

impl WorldSdfRuntimePayloadPackage {
    pub fn new(
        descriptor: FieldProductDescriptor,
        payloads: Vec<SdfChunkPayload>,
        region_summary: RegionSdfSummary,
    ) -> Self {
        Self {
            descriptor,
            payloads,
            region_summary,
        }
    }
}

fn validate_runtime_sdf_payload_package(
    package: &WorldSdfRuntimePayloadPackage,
) -> Result<(), WorldSdfRuntimePayloadPackageError> {
    if package.descriptor.kind != FieldProductKind::WorldSdfChunkPages {
        return Err(WorldSdfRuntimePayloadPackageError::UnsupportedProductKind {
            kind: package.descriptor.kind,
        });
    }
    if package.payloads.len() != 1 {
        return Err(WorldSdfRuntimePayloadPackageError::PayloadCount {
            count: package.payloads.len(),
        });
    }
    if package.descriptor.payload_refs.len() != 1 {
        return Err(WorldSdfRuntimePayloadPackageError::PayloadRefCount {
            count: package.descriptor.payload_refs.len(),
        });
    }

    let payload = &package.payloads[0];
    if package.descriptor.scope.chunk_ids.len() != 1
        || !package.descriptor.scope.region_ids.is_empty()
        || !package
            .descriptor
            .scope
            .chunk_ids
            .contains(&payload.chunk_id)
    {
        return Err(WorldSdfRuntimePayloadPackageError::ScopeMismatch);
    }

    let payload_ref = &package.descriptor.payload_refs[0];
    if payload_ref.chunk_id != payload.chunk_id
        || payload_ref.chunk_revision != payload.chunk_revision
        || payload_ref.checksum != payload.checksum
    {
        return Err(WorldSdfRuntimePayloadPackageError::PayloadRefMismatch);
    }

    let report = ratify_field_product_candidate(&FieldProductCandidate::new(
        package.descriptor.clone(),
    ));
    if report.has_blocking_issues() {
        return Err(
            WorldSdfRuntimePayloadPackageError::ProductRatificationRejected {
                issue_count: report.len(),
            },
        );
    }
    Ok(())
}

pub fn enqueue_ratified_world_sdf_payload_package(
    completed: &mut WorldCompletedBuildQueueResource,
    chunks: &mut WorldChunkRuntimeMapResource,
    products: &mut WorldRuntimeSdfProductCatalogResource,
    package: WorldSdfRuntimePayloadPackage,
) -> Result<usize, WorldSdfRuntimePayloadPackageError> {
    validate_runtime_sdf_payload_package(&package)?;

    let product_id = package.descriptor.product_core().identity;
    let descriptor = package.descriptor.clone();
    let mut enqueued = 0usize;
    for payload in package.payloads {
        let chunk_id = payload.chunk_id;
        let build_generation = BuildGeneration(payload.chunk_generation.0);
        let record = chunks.ensure_chunk(chunk_id);
        record.pending_build_generation = Some(build_generation);
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: payload.chunk_revision,
            target_build_generation: build_generation,
            staleness: WorldBuildStaleness::Current,
            chunk_payload: payload,
            region_summary: package.region_summary,
        });
        enqueued = enqueued.saturating_add(1);
    }
    products.products.insert(product_id, descriptor);
    Ok(enqueued)
}

#[allow(clippy::too_many_arguments)]
pub fn integrate_completed_build_outputs_system(
    mut completed: ResMut<WorldCompletedBuildQueueResource>,
    mut chunks: ResMut<WorldChunkRuntimeMapResource>,
    mut sdf_store: ResMut<SdfChunkStoreResource>,
    partition: Res<PartitionConfigResource>,
    mut runtime: ResMut<WorldRuntimeState>,
    mut authority: ResMut<WorldAuthorityState>,
    mut render_cache_invalidation: ResMut<WorldRenderCacheInvalidationQueueResource>,
    mut region_invalidation_journal: ResMut<RegionInvalidationJournalResource>,
) {
    let mut integrated = 0_u64;
    let mut dropped = 0_u64;

    while let Some(output) = completed.outputs.pop_front() {
        if !matches!(output.staleness, WorldBuildStaleness::Current) {
            dropped = dropped.saturating_add(1);
            continue;
        }

        let record = chunks.ensure_chunk(output.chunk_id);
        if record.pending_build_generation != Some(output.target_build_generation) {
            dropped = dropped.saturating_add(1);
            continue;
        }
        if !payload_matches_target_contract(&output) {
            dropped = dropped.saturating_add(1);
            continue;
        }

        record.chunk_revision = output.target_chunk_revision;
        record.chunk_generation = output.chunk_payload.chunk_generation;
        record.build_generation = output.target_build_generation;
        record.pending_build_generation = None;
        if record.dirty_reasons.is_empty() {
            if !matches!(record.lifecycle, ChunkLifecycleState::Resident) {
                record.lifecycle = ChunkLifecycleState::Ready;
            }
        } else {
            record.lifecycle = ChunkLifecycleState::Dirty;
        }

        sdf_store
            .chunks
            .insert(output.chunk_id, output.chunk_payload);
        let region_id = partition.region_id_from_chunk_id(output.chunk_id);
        sdf_store
            .region_summaries
            .insert(region_id, output.region_summary);
        render_cache_invalidation.enqueue_integrated_chunk(&partition, output.chunk_id);
        integrated = integrated.saturating_add(1);
        authority.world_revision.0 = authority.world_revision.0.saturating_add(1);
        region_invalidation_journal.append_integration_record(
            &partition,
            output.chunk_id,
            authority.world_revision,
        );
    }

    runtime.integrated_build_outputs = runtime.integrated_build_outputs.saturating_add(integrated);
    runtime.dropped_stale_build_outputs =
        runtime.dropped_stale_build_outputs.saturating_add(dropped);
}

pub fn sync_world_runtime_debug_metrics_system(
    runtime: Res<WorldRuntimeState>,
    authority: Res<WorldAuthorityState>,
    mut debug: ResMut<WorldDebugMetricsResource>,
) {
    debug.integrated_build_outputs = runtime.integrated_build_outputs;
    debug.dropped_stale_build_outputs = runtime.dropped_stale_build_outputs;
    debug.last_world_revision = authority.world_revision.0;
}

fn payload_matches_target_contract(output: &WorldCompletedBuildOutput) -> bool {
    if output.chunk_payload.chunk_id != output.chunk_id {
        return false;
    }
    if output.chunk_payload.chunk_revision != output.target_chunk_revision {
        return false;
    }
    output.chunk_payload.chunk_generation == ChunkGeneration(output.target_build_generation.0)
}
