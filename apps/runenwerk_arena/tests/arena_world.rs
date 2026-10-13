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
    arena_exact_formation, arena_field_product_descriptor, arena_geometry_source,
    arena_metric_encoding, arena_metric_error_components_meters, arena_metric_payload_ref,
    arena_partition_config, arena_signed_distance_from_source, arena_signed_distance_meters,
    build_arena_chunk_payload, build_arena_chunk_payload_from_source, build_headless_game_app,
    player_physical_history_for,
};
use world_sdf::{
    SDF_METRIC_BRICK_EDGE_SAMPLES, SDF_METRIC_BRICK_SAMPLE_COUNT, SDF_PAGE_EDGE_BRICKS,
    WorldSdfOpenTopRectInteriorSource, WorldSdfPayloadRef, sample_world_sdf_metric_distance,
    validate_world_sdf_metric_payload,
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
fn normalized_arena_source_controls_exact_shape_and_both_formed_outputs() {
    let original = arena_geometry_source();
    assert_eq!(original.x_bounds_meters(), [0.5, 3.5]);
    assert_eq!(original.floor_y_meters(), 0.5);
    assert_eq!(original.z_bounds_meters(), [0.5, 3.5]);
    let alternate = WorldSdfOpenTopRectInteriorSource::try_new([1.25, 3.5], 0.5, [0.5, 3.5])
        .expect("alternate open-top interior is valid");

    let point = [0.75, 1.0, 1.0];
    assert_eq!(arena_signed_distance_from_source(original, point), 0.25);
    assert_eq!(arena_signed_distance_from_source(alternate, point), -0.5);
    assert_eq!(
        arena_signed_distance_from_source(original, [2.0, 5.0, 2.0]),
        1.5,
        "the source has no artificial upper roof"
    );

    let default_payload = build_arena_chunk_payload();
    let original_payload = build_arena_chunk_payload_from_source(original);
    let alternate_payload = build_arena_chunk_payload_from_source(alternate);
    assert_eq!(default_payload, original_payload);
    assert_ne!(original_payload.page_table, alternate_payload.page_table);

    // A declared ref can stay identical while actual formed bytes differ.
    // World admission must later validate source/formed correspondence.
    assert_eq!(
        WorldSdfPayloadRef::from(&original_payload),
        WorldSdfPayloadRef::from(&alternate_payload)
    );

    let default_page = original_payload.page_table.values().next().unwrap();
    let alternate_page = alternate_payload.page_table.values().next().unwrap();
    let default_edge = &default_page.bricks[&[0, 2, 2]];
    let alternate_edge = &alternate_page.bricks[&[0, 2, 2]];
    assert_ne!(
        default_edge.samples.distances,
        alternate_edge.samples.distances
    );
    assert_ne!(
        default_edge.metadata.occupancy_mask,
        alternate_edge.metadata.occupancy_mask
    );
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

#[test]
fn exact_source_is_not_enqueue_ready_and_limits_world_scope_to_formed_bytes() {
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, WorldSdfExactReadError,
        qualify_current_world_sdf_exact_source_read,
    };
    let bounds = WorldSdfExactQueryBounds::try_new([0.75, 0.75, 0.75], [1.0, 1.0, 1.0]).unwrap();
    let query = |app: &App, b: WorldSdfExactQueryBounds| {
        let w = app.world();
        let products = w
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap();
        let descriptor = products
            .products()
            .values()
            .find(|d| d.product_id == ARENA_FIELD_PRODUCT_ID)
            .unwrap();
        qualify_current_world_sdf_exact_source_read(
            descriptor.product_core().identity,
            b,
            w.resource::<PartitionConfigResource>().unwrap(),
            products,
            w.resource::<engine::plugins::world::chunks::DirtyChunkMapResource>()
                .unwrap(),
            w.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            w.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .map(|read| {
            read.sample_exact_distance_meters([0.75, 0.75, 0.75])
                .unwrap()
        })
    };
    assert!(
        query(&started_app(), bounds).is_err(),
        "catalog enqueue is not CPU readiness"
    );
    let mut app = integrated_app();
    assert_eq!(query(&app, bounds).unwrap(), 0.25);
    let original_partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .unwrap()
        .clone();
    *app.world_mut()
        .resource_mut::<PartitionConfigResource>()
        .unwrap() = PartitionConfigResource(
        runen_spatial::GridPartitionConfig::try_new(2.0, [8, 8, 8]).unwrap(),
    );
    assert!(matches!(
        query(&app, bounds),
        Err(WorldSdfExactReadError::PartitionMismatch)
    ));
    *app.world_mut()
        .resource_mut::<PartitionConfigResource>()
        .unwrap() = original_partition;
    let cross_chunk =
        WorldSdfExactQueryBounds::try_new([3.99, 1.0, 1.0], [4.01, 1.0, 1.0]).unwrap();
    assert!(matches!(
        query(&app, cross_chunk),
        Err(WorldSdfExactReadError::IncompleteCoverage)
    ));
    let high = WorldSdfExactQueryBounds::try_new([1.0, 4.0, 1.0], [1.0, 4.5, 1.0]).unwrap();
    assert!(matches!(
        query(&app, high),
        Err(WorldSdfExactReadError::IncompleteCoverage)
    ));

    let payload = app
        .world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .get_mut(&arena_chunk_id())
        .unwrap();
    payload
        .page_table
        .get_mut(&world_sdf::SdfPageCoord3::default())
        .unwrap()
        .bricks
        .get_mut(&[1, 1, 1])
        .unwrap()
        .samples
        .distances[0] += 1;
    assert!(matches!(
        query(&app, bounds),
        Err(WorldSdfExactReadError::InvalidFormation(_))
    ));
}

#[test]
fn verified_exact_formation_checks_all_occupancy_samples_metadata_and_summary() {
    use world_sdf::{
        RegionSdfSummary, WorldSdfExactFormationError, verify_world_sdf_open_top_exact_formation,
    };
    let source = arena_geometry_source();
    let payload = build_arena_chunk_payload();
    let descriptor = arena_field_product_descriptor(&payload);
    let metric = arena_metric_payload_ref(&payload);
    let bricks = payload.page_table.values().flat_map(|p| p.bricks.values());
    let all = bricks.collect::<Vec<_>>();
    let summary = RegionSdfSummary {
        min_distance: all.iter().map(|b| b.metadata.min_distance).min().unwrap(),
        max_distance: all.iter().map(|b| b.metadata.max_distance).max().unwrap(),
        occupied_chunk_count: u32::from(all.iter().any(|b| b.metadata.occupancy_mask != 0)),
        surface_chunk_count: u32::from(all.iter().any(|b| b.metadata.surface_band_present)),
    };
    let verify = |src, p: &world_sdf::SdfChunkPayload, s| {
        verify_world_sdf_open_top_exact_formation(
            src,
            &arena_partition_config(),
            &descriptor,
            metric,
            p,
            s,
        )
    };
    assert!(verify(source, &payload, summary).is_ok());
    // The independent byte verifier is deliberately NOT a source-identity
    // oracle: sub-quantization changes may preserve every formed byte.
    let epsilon =
        WorldSdfOpenTopRectInteriorSource::try_new([0.500001, 3.5], 0.5, [0.5, 3.5]).unwrap();
    assert!(verify(epsilon, &payload, summary).is_ok());
    let p = [0.5000005, 1.0, 1.0];
    assert!(source.signed_distance_meters(p) > 0.0);
    assert!(epsilon.signed_distance_meters(p) < 0.0);
    let alternate =
        WorldSdfOpenTopRectInteriorSource::try_new([0.75, 3.5], 0.5, [0.5, 3.5]).unwrap();
    assert!(verify(alternate, &payload, summary).is_err());

    let mut changed = payload.clone();
    changed
        .page_table
        .get_mut(&world_sdf::SdfPageCoord3::default())
        .unwrap()
        .bricks
        .get_mut(&[0, 0, 0])
        .unwrap()
        .metadata
        .occupancy_mask ^= 1;
    assert!(matches!(
        verify(source, &changed, summary),
        Err(WorldSdfExactFormationError::InvalidSampleOrMetadata { .. })
    ));
    changed = payload.clone();
    changed
        .page_table
        .get_mut(&world_sdf::SdfPageCoord3::default())
        .unwrap()
        .bricks
        .get_mut(&[0, 0, 0])
        .unwrap()
        .samples
        .distances[0] += 1;
    assert!(verify(source, &changed, summary).is_err());
    assert!(verify(source, &payload, RegionSdfSummary::default()).is_err());
    assert!(verify(source, &payload, summary).is_ok());
}

fn w1_region_summary(payload: &world_sdf::SdfChunkPayload) -> world_sdf::RegionSdfSummary {
    let bricks = payload
        .page_table
        .values()
        .flat_map(|page| page.bricks.values())
        .collect::<Vec<_>>();
    world_sdf::RegionSdfSummary {
        min_distance: bricks
            .iter()
            .map(|b| b.metadata.min_distance)
            .min()
            .unwrap(),
        max_distance: bricks
            .iter()
            .map(|b| b.metadata.max_distance)
            .max()
            .unwrap(),
        occupied_chunk_count: u32::from(bricks.iter().any(|b| b.metadata.occupancy_mask != 0)),
        surface_chunk_count: u32::from(bricks.iter().any(|b| b.metadata.surface_band_present)),
    }
}

fn w1_enqueue(
    app: &mut App,
    package: engine::plugins::world::build::WorldSdfRuntimePayloadPackage,
) -> Result<usize, engine::plugins::world::build::WorldSdfRuntimePayloadPackageError> {
    use engine::plugins::world::build::enqueue_ratified_world_sdf_payload_package;
    let mut queue = app
        .world_mut()
        .remove_resource::<WorldCompletedBuildQueueResource>()
        .unwrap();
    let mut products = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfProductCatalogResource>()
        .unwrap();
    let mut metric = app
        .world_mut()
        .remove_resource::<WorldRuntimeSdfMetricCapabilityCatalogResource>()
        .unwrap();
    let result = {
        let store = app
            .world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .clone();
        let runtime = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .unwrap();
        enqueue_ratified_world_sdf_payload_package(
            &mut queue,
            runtime,
            &store,
            &mut products,
            &mut metric,
            package,
        )
    };
    app.world_mut().insert_resource(queue);
    app.world_mut().insert_resource(products);
    app.world_mut().insert_resource(metric);
    result
}

#[test]
fn exact_pending_alias_requires_independent_source_proof_and_generic_overwrite_retires_it() {
    use engine::plugins::world::build::WorldSdfRuntimePayloadPackage;
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, qualify_current_world_sdf_exact_source_read,
    };
    use world_ops::{ChunkGeneration, ChunkRevision};
    let mut app = started_app();
    let payload = build_arena_chunk_payload();
    let mut alias = arena_field_product_descriptor(&payload);
    alias.product_id = world_sdf::FieldProductId(198);
    let alias_id = alias.product_core().identity;
    let primary = arena_field_product_descriptor(&payload)
        .product_core()
        .identity;
    let package = |descriptor: world_sdf::FieldProductDescriptor, source| {
        WorldSdfRuntimePayloadPackage::from_exact_formation(arena_exact_formation(
            source,
            descriptor.product_id,
            descriptor.lineage.source_revision,
            payload.chunk_revision,
            payload.chunk_generation,
        ))
    };
    let divergent =
        WorldSdfOpenTopRectInteriorSource::try_new([0.75, 3.5], 0.5, [0.5, 3.5]).unwrap();
    assert!(w1_enqueue(&mut app, package(alias.clone(), divergent)).is_err());
    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        1,
        "rejected alias did not alter the pending completion"
    );
    assert_eq!(
        w1_enqueue(&mut app, package(alias, arena_geometry_source())).unwrap(),
        0
    );
    app = app.run_for_fixed_steps(1).unwrap();
    let bounds = WorldSdfExactQueryBounds::try_new([0.75, 0.75, 0.75], [0.75, 0.75, 0.75]).unwrap();
    let is_exact = |app: &App, id| {
        let w = app.world();
        qualify_current_world_sdf_exact_source_read(
            id,
            bounds,
            w.resource::<PartitionConfigResource>().unwrap(),
            w.resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap(),
            w.resource::<engine::plugins::world::chunks::DirtyChunkMapResource>()
                .unwrap(),
            w.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            w.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .is_ok()
    };
    assert!(is_exact(&app, primary));
    assert!(is_exact(&app, alias_id));

    let mut next = payload.clone();
    next.chunk_revision = ChunkRevision(1);
    next.chunk_generation = ChunkGeneration(1);
    let mut next_desc = arena_field_product_descriptor(&next);
    next_desc.lineage.source_revision = 2;
    assert_eq!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::new(
                next_desc,
                vec![next.clone()],
                w1_region_summary(&next),
            )
            .with_metric_capability(arena_metric_payload_ref(&next))
        )
        .unwrap(),
        1
    );
    assert!(
        !is_exact(&app, primary),
        "pending replacement is never current"
    );
    app = app.run_for_fixed_steps(1).unwrap();
    assert!(
        !is_exact(&app, primary),
        "non-exact replacement retired the old source"
    );
    assert!(!is_exact(&app, alias_id));
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&arena_chunk_id()),
        Some(&next)
    );
}

