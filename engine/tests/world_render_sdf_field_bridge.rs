use engine::plugins::render::features::world::{
    RenderSdfResidencyResource, RenderSdfResidencyStatus,
};
use engine::plugins::render::field_input::RenderFieldSemanticInput;
use engine::plugins::render::frame::PreparedRenderProductSelectionResource;
use engine::plugins::render::{RenderFrameProducerId, RenderPlugin};
use engine::plugins::world::WorldPlugin;
use engine::plugins::world::adapters::{PartitionConfigResource, SdfChunkStoreResource};
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfMetricCapabilityCatalogResource,
    WorldRuntimeSdfProductCatalogResource, WorldSdfRuntimePayloadPackage,
    enqueue_ratified_world_sdf_payload_package,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::plugins::world::prepare::{
    PreparedWorldSdfFieldSource, PreparedWorldSdfFieldSourceResource,
};
use engine::plugins::{FixedStepPlugin, SimulationPlugin, TimePlugin};
use engine::prelude::*;
use product::{
    ProductIdentity, ProductResidency, RenderProductSelection, RenderResidencyRequest,
    RenderSelectedProduct,
};
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision};
use world_sdf::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, RegionSdfSummary, SDF_METRIC_BRICK_SAMPLE_COUNT,
    SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfChunkPayload,
    SdfPageCoord3, SdfPageRecord, WorldSdfMetricEncoding, WorldSdfMetricPayloadRef,
    WorldSdfPayloadRef,
};

const PRODUCER_RAW: u64 = 9101;

fn metric_payload(chunk_id: ChunkId, checksum: u64) -> SdfChunkPayload {
    let mut page = SdfPageRecord {
        page_generation: 5,
        bricks: Default::default(),
    };
    for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                page.bricks.insert(
                    [x, y, z],
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
    SdfChunkPayload {
        chunk_id,
        chunk_revision: ChunkRevision(3),
        chunk_generation: ChunkGeneration(5),
        page_table: [(SdfPageCoord3::default(), page)].into_iter().collect(),
        hierarchy_revision: 0,
        checksum,
    }
}

fn descriptor(product_id: u64, payload: &SdfChunkPayload) -> FieldProductDescriptor {
    let mut descriptor = FieldProductDescriptor::new(
        FieldProductId(product_id),
        FieldProductKind::WorldSdfChunkPages,
        FieldProductScope::from_chunks([payload.chunk_id]),
        FieldProductLineage::new(5, "tests.world_render_sdf_field_bridge"),
    );
    descriptor.consumer_class = FieldProductConsumerClass::RuntimeRead;
    descriptor
        .payload_refs
        .push(WorldSdfPayloadRef::from(payload));
    descriptor
}

fn metric_ref(payload: &SdfChunkPayload) -> WorldSdfMetricPayloadRef {
    WorldSdfMetricPayloadRef::try_new(
        WorldSdfPayloadRef::from(payload),
        WorldSdfMetricEncoding::try_new(100, 2).expect("metric encoding"),
    )
    .expect("metric payload ref")
}

fn app_with_render() -> App {
    let mut app = App::headless();
    app.add_plugins((
        TimePlugin,
        FixedStepPlugin,
        SimulationPlugin,
        WorldPlugin,
        RenderPlugin,
    ));
    *app.world_mut()
        .resource_mut::<PartitionConfigResource>()
        .expect("World should own partition") =
        PartitionConfigResource(
            GridPartitionConfig::try_new(4.0, [8, 8, 8]).expect("test partition"),
        );
    app
}

fn enqueue(
    app: &mut App,
    descriptor: FieldProductDescriptor,
    payload: SdfChunkPayload,
    metric: bool,
) {
    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed queue");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("product catalog");
    let mut metric_capabilities = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("metric catalog");
    let package = WorldSdfRuntimePayloadPackage::new(
        descriptor,
        vec![payload.clone()],
        RegionSdfSummary::default(),
    );
    let package = if metric {
        package.with_metric_capability(metric_ref(&payload))
    } else {
        package
    };
    {
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &mut products,
            &mut metric_capabilities,
            package,
        )
        .expect("test package should be valid");
    }
    app.world_mut().insert_resource(completed);
    app.world_mut().insert_resource(products);
    app.world_mut().insert_resource(metric_capabilities);
}

