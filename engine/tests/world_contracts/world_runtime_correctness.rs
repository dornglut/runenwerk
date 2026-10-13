use engine::plugins::world::adapters::resources::{
    BuildQueueResource, OperationLogResource, SdfChunkStoreResource, WorldQuantizationScaleResource,
};
use engine::plugins::world::chunks::DirtyChunkMapResource;
use engine::plugins::world::chunks::lifecycle::{
    ChunkLifecycleState, WorldChunkRuntimeMapResource, WorldChunkRuntimeRecord,
};
use engine::plugins::world::edits::ingress::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::plugin::{
    WorldAuthorityState, WorldPlugin, WorldRuntimeConfig, WorldRuntimeMode, WorldRuntimeState,
};
use engine::plugins::world::{
    build::integration::{
        WorldCompletedBuildOutput, WorldCompletedBuildQueueResource,
        WorldRuntimeSdfMetricCapabilityCatalogResource, WorldRuntimeSdfProductCatalogResource,
        WorldSdfRuntimePayloadPackage, WorldSdfRuntimePayloadPackageError,
        enqueue_ratified_world_sdf_payload_package,
    },
    build::jobs::{WorldBuildJobRuntimeResource, WorldBuildStaleness},
};
use engine::plugins::{FixedStepPlugin, SimulationPlugin};
use engine::prelude::{App, AppFixedStepExt, AppSimulationExt, AuthorityRole};
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{
    BrushShape, BuildGeneration, ChunkGeneration, ChunkRevision, DirtyReason, Operation,
    WorldQuantizationScale, quantize_aabb, quantize_position,
};
use world_sdf::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, RegionSdfSummary, SDF_METRIC_BRICK_SAMPLE_COUNT,
    SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfChunkPayload,
    SdfPageCoord3, SdfPageRecord, WORLD_SDF_METRIC_LAYOUT_REVISION, WorldSdfMetricEncoding,
    WorldSdfMetricError, WorldSdfMetricPayloadRef, WorldSdfPayloadRef,
};

fn world_app() -> App {
    let mut app = App::headless();
    app.add_plugin(SimulationPlugin);
    app.add_plugin(WorldPlugin);
    app
}

fn fixed_world_app() -> App {
    let mut app = world_app();
    app.add_plugin(FixedStepPlugin);
    app
}

fn test_quantization_scale() -> WorldQuantizationScale {
    WorldQuantizationScale::try_new(1024).expect("test quantization scale is valid")
}

fn sdf_chunk_payload(
    chunk_id: ChunkId,
    chunk_revision: ChunkRevision,
    chunk_generation: ChunkGeneration,
    checksum: u64,
) -> SdfChunkPayload {
    SdfChunkPayload {
        chunk_id,
        chunk_revision,
        chunk_generation,
        page_table: Default::default(),
        hierarchy_revision: 0,
        checksum,
    }
}

fn runtime_sdf_descriptor(product_id: u64, payload: &SdfChunkPayload) -> FieldProductDescriptor {
    let mut descriptor = FieldProductDescriptor::new(
        FieldProductId(product_id),
        FieldProductKind::WorldSdfChunkPages,
        FieldProductScope::from_chunks([payload.chunk_id]),
        FieldProductLineage::new(1, "tests.world_runtime"),
    );
    descriptor.consumer_class = FieldProductConsumerClass::RuntimeRead;
    descriptor
        .payload_refs
        .push(WorldSdfPayloadRef::from(payload));
    descriptor
}

fn metric_sdf_payload(
    chunk_id: ChunkId,
    chunk_revision: ChunkRevision,
    chunk_generation: ChunkGeneration,
    checksum: u64,
) -> SdfChunkPayload {
    let mut payload = sdf_chunk_payload(chunk_id, chunk_revision, chunk_generation, checksum);
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };
    for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                page.bricks.insert(
                    [brick_x, brick_y, brick_z],
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata::default(),
                        samples: SdfBrickSamples {
                            distances: vec![0; SDF_METRIC_BRICK_SAMPLE_COUNT],
                        },
                    },
                );
            }
        }
    }
    payload.page_table.insert(SdfPageCoord3::default(), page);
    payload
}

fn metric_payload_ref(
    payload: &SdfChunkPayload,
    max_absolute_error_units: u32,
) -> WorldSdfMetricPayloadRef {
    WorldSdfMetricPayloadRef::try_new(
        WorldSdfPayloadRef::from(payload),
        WorldSdfMetricEncoding::try_new(1024, max_absolute_error_units)
            .expect("positive metric scale should be valid"),
    )
    .expect("supported metric payload ref should be valid")
}

fn enqueue_runtime_sdf_package(
    app: &mut App,
    package: WorldSdfRuntimePayloadPackage,
) -> Result<usize, WorldSdfRuntimePayloadPackageError> {
    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed build queue should exist");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let mut metric_capabilities = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("runtime SDF metric capability catalog should exist");
    let result = {
        let sdf_store = app
            .world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF CPU store should exist")
            .clone();
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &sdf_store,
            &mut products,
            &mut metric_capabilities,
            package,
        )
    };
    app.world_mut().insert_resource(completed);
    app.world_mut().insert_resource(products);
    app.world_mut().insert_resource(metric_capabilities);
    result
}

#[test]
fn dirty_chunk_without_runtime_record_is_bootstrapped_and_built() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }

    let app = app
        .run_for_fixed_steps(1)
        .expect("world plugin systems should run for one fixed step");

    let chunk_runtime = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime map should be available");
    assert!(
        chunk_runtime.by_chunk_id.contains_key(&chunk_id),
        "dirty chunk should create a runtime record before build dispatch"
    );

    let sdf_store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should be available");
    assert!(
        sdf_store.chunks.contains_key(&chunk_id),
        "dirty chunk should produce an integrated chunk payload in the same fixed step"
    );

    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("world authority state should be available");
    assert!(
        authority.world_revision.0 > 0,
        "world revision should advance when build outputs integrate"
    );
}

#[test]
fn ratified_world_sdf_payload_package_flows_through_runtime_intake() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 1, y: 2, z: 3 });
    {
        let mut completed = app
            .world_mut()
            .remove_resource::<WorldCompletedBuildQueueResource>()
            .expect("completed build queue should exist");
        let mut products = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
            .expect("runtime SDF product catalog should exist");
        let mut metric_capabilities = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .expect("runtime SDF metric capability catalog should exist");
        let payload = sdf_chunk_payload(chunk_id, ChunkRevision(11), ChunkGeneration(12), 99);
        let descriptor = runtime_sdf_descriptor(7001, &payload);
        let product_id = descriptor.product_core().identity;
        let enqueued = {
            let sdf_store = app
                .world()
                .resource::<SdfChunkStoreResource>()
                .expect("SDF CPU store should exist")
                .clone();
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
                &sdf_store,
                &mut products,
                &mut metric_capabilities,
                WorldSdfRuntimePayloadPackage::new(
                    descriptor,
                    vec![payload],
                    RegionSdfSummary::default(),
                ),
            )
            .expect("valid runtime SDF product should enqueue")
        };
        assert_eq!(enqueued, 1);
        assert!(
            products.product(product_id).is_some(),
            "accepted runtime SDF package must retain its source-owned product identity"
        );
        app.world_mut().insert_resource(completed);
        app.world_mut().insert_resource(products);
        app.world_mut().insert_resource(metric_capabilities);
    }

    let app = app
        .run_for_fixed_steps(1)
        .expect("runtime intake should integrate through world systems");
    let sdf_store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should be available");
    let payload = sdf_store
        .chunks
        .get(&chunk_id)
        .expect("payload should reach the runtime chunk store through integration");

    assert_eq!(payload.checksum, 99);
    assert_eq!(payload.chunk_revision, ChunkRevision(11));
}

