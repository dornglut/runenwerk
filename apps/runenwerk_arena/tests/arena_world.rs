use engine::plugins::render::frame::PreparedRenderProductSelectionResource;
use engine::plugins::render::{RenderFrameProducerId, RenderPlugin};
use engine::plugins::world::adapters::{
    CollisionQueryServiceResource, PartitionConfigResource, SdfChunkStoreResource,
};
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfMetricCapabilityCatalogResource,
    WorldRuntimeSdfProductCatalogResource,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::plugins::world::prepare::PreparedWorldSdfFieldSourceResource;
use engine::prelude::*;
use product::{
    ProductResidency, RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct,
};
use runen_spatial::WorldId;
use runenwerk_arena::{
    ARENA_CHUNK_EDGE_METERS, ARENA_FIELD_PRODUCT_ID, ARENA_METRIC_DISTANCE_UNITS_PER_METER,
    ARENA_METRIC_MAX_ABSOLUTE_ERROR_UNITS, ARENA_PLAYER_SPAWN, LOCAL_PARTICIPANT_ID,
    ParticipantCommand, PlayerCommand, TickCommandBatch, apply_game_commands, arena_chunk_id,
    arena_field_product_descriptor, arena_metric_encoding, arena_metric_error_components_meters,
    arena_metric_payload_ref, arena_partition_config, arena_signed_distance_meters,
    build_arena_chunk_payload, build_headless_game_app, player_physical_history_for,
};
use world_sdf::{
    SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_METRIC_BRICK_SAMPLE_COUNT, SDF_PAGE_EDGE_BRICKS,
    WorldSdfPayloadRef, sample_world_sdf_metric_distance, validate_world_sdf_metric_payload,
};

fn started_app() -> App {
    build_headless_game_app()
        .run_for_frames(0)
        .expect("maintained arena startup should succeed")
}

fn integrated_app() -> App {
    build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("first fixed tick should integrate the maintained arena")
}

fn signed_distance(app: &App, position: [f32; 3]) -> f32 {
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("WorldPlugin should install partition configuration");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("WorldPlugin should install the SDF store");
    let collision = app
        .world()
        .resource::<CollisionQueryServiceResource>()
        .expect("WorldPlugin should install collision queries");
    collision
        .sample_signed_distance(partition, store, WorldId::new(0), position)
        .expect("arena sample position should be valid")
        .expect("arena chunk payload should exist")
        .distance
}

fn local_batch(tick: u64, move_x: i8) -> TickCommandBatch {
    TickCommandBatch {
        tick: SimulationTick(tick),
        commands: vec![ParticipantCommand {
            participant: LOCAL_PARTICIPANT_ID,
            command: PlayerCommand {
                move_x,
                ..PlayerCommand::default()
            },
        }],
    }
}

#[test]
fn maintained_startup_enqueues_arena_through_world_runtime_intake() {
    let app = started_app();
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("arena partition should exist");
    assert_eq!(partition.chunk_edge_meters(), 4.0);

    let completed = app
        .world()
        .resource::<WorldCompletedBuildQueueResource>()
        .expect("world runtime intake queue should exist");
    assert_eq!(completed.outputs.len(), 1);

    let chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("world chunk runtime map should exist");
    let record = chunks
        .by_chunk_id
        .get(&arena_chunk_id())
        .expect("arena chunk should be registered with world runtime");
    assert!(record.pending_build_generation.is_some());

    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("arena SDF store should exist");
    assert!(
        store.chunks.is_empty(),
        "startup must not bypass World BuildIntegrate by writing the SDF store directly"
    );

    let products = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let descriptor = products
        .products()
        .values()
        .find(|descriptor| descriptor.product_id == ARENA_FIELD_PRODUCT_ID)
        .expect("maintained arena should retain its field product identity");
    assert_eq!(descriptor.payload_refs.len(), 1);
    assert_eq!(descriptor.payload_refs[0].chunk_id, arena_chunk_id());

    let product_id = descriptor.product_core().identity;
    let ordinary_ref = descriptor.payload_refs[0];
    let metric_capabilities = app
        .world()
        .resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .expect("maintained arena should retain metric capability in headless World");
    let metric_ref = metric_capabilities
        .capability(product_id)
        .expect("maintained arena should publish its metric wrapper through World intake");
    assert_eq!(metric_ref.payload_ref, ordinary_ref);
    assert_eq!(metric_ref.encoding, arena_metric_encoding());
}