#[test]
fn terminal_rejected_exact_replacement_preserves_certificate_but_requires_successful_retry() {
    use engine::plugins::world::build::WorldSdfRuntimePayloadPackage;
    use engine::plugins::world::chunks::lifecycle::ChunkLifecycleState;
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, qualify_current_world_sdf_exact_source_read,
    };
    use world_ops::{ChunkGeneration, ChunkRevision};
    let mut app = started_app();
    // Qualify two aliases before the first CPU commit. On failed N+1,
    // the untouched older alias must not expose retained geometry as current.
    let initial = build_arena_chunk_payload();
    let mut prior_alias = arena_field_product_descriptor(&initial);
    prior_alias.product_id = world_sdf::FieldProductId(199);
    let alias_id = prior_alias.product_core().identity;
    assert_eq!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::from_exact_formation(arena_exact_formation(
                arena_geometry_source(),
                prior_alias.product_id,
                prior_alias.lineage.source_revision,
                initial.chunk_revision,
                initial.chunk_generation,
            )),
        )
        .unwrap(),
        0
    );
    app = app.run_for_fixed_steps(1).unwrap();
    let prior = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .get(&arena_chunk_id())
        .unwrap()
        .clone();
    let mut next = prior.clone();
    next.chunk_revision = ChunkRevision(1);
    next.chunk_generation = ChunkGeneration(1);
    let mut descriptor = arena_field_product_descriptor(&next);
    descriptor.lineage.source_revision = 2;
    let id = descriptor.product_core().identity;
    let package = WorldSdfRuntimePayloadPackage::from_exact_formation(arena_exact_formation(
        arena_geometry_source(),
        descriptor.product_id,
        descriptor.lineage.source_revision,
        next.chunk_revision,
        next.chunk_generation,
    ));
    assert_eq!(w1_enqueue(&mut app, package.clone()).unwrap(), 1);
    {
        let queue = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .unwrap();
        let output = queue.outputs.front_mut().unwrap();
        output
            .chunk_payload
            .page_table
            .get_mut(&world_sdf::SdfPageCoord3::default())
            .unwrap()
            .bricks
            .get_mut(&[0, 0, 0])
            .unwrap()
            .samples
            .distances[0] += 1;
    }
    app = app.run_for_fixed_steps(1).unwrap();
    let w = app.world();
    assert_eq!(
        w.resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&arena_chunk_id()),
        Some(&prior)
    );
    let record = w
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&arena_chunk_id())
        .unwrap();
    assert!(
        record.has_exact_source(),
        "committed prior source preserved"
    );
    assert!(record.pending_build_generation.is_none());
    assert!(record.exact_read_recovery_required());
    assert!(matches!(record.lifecycle, ChunkLifecycleState::Ready));

    // New enqueue-time descriptor is not accepted as the old committed source.
    let bounds = WorldSdfExactQueryBounds::try_new([0.75; 3], [0.75; 3]).unwrap();
    let is_exact = |app: &App, selected_id| {
        let w = app.world();
        qualify_current_world_sdf_exact_source_read(
            selected_id,
            bounds,
            w.resource::<PartitionConfigResource>().unwrap(),
            w.resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap(),
            w.resource::<engine::plugins::world::chunks::DirtyChunkMapResource>()
                .unwrap(),
            w.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            w.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .is_ok()
    };
    assert!(!is_exact(&app, id));
    assert!(
        !is_exact(&app, alias_id),
        "old alias must be revoked after terminal failure"
    );
    assert_eq!(w1_enqueue(&mut app, package).unwrap(), 1);
    app = app.run_for_fixed_steps(1).unwrap();
    assert!(
        is_exact(&app, id),
        "valid newer generation restores exact readiness"
    );
    assert!(
        !is_exact(&app, alias_id),
        "old alias remains bound to the retired generation"
    );
}