#[test]
fn conflicting_same_generation_runtime_packages_preserve_first_catalog_and_payload() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 43, y: 0, z: 0 });
    let first = metric_sdf_payload(chunk_id, ChunkRevision(7), ChunkGeneration(5), 221);
    let first_descriptor = runtime_sdf_descriptor(7120, &first);
    let product_id = first_descriptor.product_core().identity;
    let first_metric = metric_payload_ref(&first, 2);

    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                first_descriptor.clone(),
                vec![first.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(first_metric),
        )
        .expect("first package should enqueue"),
        1
    );

    let conflicting = metric_sdf_payload(chunk_id, ChunkRevision(8), ChunkGeneration(5), 222);
    let conflicting_descriptor = runtime_sdf_descriptor(7120, &conflicting);
    let error = enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            conflicting_descriptor,
            vec![conflicting.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&conflicting, 2)),
    )
    .expect_err("same generation must not replace different pending source bytes");
    assert!(matches!(
        error,
        WorldSdfRuntimePayloadPackageError::PendingGenerationConflict {
            chunk_id: rejected,
            generation: BuildGeneration(5),
        } if rejected == chunk_id
    ));

    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(product_id),
        Some(&first_descriptor)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .unwrap()
            .capability(product_id),
        Some(&first_metric)
    );
    assert_eq!(
        app.world()
            .resource::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get(&chunk_id)
            .unwrap()
            .pending_build_generation,
        Some(BuildGeneration(5))
    );

    app = app
        .run_for_fixed_steps(1)
        .expect("first package should integrate");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&chunk_id),
        Some(&first)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(product_id),
        Some(&first_descriptor)
    );
}

#[test]
fn identical_pending_runtime_package_is_idempotent_but_changed_bytes_are_not() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 44, y: 0, z: 0 });
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(3), ChunkGeneration(6), 223);
    let descriptor = runtime_sdf_descriptor(7121, &payload);
    let metric = metric_payload_ref(&payload, 2);
    let package = WorldSdfRuntimePayloadPackage::new(
        descriptor.clone(),
        vec![payload.clone()],
        RegionSdfSummary::default(),
    )
    .with_metric_capability(metric);
    assert_eq!(
        enqueue_runtime_sdf_package(&mut app, package.clone()).unwrap(),
        1
    );
    assert_eq!(
        enqueue_runtime_sdf_package(&mut app, package).expect("identical retry is a no-op"),
        0
    );

    // The existing payload reference/checksum is only declared metadata; exact
    // byte equality must still reject an altered candidate with the same ref.
    let mut altered = payload.clone();
    altered
        .page_table
        .get_mut(&SdfPageCoord3::default())
        .unwrap()
        .bricks
        .get_mut(&[0, 0, 0])
        .unwrap()
        .samples
        .distances[0] = 1;
    let error = enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor.clone(),
            vec![altered],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric),
    )
    .expect_err("same ref and generation must not hide changed samples");
    assert!(matches!(
        error,
        WorldSdfRuntimePayloadPackageError::PendingGenerationConflict { .. }
    ));
    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        1
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("one candidate should integrate once");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&chunk_id),
        Some(&payload)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(descriptor.product_core().identity),
        Some(&descriptor)
    );
}

#[test]
fn matching_pending_bytes_support_an_independent_product_identity() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 46, y: 0, z: 0 });
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(4), ChunkGeneration(8), 226);
    let primary = runtime_sdf_descriptor(7123, &payload);
    let secondary = runtime_sdf_descriptor(7124, &payload);
    let metric = metric_payload_ref(&payload, 2);
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                primary.clone(),
                vec![payload.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric),
        )
        .unwrap(),
        1
    );
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                secondary.clone(),
                vec![payload.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric),
        )
        .expect("identical geometry may be referenced by another product"),
        0
    );
    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        1
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("one shared chunk integrates");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&chunk_id),
        Some(&payload)
    );
    for descriptor in [primary, secondary] {
        let product_id = descriptor.product_core().identity;
        assert_eq!(
            app.world()
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap()
                .product(product_id),
            Some(&descriptor)
        );
        assert_eq!(
            app.world()
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap()
                .capability(product_id),
            Some(&metric)
        );
    }
}

#[test]
fn newer_generation_supersedes_older_queued_runtime_package() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 45, y: 0, z: 0 });
    let old = sdf_chunk_payload(chunk_id, ChunkRevision(1), ChunkGeneration(1), 224);
    let old_descriptor = runtime_sdf_descriptor(7122, &old);
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                old_descriptor,
                vec![old],
                RegionSdfSummary::default(),
            ),
        )
        .unwrap(),
        1
    );
    let newer = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(2), 225);
    let mut newer_descriptor = runtime_sdf_descriptor(7122, &newer);
    newer_descriptor.lineage.source_revision = 2;
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                newer_descriptor.clone(),
                vec![newer.clone()],
                RegionSdfSummary::default(),
            ),
        )
        .unwrap(),
        1
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("newer generation should supersede the old pending candidate");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&chunk_id),
        Some(&newer)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(newer_descriptor.product_core().identity),
        Some(&newer_descriptor)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .unwrap()
            .capability(newer_descriptor.product_core().identity),
        None
    );
}

#[test]
fn queued_dirty_build_does_not_consume_runtime_package_pending_generation() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 47, y: 0, z: 0 });

    // Leave an already-admitted dirty BuildQueue item unprocessed for one tick.
    app.world_mut()
        .resource_mut::<WorldBuildJobRuntimeResource>()
        .unwrap()
        .max_jobs_per_update = 0;
    app.world_mut()
        .resource_mut::<DirtyChunkMapResource>()
        .unwrap()
        .mark_dirty(chunk_id, DirtyReason::Geometry);
    app = app
        .run_for_fixed_steps(1)
        .expect("dirty chunk should enter the queue without dispatch");
    assert!(
        app.world()
            .resource::<BuildQueueResource>()
            .unwrap()
            .contains_chunk(chunk_id),
        "dirty build item must remain queued"
    );

    // Runtime-package intake claims generation 1, while that older queue item
    // still exists. Dispatch must not replace the package or erase dirtiness.
    app.world_mut()
        .resource_mut::<WorldBuildJobRuntimeResource>()
        .unwrap()
        .max_jobs_per_update = 4;
    let package_payload = sdf_chunk_payload(chunk_id, ChunkRevision(1), ChunkGeneration(1), 231);
    let package_descriptor = runtime_sdf_descriptor(7125, &package_payload);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            package_descriptor,
            vec![package_payload.clone()],
            RegionSdfSummary::default(),
        ),
    )
    .expect("external package should claim its own pending build generation");

    app = app
        .run_for_fixed_steps(1)
        .expect("external payload should integrate without losing dirty reasons");
    let store = app.world().resource::<SdfChunkStoreResource>().unwrap();
    assert_eq!(store.chunks.get(&chunk_id), Some(&package_payload));
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&chunk_id)
        .unwrap();
    assert!(
        matches!(record.lifecycle, ChunkLifecycleState::Dirty),
        "pending package must not consume an unrelated queued dirty build"
    );
    assert!(
        !record.dirty_reasons.is_empty(),
        "dirty reasons must survive the external payload integration"
    );
    assert!(record.pending_build_generation.is_none());
    assert!(
        app.world()
            .resource::<WorldBuildJobRuntimeResource>()
            .unwrap()
            .dropped_jobs
            > 0,
        "stale queued dispatch should be rejected"
    );

    // With no pending package left, the still-dirty chunk rebuilds ordinarily.
    app = app
        .run_for_fixed_steps(1)
        .expect("preserved dirty source should rebuild at the next generation");
    let next = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .get(&chunk_id)
        .unwrap();
    assert_eq!(next.chunk_generation, ChunkGeneration(2));
    assert_eq!(next.chunk_revision, ChunkRevision(2));
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&chunk_id)
        .unwrap();
    assert!(
        matches!(record.lifecycle, ChunkLifecycleState::Ready),
        "follow-up rebuild should recover the ordinary ready lifecycle"
    );
    assert!(record.dirty_reasons.is_empty());
    assert!(record.pending_build_generation.is_none());
}

