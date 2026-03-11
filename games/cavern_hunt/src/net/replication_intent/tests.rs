use super::mvp_slice::{
    MvpClientEntityMap, apply_mvp_snapshot_payload, build_mvp_full_snapshot,
    build_mvp_snapshot_payload, health_component_name, player_input_component_name,
};
use super::registry::CavernReplicationIntent;
use crate::{
    CavernControlState, CavernPlayerOwnershipState, CavernServerControlMap, Health, Player,
    PlayerId, Transform2, Velocity2,
};
use engine::prelude::World;
use engine_net::replication::{
    InterestPolicy, NetEntityMap, ReplicationProfilePreset, SnapshotTimeline,
};
use engine_net::{ConnectionId, SimulationTick, decode_snapshot_payload};

fn build_player_world() -> World {
    let mut world = World::new();
    let player_a = world.spawn(Player);
    let _ = world.insert(player_a, PlayerId(1));
    let _ = world.insert(player_a, Transform2::new(1.0, 2.0, 0.1));
    let _ = world.insert(player_a, Velocity2 { x: 0.5, y: 0.0 });
    let _ = world.insert(player_a, Health::new(10.0));

    let player_b = world.spawn(Player);
    let _ = world.insert(player_b, PlayerId(2));
    let _ = world.insert(player_b, Transform2::new(4.0, -1.0, 0.2));
    let _ = world.insert(player_b, Velocity2 { x: -0.25, y: 0.75 });
    let _ = world.insert(player_b, Health::new(12.0));
    world
}

#[test]
fn registry_contains_expected_roadmap_component_intent() {
    let intent = CavernReplicationIntent::default();
    let player_state = intent
        .descriptor("PlayerStateReplicated")
        .expect("PlayerStateReplicated should be registered");
    assert_eq!(
        player_state.profile,
        ReplicationProfilePreset::PredictedMovement
    );
    assert_eq!(player_state.interest, InterestPolicy::Spatial);
    assert!(player_state.owner_prediction);

    let player_input = intent
        .descriptor("PlayerInputReplicated")
        .expect("PlayerInputReplicated should be registered");
    assert_eq!(player_input.profile, ReplicationProfilePreset::InputCommand);
    assert_eq!(player_input.interest, InterestPolicy::OwnerOnly);

    let health = intent
        .descriptor("HealthReplicated")
        .expect("HealthReplicated should be registered");
    assert_eq!(health.profile, ReplicationProfilePreset::ReliableState);
    assert_eq!(health.interest, InterestPolicy::Global);
}

#[test]
fn mvp_payload_only_includes_owner_input_component() {
    let world = build_player_world();
    let mut ownership = CavernPlayerOwnershipState::default();
    ownership.by_connection_id.insert(7, 1);
    ownership.by_connection_id.insert(8, 2);

    let mut controls = CavernServerControlMap::default();
    controls.by_player_id.insert(
        1,
        CavernControlState {
            movement: [1.0, 0.0],
            aim_world: [10.0, 0.0],
            fire_pressed: true,
            dash_pressed: false,
            interact_pressed: false,
            source_tick: SimulationTick(11),
        },
    );
    controls.by_player_id.insert(
        2,
        CavernControlState {
            movement: [0.0, -1.0],
            aim_world: [0.0, 10.0],
            fire_pressed: false,
            dash_pressed: true,
            interact_pressed: false,
            source_tick: SimulationTick(12),
        },
    );

    let mut net_ids = NetEntityMap::default();
    let payload_for_connection_7 =
        build_mvp_snapshot_payload(&world, ConnectionId(7), &ownership, &controls, &mut net_ids)
            .expect("payload build should succeed");
    let input_upserts = payload_for_connection_7
        .upserts
        .iter()
        .filter(|upsert| upsert.component_name == player_input_component_name())
        .collect::<Vec<_>>();
    assert_eq!(input_upserts.len(), 1);
    let input_state: super::components::PlayerInputReplicated =
        postcard::from_bytes(&input_upserts[0].payload).expect("input payload should decode");
    assert_eq!(input_state.player_id, 1);

    let payload_for_connection_8 =
        build_mvp_snapshot_payload(&world, ConnectionId(8), &ownership, &controls, &mut net_ids)
            .expect("payload build should succeed");
    let input_upserts_for_connection_8 = payload_for_connection_8
        .upserts
        .iter()
        .filter(|upsert| upsert.component_name == player_input_component_name())
        .collect::<Vec<_>>();
    assert_eq!(input_upserts_for_connection_8.len(), 1);
    let input_state_for_connection_8: super::components::PlayerInputReplicated =
        postcard::from_bytes(&input_upserts_for_connection_8[0].payload)
            .expect("input payload should decode");
    assert_eq!(input_state_for_connection_8.player_id, 2);
}

#[test]
fn mvp_full_snapshot_roundtrip_applies_player_state_and_health() {
    let world = build_player_world();
    let mut ownership = CavernPlayerOwnershipState::default();
    ownership.by_connection_id.insert(7, 1);
    let mut controls = CavernServerControlMap::default();
    controls.by_player_id.insert(
        1,
        CavernControlState {
            movement: [1.0, 0.0],
            aim_world: [1.0, 1.0],
            fire_pressed: false,
            dash_pressed: false,
            interact_pressed: false,
            source_tick: SimulationTick(20),
        },
    );

    let mut timeline = SnapshotTimeline::default();
    let mut net_ids = NetEntityMap::default();
    let snapshot = build_mvp_full_snapshot(
        &mut timeline,
        SimulationTick(21),
        &world,
        ConnectionId(7),
        &ownership,
        &controls,
        &mut net_ids,
    )
    .expect("full snapshot should build");
    let decoded_payload =
        decode_snapshot_payload(&snapshot.payload).expect("payload should decode");
    assert!(
        decoded_payload
            .upserts
            .iter()
            .any(|upsert| upsert.component_name == health_component_name())
    );

    let mut client_world = World::new();
    let mut client_map = MvpClientEntityMap::default();
    apply_mvp_snapshot_payload(&mut client_world, &decoded_payload, &mut client_map)
        .expect("snapshot payload should apply");

    let found_player_1 = client_world
        .query::<(engine::prelude::Entity, &PlayerId)>()
        .iter()
        .find(|(_, player_id)| player_id.0 == 1)
        .map(|(entity, _)| {
            (
                client_world
                    .get::<Transform2>(entity)
                    .copied()
                    .expect("transform should exist"),
                client_world
                    .get::<Health>(entity)
                    .copied()
                    .expect("health should exist"),
            )
        })
        .expect("player 1 should exist after apply");
    assert_eq!(found_player_1.0.x, 1.0);
    assert_eq!(found_player_1.0.y, 2.0);
    assert_eq!(found_player_1.1.current, 10.0);
}
