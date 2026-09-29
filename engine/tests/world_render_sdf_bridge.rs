use engine::plugins::render::features::world::{
    RenderSdfRaymarchAccelerationResource, RenderSdfResidencyResource,
    RenderSdfResidencySourceResource,
};
use engine::plugins::render::frame::PreparedRenderProductSelectionResource;
use engine::plugins::render::{RenderFrameProducerId, RenderPlugin};
use engine::plugins::world::WorldPlugin;
use engine::plugins::world::adapters::SdfChunkStoreResource;
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfProductCatalogResource,
    WorldSdfRuntimePayloadPackage, enqueue_ratified_world_sdf_payload_package,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::plugins::{FixedStepPlugin, SimulationPlugin, TimePlugin};
use engine::prelude::*;
use product::{
    ProductIdentity, ProductResidency, RenderProductSelection, RenderResidencyRequest,
    RenderSelectedProduct,
};
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision};
use world_sdf::{
    FieldProductConsumerClass, FieldProductDescriptor, FieldProductId, FieldProductKind,
    FieldProductLineage, FieldProductScope, RegionSdfSummary, SdfBrickMetadata, SdfBrickRecord,
    SdfChunkPayload, SdfPageCoord3, SdfPageRecord, WorldSdfPayloadRef,
};

fn payload(chunk_id: ChunkId) -> SdfChunkPayload {
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };
    page.bricks.insert(
        [0, 0, 0],
        SdfBrickRecord {
            metadata: SdfBrickMetadata {
                occupancy_mask: u8::MAX,
                ..SdfBrickMetadata::default()
            },
            samples: Default::default(),
        },
    );
    SdfChunkPayload {
        chunk_id,
        chunk_revision: ChunkRevision(3),
        chunk_generation: ChunkGeneration(5),
        page_table: [(SdfPageCoord3 { x: 0, y: 0, z: 0 }, page)]
            .into_iter()
            .collect(),
        hierarchy_revision: 0,
        checksum: 77,
    }
}

fn descriptor(product_id: u64, payload: &SdfChunkPayload) -> FieldProductDescriptor {
    let mut descriptor = FieldProductDescriptor::new(
        FieldProductId(product_id),
        FieldProductKind::WorldSdfChunkPages,
        FieldProductScope::from_chunks([payload.chunk_id]),
        FieldProductLineage::new(5, "tests.world_render_sdf_bridge"),
    );
    descriptor.consumer_class = FieldProductConsumerClass::RuntimeRead;
    descriptor
        .payload_refs
        .push(WorldSdfPayloadRef::from(payload));
    descriptor
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
    app
}

fn enqueue(app: &mut App, descriptor: FieldProductDescriptor, payload: SdfChunkPayload) {
    let mut completed = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .expect("completed queue should exist");
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("product catalog should exist");
    {
        let chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        enqueue_ratified_world_sdf_payload_package(
            &mut completed,
            chunks,
            &mut products,
            WorldSdfRuntimePayloadPackage::new(
                descriptor,
                vec![payload],
                RegionSdfSummary::default(),
            ),
        )
        .expect("bridge test package should be valid");
    }
    app.world_mut().insert_resource(completed);
    app.world_mut().insert_resource(products);
}

fn selection(descriptor: &FieldProductDescriptor) -> RenderProductSelection {
    let core = descriptor.product_core();
    RenderProductSelection::new("world-sdf-test")
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

#[test]
fn headless_world_without_render_remains_valid() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin, WorldPlugin));
    let app = app
        .run_for_frames(1)
        .expect("World-only headless frame should not require Render resources");
    assert!(
        app.world()
            .resource::<RenderSdfResidencySourceResource>()
            .is_err()
    );
}

#[test]
fn world_sdf_requires_explicit_selection_before_renderer_residency() {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = payload(chunk_id);
    let descriptor = descriptor(501, &payload);
    enqueue(&mut app, descriptor, payload);

    let app = app
        .run_for_fixed_steps(1)
        .expect("payload should integrate")
        .run_for_frames(1)
        .expect("RenderPrepare bridge should execute");

    let residency = app
        .world()
        .resource::<RenderSdfResidencyResource>()
        .expect("Render should own SDF residency");
    assert!(residency.entries().is_empty());
}