#[test]
fn completed_runtime_generation_cannot_be_replayed_after_integration() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 48, y: 0, z: 0 });
    let accepted = metric_sdf_payload(chunk_id, ChunkRevision(2), ChunkGeneration(2), 240);
    let descriptor = runtime_sdf_descriptor(7126, &accepted);
    let product_id = descriptor.product_core().identity;
    let metric = metric_payload_ref(&accepted, 2);
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                descriptor.clone(),
                vec![accepted.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric),
        )
        .unwrap(),
        1
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("generation two should integrate");

    let revision = app
        .world()
        .resource::<WorldAuthorityState>()
        .unwrap()
        .world_revision;
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&chunk_id)
        .unwrap()
        .clone();

    for (generation, hierarchy_revision) in [
        (ChunkGeneration(1), 0),
        (ChunkGeneration(2), 0),
        (ChunkGeneration(2), 1),
    ] {
        // Exact same-generation retries and altered source bytes that reuse
        // their declared checksum are both rejected after integration.
        let mut replay = metric_sdf_payload(chunk_id, ChunkRevision(2), generation, 240);
        replay.hierarchy_revision = hierarchy_revision;
        let replay_descriptor = runtime_sdf_descriptor(7126, &replay);
        let error = enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                replay_descriptor,
                vec![replay.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric_payload_ref(&replay, 2)),
        )
        .expect_err("completed source generation must not be re-admitted");
        assert!(matches!(
            error,
            WorldSdfRuntimePayloadPackageError::IntegratedGenerationConflict {
                chunk_id: rejected,
                requested,
                integrated: BuildGeneration(2),
            } if rejected == chunk_id && requested.0 == generation.0
        ));
        assert_eq!(
            app.world()
                .resource::<WorldCompletedBuildQueueResource>()
                .unwrap()
                .outputs
                .len(),
            0
        );
        assert_eq!(
            app.world()
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap()
                .product(product_id),
            Some(&descriptor)
        );
        assert_eq!(
            app.world()
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap()
                .capability(product_id),
            Some(&metric)
        );
        assert_eq!(
            app.world()
                .resource::<SdfChunkStoreResource>()
                .unwrap()
                .chunks
                .get(&chunk_id),
            Some(&accepted)
        );
        let unchanged_record = app
            .world()
            .resource::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get(&chunk_id)
            .unwrap();
        assert_eq!(unchanged_record.chunk_revision, record.chunk_revision);
        assert_eq!(unchanged_record.chunk_generation, record.chunk_generation);
        assert_eq!(unchanged_record.build_generation, record.build_generation);
        assert_eq!(
            unchanged_record.pending_build_generation,
            record.pending_build_generation
        );
        assert_eq!(unchanged_record.lifecycle, record.lifecycle);
        assert_eq!(unchanged_record.gameplay_locked, record.gameplay_locked);
        assert_eq!(
            unchanged_record.dirty_reasons.is_empty(),
            record.dirty_reasons.is_empty()
        );
        assert_eq!(
            app.world()
                .resource::<WorldAuthorityState>()
                .unwrap()
                .world_revision,
            revision
        );
    }

    let next = metric_sdf_payload(chunk_id, ChunkRevision(3), ChunkGeneration(3), 241);
    let mut next_descriptor = runtime_sdf_descriptor(7126, &next);
    next_descriptor.lineage.source_revision = 2;
    assert_eq!(
        enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                next_descriptor.clone(),
                vec![next.clone()],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric_payload_ref(&next, 2)),
        )
        .expect("strictly newer generation remains accepted"),
        1
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("generation three should integrate");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&chunk_id),
        Some(&next)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(product_id),
        Some(&next_descriptor)
    );
    assert!(
        app.world()
            .resource::<WorldAuthorityState>()
            .unwrap()
            .world_revision
            .0
            > revision.0
    );
}

#[test]
fn current_metric_read_rejects_enqueue_only_and_qualifies_integrated_cpu_payload() {
    use engine::plugins::world::queries::{
        WorldSdfCurrentMetricReadError, qualify_current_world_sdf_metric_read,
    };
    use runen_spatial::GridPartitionConfig;

    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(4), ChunkGeneration(4), 330);
    let descriptor = runtime_sdf_descriptor(7200, &payload);
    let product_id = descriptor.product_core().identity;
    let metric = metric_payload_ref(&payload, 2);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor,
            vec![payload.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric),
    )
    .expect("metric source enqueues before CPU integration");

    let read = |app: &App| {
        let world = app.world();
        qualify_current_world_sdf_metric_read(
            product_id,
            world
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap(),
            world
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap(),
            world.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            world.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .map(|view| view.metric_ref())
    };
    assert!(matches!(
        read(&app),
        Err(WorldSdfCurrentMetricReadError::MissingCpuPayload)
    ));
    app = app
        .run_for_fixed_steps(1)
        .expect("source must integrate on a fixed tick");

    let world = app.world();
    let view = qualify_current_world_sdf_metric_read(
        product_id,
        world
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap(),
        world
            .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .unwrap(),
        world.resource::<WorldChunkRuntimeMapResource>().unwrap(),
        world.resource::<SdfChunkStoreResource>().unwrap(),
    )
    .expect("integrated matching metric source must qualify");
    assert_eq!(view.payload(), &payload);
    assert_eq!(view.metric_ref(), metric);
    let partition = GridPartitionConfig::try_new(4.0, [8, 8, 8]).expect("valid metric partition");
    let sample = view
        .sample(&partition, [0.5, 0.5, 0.5])
        .expect("declared zero metric samples are readable");
    assert_eq!(sample.signed_distance_estimate_meters(), 0.0);
    assert_eq!(sample.max_absolute_error_meters(), 2.0 / 1024.0);
    // Enqueue-time metadata now refers to B, while the integrated CPU
    // payload still contains A. That is not a current metric read.
    let replacement = metric_sdf_payload(chunk_id, ChunkRevision(5), ChunkGeneration(5), 332);
    let mut new_descriptor = runtime_sdf_descriptor(7200, &replacement);
    new_descriptor.lineage.source_revision = 5;
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            new_descriptor,
            vec![replacement.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&replacement, 2)),
    )
    .expect("strictly newer pending product should enqueue");
    assert!(matches!(
        read(&app),
        Err(WorldSdfCurrentMetricReadError::PayloadReferenceMismatch)
    ));
    app = app
        .run_for_fixed_steps(1)
        .expect("replacement geometry should integrate");
    assert_eq!(read(&app), Ok(metric_payload_ref(&replacement, 2)));
}

#[test]
fn current_metric_read_fails_closed_on_dirty_pending_or_wrong_cpu_revision() {
    use engine::plugins::world::queries::{
        WorldSdfCurrentMetricReadError, qualify_current_world_sdf_metric_read,
    };

    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 49, y: 0, z: 0 });
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(3), ChunkGeneration(3), 331);
    let descriptor = runtime_sdf_descriptor(7201, &payload);
    let product_id = descriptor.product_core().identity;
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor,
            vec![payload.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&payload, 2)),
    )
    .unwrap();
    app = app.run_for_fixed_steps(1).unwrap();

    let status = |app: &App| {
        let world = app.world();
        qualify_current_world_sdf_metric_read(
            product_id,
            world
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap(),
            world
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap(),
            world.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            world.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .map(|_| ())
    };
    assert!(status(&app).is_ok());

    {
        let record = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get_mut(&chunk_id)
            .unwrap();
        record.pending_build_generation = Some(BuildGeneration(4));
    }
    assert!(matches!(
        status(&app),
        Err(WorldSdfCurrentMetricReadError::RuntimeNotReady)
    ));
    {
        let record = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get_mut(&chunk_id)
            .unwrap();
        record.pending_build_generation = None;
        record.lifecycle = ChunkLifecycleState::Dirty;
    }
    assert!(matches!(
        status(&app),
        Err(WorldSdfCurrentMetricReadError::RuntimeNotReady)
    ));
    {
        let record = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get_mut(&chunk_id)
            .unwrap();
        record.lifecycle = ChunkLifecycleState::Ready;
        record.chunk_generation = ChunkGeneration(99);
    }
    assert!(matches!(
        status(&app),
        Err(WorldSdfCurrentMetricReadError::RuntimeVersionMismatch)
    ));
    {
        let record = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get_mut(&chunk_id)
            .unwrap();
        record.chunk_generation = ChunkGeneration(3);
    }
    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .get_mut(&chunk_id)
        .unwrap()
        .checksum = 77;
    assert!(matches!(
        status(&app),
        Err(WorldSdfCurrentMetricReadError::PayloadReferenceMismatch)
    ));
}