#[test]
fn first_fixed_tick_integrates_arena_through_world_authority() {
    let app = integrated_app();
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("arena SDF store should exist after integration");
    assert_eq!(store.chunks.len(), 1);
    assert!(store.chunks.contains_key(&arena_chunk_id()));

    let completed = app
        .world()
        .resource::<WorldCompletedBuildQueueResource>()
        .expect("world runtime intake queue should exist");
    assert!(completed.outputs.is_empty());
}

#[test]
fn maintained_arena_collision_truth_contains_floor_clear_interior_and_four_walls() {
    let app = integrated_app();

    assert!(signed_distance(&app, [1.0, 0.25, 1.0]) < 0.0);
    assert!(signed_distance(&app, [1.0, 0.75, 1.0]) > 0.0);

    for wall in [
        [0.25, 1.0, 1.0],
        [3.75, 1.0, 1.0],
        [1.0, 1.0, 0.25],
        [1.0, 1.0, 3.75],
    ] {
        assert!(signed_distance(&app, wall) < 0.0, "{wall:?}");
    }
}

#[test]
fn maintained_player_spawns_in_clear_runtime_arena_interior() {
    let app = integrated_app();
    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should exist");
    assert_eq!(history.current.position, ARENA_PLAYER_SPAWN);
    assert!(signed_distance(&app, history.current.position) > 0.0);
}

#[test]
fn ordinary_fixed_tick_grounds_player_against_installed_floor() {
    let app = build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("maintained game should advance against its own arena payload");
    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should retain physical history");

    assert!(history.current.grounded);
    assert!(history.current.support_normal.is_some());
    assert!(history.current.position[1] <= ARENA_PLAYER_SPAWN[1] + 0.001);
}

#[test]
fn movement_command_advances_player_through_shared_p1_path_without_test_payload_injection() {
    let mut app = integrated_app();
    let before = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();

    let batch = local_batch(2, 1);
    apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();

    let after = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert!(after.current.position[0] > before.current.position[0]);
}

#[test]
fn repeated_movement_stops_at_boundary_without_crossing_into_missing_payload_space() {
    let mut app = integrated_app();

    for tick in 2..=121_u64 {
        let batch = local_batch(tick, -1);
        apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();
    }

    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert!(history.current.position[0] >= 0.49);
    assert!(history.current.position[0] < ARENA_PLAYER_SPAWN[0]);
}

#[test]
fn analytic_arena_field_has_expected_faces_sign_and_open_top() {
    for interior in [[1.0, 0.75, 1.0], [2.0, 2.0, 2.0], [2.0, 3.75, 2.0]] {
        assert!(arena_signed_distance_meters(interior) > 0.0, "{interior:?}");
    }
    for solid in [
        [1.0, 0.25, 1.0],
        [0.25, 1.0, 1.0],
        [3.75, 1.0, 1.0],
        [1.0, 1.0, 0.25],
        [1.0, 1.0, 3.75],
    ] {
        assert!(arena_signed_distance_meters(solid) < 0.0, "{solid:?}");
    }
    for surface in [
        [0.5, 1.0, 1.0],
        [3.5, 1.0, 1.0],
        [1.0, 0.5, 1.0],
        [1.0, 1.0, 0.5],
        [1.0, 1.0, 3.5],
    ] {
        assert!(
            arena_signed_distance_meters(surface).abs() <= 1.0e-12,
            "{surface:?}"
        );
    }
}

#[test]
fn arena_metric_error_budget_is_derived_from_source_lipschitz_and_quantization_bounds() {
    let components = arena_metric_error_components_meters();
    assert_eq!(components[0], 0.0, "analytic source is the exact arena SDF");

    let brick_edge_meters = ARENA_CHUNK_EDGE_METERS / SDF_PAGE_EDGE_BRICKS as f64;
    let subcell_edge_meters = brick_edge_meters / (SDF_METRIC_BRICK_EDGE_SAMPLES - 1) as f64;
    let expected_trilinear = (3.0_f64).sqrt() * subcell_edge_meters / 2.0;
    let expected_quantization = 0.5 / f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER);
    assert!((components[1] - expected_trilinear).abs() <= 1.0e-15);
    assert!((components[2] - expected_quantization).abs() <= 1.0e-15);

    let required_units = ((components.iter().sum::<f64>())
        * f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER))
    .ceil() as u32;
    assert_eq!(required_units, ARENA_METRIC_MAX_ABSOLUTE_ERROR_UNITS);
    assert!(arena_metric_encoding().max_absolute_error_meters() >= components.iter().sum::<f64>());
}

