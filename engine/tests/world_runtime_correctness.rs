use engine::plugins::world::adapters::resources::{
    OperationLogResource, SdfChunkStoreResource, WorldQuantizationScaleResource,
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
    build::jobs::WorldBuildStaleness,
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
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
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
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
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
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
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
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
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
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
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
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
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
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
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
            let chunks = app
                .world_mut()
                .resource_mut::<WorldChunkRuntimeMapResource>()
                .expect("chunk runtime should exist");
            enqueue_ratified_world_sdf_payload_package(
                &mut completed,
                chunks,
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
        Some(BuildGeneration(11)),
        "rejected integration must keep pending generation unchanged"
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
                metric_ref.payload_ref.checksum =
                    metric_ref.payload_ref.checksum.saturating_add(1);
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