#[test]
fn current_metric_read_rejects_missing_mismatched_and_invalid_cpu_authorities() {
    use engine::plugins::world::queries::{
        WorldSdfCurrentMetricReadError, qualify_current_world_sdf_metric_read,
    };

    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 50, y: 0, z: 0 });
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(4), ChunkGeneration(4), 440);
    let descriptor = runtime_sdf_descriptor(7202, &payload);
    let product_id = descriptor.product_core().identity;
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor,
            vec![payload.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&payload, 2)),
    )
    .expect("qualified metric source enqueues");
    app = app.run_for_fixed_steps(1).expect("CPU payload integrates");

    let world = app.world();
    let products = world
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .unwrap();
    let metrics = world
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .unwrap();
    let runtime = world.resource::<WorldChunkRuntimeMapResource>().unwrap();
    let store = world.resource::<SdfChunkStoreResource>().unwrap();
    let admit = |selected_metrics: &WorldRuntimeSdfMetricCapabilityCatalogResource,
                 selected_runtime: &WorldChunkRuntimeMapResource,
                 selected_store: &SdfChunkStoreResource| {
        qualify_current_world_sdf_metric_read(
            product_id,
            products,
            selected_metrics,
            selected_runtime,
            selected_store,
        )
        .map(|_| ())
    };

    assert_eq!(admit(metrics, runtime, store), Ok(()));
    let unknown_id = runtime_sdf_descriptor(7203, &payload)
        .product_core()
        .identity;
    assert!(matches!(
        qualify_current_world_sdf_metric_read(unknown_id, products, metrics, runtime, store),
        Err(WorldSdfCurrentMetricReadError::MissingProduct)
    ));
    assert_eq!(
        admit(
            &WorldRuntimeSdfMetricCapabilityCatalogResource::default(),
            runtime,
            store,
        ),
        Err(WorldSdfCurrentMetricReadError::MissingMetricCapability)
    );
    assert_eq!(
        admit(metrics, &WorldChunkRuntimeMapResource::default(), store),
        Err(WorldSdfCurrentMetricReadError::MissingRuntimeRecord)
    );
    assert_eq!(
        admit(metrics, runtime, &SdfChunkStoreResource::default()),
        Err(WorldSdfCurrentMetricReadError::MissingCpuPayload)
    );

    // Catalog references must belong to the selected product's actual CPU
    // source, even when each catalog was independently ratified at intake.
    let mut other_app = fixed_world_app();
    let other_payload = metric_sdf_payload(chunk_id, ChunkRevision(5), ChunkGeneration(5), 441);
    enqueue_runtime_sdf_package(
        &mut other_app,
        WorldSdfRuntimePayloadPackage::new(
            runtime_sdf_descriptor(7202, &other_payload),
            vec![other_payload.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&other_payload, 2)),
    )
    .expect("separate later source enqueues");
    let other_world = other_app.world();
    let other_metrics = other_world
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .unwrap();
    assert_eq!(
        admit(other_metrics, runtime, store),
        Err(WorldSdfCurrentMetricReadError::MetricReferenceMismatch)
    );

    let mut unloaded = (*runtime).clone();
    unloaded.by_chunk_id.get_mut(&chunk_id).unwrap().lifecycle = ChunkLifecycleState::Unloaded;
    assert_eq!(
        admit(metrics, &unloaded, store),
        Err(WorldSdfCurrentMetricReadError::RuntimeNotReady)
    );
    let mut mismatched_generation = (*runtime).clone();
    mismatched_generation
        .by_chunk_id
        .get_mut(&chunk_id)
        .unwrap()
        .build_generation = BuildGeneration(99);
    assert_eq!(
        admit(metrics, &mismatched_generation, store),
        Err(WorldSdfCurrentMetricReadError::RuntimeVersionMismatch)
    );

    // A producer-declared checksum is not a content hash: layout validation
    // must still reject corrupted stored bytes with unchanged declared ref.
    let mut invalid_store = (*store).clone();
    invalid_store
        .chunks
        .get_mut(&chunk_id)
        .unwrap()
        .page_table
        .get_mut(&SdfPageCoord3::default())
        .unwrap()
        .bricks
        .get_mut(&[0, 0, 0])
        .unwrap()
        .samples
        .distances
        .clear();
    assert_eq!(
        admit(metrics, runtime, &invalid_store),
        Err(WorldSdfCurrentMetricReadError::InvalidMetricPayload(
            WorldSdfMetricError::InvalidMetricSampleCount { actual: 0 }
        ))
    );
}

#[test]
fn malformed_runtime_sdf_product_is_rejected_before_intake_mutation() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 4, y: 0, z: 0 });
    let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 44);
    let descriptor = runtime_sdf_descriptor(0, &payload);

    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed build queue should exist");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let mut metric_capabilities = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("runtime SDF metric capability catalog should exist");
    let error = {
        let sdf_store = app
            .world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF CPU store should exist")
            .clone();
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &sdf_store,
            &mut products,
            &mut metric_capabilities,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            ),
        )
        .expect_err("zero product identity must be rejected")
    };

    assert!(completed.outputs.is_empty());
    assert!(products.products().is_empty());
    let chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
    assert!(error.to_string().contains("ratification"));
}

#[test]
fn wrong_runtime_sdf_product_kind_is_rejected_before_intake_mutation() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 5, y: 0, z: 0 });
    let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 45);
    let mut descriptor = runtime_sdf_descriptor(7002, &payload);
    descriptor.kind = FieldProductKind::ScalarDistance;

    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed build queue should exist");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let mut metric_capabilities = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("runtime SDF metric capability catalog should exist");
    let error = {
        let sdf_store = app
            .world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF CPU store should exist")
            .clone();
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &sdf_store,
            &mut products,
            &mut metric_capabilities,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            ),
        )
        .expect_err("non-WorldSdfChunkPages products must be rejected")
    };

    assert!(matches!(
        error,
        WorldSdfRuntimePayloadPackageError::UnsupportedProductKind { .. }
    ));
    assert!(completed.outputs.is_empty());
    assert!(products.products().is_empty());
    let chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
}

#[test]
fn runtime_sdf_package_rejects_zero_or_multiple_payloads_before_intake_mutation() {
    for payload_count in [0_usize, 2] {
        let mut app = fixed_world_app();
        let chunk_id = ChunkId::new(
            WorldId::new(0),
            ChunkCoord3 {
                x: 6 + payload_count as i64,
                y: 0,
                z: 0,
            },
        );
        let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 46);
        let descriptor = runtime_sdf_descriptor(7003 + payload_count as u64, &payload);
        let payloads = match payload_count {
            0 => Vec::new(),
            2 => vec![payload.clone(), payload],
            _ => unreachable!(),
        };

        let mut completed = app
            .world_mut()
            .remove_resource::<WorldCompletedBuildQueueResource>()
            .expect("completed build queue should exist");
        let mut products = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
            .expect("runtime SDF product catalog should exist");
        let mut metric_capabilities = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .expect("runtime SDF metric capability catalog should exist");
        let error = {
            let sdf_store = app
                .world()
                .resource::<SdfChunkStoreResource>()
                .expect("SDF CPU store should exist")
                .clone();
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
                &sdf_store,
                &mut products,
                &mut metric_capabilities,
                WorldSdfRuntimePayloadPackage::new(
                    descriptor,
                    payloads,
                    RegionSdfSummary::default(),
                ),
            )
            .expect_err("runtime SDF package cardinality must be exactly one")
        };

        assert!(matches!(
            error,
            WorldSdfRuntimePayloadPackageError::PayloadCount { count }
                if count == payload_count
        ));
        assert!(completed.outputs.is_empty());
        assert!(products.products().is_empty());
        let chunks = app
            .world()
            .resource::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
    }
}

