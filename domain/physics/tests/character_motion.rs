use physics::*;
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision};
use world_sdf::{
    CollisionQueryService, SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples,
    SdfChunkPayload, SdfChunkStore, SdfPageCoord3, SdfPageRecord,
};

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(1.0, [8, 8, 8]).expect("test partition is valid")
}

fn config() -> CharacterMotionConfig {
    CharacterMotionConfig {
        radius: 0.05,
        gravity_acceleration: 0.0,
        up: [0.0, 1.0, 0.0],
        max_walkable_slope_radians: 45.0_f32.to_radians(),
        contact_skin: 0.005,
        support_probe_distance: 0.05,
        max_slide_iterations: 4,
    }
}

fn clear_payload(chunk_id: ChunkId) -> SdfChunkPayload {
    SdfChunkPayload {
        chunk_id,
        chunk_revision: ChunkRevision::default(),
        chunk_generation: ChunkGeneration::default(),
        page_table: Default::default(),
        hierarchy_revision: 0,
        checksum: 0,
    }
}

fn plane_payload(chunk_id: ChunkId, signed_distance: impl Fn([f32; 3]) -> f32) -> SdfChunkPayload {
    let mut payload = clear_payload(chunk_id);
    let mut page = SdfPageRecord {
        page_generation: 0,
        bricks: Default::default(),
    };
    let edge = SDF_PAGE_EDGE_BRICKS as f32;
    for brick_z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
        for brick_y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for brick_x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                let mut distances = Vec::with_capacity(8);
                for sample_z in 0..2_u8 {
                    for sample_y in 0..2_u8 {
                        for sample_x in 0..2_u8 {
                            let point = [
                                (f32::from(brick_x) + f32::from(sample_x)) / edge,
                                (f32::from(brick_y) + f32::from(sample_y)) / edge,
                                (f32::from(brick_z) + f32::from(sample_z)) / edge,
                            ];
                            let encoded = (signed_distance(point) * 10_000.0)
                                .round()
                                .clamp(f32::from(i16::MIN) + 1.0, f32::from(i16::MAX));
                            distances.push(encoded as i16);
                        }
                    }
                }
                page.bricks.insert(
                    [brick_x, brick_y, brick_z],
                    SdfBrickRecord {
                        metadata: SdfBrickMetadata::default(),
                        samples: SdfBrickSamples { distances },
                    },
                );
            }
        }
    }
    payload
        .page_table
        .insert(SdfPageCoord3 { x: 0, y: 0, z: 0 }, page);
    payload
}

fn store_with(payload: SdfChunkPayload) -> SdfChunkStore {
    let mut store = SdfChunkStore::default();
    store.chunks.insert(payload.chunk_id, payload);
    store
}

fn chunk0() -> ChunkId {
    ChunkId::new(WorldId::new(0), ChunkCoord3::default())
}

fn input(
    state: CharacterPhysicalState,
    desired_planar_velocity: [f32; 3],
    fixed_step_seconds: f32,
    config: CharacterMotionConfig,
) -> CharacterMotionInput {
    CharacterMotionInput {
        state,
        desired_planar_velocity,
        fixed_step_seconds,
        config,
        world_id: WorldId::new(0),
    }
}

fn completed(
    outcome: CharacterMotionOutcome,
) -> (CharacterPhysicalState, CharacterMotionDiagnostics) {
    match outcome {
        CharacterMotionOutcome::Completed { state, diagnostics } => (state, diagnostics),
        other => panic!("expected completed motion, got {other:?}"),
    }
}