#[test]
fn queued_dirtiness_rejects_exact_reads_and_ordinary_world_build_retires_source() {
    use engine::plugins::world::chunks::DirtyChunkMapResource;
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, WorldSdfExactReadError,
        qualify_current_world_sdf_exact_source_read,
    };
    use world_ops::{ChunkGeneration, DirtyReason};
    let mut app = integrated_app();
    let product_id = arena_field_product_descriptor(&build_arena_chunk_payload())
        .product_core()
        .identity;
    let bounds = WorldSdfExactQueryBounds::try_new([0.75; 3], [0.75; 3]).unwrap();
    let qualify = |app: &App| {
        let w = app.world();
        qualify_current_world_sdf_exact_source_read(
            product_id,
            bounds,
            w.resource::<PartitionConfigResource>().unwrap(),
            w.resource::<WorldRuntimeSdfProductCatalogResource>()
                .unwrap(),
            w.resource::<DirtyChunkMapResource>().unwrap(),
            w.resource::<WorldChunkRuntimeMapResource>().unwrap(),
            w.resource::<SdfChunkStoreResource>().unwrap(),
        )
        .map(|_| ())
    };
    assert!(qualify(&app).is_ok());
    app.world_mut()
        .resource_mut::<DirtyChunkMapResource>()
        .unwrap()
        .mark_dirty(arena_chunk_id(), DirtyReason::Geometry);
    assert!(
        matches!(qualify(&app), Err(WorldSdfExactReadError::RuntimeNotReady)),
        "unconsumed geometry dirtiness must revoke strict-current exactness"
    );
    app = app
        .run_for_fixed_steps(1)
        .expect("ordinary World dirty build integrates");
    let record = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .unwrap()
        .by_chunk_id
        .get(&arena_chunk_id())
        .unwrap();
    assert_eq!(record.chunk_generation, ChunkGeneration(1));
    assert!(
        !record.has_exact_source(),
        "generic build must retire certified source"
    );
    assert!(matches!(
        qualify(&app),
        Err(WorldSdfExactReadError::UnsupportedExactSource)
    ));
}