#[test]
fn runtime_sdf_package_rejects_zero_or_multiple_payload_refs_before_intake_mutation() {
    for payload_ref_count in [0_usize, 2] {
        let mut app = fixed_world_app();
        let chunk_id = ChunkId::new(
            WorldId::new(0),
            ChunkCoord3 {
                x: 9 + payload_ref_count as i64,
                y: 0,
                z: 0,
            },
        );
        let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 47);
        let mut descriptor = runtime_sdf_descriptor(7006 + payload_ref_count as u64, &payload);
        descriptor.payload_refs = match payload_ref_count {
            0 => Vec::new(),
            2 => {
                let payload_ref = WorldSdfPayloadRef::from(&payload);
                vec![
                    WorldSdfPayloadRef {
                        chunk_id: payload_ref.chunk_id,
                        chunk_revision: payload_ref.chunk_revision,
                        checksum: payload_ref.checksum,
                    },
                    payload_ref,
                ]
            }
            _ => unreachable!(),
        };

        let mut completed = app
            .world_mut()
            .remove_resource::<WorldCompletedBuildQueueResource>()
            .expect("completed build queue should exist");
        let mut products = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
            .expect("runtime SDF product catalog should exist");
        let mut metric_capabilities = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .expect("runtime SDF metric capability catalog should exist");
        let error = {
            let sdf_store = app
                .world()
                .resource::<SdfChunkStoreResource>()
                .expect("SDF CPU store should exist")
                .clone();
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
                &sdf_store,
                &mut products,
                &mut metric_capabilities,
                WorldSdfRuntimePayloadPackage::new(
                    descriptor,
                    vec![payload],
                    RegionSdfSummary::default(),
                ),
            )
            .expect_err("runtime SDF package must contain exactly one payload ref")
        };

        assert!(matches!(
            error,
            WorldSdfRuntimePayloadPackageError::PayloadRefCount { count }
                if count == payload_ref_count
        ));
        assert!(completed.outputs.is_empty());
        assert!(products.products().is_empty());
        let chunks = app
            .world()
            .resource::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
    }
}

#[test]
fn runtime_sdf_scope_mismatch_is_rejected_before_intake_mutation() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 12, y: 0, z: 0 });
    let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 48);
    let mut descriptor = runtime_sdf_descriptor(7009, &payload);
    descriptor.scope = FieldProductScope::from_chunks([ChunkId::new(
        WorldId::new(0),
        ChunkCoord3 { x: 13, y: 0, z: 0 },
    )]);

    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed build queue should exist");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let mut metric_capabilities = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("runtime SDF metric capability catalog should exist");
    let error = {
        let sdf_store = app
            .world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF CPU store should exist")
            .clone();
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &sdf_store,
            &mut products,
            &mut metric_capabilities,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            ),
        )
        .expect_err("runtime SDF product scope must match the packaged chunk")
    };

    assert!(matches!(
        error,
        WorldSdfRuntimePayloadPackageError::ScopeMismatch
    ));
    assert!(completed.outputs.is_empty());
    assert!(products.products().is_empty());
    let chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
}

#[test]
fn runtime_sdf_payload_ref_chunk_revision_and_checksum_mismatches_are_rejected_before_intake_mutation()
 {
    for mismatch in 0_u8..3 {
        let mut app = fixed_world_app();
        let chunk_id = ChunkId::new(
            WorldId::new(0),
            ChunkCoord3 {
                x: 14 + i64::from(mismatch),
                y: 0,
                z: 0,
            },
        );
        let payload = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(3), 49);
        let mut descriptor = runtime_sdf_descriptor(7010 + u64::from(mismatch), &payload);
        match mismatch {
            0 => {
                descriptor.payload_refs[0].chunk_id = ChunkId::new(
                    WorldId::new(0),
                    ChunkCoord3 {
                        x: chunk_id.coord.x + 1,
                        y: chunk_id.coord.y,
                        z: chunk_id.coord.z,
                    },
                );
            }
            1 => {
                descriptor.payload_refs[0].chunk_revision =
                    ChunkRevision(payload.chunk_revision.0 + 1);
            }
            2 => descriptor.payload_refs[0].checksum = payload.checksum + 1,
            _ => unreachable!(),
        }

        let mut completed = app
            .world_mut()
            .remove_resource::<WorldCompletedBuildQueueResource>()
            .expect("completed build queue should exist");
        let mut products = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
            .expect("runtime SDF product catalog should exist");
        let mut metric_capabilities = app
            .world_mut()
            .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .expect("runtime SDF metric capability catalog should exist");
        let error = {
            let sdf_store = app
                .world()
                .resource::<SdfChunkStoreResource>()
                .expect("SDF CPU store should exist")
                .clone();
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
                &sdf_store,
                &mut products,
                &mut metric_capabilities,
                WorldSdfRuntimePayloadPackage::new(
                    descriptor,
                    vec![payload],
                    RegionSdfSummary::default(),
                ),
            )
            .expect_err("descriptor payload ref mismatch must be rejected")
        };

        assert!(matches!(
            error,
            WorldSdfRuntimePayloadPackageError::PayloadRefMismatch
        ));
        assert!(completed.outputs.is_empty());
        assert!(products.products().is_empty());
        let chunks = app
            .world()
            .resource::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        assert!(!chunks.by_chunk_id.contains_key(&chunk_id));
    }
}

#[test]
fn ingress_rejects_operations_in_client_replica_mode() {
    let mut app = world_app();
    {
        let world_runtime = app
            .world_mut()
            .resource_mut::<WorldRuntimeConfig>()
            .expect("world runtime config should exist");
        world_runtime.mode = WorldRuntimeMode::ReadOnly;
    }

    let quantization_scale = test_quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::CsgAdd {
            brush: BrushShape::Sphere {
                center_q: quantize_position([0.0, 0.0, 0.0], quantization_scale),
                radius_q: 256,
            },
            material_channel: 1,
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], quantization_scale),
        WorldEditIngressMeta {
            planet_id: WorldId::new(0),
            deterministic_seed: 404,
        },
    );
    assert!(
        op_id.is_none(),
        "client-replica world runtime mode must reject world edit ingress mutations"
    );

    let op_log = app
        .world()
        .resource::<OperationLogResource>()
        .expect("world operation log should exist");
    assert!(
        op_log.operations.is_empty(),
        "rejected ingress should not append world operation records"
    );
    let dirty = app
        .world()
        .resource::<DirtyChunkMapResource>()
        .expect("dirty map should exist");
    assert!(
        dirty.by_chunk.is_empty(),
        "rejected ingress should not mutate dirty chunk invalidation state"
    );
}

#[test]
fn world_runtime_mode_tracks_authority_role() {
    let mut app = world_app();

    let initial_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        initial_mode,
        WorldRuntimeMode::Writable,
        "local default authority should initialize world runtime in authoritative mode"
    );

    app.set_authority_role(AuthorityRole::Client);
    let client_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        client_mode,
        WorldRuntimeMode::ReadOnly,
        "set_authority_role(Client) should immediately switch world runtime mode"
    );

    app.set_authority_role(AuthorityRole::Server);
    let server_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        server_mode,
        WorldRuntimeMode::Writable,
        "set_authority_role(Server) should switch world runtime mode back to authoritative"
    );
}

#[test]
fn ingress_invalidation_uses_world_quantization_scale() {
    let mut app = world_app();
    let quantization_scale =
        WorldQuantizationScale::try_new(1).expect("test quantization scale should be valid");
    {
        let configured_scale = app
            .world_mut()
            .resource_mut::<WorldQuantizationScaleResource>()
            .expect("world quantization scale should be available");
        **configured_scale = quantization_scale;
    }

    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::Stamp {
            stamp_id: "tests.world-quantization-scale-ingress".to_string(),
            anchor_q: Default::default(),
            payload: vec![1, 2, 3],
        },
        quantize_aabb([40.0, 0.0, 0.0], [40.0, 0.0, 0.0], quantization_scale),
        WorldEditIngressMeta {
            planet_id: WorldId::new(0),
            deterministic_seed: 77,
        },
    );
    assert!(op_id.is_some(), "world ingress should append operation");

    let dirty = app
        .world()
        .resource::<DirtyChunkMapResource>()
        .expect("world dirty map should be available");
    let expected_chunk = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 1, y: 0, z: 0 });
    assert!(
        dirty.by_chunk.contains_key(&expected_chunk),
        "ingress invalidation must dequantize bounds using Runenwerk world quantization policy"
    );
}