#[test]
fn arena_payload_metric_samples_and_metadata_derive_from_one_analytic_source() {
    let payload = build_arena_chunk_payload();
    let page = payload
        .page_table
        .values()
        .next()
        .expect("maintained arena has one page");
    assert_eq!(
        page.bricks.len(),
        SDF_PAGE_EDGE_BRICKS * SDF_PAGE_EDGE_BRICKS * SDF_PAGE_EDGE_BRICKS
    );

    let brick_edge_meters = ARENA_CHUNK_EDGE_METERS / SDF_PAGE_EDGE_BRICKS as f64;
    let sample_step_meters = brick_edge_meters / (SDF_METRIC_BRICK_EDGE_SAMPLES - 1) as f64;
    let cell_edge_meters = ARENA_CHUNK_EDGE_METERS / (SDF_PAGE_EDGE_BRICKS * 2) as f64;

    for (brick_coord, brick) in &page.bricks {
        assert_eq!(brick.samples.distances.len(), SDF_METRIC_BRICK_SAMPLE_COUNT);
        let mut index = 0;
        for sample_z in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
            for sample_y in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                for sample_x in 0..SDF_METRIC_BRICK_EDGE_SAMPLES {
                    let point = [
                        f64::from(brick_coord[0]) * brick_edge_meters
                            + sample_x as f64 * sample_step_meters,
                        f64::from(brick_coord[1]) * brick_edge_meters
                            + sample_y as f64 * sample_step_meters,
                        f64::from(brick_coord[2]) * brick_edge_meters
                            + sample_z as f64 * sample_step_meters,
                    ];
                    let expected = (arena_signed_distance_meters(point)
                        * f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER))
                    .round() as i16;
                    assert_eq!(brick.samples.distances[index], expected, "{point:?}");
                    index += 1;
                }
            }
        }

        assert_eq!(
            brick.metadata.min_distance,
            *brick.samples.distances.iter().min().unwrap()
        );
        assert_eq!(
            brick.metadata.max_distance,
            *brick.samples.distances.iter().max().unwrap()
        );
        assert_eq!(
            brick.metadata.surface_band_present,
            brick.metadata.min_distance <= 0 && brick.metadata.max_distance >= 0
        );

        let mut expected_occupancy = 0_u8;
        for octant_z in 0..2_u8 {
            for octant_y in 0..2_u8 {
                for octant_x in 0..2_u8 {
                    let point = [
                        f64::from(brick_coord[0] * 2 + octant_x) * cell_edge_meters
                            + cell_edge_meters * 0.5,
                        f64::from(brick_coord[1] * 2 + octant_y) * cell_edge_meters
                            + cell_edge_meters * 0.5,
                        f64::from(brick_coord[2] * 2 + octant_z) * cell_edge_meters
                            + cell_edge_meters * 0.5,
                    ];
                    if arena_signed_distance_meters(point) < 0.0 {
                        expected_occupancy |= 1 << (octant_x | (octant_y << 1) | (octant_z << 2));
                    }
                }
            }
        }
        assert_eq!(brick.metadata.occupancy_mask, expected_occupancy);
        assert_eq!(
            brick.metadata.material_channel_mask,
            u16::from(expected_occupancy != 0)
        );
    }
}

#[test]
fn arena_metric_wrapper_preserves_ordinary_product_ref_and_lineage() {
    let payload = build_arena_chunk_payload();
    let descriptor = arena_field_product_descriptor(&payload);
    let ordinary_ref = WorldSdfPayloadRef::from(&payload);
    let metric_ref = arena_metric_payload_ref(&payload);

    assert_eq!(descriptor.product_id, ARENA_FIELD_PRODUCT_ID);
    assert_eq!(descriptor.lineage.source_revision, 1);
    assert_eq!(descriptor.lineage.producer, "runenwerk_arena.runtime_sdf");
    assert_eq!(descriptor.payload_refs, vec![ordinary_ref]);
    assert_eq!(
        ordinary_ref,
        WorldSdfPayloadRef {
            chunk_id: arena_chunk_id(),
            chunk_revision: Default::default(),
            checksum: 1,
        }
    );
    assert_eq!(metric_ref.payload_ref, ordinary_ref);
    assert_eq!(metric_ref.encoding, arena_metric_encoding());
    validate_world_sdf_metric_payload(&metric_ref, &payload)
        .expect("maintained arena metric payload should satisfy GP1B1");
}