fn selection(descriptor: &FieldProductDescriptor) -> RenderProductSelection {
    let core = descriptor.product_core();
    RenderProductSelection::new("world-sdf-field-test")
        .with_selected_product(RenderSelectedProduct {
            product_id: core.identity,
            scale_band: core.scale_band,
            generation: core.lineage.generation,
            freshness: core.freshness,
            residency: core.residency,
            authority_class: core.authority_class,
            query_policy: core.query_policy,
        })
        .with_residency_request(RenderResidencyRequest::new(
            core.identity,
            ProductResidency::Resident,
            100,
            true,
        ))
}

fn publish_selection(app: &mut App, selection: RenderProductSelection) {
    app.world_mut()
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .expect("Render should own product selections")
        .replace_contribution(
            RenderFrameProducerId::try_from_raw(PRODUCER_RAW).expect("producer id"),
            [selection],
        )
        .expect("selection should ratify");
}

fn selected_metric_app(product_raw: u64) -> (App, FieldProductDescriptor, SdfChunkPayload) {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = metric_payload(chunk_id, 77);
    let descriptor = descriptor(product_raw, &payload);
    enqueue(&mut app, descriptor.clone(), payload.clone(), true);
    app = app.run_for_fixed_steps(1).expect("payload should integrate");
    publish_selection(&mut app, selection(&descriptor));
    (app, descriptor, payload)
}

#[test]
fn headless_world_owns_empty_projection_without_render_dependency() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin, WorldPlugin));
    let app = app
        .run_for_frames(1)
        .expect("World-only headless frame should remain valid");

    let projected = app
        .world()
        .resource::<PreparedWorldSdfFieldSourceResource>()
        .expect("World should own projection cache without Render");
    assert!(projected.sources().is_empty());
    assert!(
        app.world()
            .resource::<RenderSdfResidencyResource>()
            .is_err()
    );
}

#[test]
fn metric_product_requires_explicit_selection_and_residency_before_projection() {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = metric_payload(chunk_id, 77);
    let descriptor = descriptor(601, &payload);
    enqueue(&mut app, descriptor.clone(), payload, true);

    let app = app
        .run_for_fixed_steps(1)
        .expect("payload should integrate")
        .run_for_frames(1)
        .expect("RenderPrepare should execute");

    let product_id = descriptor.product_core().identity;
    assert!(
        app.world()
            .resource::<PreparedWorldSdfFieldSourceResource>()
            .unwrap()
            .source(product_id)
            .is_none()
    );
}

#[test]
fn selected_resident_metric_product_projects_to_regular_render_field() {
    let (mut app, descriptor, payload) = selected_metric_app(602);
    app = app.run_for_frames(1).expect("projection frame");
    let product_id = descriptor.product_core().identity;

    let source = app
        .world()
        .resource::<PreparedWorldSdfFieldSourceResource>()
        .unwrap()
        .source(product_id)
        .expect("selected resident metric product should project");
    assert_eq!(source.product_generation(), 5);
    assert_eq!(source.payload_ref(), WorldSdfPayloadRef::from(&payload));
    assert_eq!(source.input().origin_local_meters(), [0.0; 3]);
    assert_eq!(source.input().sample_spacing_meters(), [0.5; 3]);
    assert_eq!(source.input().dimensions(), [9, 9, 9]);
    assert_eq!(source.input().sample_count(), 9 * 9 * 9);
    assert_eq!(source.input().max_absolute_query_error_local_meters(), 0.02);
    assert!(
        (0..source.input().sample_count())
            .all(|index| source.input().signed_distance_sample_meters(index) == Some(0.0))
    );
}