#[test]
fn clear_motion_admits_full_displacement() {
    let partition = partition();
    let store = store_with(clear_payload(chunk0()));
    let state = CharacterPhysicalState {
        position: [0.25, 0.5, 0.25],
        ..CharacterPhysicalState::default()
    };
    let (state, diagnostics) = completed(
        evaluate_character_motion(
            input(state, [1.0, 0.0, 0.0], 0.1, config()),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!((state.position[0] - 0.35).abs() < 1.0e-4);
    for axis in 0..3 {
        assert!(
            (diagnostics.requested_displacement[axis] - diagnostics.admitted_displacement[axis])
                .abs()
                < 1.0e-6
        );
    }
}

#[test]
fn wall_contact_stops_before_penetration() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| 0.5 - point[0]));
    let state = CharacterPhysicalState {
        position: [0.25, 0.5, 0.25],
        ..CharacterPhysicalState::default()
    };
    let (state, diagnostics) = completed(
        evaluate_character_motion(
            input(state, [1.0, 0.0, 0.0], 0.5, config()),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!(state.position[0] < 0.5);
    assert!(!diagnostics.contacts.is_empty());
}

#[test]
fn oblique_wall_contact_preserves_tangent_slide() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| 0.5 - point[0]));
    let state = CharacterPhysicalState {
        position: [0.25, 0.5, 0.2],
        ..CharacterPhysicalState::default()
    };
    let (state, diagnostics) = completed(
        evaluate_character_motion(
            input(state, [1.0, 0.0, 0.5], 0.5, config()),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!(state.position[0] < 0.5);
    assert!(state.position[2] > 0.2);
    assert!(!diagnostics.contacts.is_empty());
}

#[test]
fn floor_support_is_grounded() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| point[1] - 0.25));
    let mut cfg = config();
    cfg.support_probe_distance = 0.2;
    let state = CharacterPhysicalState {
        position: [0.5, 0.35, 0.5],
        ..CharacterPhysicalState::default()
    };
    let (state, diagnostics) = completed(
        evaluate_character_motion(
            input(state, [0.0; 3], 0.1, cfg),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!(state.grounded);
    assert!(diagnostics.support_normal.unwrap()[1] > 0.9);
}

#[test]
fn walkable_and_steep_slopes_are_classified() {
    let partition = partition();
    let mut cfg = config();
    cfg.support_probe_distance = 0.25;

    let walkable = store_with(plane_payload(chunk0(), |point| {
        point[1] - (0.5 * point[0] + 0.1)
    }));
    let walkable_state = CharacterPhysicalState {
        position: [0.5, 0.5, 0.5],
        ..CharacterPhysicalState::default()
    };
    let (walkable_state, _) = completed(
        evaluate_character_motion(
            input(walkable_state, [0.0; 3], 0.1, cfg),
            &partition,
            &walkable,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!(walkable_state.grounded);

    let steep = store_with(plane_payload(chunk0(), |point| {
        point[1] - (2.0 * point[0] - 0.5)
    }));
    let steep_state = CharacterPhysicalState {
        position: [0.5, 0.65, 0.5],
        ..CharacterPhysicalState::default()
    };
    let mut steep_cfg = cfg;
    steep_cfg.max_walkable_slope_radians = 30.0_f32.to_radians();
    let (steep_state, diagnostics) = completed(
        evaluate_character_motion(
            input(steep_state, [0.0; 3], 0.1, steep_cfg),
            &partition,
            &steep,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!(!steep_state.grounded);
    assert!(diagnostics.rejected_steep_support);
}

#[test]
fn gravity_changes_airborne_velocity_and_motion() {
    let partition = partition();
    let store = store_with(clear_payload(chunk0()));
    let mut cfg = config();
    cfg.gravity_acceleration = 10.0;
    let state = CharacterPhysicalState {
        position: [0.5, 0.5, 0.5],
        ..CharacterPhysicalState::default()
    };
    let (state, _) = completed(
        evaluate_character_motion(
            input(state, [0.0; 3], 0.1, cfg),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert!((state.velocity[1] + 1.0).abs() < 1.0e-4);
    assert!(state.position[1] < 0.5);
}

#[test]
fn missing_first_sweep_fails_closed() {
    let partition = partition();
    let state = CharacterPhysicalState {
        position: [0.5, 0.5, 0.5],
        ..CharacterPhysicalState::default()
    };
    let outcome = evaluate_character_motion(
        input(state, [1.0, 0.0, 0.0], 0.1, config()),
        &partition,
        &SdfChunkStore::default(),
        &CollisionQueryService,
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CharacterMotionOutcome::MissingPayload { .. }
    ));
}

#[test]
fn missing_support_payload_reports_partial_candidate_but_no_committed_state() {
    let partition = partition();
    let store = store_with(clear_payload(chunk0()));
    let mut cfg = config();
    cfg.support_probe_distance = 0.4;
    let state = CharacterPhysicalState {
        position: [0.25, 0.2, 0.25],
        ..CharacterPhysicalState::default()
    };
    let outcome = evaluate_character_motion(
        input(state, [0.2, 0.0, 0.0], 0.5, cfg),
        &partition,
        &store,
        &CollisionQueryService,
    )
    .unwrap();
    match outcome {
        CharacterMotionOutcome::MissingPayload { diagnostics, .. } => {
            assert!(diagnostics.admitted_displacement[0] > 0.0);
        }
        other => panic!("expected missing support payload, got {other:?}"),
    }
    assert_eq!(state.position, [0.25, 0.2, 0.25]);
}

#[test]
fn initial_overlap_is_explicit() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| 0.5 - point[0]));
    let state = CharacterPhysicalState {
        position: [0.75, 0.5, 0.5],
        ..CharacterPhysicalState::default()
    };
    let outcome = evaluate_character_motion(
        input(state, [0.0; 3], 0.1, config()),
        &partition,
        &store,
        &CollisionQueryService,
    )
    .unwrap();
    assert!(matches!(
        outcome,
        CharacterMotionOutcome::InitialOverlap { .. }
    ));
}

#[test]
fn slide_iteration_count_is_bounded() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| 0.5 - point[0]));
    let mut cfg = config();
    cfg.max_slide_iterations = 1;
    let state = CharacterPhysicalState {
        position: [0.25, 0.5, 0.2],
        ..CharacterPhysicalState::default()
    };
    let (_, diagnostics) = completed(
        evaluate_character_motion(
            input(state, [1.0, 0.0, 0.5], 0.5, cfg),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap(),
    );
    assert_eq!(diagnostics.slide_iterations, 1);
    assert!(diagnostics.iteration_limit_reached);
}

#[test]
fn invalid_config_is_rejected_explicitly() {
    let mut cfg = config();
    cfg.radius = 0.0;
    let error = evaluate_character_motion(
        input(CharacterPhysicalState::default(), [0.0; 3], 0.1, cfg),
        &partition(),
        &SdfChunkStore::default(),
        &CollisionQueryService,
    )
    .unwrap_err();
    assert_eq!(error, CharacterMotionError::InvalidRadius);
}

#[test]
fn excessive_slide_iteration_limit_is_rejected_before_world_queries() {
    let mut cfg = config();
    cfg.max_slide_iterations = 17;
    let error = evaluate_character_motion(
        input(CharacterPhysicalState::default(), [0.0; 3], 0.1, cfg),
        &partition(),
        &SdfChunkStore::default(),
        &CollisionQueryService,
    )
    .unwrap_err();
    assert_eq!(error, CharacterMotionError::InvalidSlideIterationLimit);
}

#[test]
fn identical_evaluations_produce_identical_results() {
    let partition = partition();
    let store = store_with(plane_payload(chunk0(), |point| 0.5 - point[0]));
    let request = input(
        CharacterPhysicalState {
            position: [0.25, 0.5, 0.2],
            ..CharacterPhysicalState::default()
        },
        [1.0, 0.0, 0.5],
        0.5,
        config(),
    );
    let first =
        evaluate_character_motion(request, &partition, &store, &CollisionQueryService).unwrap();
    let second =
        evaluate_character_motion(request, &partition, &store, &CollisionQueryService).unwrap();
    assert_eq!(first, second);
}

#[test]
fn pathological_finite_motion_bounds_are_rejected_before_world_queries() {
    let partition = partition();
    let store = SdfChunkStore::default();
    let base_state = CharacterPhysicalState::default();

    let mut huge_radius = config();
    huge_radius.radius = 2048.0;
    assert_eq!(
        evaluate_character_motion(
            input(base_state, [0.0; 3], 0.1, huge_radius),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidRadius
    );

    let mut invalid_skin = config();
    invalid_skin.contact_skin = invalid_skin.radius;
    assert_eq!(
        evaluate_character_motion(
            input(base_state, [0.0; 3], 0.1, invalid_skin),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidContactSkin
    );

    let mut huge_probe = config();
    huge_probe.support_probe_distance = 2048.0;
    assert_eq!(
        evaluate_character_motion(
            input(base_state, [0.0; 3], 0.1, huge_probe),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidSupportProbe
    );

    assert_eq!(
        evaluate_character_motion(
            input(base_state, [20_000.0, 0.0, 0.0], 0.1, config()),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidDesiredVelocity
    );
}

#[test]
fn sweep_chunk_volume_is_bounded_before_world_sdf_enumeration() {
    let tiny_partition =
        GridPartitionConfig::try_new(0.001, [8, 8, 8]).expect("tiny test partition is valid");
    let mut cfg = config();
    cfg.radius = 0.05;

    let error = evaluate_character_motion(
        input(
            CharacterPhysicalState {
                position: [0.25, 0.5, 0.25],
                ..CharacterPhysicalState::default()
            },
            [0.0; 3],
            0.1,
            cfg,
        ),
        &tiny_partition,
        &SdfChunkStore::default(),
        &CollisionQueryService,
    )
    .unwrap_err();

    assert_eq!(error, CharacterMotionError::SweepTooBroad);
}

#[test]
fn overflow_length_vectors_are_rejected_before_normalization_or_queries() {
    let partition = partition();
    let store = SdfChunkStore::default();

    let mut invalid_up = config();
    invalid_up.up = [f32::MAX; 3];
    assert_eq!(
        evaluate_character_motion(
            input(CharacterPhysicalState::default(), [0.0; 3], 0.1, invalid_up),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidUpVector
    );

    let invalid_velocity = CharacterPhysicalState {
        velocity: [f32::MAX; 3],
        ..CharacterPhysicalState::default()
    };
    assert_eq!(
        evaluate_character_motion(
            input(invalid_velocity, [0.0; 3], 0.1, config()),
            &partition,
            &store,
            &CollisionQueryService,
        )
        .unwrap_err(),
        CharacterMotionError::InvalidPhysicalState
    );
}