#[test]
fn exact_source_identity_rejects_epsilon_equivalent_formed_product_alias() {
    use engine::plugins::world::build::{
        WorldSdfRuntimePayloadPackage, WorldSdfRuntimePayloadPackageError,
    };
    let mut app = started_app();
    let original = arena_exact_formation(
        arena_geometry_source(),
        ARENA_FIELD_PRODUCT_ID,
        1,
        world_ops::ChunkRevision(0),
        world_ops::ChunkGeneration(0),
    );
    let epsilon =
        WorldSdfOpenTopRectInteriorSource::try_new([0.500001, 3.5], 0.5, [0.5, 3.5]).unwrap();
    let mut candidate = arena_exact_formation(
        epsilon,
        ARENA_FIELD_PRODUCT_ID,
        1,
        world_ops::ChunkRevision(0),
        world_ops::ChunkGeneration(0),
    );
    assert_eq!(
        original.payload(),
        candidate.payload(),
        "all quantized bytes coincide"
    );
    assert_eq!(
        original.metric_ref(),
        candidate.metric_ref(),
        "declared refs coincide"
    );
    assert_eq!(original.region_summary(), candidate.region_summary());
    assert_ne!(original.source(), candidate.source());
    // Cannot combine canonical producer bytes with an independently supplied
    // exact evaluator. A minted source/formation pair carries its own bytes.
    let mut old_alias = arena_field_product_descriptor(original.payload());
    old_alias.product_id = world_sdf::FieldProductId(997);
    let descriptor = candidate.descriptor().clone();
    assert_eq!(descriptor.lineage, original.descriptor().lineage);
    assert_eq!(descriptor.payload_refs, original.descriptor().payload_refs);

    let old_alias_formation = arena_exact_formation(
        arena_geometry_source(),
        old_alias.product_id,
        1,
        world_ops::ChunkRevision(0),
        world_ops::ChunkGeneration(0),
    );
    assert_eq!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::from_exact_formation(old_alias_formation,)
        )
        .unwrap(),
        0,
        "canonical source-qualified alias shares pending payload",
    );
    // A second product with the *same declared* source revision and bytes
    // must not inherit the canonical exact source if its f64 wall differs.
    assert!(matches!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::from_exact_formation(candidate.clone(),)
        ),
        Err(WorldSdfRuntimePayloadPackageError::PendingGenerationConflict { .. })
    ));
    assert_eq!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .len(),
        1
    );
    candidate = arena_exact_formation(
        arena_geometry_source(),
        ARENA_FIELD_PRODUCT_ID,
        1,
        world_ops::ChunkRevision(0),
        world_ops::ChunkGeneration(0),
    );
    assert_eq!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::from_exact_formation(candidate),
        )
        .unwrap(),
        0
    );
    app = app.run_for_fixed_steps(1).unwrap();
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, qualify_current_world_sdf_exact_source_read,
    };
    let world = app.world();
    let p = [0.5000005, 1.0, 1.0];
    let bounds = WorldSdfExactQueryBounds::try_new(p, p).unwrap();
    let view = qualify_current_world_sdf_exact_source_read(
        original.descriptor().product_core().identity,
        bounds,
        world.resource::<PartitionConfigResource>().unwrap(),
        world
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap(),
        world
            .resource::<engine::plugins::world::chunks::DirtyChunkMapResource>()
            .unwrap(),
        world.resource::<WorldChunkRuntimeMapResource>().unwrap(),
        world.resource::<SdfChunkStoreResource>().unwrap(),
    )
    .unwrap();
    assert!(
        view.sample_exact_distance_meters(p).unwrap() > 0.0,
        "read uses original producer-authored exact geometry"
    );
}