#[test]
fn preserved_residency_remains_eligible_for_same_projection() {
    let (mut app, descriptor, _) = selected_metric_app(603);
    app = app.run_for_frames(1).expect("first projection frame");
    app = app.run_for_frames(1).expect("preserved projection frame");
    let product_id = descriptor.product_core().identity;

    let residency = app.world().resource::<RenderSdfResidencyResource>().unwrap();
    assert_eq!(
        residency.entry(product_id).unwrap().status,
        RenderSdfResidencyStatus::Preserved
    );
    assert!(
        app.world()
            .resource::<PreparedWorldSdfFieldSourceResource>()
            .unwrap()
            .source(product_id)
            .is_some()
    );
}

#[test]
fn non_metric_resident_product_fails_closed() {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = metric_payload(chunk_id, 77);
    let descriptor = descriptor(604, &payload);
    let product_id = descriptor.product_core().identity;
    enqueue(&mut app, descriptor.clone(), payload, false);
    app = app.run_for_fixed_steps(1).expect("payload should integrate");
    publish_selection(&mut app, selection(&descriptor));
    app = app.run_for_frames(1).expect("projection frame");

    assert!(
        app.world()
            .resource::<RenderSdfResidencyResource>()
            .unwrap()
            .entry(product_id)
            .is_some(),
        "ordinary residency should remain available without metric capability"
    );
    assert!(
        app.world()
            .resource::<PreparedWorldSdfFieldSourceResource>()
            .unwrap()
            .source(product_id)
            .is_none()
    );
}

#[test]
fn selection_removal_removes_only_world_owned_projection() {
    let (mut app, descriptor, payload) = selected_metric_app(605);
    app = app.run_for_frames(1).expect("first projection frame");
    let product_id = descriptor.product_core().identity;
    let unrelated_id = ProductIdentity::new(9999);
    let unrelated_input = RenderFieldSemanticInput::dense(
        [0.0; 3],
        [1.0; 3],
        [2, 2, 2],
        vec![1.0; 8],
        0.0,
        engine::plugins::render::space_time::RenderTemporalSupport::unbounded(),
    )
    .expect("unrelated field input");
    app.world_mut()
        .resource_mut::<PreparedWorldSdfFieldSourceResource>()
        .unwrap()
        .insert_source(PreparedWorldSdfFieldSource::new(
            unrelated_id,
            1,
            WorldSdfPayloadRef::from(&payload),
            unrelated_input,
        ));

    app.world_mut()
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .unwrap()
        .remove_contribution(
            RenderFrameProducerId::try_from_raw(PRODUCER_RAW).expect("producer id"),
        );
    app = app.run_for_frames(1).expect("selection removal frame");

    let projected = app
        .world()
        .resource::<PreparedWorldSdfFieldSourceResource>()
        .unwrap();
    assert!(projected.source(product_id).is_none());
    assert!(
        projected.source(unrelated_id).is_some(),
        "World adapter must not globally clear unrelated prepared field sources"
    );
}

#[test]
fn descriptor_store_mismatch_removes_world_projection_without_unrelated_eviction() {
    let (mut app, descriptor, payload) = selected_metric_app(606);
    app = app.run_for_frames(1).expect("first projection frame");
    let product_id = descriptor.product_core().identity;
    let unrelated_id = ProductIdentity::new(9998);
    let unrelated_input = RenderFieldSemanticInput::dense(
        [0.0; 3],
        [1.0; 3],
        [2, 2, 2],
        vec![1.0; 8],
        0.0,
        engine::plugins::render::space_time::RenderTemporalSupport::unbounded(),
    )
    .expect("unrelated field input");
    app.world_mut()
        .resource_mut::<PreparedWorldSdfFieldSourceResource>()
        .unwrap()
        .insert_source(PreparedWorldSdfFieldSource::new(
            unrelated_id,
            1,
            WorldSdfPayloadRef::from(&payload),
            unrelated_input,
        ));

    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .get_mut(&payload.chunk_id)
        .unwrap()
        .checksum += 1;
    app = app.run_for_frames(1).expect("mismatch frame");

    let projected = app
        .world()
        .resource::<PreparedWorldSdfFieldSourceResource>()
        .unwrap();
    assert!(projected.source(product_id).is_none());
    assert!(projected.source(unrelated_id).is_some());
}