#[test]
fn reconstructed_metric_distance_stays_inside_declared_error_on_dense_grid() {
    let payload = build_arena_chunk_payload();
    let metric_ref = arena_metric_payload_ref(&payload);
    let partition = arena_partition_config();
    let declared_error = metric_ref.encoding.max_absolute_error_meters();

    let mut maximum_observed_error = 0.0_f64;
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let point = [x as f64 * 0.25, y as f64 * 0.25, z as f64 * 0.25];
                let intended = arena_signed_distance_meters(point);
                let reconstructed =
                    sample_world_sdf_metric_distance(&metric_ref, &payload, &partition, point)
                        .expect("dense arena metric query should succeed")
                        .signed_distance_estimate_meters();
                let error = (reconstructed - intended).abs();
                maximum_observed_error = maximum_observed_error.max(error);
                assert!(
                    error <= declared_error + 1.0e-12,
                    "point {point:?}: reconstructed={reconstructed}, intended={intended}, error={error}, declared={declared_error}"
                );
            }
        }
    }

    assert!(maximum_observed_error > 0.0);
    for point in [
        [0.25, 0.25, 0.25],
        [0.25, 1.0, 0.25],
        [0.75, 0.75, 0.75],
        [2.0, 2.0, 2.0],
        [3.75, 0.25, 3.75],
    ] {
        let sample =
            sample_world_sdf_metric_distance(&metric_ref, &payload, &partition, point).unwrap();
        assert_eq!(
            sample.signed_distance_estimate_meters() > 0.0,
            arena_signed_distance_meters(point) > 0.0,
            "{point:?}"
        );
        assert_eq!(sample.max_absolute_error_meters(), declared_error);
    }
}

#[test]
fn collision_sample_sign_matches_analytic_source_on_dense_off_surface_grid() {
    let app = integrated_app();
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let point = [
                    (x as f32 + 0.5) * 0.25,
                    (y as f32 + 0.5) * 0.25,
                    (z as f32 + 0.5) * 0.25,
                ];
                let intended = arena_signed_distance_meters([
                    f64::from(point[0]),
                    f64::from(point[1]),
                    f64::from(point[2]),
                ]);
                assert!(intended.abs() > 1.0e-12, "{point:?}");
                assert_eq!(
                    signed_distance(&app, point) > 0.0,
                    intended > 0.0,
                    "{point:?}"
                );
            }
        }
    }
}

#[test]
fn metric_samples_preserve_accepted_g2a_cell_classification() {
    let app = integrated_app();
    let cells_per_axis = (SDF_PAGE_EDGE_BRICKS * 2) as u8;
    let cell_edge = ARENA_CHUNK_EDGE_METERS as f32 / f32::from(cells_per_axis);

    for z in 0..cells_per_axis {
        for y in 0..cells_per_axis {
            for x in 0..cells_per_axis {
                let position = [
                    (f32::from(x) + 0.5) * cell_edge,
                    (f32::from(y) + 0.5) * cell_edge,
                    (f32::from(z) + 0.5) * cell_edge,
                ];
                let legacy_solid = y == 0
                    || x == 0
                    || x == cells_per_axis - 1
                    || z == 0
                    || z == cells_per_axis - 1;
                assert_eq!(
                    signed_distance(&app, position) < 0.0,
                    legacy_solid,
                    "cell [{x}, {y}, {z}] at {position:?}"
                );
            }
        }
    }
}

#[test]
fn collision_classification_is_preserved_immediately_on_both_sides_of_all_boundaries() {
    let app = integrated_app();
    for (solid, clear) in [
        ([2.0, 0.499, 2.0], [2.0, 0.501, 2.0]),
        ([0.499, 1.0, 2.0], [0.501, 1.0, 2.0]),
        ([3.501, 1.0, 2.0], [3.499, 1.0, 2.0]),
        ([2.0, 1.0, 0.499], [2.0, 1.0, 0.501]),
        ([2.0, 1.0, 3.501], [2.0, 1.0, 3.499]),
        ([0.499, 0.499, 2.0], [0.501, 0.501, 2.0]),
        ([0.499, 1.0, 0.499], [0.501, 1.0, 0.501]),
    ] {
        assert!(signed_distance(&app, solid) < 0.0, "{solid:?}");
        assert!(signed_distance(&app, clear) > 0.0, "{clear:?}");
    }
}