#[test]
fn world_revision_advances_only_for_integrated_outputs() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 1, y: 1, z: 1 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("first dirty build should integrate");
    let revision_after_integrate = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert!(revision_after_integrate > 0);

    app = app
        .run_for_fixed_steps(1)
        .expect("idle fixed step should not change world revision");
    let revision_after_idle = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert_eq!(
        revision_after_idle, revision_after_integrate,
        "world revision must remain stable without accepted integrations"
    );

    {
        let runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        let record = runtime_chunks.ensure_chunk(chunk_id);
        record.lifecycle = ChunkLifecycleState::Rebuilding;
        record.pending_build_generation = Some(BuildGeneration(9));
    }
    {
        let completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(99),
            target_build_generation: BuildGeneration(8),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: sdf_chunk_payload(
                chunk_id,
                ChunkRevision(99),
                ChunkGeneration::default(),
                0,
            ),
            region_summary: RegionSdfSummary::default(),
            exact_source: None,
            origin: engine::plugins::world::build::WorldCompletedBuildOrigin::ExternalPackage,
        });
    }

    app = app
        .run_for_fixed_steps(1)
        .expect("stale output should be dropped without revision bump");
    let revision_after_stale = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert_eq!(
        revision_after_stale, revision_after_integrate,
        "world revision must not advance for dropped stale outputs"
    );
    let runtime = app
        .world()
        .resource::<WorldRuntimeState>()
        .expect("runtime state should exist");
    assert!(
        runtime.dropped_stale_build_outputs > 0,
        "stale output path should increment dropped count"
    );
}

#[test]
fn dirty_reasons_while_rebuilding_are_preserved_for_followup_build() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 3, y: 2, z: -1 });
    {
        let runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(4),
                chunk_generation: Default::default(),
                build_generation: BuildGeneration(4),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(5)),
                gameplay_locked: false,
                exact_source: None,
                exact_read_recovery_required: false,
            },
        );
    }
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }
    {
        let completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(5),
            target_build_generation: BuildGeneration(5),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: sdf_chunk_payload(chunk_id, ChunkRevision(5), ChunkGeneration(5), 0),
            region_summary: RegionSdfSummary::default(),
            exact_source: None,
            origin: engine::plugins::world::build::WorldCompletedBuildOrigin::ExternalPackage,
        });
    }

    let app = app
        .run_for_fixed_steps(1)
        .expect("integration should preserve rebuild-time dirty reasons");
    let runtime_chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    let record = runtime_chunks
        .by_chunk_id
        .get(&chunk_id)
        .expect("runtime record should remain present");
    assert!(
        matches!(record.lifecycle, ChunkLifecycleState::Dirty),
        "chunk must re-enter Dirty lifecycle when new dirty reasons arrive during rebuild"
    );
    assert!(
        !record.dirty_reasons.is_empty(),
        "dirty reasons merged during rebuild must be preserved for follow-up dispatch"
    );
    assert!(
        record.pending_build_generation.is_none(),
        "accepted integration clears pending generation before follow-up rebuild"
    );
}

#[test]
fn stamp_operation_produces_authoritative_chunk_payload() {
    let mut app = fixed_world_app();

    let quantization_scale = test_quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::Stamp {
            stamp_id: "tests.stamp-authority".to_string(),
            anchor_q: Default::default(),
            payload: vec![9, 9, 9],
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], quantization_scale),
        WorldEditIngressMeta {
            planet_id: WorldId::new(0),
            deterministic_seed: 101,
        },
    );
    assert!(
        op_id.is_some(),
        "stamp operation should be accepted by ingress"
    );

    let app = app
        .run_for_fixed_steps(1)
        .expect("stamp operation should build and integrate");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist after integration");
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = store
        .chunks
        .get(&chunk_id)
        .expect("stamp build should produce chunk payload");

    assert!(
        !payload.page_table.is_empty(),
        "stamp operation should produce non-empty authoritative payload content"
    );
    assert!(
        payload
            .page_table
            .values()
            .flat_map(|page| page.bricks.values())
            .any(|brick| brick.metadata.occupancy_mask != 0),
        "stamp payload should carry occupied brick metadata for collision authority"
    );
}

#[test]
fn material_field_edit_preserves_existing_chunk_solidity() {
    let mut app = fixed_world_app();

    let quantization_scale = test_quantization_scale();
    let add_op = submit_world_operation(
        app.world_mut(),
        Operation::CsgAdd {
            brush: BrushShape::Sphere {
                center_q: Default::default(),
                radius_q: 128,
            },
            material_channel: 1,
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], quantization_scale),
        WorldEditIngressMeta {
            planet_id: WorldId::new(0),
            deterministic_seed: 201,
        },
    );
    assert!(add_op.is_some(), "add operation should be accepted");

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("initial csg add should integrate into chunk payload");

    let edit_op = submit_world_operation(
        app.world_mut(),
        Operation::MaterialFieldEdit {
            bounds_q: quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], quantization_scale),
            channel_mask: 0b0100,
            payload: vec![1],
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], quantization_scale),
        WorldEditIngressMeta {
            planet_id: WorldId::new(0),
            deterministic_seed: 202,
        },
    );
    assert!(edit_op.is_some(), "material field edit should be accepted");

    app = app
        .run_for_fixed_steps(1)
        .expect("material field edit should rebuild payload without topology loss");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist");
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = store
        .chunks
        .get(&chunk_id)
        .expect("chunk payload should remain present after material edit");
    let brick_masks = payload
        .page_table
        .values()
        .flat_map(|page| page.bricks.values())
        .map(|brick| brick.metadata.material_channel_mask)
        .collect::<Vec<_>>();

    assert!(
        !brick_masks.is_empty(),
        "material edit should preserve occupied payload pages after csg add"
    );
    assert!(
        brick_masks.iter().any(|mask| (mask & 0b0100) != 0),
        "material edit mask should be reflected in authoritative payload metadata"
    );
}

#[test]
fn integration_drops_output_when_payload_revision_contract_mismatches() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 6, y: 0, z: -2 });
    {
        let runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(10),
                chunk_generation: ChunkGeneration(10),
                build_generation: BuildGeneration(10),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(11)),
                gameplay_locked: false,
                exact_source: None,
                exact_read_recovery_required: false,
            },
        );
    }
    {
        let completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(11),
            target_build_generation: BuildGeneration(11),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: sdf_chunk_payload(chunk_id, ChunkRevision(10), ChunkGeneration(11), 0),
            region_summary: RegionSdfSummary::default(),
            exact_source: None,
            origin: engine::plugins::world::build::WorldCompletedBuildOrigin::ExternalPackage,
        });
    }

    let app = app
        .run_for_fixed_steps(1)
        .expect("integration should reject mismatched payload revision contract");
    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist");
    assert_eq!(
        authority.world_revision.0, 0,
        "authority revision must not advance when integration rejects malformed output"
    );
    let runtime = app
        .world()
        .resource::<WorldRuntimeState>()
        .expect("runtime state should exist");
    assert!(
        runtime.dropped_stale_build_outputs > 0,
        "malformed output should be counted as dropped"
    );
    let runtime_chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    let record = runtime_chunks
        .by_chunk_id
        .get(&chunk_id)
        .expect("runtime record should remain present");
    assert_eq!(
        record.pending_build_generation,
        None,
        "terminally rejected integration must release the failed pending generation"
    );
    assert!(
        record.exact_read_recovery_required,
        "old exact sources must not appear current after a failed replacement"
    );
    assert_eq!(record.chunk_revision, ChunkRevision(10));
    assert_eq!(record.chunk_generation, ChunkGeneration(10));
    assert_eq!(record.build_generation, BuildGeneration(10));
    assert!(
        matches!(record.lifecycle, ChunkLifecycleState::Unloaded),
        "without committed CPU bytes a failed first generation remains unloaded"
    );
}

#[test]
fn integration_drops_output_when_payload_chunk_id_contract_mismatches() {
    let mut app = fixed_world_app();

    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: -4, y: 1, z: 3 });
    let wrong_chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: -3, y: 1, z: 3 });
    {
        let runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(2),
                chunk_generation: ChunkGeneration(2),
                build_generation: BuildGeneration(2),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(3)),
                gameplay_locked: false,
                exact_source: None,
                exact_read_recovery_required: false,
            },
        );
    }
    {
        let completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(3),
            target_build_generation: BuildGeneration(3),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: sdf_chunk_payload(
                wrong_chunk_id,
                ChunkRevision(3),
                ChunkGeneration(3),
                0,
            ),
            region_summary: RegionSdfSummary::default(),
            exact_source: None,
            origin: engine::plugins::world::build::WorldCompletedBuildOrigin::ExternalPackage,
        });
    }

    let app = app
        .run_for_fixed_steps(1)
        .expect("integration should reject mismatched payload chunk-id contract");
    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist");
    assert_eq!(
        authority.world_revision.0, 0,
        "authority revision must not advance when payload chunk-id contract mismatches"
    );
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist");
    assert!(
        !store.chunks.contains_key(&chunk_id),
        "rejected integration must not publish malformed payload into authoritative chunk store"
    );
}