#[test]
fn full_precision_replacement_requires_explicit_source_revision_and_sealed_descriptor() {
    use engine::plugins::world::build::{
        WorldSdfRuntimePayloadPackage, WorldSdfRuntimePayloadPackageError,
    };
    use engine::plugins::world::queries::{
        WorldSdfExactQueryBounds, qualify_current_world_sdf_exact_source_read,
    };
    let original = build_arena_chunk_payload();
    let original_descriptor = arena_field_product_descriptor(&original);
    let mut forged = WorldSdfRuntimePayloadPackage::from_exact_formation(arena_exact_formation(
        arena_geometry_source(),
        ARENA_FIELD_PRODUCT_ID,
        1,
        world_ops::ChunkRevision(0),
        world_ops::ChunkGeneration(0),
    ));
    forged.descriptor.lineage.source_revision = 7;
    let mut app = integrated_app();
    assert!(
        matches!(
            w1_enqueue(&mut app, forged),
            Err(WorldSdfRuntimePayloadPackageError::ExactSourceRejected(
                world_sdf::WorldSdfExactFormationError::WrongPayloadIdentity
            ))
        ),
        "changing a public descriptor cannot rewrite the sealed producer lineage"
    );
    assert_eq!(
        app.world()
            .resource::<SdfChunkStoreResource>()
            .unwrap()
            .chunks
            .get(&arena_chunk_id()),
        Some(&original)
    );
    let shifted =
        WorldSdfOpenTopRectInteriorSource::try_new([0.500001, 3.5], 0.5, [0.5, 3.5]).unwrap();
    let next_same_rev = arena_exact_formation(
        shifted,
        ARENA_FIELD_PRODUCT_ID,
        1,
        world_ops::ChunkRevision(1),
        world_ops::ChunkGeneration(1),
    );
    assert_eq!(
        next_same_rev.payload().page_table,
        original.page_table,
        "sub-quantization source difference is invisible to metric bytes"
    );
    assert!(
        matches!(
            w1_enqueue(
                &mut app,
                WorldSdfRuntimePayloadPackage::from_exact_formation(next_same_rev,)
            ),
            Err(WorldSdfRuntimePayloadPackageError::ExactSourceRejected(
                world_sdf::WorldSdfExactFormationError::InvalidProductOrRevision
            ))
        ),
        "replacing an exact source requires advancing the authoritative source revision"
    );
    assert!(
        app.world()
            .resource::<WorldCompletedBuildQueueResource>()
            .unwrap()
            .outputs
            .is_empty()
    );

    let explicit_next = arena_exact_formation(
        shifted,
        ARENA_FIELD_PRODUCT_ID,
        2,
        world_ops::ChunkRevision(1),
        world_ops::ChunkGeneration(1),
    );
    assert_eq!(
        w1_enqueue(
            &mut app,
            WorldSdfRuntimePayloadPackage::from_exact_formation(explicit_next,)
        )
        .unwrap(),
        1
    );
    app = app.run_for_fixed_steps(1).unwrap();
    let p = [0.5000005, 1.0, 1.0];
    let world = app.world();
    let view = qualify_current_world_sdf_exact_source_read(
        original_descriptor.product_core().identity,
        WorldSdfExactQueryBounds::try_new(p, p).unwrap(),
        world.resource::<PartitionConfigResource>().unwrap(),
        world
            .resource::<WorldRuntimeSdfProductCatalogResource>()
            .unwrap(),
        world
            .resource::<engine::plugins::world::chunks::DirtyChunkMapResource>()
            .unwrap(),
        world.resource::<WorldChunkRuntimeMapResource>().unwrap(),
        world.resource::<SdfChunkStoreResource>().unwrap(),
    )
    .unwrap();
    assert!(
        view.sample_exact_distance_meters(p).unwrap() < 0.0,
        "explicitly revised source changes exact truth despite matching renderer bytes"
    );
}