fn sample_projected_arena_field(
    input: &runen_render::field_input::RenderFieldSemanticInput,
    point: [f64; 3],
) -> f64 {
    let origin = input.origin_local_meters();
    let spacing = input.sample_spacing_meters();
    let dimensions = input.dimensions();
    let axis = |axis: usize| {
        let coordinate = ((point[axis] - origin[axis]) / spacing[axis])
            .clamp(0.0, f64::from(dimensions[axis] - 1));
        let lower = (coordinate.floor() as u32).min(dimensions[axis] - 2);
        let upper = lower + 1;
        (
            lower as usize,
            upper as usize,
            coordinate - f64::from(lower),
        )
    };
    let (x0, x1, tx) = axis(0);
    let (y0, y1, ty) = axis(1);
    let (z0, z1, tz) = axis(2);
    let width = dimensions[0] as usize;
    let height = dimensions[1] as usize;
    let sample = |x: usize, y: usize, z: usize| {
        input
            .signed_distance_sample_meters(z * width * height + y * width + x)
            .unwrap()
    };
    let lerp = |left: f64, right: f64, t: f64| left + (right - left) * t;
    let c00 = lerp(sample(x0, y0, z0), sample(x1, y0, z0), tx);
    let c10 = lerp(sample(x0, y1, z0), sample(x1, y1, z0), tx);
    let c01 = lerp(sample(x0, y0, z1), sample(x1, y0, z1), tx);
    let c11 = lerp(sample(x0, y1, z1), sample(x1, y1, z1), tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    lerp(c0, c1, tz)
}

#[test]
fn maintained_arena_projects_through_world_residency_into_generic_render_field_input() {
    let mut app = build_headless_game_app();
    app.add_plugins(RenderPlugin);
    app = app
        .run_for_fixed_steps(1)
        .expect("maintained arena should integrate before render projection");

    let descriptor = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .unwrap()
        .products()
        .values()
        .find(|descriptor| descriptor.product_id == ARENA_FIELD_PRODUCT_ID)
        .expect("maintained arena descriptor")
        .clone();
    let core = descriptor.product_core();
    let selection = RenderProductSelection::new("maintained-arena-gp1b3")
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
        ));
    app.world_mut()
        .resource_mut::<PreparedRenderProductSelectionResource>()
        .expect("Render should own product selection resource")
        .replace_contribution(
            RenderFrameProducerId::try_from_raw(9201).expect("producer id"),
            [selection],
        )
        .expect("maintained arena selection should ratify");

    app = app
        .run_for_frames(1)
        .expect("World-to-Render field projection should execute");

    let projected = app
        .world()
        .resource::<PreparedWorldSdfFieldSourceResource>()
        .expect("World should own prepared SDF field projections")
        .source(core.identity)
        .expect("maintained arena should project after selection and residency");
    let input = projected.input();

    assert_eq!(projected.product_generation(), core.lineage.generation);
    assert_eq!(projected.payload_ref(), descriptor.payload_refs[0]);
    assert_eq!(input.origin_local_meters(), [0.0; 3]);
    assert_eq!(input.sample_spacing_meters(), [0.5; 3]);
    assert_eq!(input.dimensions(), [9, 9, 9]);
    assert_eq!(input.sample_count(), 9 * 9 * 9);
    assert_eq!(
        input.max_absolute_query_error_local_meters(),
        arena_metric_encoding().max_absolute_error_meters(),
        "exact duplicated arena samples must add zero GP1B3 canonicalization error"
    );

    for z in 0..9 {
        for y in 0..9 {
            for x in 0..9 {
                let point = [x as f64 * 0.5, y as f64 * 0.5, z as f64 * 0.5];
                let expected = (arena_signed_distance_meters(point)
                    * f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER))
                .round()
                    / f64::from(ARENA_METRIC_DISTANCE_UNITS_PER_METER);
                let index = z * 9 * 9 + y * 9 + x;
                assert_eq!(
                    input.signed_distance_sample_meters(index),
                    Some(expected),
                    "{point:?}"
                );
            }
        }
    }

    let declared_error = input.max_absolute_query_error_local_meters();
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let point = [
                    (x as f64 + 0.5) * 0.25,
                    (y as f64 + 0.5) * 0.25,
                    (z as f64 + 0.5) * 0.25,
                ];
                let intended = arena_signed_distance_meters(point);
                let rendered = sample_projected_arena_field(input, point);
                assert!(
                    (rendered - intended).abs() <= declared_error + 1.0e-12,
                    "{point:?}: projected={rendered}, intended={intended}, declared={declared_error}"
                );
            }
        }
    }
}