#[test]
fn matching_selection_bridges_integrated_world_payload_into_residency_and_acceleration() {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = payload(chunk_id);
    let descriptor = descriptor(502, &payload);
    let product_id = descriptor.product_core().identity;
    enqueue(&mut app, descriptor.clone(), payload.clone());

    app = app
        .run_for_fixed_steps(1)
        .expect("payload should integrate");
    app.world_mut()
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .expect("Render should own prepared product selections")
        .replace_contribution(
            RenderFrameProducerId::try_from_raw(9001).expect("producer id should be nonzero"),
            [selection(&descriptor)],
        )
        .expect("selection should ratify");

    app = app
        .run_for_frames(1)
        .expect("RenderPrepare bridge should execute");

    let sources = app
        .world()
        .resource::<RenderSdfResidencySourceResource>()
        .expect("Render should own SDF sources");
    let source = sources
        .product(product_id)
        .expect("World adapter should publish exact integrated payload");
    assert_eq!(source.payload.chunk_id, payload.chunk_id);
    assert_eq!(source.payload.chunk_revision, payload.chunk_revision);
    assert_eq!(source.payload.checksum, payload.checksum);

    let residency = app
        .world()
        .resource::<RenderSdfResidencyResource>()
        .expect("Render should own SDF residency");
    let entry = residency
        .entry(product_id)
        .expect("matching selection should become renderer resident");
    assert_eq!(entry.product_id, product_id);
    assert_eq!(
        entry.product_generation,
        descriptor.product_core().lineage.generation
    );
    assert_eq!(entry.chunk_id, payload.chunk_id);
    assert_eq!(entry.chunk_revision, payload.chunk_revision.0);
    assert_eq!(entry.chunk_generation, payload.chunk_generation.0);

    let acceleration = app
        .world()
        .resource::<RenderSdfRaymarchAccelerationResource>()
        .expect("Render should own SDF raymarch acceleration");
    assert!(acceleration.last_report().is_acceleration_ready());
    assert_eq!(acceleration.last_report().resident_product_count, 1);
    assert!(acceleration.last_report().resident_page_count > 0);
    assert!(acceleration.last_report().resident_brick_count > 0);
}

#[test]
fn descriptor_store_mismatch_removes_world_owned_source_without_touching_unrelated_source() {
    let mut app = app_with_render();
    let chunk_id = ChunkId::new(WorldId::new(0), ChunkCoord3::default());
    let payload = payload(chunk_id);
    let descriptor = descriptor(503, &payload);
    let product_id = descriptor.product_core().identity;
    enqueue(&mut app, descriptor.clone(), payload.clone());

    app = app
        .run_for_fixed_steps(1)
        .expect("payload should integrate");
    let unrelated_id = ProductIdentity::new(9999);
    app.world_mut()
        .resource_mut::<RenderSdfResidencySourceResource>()
        .expect("Render should own source registry")
        .upsert_payload(unrelated_id, 1, payload.clone());
    let mut mixed_selection = selection(&descriptor);
    let mut unrelated_selected = mixed_selection
        .selected_products
        .first()
        .expect("world selection should contain one selected product")
        .clone();
    unrelated_selected.product_id = unrelated_id;
    unrelated_selected.generation = 1;
    mixed_selection.selected_products.push(unrelated_selected);
    mixed_selection
        .residency_requests
        .push(RenderResidencyRequest::new(
            unrelated_id,
            ProductResidency::Resident,
            90,
            false,
        ));
    app.world_mut()
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .expect("Render should own prepared product selections")
        .replace_contribution(
            RenderFrameProducerId::try_from_raw(9002).expect("producer id should be nonzero"),
            [mixed_selection],
        )
        .expect("selection should ratify");

    app = app
        .run_for_frames(1)
        .expect("initial bridge should execute");
    assert!(
        app.world()
            .resource::<RenderSdfResidencySourceResource>()
            .unwrap()
            .product(product_id)
            .is_some()
    );
    assert!(
        app.world()
            .resource::<RenderSdfResidencyResource>()
            .unwrap()
            .entry(unrelated_id)
            .is_some(),
        "World bridge derivation must preserve residency selected for an unrelated SDF source"
    );

    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .expect("World should own SDF store")
        .chunks
        .get_mut(&chunk_id)
        .expect("integrated payload should exist")
        .checksum = payload.checksum + 1;

    app = app
        .run_for_frames(1)
        .expect("mismatch bridge should fail closed");

    let sources = app
        .world()
        .resource::<RenderSdfResidencySourceResource>()
        .unwrap();
    assert!(sources.product(product_id).is_none());
    assert!(sources.product(unrelated_id).is_some());
    let residency = app
        .world()
        .resource::<RenderSdfResidencyResource>()
        .unwrap();
    assert!(residency.entry(product_id).is_none());
    assert!(
        residency.entry(unrelated_id).is_some(),
        "invalidating a World-owned SDF source must not evict unrelated selected SDF residency"
    );
}
