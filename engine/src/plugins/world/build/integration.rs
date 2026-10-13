use super::super::adapters::resources::{
    PartitionConfigResource, RegionInvalidationJournalResource, SdfChunkStoreResource,
};
use super::super::chunks::lifecycle::{
    ChunkLifecycleState, WorldChunkRuntimeMapResource, WorldSdfCommittedExactSource,
};
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::plugin::{WorldAuthorityState, WorldRuntimeState};
use super::jobs::WorldBuildStaleness;
use crate::runtime::{Res, ResMut};
use product::ProductIdentity;
use runen_spatial::ChunkId;
use std::collections::{BTreeMap, VecDeque};
use world_ops::{BuildGeneration, ChunkGeneration, ChunkRevision, DirtyReason};
use world_sdf::{
    FieldProductCandidate, FieldProductDescriptor, FieldProductKind, RegionSdfSummary,
    SdfChunkPayload, WorldSdfExactFormationError, WorldSdfMetricError, WorldSdfMetricPayloadRef,
    WorldSdfMetricProductCandidate, WorldSdfOpenTopExactFormation, WorldSdfVerifiedExactSource,
    ratify_field_product_candidate, ratify_world_sdf_metric_product_candidate,
    validate_world_sdf_metric_payload, verify_world_sdf_open_top_exact_formation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCompletedBuildOrigin {
    ExternalPackage,
    OperationWindow,
}

#[derive(Debug, Clone, runen_ecs::Resource)]
pub struct WorldCompletedBuildOutput {
    pub chunk_id: ChunkId,
    pub target_chunk_revision: ChunkRevision,
    pub target_build_generation: BuildGeneration,
    pub staleness: WorldBuildStaleness,
    pub chunk_payload: SdfChunkPayload,
    pub region_summary: RegionSdfSummary,
    pub(crate) exact_source: Option<WorldSdfCommittedExactSource>,
    pub origin: WorldCompletedBuildOrigin,
}

impl WorldCompletedBuildOutput {
    /// External authors may construct ordinary outputs, never mint or swap
    /// an exact-source certificate directly in the mutable completion queue.
    pub fn generic(
        chunk_id: ChunkId,
        target_chunk_revision: ChunkRevision,
        target_build_generation: BuildGeneration,
        staleness: WorldBuildStaleness,
        chunk_payload: SdfChunkPayload,
        region_summary: RegionSdfSummary,
        origin: WorldCompletedBuildOrigin,
    ) -> Self {
        Self {
            chunk_id, target_chunk_revision, target_build_generation,
            staleness, chunk_payload, region_summary, origin,
            exact_source: None,
        }
    }
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

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct WorldRuntimeSdfMetricCapabilityCatalogResource {
    capabilities: BTreeMap<ProductIdentity, WorldSdfMetricPayloadRef>,
}

impl WorldRuntimeSdfMetricCapabilityCatalogResource {
    pub fn capabilities(&self) -> &BTreeMap<ProductIdentity, WorldSdfMetricPayloadRef> {
        &self.capabilities
    }

    pub fn capability(&self, product_id: ProductIdentity) -> Option<&WorldSdfMetricPayloadRef> {
        self.capabilities.get(&product_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSdfRuntimePayloadPackageError {
    UnsupportedProductKind {
        kind: FieldProductKind,
    },
    PayloadCount {
        count: usize,
    },
    PayloadRefCount {
        count: usize,
    },
    ScopeMismatch,
    PayloadRefMismatch,
    ProductRatificationRejected {
        issue_count: usize,
    },
    MetricProductRatificationRejected {
        issue_count: usize,
    },
    MetricPayloadRejected {
        error: WorldSdfMetricError,
    },
    ExactSourceRequiresMetric,
    ExactSourceRejected(WorldSdfExactFormationError),
    PendingGenerationConflict {
        chunk_id: ChunkId,
        generation: BuildGeneration,
    },
    IntegratedGenerationConflict {
        chunk_id: ChunkId,
        requested: BuildGeneration,
        integrated: BuildGeneration,
    },
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
            Self::ScopeMismatch => formatter
                .write_str("runtime SDF product scope must identify exactly the packaged chunk"),
            Self::PayloadRefMismatch => formatter.write_str(
                "runtime SDF product payload ref must match packaged chunk, revision, and checksum",
            ),
            Self::ProductRatificationRejected { issue_count } => write!(
                formatter,
                "runtime SDF product descriptor failed ratification with {issue_count} issue(s)"
            ),
            Self::MetricProductRatificationRejected { issue_count } => write!(
                formatter,
                "runtime metric SDF capability failed ratification with {issue_count} issue(s)"
            ),
            Self::MetricPayloadRejected { error } => {
                write!(formatter, "runtime metric SDF payload is invalid: {error}")
            }
            Self::ExactSourceRequiresMetric => {
                formatter.write_str("exact source requires a metric product")
            }
            Self::ExactSourceRejected(error) => {
                write!(formatter, "exact source formation rejected: {error}")
            }
            Self::PendingGenerationConflict {
                chunk_id,
                generation,
            } => write!(
                formatter,
                "conflicting runtime SDF package for chunk {chunk_id:?} at pending generation {generation:?}"
            ),
            Self::IntegratedGenerationConflict {
                chunk_id,
                requested,
                integrated,
            } => write!(
                formatter,
                "runtime SDF package for chunk {chunk_id:?} requests generation {requested:?} but integrated generation {integrated:?} is not older"
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
    pub metric_payload_ref: Option<WorldSdfMetricPayloadRef>,
    exact_source: Option<WorldSdfOpenTopExactFormation>,
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
            metric_payload_ref: None,
            exact_source: None,
        }
    }

    pub fn with_metric_capability(mut self, metric_payload_ref: WorldSdfMetricPayloadRef) -> Self {
        self.metric_payload_ref = Some(metric_payload_ref);
        self
    }

    /// Only a domain-formed source+CPU artifact can claim exact provenance.
    /// There is no overload that attaches an independent analytic source to
    /// an already constructed payload with indistinguishable quantized bytes.
    pub fn from_exact_formation(formation: WorldSdfOpenTopExactFormation) -> Self {
        Self {
            descriptor: formation.descriptor().clone(),
            payloads: vec![formation.payload().clone()],
            region_summary: formation.region_summary(),
            metric_payload_ref: Some(formation.metric_ref()),
            exact_source: Some(formation),
        }
    }
}

fn validate_runtime_sdf_payload_package(
    package: &WorldSdfRuntimePayloadPackage,
) -> Result<Option<WorldSdfVerifiedExactSource>, WorldSdfRuntimePayloadPackageError> {
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

    let report =
        ratify_field_product_candidate(&FieldProductCandidate::new(package.descriptor.clone()));
    if report.has_blocking_issues() {
        return Err(
            WorldSdfRuntimePayloadPackageError::ProductRatificationRejected {
                issue_count: report.len(),
            },
        );
    }

    if let Some(metric_payload_ref) = package.metric_payload_ref {
        let metric_report = ratify_world_sdf_metric_product_candidate(
            &WorldSdfMetricProductCandidate::new(package.descriptor.clone(), metric_payload_ref),
        );
        if metric_report.has_blocking_issues() {
            return Err(
                WorldSdfRuntimePayloadPackageError::MetricProductRatificationRejected {
                    issue_count: metric_report.len(),
                },
            );
        }
        validate_world_sdf_metric_payload(&metric_payload_ref, payload)
            .map_err(|error| WorldSdfRuntimePayloadPackageError::MetricPayloadRejected { error })?;
    }

    let qualified = match package.exact_source.as_ref() {
        Some(formation) => {
            // Do not permit public package fields to substitute a product,
            // metric ref, region summary or bytes after the source was formed.
            if package.descriptor != *formation.descriptor()
                || payload != formation.payload()
                || package.region_summary != formation.region_summary()
                || package.metric_payload_ref != Some(formation.metric_ref())
            {
                return Err(WorldSdfRuntimePayloadPackageError::ExactSourceRejected(
                    WorldSdfExactFormationError::WrongPayloadIdentity,
                ));
            }
            Some(
                verify_world_sdf_open_top_exact_formation(
                    formation.source(),
                    formation.partition(),
                    &package.descriptor,
                    formation.metric_ref(),
                    payload,
                    package.region_summary,
                )
                .map_err(WorldSdfRuntimePayloadPackageError::ExactSourceRejected)?,
            )
        }
        None => None,
    };
    Ok(qualified)
}

pub fn enqueue_ratified_world_sdf_payload_package(
    completed: &mut WorldCompletedBuildQueueResource,
    chunks: &mut WorldChunkRuntimeMapResource,
    sdf_store: &SdfChunkStoreResource,
    products: &mut WorldRuntimeSdfProductCatalogResource,
    metric_capabilities: &mut WorldRuntimeSdfMetricCapabilityCatalogResource,
    package: WorldSdfRuntimePayloadPackage,
) -> Result<usize, WorldSdfRuntimePayloadPackageError> {
    let qualified = validate_runtime_sdf_payload_package(&package)?;

    // The catalog is published at enqueue, but integration accepts only the first
    // matching pending generation. Reject a conflicting retry before changing either.
    let payload = &package.payloads[0];
    let generation = BuildGeneration(payload.chunk_generation.0);

    // Source changes on replacement require explicit lineage revision progress;
    // matching formed bytes alone must not silently change the committed f64
    // source under an unchanged source revision.
    if let Some(candidate) = qualified.as_ref() {
        if let Some(previous) = chunks.by_chunk_id.get(&payload.chunk_id)
            .and_then(|record| record.exact_source.as_ref())
        {
            if candidate.source() != previous.evidence.source()
                && candidate.source_revision() <= previous.evidence.source_revision()
            {
                return Err(WorldSdfRuntimePayloadPackageError::ExactSourceRejected(
                    WorldSdfExactFormationError::InvalidProductOrRevision,
                ));
            }
        }
    }

    // A pending retry is distinct from replaying an already integrated chunk.
    // Check actual CPU geometry first: the catalog may still advertise a newer
    // candidate and its declared checksum is not proof of integrated content.
    let integrated = sdf_store
        .chunks
        .get(&payload.chunk_id)
        .map(|chunk| BuildGeneration(chunk.chunk_generation.0));
    let recorded = chunks
        .by_chunk_id
        .get(&payload.chunk_id)
        .map(|record| record.build_generation);
    let stale_generation = integrated
        .filter(|current| generation.0 <= current.0)
        .or_else(|| recorded.filter(|current| generation.0 < current.0));
    if let Some(integrated) = stale_generation {
        return Err(
            WorldSdfRuntimePayloadPackageError::IntegratedGenerationConflict {
                chunk_id: payload.chunk_id,
                requested: generation,
                integrated,
            },
        );
    }

    if let Some(existing) = completed.outputs.iter().find(|output| {
        output.chunk_id == payload.chunk_id
            && output.target_build_generation == generation
            && matches!(output.staleness, WorldBuildStaleness::Current)
    }) {
        let product_id = package.descriptor.product_core().identity;
        let same_metadata = match products.product(product_id) {
            Some(descriptor) => {
                descriptor == &package.descriptor
                    && metric_capabilities.capability(product_id).copied()
                        == package.metric_payload_ref
            }
            None => metric_capabilities.capability(product_id).is_none(),
        };
        let same_pending = chunks
            .by_chunk_id
            .get(&payload.chunk_id)
            .and_then(|record| record.pending_build_generation)
            == Some(generation);
        let same_exact = match (&existing.exact_source, &qualified) {
            (None, None) => true,
            (Some(committed), Some(evidence)) => committed.evidence == *evidence,
            (Some(committed), None) => !committed.eligible_products.contains_key(&product_id),
            (None, Some(_)) => false, // No late promotion of ordinary pending output.
        };
        if same_pending
            && same_metadata
            && same_exact
            && existing.chunk_payload == *payload
            && existing.region_summary == package.region_summary
        {
            if qualified.is_some() {
                let pending = completed
                    .outputs
                    .iter_mut()
                    .find(|output| {
                        output.chunk_id == payload.chunk_id
                            && output.target_build_generation == generation
                            && matches!(output.staleness, WorldBuildStaleness::Current)
                    })
                    .expect("matched pending exact output");
                pending
                    .exact_source
                    .as_mut()
                    .expect("qualified exact alias")
                    .eligible_products
                    .insert(product_id, package.descriptor.clone());
            }
            // A different product identity may legitimately reference the exact same
            // pending bytes; publish its metadata without queueing another build.
            products.products.insert(product_id, package.descriptor);
            match package.metric_payload_ref {
                Some(metric) => {
                    metric_capabilities.capabilities.insert(product_id, metric);
                }
                None => {
                    metric_capabilities.capabilities.remove(&product_id);
                }
            }
            return Ok(0);
        }
        return Err(
            WorldSdfRuntimePayloadPackageError::PendingGenerationConflict {
                chunk_id: payload.chunk_id,
                generation,
            },
        );
    }

    let product_id = package.descriptor.product_core().identity;
    let descriptor = package.descriptor.clone();
    let metric_payload_ref = package.metric_payload_ref;
    let exact_source = qualified.map(|evidence| WorldSdfCommittedExactSource {
        evidence,
        eligible_products: BTreeMap::from([(product_id, descriptor.clone())]),
    });
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
            exact_source: exact_source.clone(),
            origin: WorldCompletedBuildOrigin::ExternalPackage,
        });
        enqueued = enqueued.saturating_add(1);
    }
    products.products.insert(product_id, descriptor);
    match metric_payload_ref {
        Some(metric_payload_ref) => {
            metric_capabilities
                .capabilities
                .insert(product_id, metric_payload_ref);
        }
        None => {
            metric_capabilities.capabilities.remove(&product_id);
        }
    }
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
        let pending_matches = chunks
            .by_chunk_id
            .get(&output.chunk_id)
            .is_some_and(|record| {
                record.pending_build_generation == Some(output.target_build_generation)
            });
        let previous_is_older = sdf_store
            .chunks
            .get(&output.chunk_id)
            .is_none_or(|previous| previous.chunk_generation.0 < output.target_build_generation.0);
        let source_is_valid = output.exact_source.as_ref().is_none_or(|committed| {
            committed.evidence.partition() == &**partition
                && !committed.eligible_products.is_empty()
                && committed.eligible_products.values().all(|descriptor| {
                    committed
                        .evidence
                        .revalidate(
                            descriptor,
                            &output.chunk_payload,
                            output.region_summary,
                            &partition,
                        )
                        .is_ok()
                })
        });
        if !matches!(output.staleness, WorldBuildStaleness::Current)
            || !pending_matches
            || !previous_is_older
            || !payload_matches_target_contract(&output)
            || !source_is_valid
        {
            dropped = dropped.saturating_add(1);
            if pending_matches {
                // Terminally reject only this intent: preserve committed CPU/source
                // truth and release the pending generation for a valid future retry.
                if let Some(record) = chunks.by_chunk_id.get_mut(&output.chunk_id) {
                    record.pending_build_generation = None;
                    if matches!(output.origin, WorldCompletedBuildOrigin::OperationWindow) {
                        record.dirty_reasons.reasons.insert(DirtyReason::Geometry);
                    }
                    record.lifecycle = if !record.dirty_reasons.is_empty() {
                        ChunkLifecycleState::Dirty
                    } else if sdf_store.chunks.contains_key(&output.chunk_id) {
                        ChunkLifecycleState::Ready
                    } else {
                        ChunkLifecycleState::Unloaded
                    };
                }
            }
            continue;
        }

        let record = chunks.ensure_chunk(output.chunk_id);
        record.chunk_revision = output.target_chunk_revision;
        record.chunk_generation = output.chunk_payload.chunk_generation;
        record.build_generation = output.target_build_generation;
        record.pending_build_generation = None;
        record.exact_source = output.exact_source;
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