#[test]
fn metric_runtime_sdf_capability_is_retained_and_bytes_integrate_only_in_build_integrate() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 30, y: 0, z: 0 });
    let payload = metric_sdf_payload(chunk_id, ChunkRevision(4), ChunkGeneration(5), 80);
    let descriptor = runtime_sdf_descriptor(7100, &payload);
    let product_id = descriptor.product_core().identity;
    let metric_ref = metric_payload_ref(&payload, 2);

    let enqueued = enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor.clone(),
            vec![payload.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_ref),
    )
    .expect("valid metric runtime SDF package should enqueue");
    assert_eq!(enqueued, 1);

    let products = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("ordinary runtime product catalog should remain available");
    assert_eq!(products.product(product_id), Some(&descriptor));
    let metric_capabilities = app
        .world()
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("metric capability catalog should exist in headless World");
    assert_eq!(
        metric_capabilities.capability(product_id),
        Some(&metric_ref)
    );
    assert!(
        !app.world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF store should exist")
            .chunks
            .contains_key(&chunk_id),
        "runtime intake must not bypass BuildIntegrate byte authority"
    );

    let app = app
        .run_for_fixed_steps(1)
        .expect("accepted metric package should integrate on the fixed World path");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .expect("SDF store should exist")
            .chunks
            .get(&chunk_id),
        Some(&payload)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .expect("metric capability catalog should persist")
            .capability(product_id),
        Some(&metric_ref)
    );
}

#[test]
fn metric_runtime_sdf_rejects_wrapper_exact_ref_mismatches_before_any_mutation() {
    for mismatch in 0_u8..3 {
        let mut app = fixed_world_app();
        let chunk_id = ChunkId::new(
            WorldId::new(0),
            ChunkCoord3 {
                x: 31 + i64::from(mismatch),
                y: 0,
                z: 0,
            },
        );
        let payload = metric_sdf_payload(
            chunk_id,
            ChunkRevision(1),
            ChunkGeneration(1),
            81 + u64::from(mismatch),
        );
        let descriptor = runtime_sdf_descriptor(7101 + u64::from(mismatch), &payload);
        let product_id = descriptor.product_core().identity;
        let mut metric_ref = metric_payload_ref(&payload, 1);
        match mismatch {
            0 => {
                metric_ref.payload_ref.chunk_id = ChunkId::new(
                    WorldId::new(0),
                    ChunkCoord3 {
                        x: chunk_id.coord.x + 1,
                        y: chunk_id.coord.y,
                        z: chunk_id.coord.z,
                    },
                );
            }
            1 => {
                metric_ref.payload_ref.chunk_revision =
                    ChunkRevision(payload.chunk_revision.0.saturating_add(1));
            }
            2 => {
                metric_ref.payload_ref.checksum = metric_ref.payload_ref.checksum.saturating_add(1);
            }
            _ => unreachable!(),
        }

        let error = enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric_ref),
        )
        .expect_err("wrapper exact ref mismatch must fail before mutation");

        assert!(matches!(
            error,
            WorldSdfRuntimePayloadPackageError::MetricProductRatificationRejected { .. }
        ));
        assert!(
            app.world()
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap()
                .product(product_id)
                .is_none()
        );
        assert!(
            app.world()
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap()
                .capability(product_id)
                .is_none()
        );
        assert!(
            app.world()
                .resource::<WorldCompletedBuildQueueResource>()
                .unwrap()
                .outputs
                .is_empty()
        );
        assert!(
            !app.world()
                .resource::<WorldChunkRuntimeMapResource>()
                .unwrap()
                .by_chunk_id
                .contains_key(&chunk_id)
        );
    }
}

#[test]
fn metric_runtime_sdf_rejects_unsupported_encoding_and_invalid_topology_before_mutation() {
    for invalid_encoding in [true, false] {
        let mut app = fixed_world_app();
        let chunk_id = ChunkId::new(
            WorldId::new(0),
            ChunkCoord3 {
                x: if invalid_encoding { 32 } else { 33 },
                y: 0,
                z: 0,
            },
        );
        let payload = if invalid_encoding {
            metric_sdf_payload(chunk_id, ChunkRevision(1), ChunkGeneration(1), 82)
        } else {
            sdf_chunk_payload(chunk_id, ChunkRevision(1), ChunkGeneration(1), 83)
        };
        let descriptor =
            runtime_sdf_descriptor(if invalid_encoding { 7102 } else { 7103 }, &payload);
        let product_id = descriptor.product_core().identity;
        let metric_ref = if invalid_encoding {
            WorldSdfMetricPayloadRef {
                payload_ref: WorldSdfPayloadRef::from(&payload),
                encoding: WorldSdfMetricEncoding {
                    layout_revision: WORLD_SDF_METRIC_LAYOUT_REVISION + 1,
                    distance_units_per_meter: 1024,
                    max_absolute_error_units: 1,
                },
            }
        } else {
            metric_payload_ref(&payload, 1)
        };

        let error = enqueue_runtime_sdf_package(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            )
            .with_metric_capability(metric_ref),
        )
        .expect_err("invalid metric contract must fail before mutation");

        if invalid_encoding {
            assert!(matches!(
                error,
                WorldSdfRuntimePayloadPackageError::MetricProductRatificationRejected { .. }
            ));
        } else {
            assert!(matches!(
                error,
                WorldSdfRuntimePayloadPackageError::MetricPayloadRejected {
                    error: WorldSdfMetricError::InvalidMetricPageCount { actual: 0 }
                }
            ));
        }
        assert!(
            app.world()
                .resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap()
                .product(product_id)
                .is_none()
        );
        assert!(
            app.world()
                .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
                .unwrap()
                .capability(product_id)
                .is_none()
        );
        assert!(
            app.world()
                .resource::<WorldCompletedBuildQueueResource>()
                .unwrap()
                .outputs
                .is_empty()
        );
        assert!(
            !app.world()
                .resource::<WorldChunkRuntimeMapResource>()
                .unwrap()
                .by_chunk_id
                .contains_key(&chunk_id)
        );
    }
}

#[test]
fn non_metric_replacement_clears_only_that_products_metric_capability() {
    let mut app = fixed_world_app();

    let chunk_a = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 34, y: 0, z: 0 });
    let payload_a = metric_sdf_payload(chunk_a, ChunkRevision(1), ChunkGeneration(1), 84);
    let descriptor_a = runtime_sdf_descriptor(7104, &payload_a);
    let product_a = descriptor_a.product_core().identity;
    let metric_a = metric_payload_ref(&payload_a, 1);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor_a,
            vec![payload_a],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_a),
    )
    .unwrap();

    let chunk_b = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 35, y: 0, z: 0 });
    let payload_b = metric_sdf_payload(chunk_b, ChunkRevision(1), ChunkGeneration(1), 85);
    let descriptor_b = runtime_sdf_descriptor(7105, &payload_b);
    let product_b = descriptor_b.product_core().identity;
    let metric_b = metric_payload_ref(&payload_b, 1);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor_b,
            vec![payload_b],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_b),
    )
    .unwrap();

    let replacement_a = sdf_chunk_payload(chunk_a, ChunkRevision(2), ChunkGeneration(2), 86);
    let mut replacement_descriptor_a = runtime_sdf_descriptor(7104, &replacement_a);
    replacement_descriptor_a.lineage.source_revision = 2;
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            replacement_descriptor_a.clone(),
            vec![replacement_a],
            RegionSdfSummary::default(),
        ),
    )
    .expect("valid non-metric replacement should remain ordinary GP1B0 intake");

    let products = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .unwrap();
    assert_eq!(products.product(product_a), Some(&replacement_descriptor_a));
    let capabilities = app
        .world()
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .unwrap();
    assert!(capabilities.capability(product_a).is_none());
    assert_eq!(capabilities.capability(product_b), Some(&metric_b));
}

#[test]
fn valid_metric_replacement_updates_only_the_target_product() {
    let mut app = fixed_world_app();

    let chunk_a = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 36, y: 0, z: 0 });
    let initial_a = metric_sdf_payload(chunk_a, ChunkRevision(1), ChunkGeneration(1), 87);
    let descriptor_a = runtime_sdf_descriptor(7106, &initial_a);
    let product_a = descriptor_a.product_core().identity;
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor_a,
            vec![initial_a.clone()],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_payload_ref(&initial_a, 1)),
    )
    .unwrap();

    let chunk_b = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 37, y: 0, z: 0 });
    let payload_b = metric_sdf_payload(chunk_b, ChunkRevision(1), ChunkGeneration(1), 88);
    let descriptor_b = runtime_sdf_descriptor(7107, &payload_b);
    let product_b = descriptor_b.product_core().identity;
    let metric_b = metric_payload_ref(&payload_b, 3);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            descriptor_b,
            vec![payload_b],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(metric_b),
    )
    .unwrap();

    let replacement_a = metric_sdf_payload(chunk_a, ChunkRevision(2), ChunkGeneration(2), 89);
    let mut replacement_descriptor_a = runtime_sdf_descriptor(7106, &replacement_a);
    replacement_descriptor_a.lineage.source_revision = 2;
    let replacement_metric_a = metric_payload_ref(&replacement_a, 2);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            replacement_descriptor_a.clone(),
            vec![replacement_a],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(replacement_metric_a),
    )
    .expect("valid metric replacement should atomically update target product");

    let products = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .unwrap();
    assert_eq!(products.product(product_a), Some(&replacement_descriptor_a));
    let capabilities = app
        .world()
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .unwrap();
    assert_eq!(
        capabilities.capability(product_a),
        Some(&replacement_metric_a)
    );
    assert_eq!(capabilities.capability(product_b), Some(&metric_b));
}

#[test]
fn rejected_metric_replacement_preserves_prior_ordinary_and_metric_state() {
    let mut app = fixed_world_app();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 38, y: 0, z: 0 });
    let initial = metric_sdf_payload(chunk_id, ChunkRevision(1), ChunkGeneration(1), 90);
    let initial_descriptor = runtime_sdf_descriptor(7108, &initial);
    let product_id = initial_descriptor.product_core().identity;
    let initial_metric = metric_payload_ref(&initial, 1);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            initial_descriptor.clone(),
            vec![initial],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(initial_metric),
    )
    .unwrap();

    let queue_len_before = app
        .world()
        .resource::<WorldCompletedBuildQueueResource>()
        .unwrap()
        .outputs
        .len();
    let pending_before = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&chunk_id)
        .and_then(|record| record.pending_build_generation);

    let invalid_replacement = sdf_chunk_payload(chunk_id, ChunkRevision(2), ChunkGeneration(2), 91);
    let mut invalid_descriptor = runtime_sdf_descriptor(7108, &invalid_replacement);
    invalid_descriptor.lineage.source_revision = 2;
    let invalid_metric = metric_payload_ref(&invalid_replacement, 1);
    let error = enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            invalid_descriptor,
            vec![invalid_replacement],
            RegionSdfSummary::default(),
        )
        .with_metric_capability(invalid_metric),
    )
    .expect_err("invalid topology must not partially replace accepted state");
    assert!(matches!(
        error,
        WorldSdfRuntimePayloadPackageError::MetricPayloadRejected { .. }
    ));

    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(product_id),
        Some(&initial_descriptor)
    );
    assert_eq!(
        app.world()
            .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
            .unwrap()
            .capability(product_id),
        Some(&initial_metric)
    );
    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        queue_len_before
    );
    assert_eq!(
        app.world()
            .resource::<WorldChunkRuntimeMapResource>()
            .unwrap()
            .by_chunk_id
            .get(&chunk_id)
            .and_then(|record| record.pending_build_generation),
        pending_before
    );
}

#[test]
fn failed_external_replacement_preserves_cpu_clears_pending_and_accepts_retry() {
    use engine::plugins::world::chunks::lifecycle::ChunkLifecycleState;
    let mut app = fixed_world_app();
    let id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 55, y: 0, z: 0 });
    let first = sdf_chunk_payload(id, ChunkRevision(1), ChunkGeneration(1), 701);
    let first_desc = runtime_sdf_descriptor(7290, &first);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            first_desc,
            vec![first.clone()],
            RegionSdfSummary::default(),
        ),
    )
    .expect("first generation enqueues");
    app = app.run_for_fixed_steps(1).expect("first commits");

    let next = sdf_chunk_payload(id, ChunkRevision(2), ChunkGeneration(2), 702);
    let next_desc = runtime_sdf_descriptor(7290, &next);
    let package = WorldSdfRuntimePayloadPackage::new(
        next_desc.clone(),
        vec![next.clone()],
        RegionSdfSummary::default(),
    );
    assert_eq!(
        enqueue_runtime_sdf_package(&mut app, package.clone()).unwrap(),
        1
    );
    let queued = app
        .world_mut()
        .resource_mut::<WorldCompletedBuildQueueResource>()
        .unwrap();
    let output = queued.outputs.front_mut().unwrap();
    // Simulate a terminally malformed completed output after enqueue metadata was published.
    output.chunk_payload.chunk_revision = ChunkRevision(300);
    app = app
        .run_for_fixed_steps(1)
        .expect("malformed completion drops");

    let w = app.world();
    assert_eq!(
        w.resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&id),
        Some(&first)
    );
    let record = w
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&id)
        .unwrap();
    assert_eq!(record.pending_build_generation, None);
    assert_eq!(record.chunk_generation, ChunkGeneration(1));
    assert!(matches!(record.lifecycle, ChunkLifecycleState::Ready));
    // Enqueue-time catalog points to N+1, yet the committed CPU is still N.
    assert_eq!(
        w.resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap()
            .product(next_desc.product_core().identity),
        Some(&next_desc)
    );

    assert_eq!(enqueue_runtime_sdf_package(&mut app, package).unwrap(), 1);
    app = app.run_for_fixed_steps(1).expect("valid retry commits");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&id),
        Some(&next)
    );
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&id)
        .unwrap();
    assert!(record.pending_build_generation.is_none());
    assert_eq!(record.chunk_generation, ChunkGeneration(2));
}

#[test]
fn failed_generic_completion_recovers_dirty_retry_authority() {
    use engine::plugins::world::chunks::lifecycle::ChunkLifecycleState;
    let mut app = fixed_world_app();
    let id = ChunkId::new(WorldId::new(0), ChunkCoord3 { x: 56, y: 0, z: 0 });
    let first = sdf_chunk_payload(id, ChunkRevision(1), ChunkGeneration(1), 800);
    enqueue_runtime_sdf_package(
        &mut app,
        WorldSdfRuntimePayloadPackage::new(
            runtime_sdf_descriptor(7291, &first),
            vec![first.clone()],
            RegionSdfSummary::default(),
        ),
    )
    .unwrap();
    app = app.run_for_fixed_steps(1).unwrap();

    // A generic job has already claimed generation N+1; its completed bytes
    // fail the revision contract. The same World lifecycle must release N+1.
    {
        let record = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap();
        let r = record.by_chunk_id.get_mut(&id).unwrap();
        r.pending_build_generation = Some(BuildGeneration(2));
        r.lifecycle = ChunkLifecycleState::Rebuilding;
    }
    app.world_mut()
        .resource_mut::<WorldCompletedBuildQueueResource>()
        .unwrap()
        .outputs
        .push_back(WorldCompletedBuildOutput {
            chunk_id: id,
            target_chunk_revision: ChunkRevision(2),
            target_build_generation: BuildGeneration(2),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: sdf_chunk_payload(id, ChunkRevision(999), ChunkGeneration(2), 801),
            region_summary: RegionSdfSummary::default(),
            exact_source: None,
            origin: engine::plugins::world::build::WorldCompletedBuildOrigin::OperationWindow,
        });
    app = app
        .run_for_fixed_steps(1)
        .expect("failed operation job releases intent");
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&id),
        Some(&first)
    );
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&id)
        .unwrap();
    assert!(record.pending_build_generation.is_none());
    assert!(matches!(record.lifecycle, ChunkLifecycleState::Dirty));
    assert!(!record.dirty_reasons.is_empty());

    // The built-in dirty dispatch, not a second lifecycle manager, retries N+1.
    app = app
        .run_for_fixed_steps(1)
        .expect("dirty job dispatched and integrated");
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&id)
        .unwrap();
    assert!(record.pending_build_generation.is_none());
    assert_eq!(record.chunk_generation, ChunkGeneration(2));
}
