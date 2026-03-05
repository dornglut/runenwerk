use super::*;

#[test]
fn v2_patch_rebinds_local_player_from_owner_connection_id() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(22, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let mut player_state = snapshot.players.first().cloned().unwrap();
    player_state.owner_connection_id = Some(22);

    let mut client = World::new();
    client.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(22)),
        connected: true,
        ..Default::default()
    });
    client.insert_resource(LocalPlayerRef {
        player_id: Some(99),
        entity: None,
    });
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());

    apply_player_patch_ops_v2(
        &mut client,
        vec![CavernPlayerPatchOpV2::Spawn {
            entity_id: NetworkEntityId(0x1000_0001),
            priority: CavernPatchPriorityV2::Critical,
            state: player_state,
        }],
        None,
        true,
    )
    .unwrap();

    let local = client.resource::<LocalPlayerRef>().unwrap().clone();
    assert_eq!(local.player_id, Some(1));
    let local_entity = local.entity.expect("local player entity should be set");
    assert_eq!(
        client
            .get::<crate::domain::PlayerId>(local_entity)
            .unwrap()
            .0,
        1
    );
}

#[test]
fn v2_local_patch_ignores_duplicate_authoritative_tick_for_transform() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(22, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let mut state_a = snapshot.players.first().cloned().unwrap();
    state_a.owner_connection_id = Some(22);
    let mut state_b = state_a.clone();
    state_b.x += 2.0;
    state_b.y += 1.5;

    let mut client = World::new();
    client.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(22)),
        connected: true,
        ..Default::default()
    });
    client.insert_resource(LocalPlayerRef {
        player_id: Some(1),
        entity: None,
    });
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(CavernPredictionState {
        pending_frames: Vec::new(),
        corrections_applied: 0,
        last_authoritative_tick: SimulationTick::default(),
    });
    client.insert_resource(SimulationTick(90));

    let entity_id = NetworkEntityId(0x1000_0001);
    apply_player_patch_ops_v2(
        &mut client,
        vec![CavernPlayerPatchOpV2::Spawn {
            entity_id,
            priority: CavernPatchPriorityV2::Critical,
            state: state_a.clone(),
        }],
        Some(SimulationTick(4)),
        true,
    )
    .unwrap();
    let local_entity = client.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
    let transform_before = client
        .get::<crate::domain::Transform2>(local_entity)
        .unwrap()
        .x;

    apply_player_patch_ops_v2(
        &mut client,
        vec![CavernPlayerPatchOpV2::Patch {
            entity_id,
            priority: CavernPatchPriorityV2::High,
            state: state_b,
        }],
        Some(SimulationTick(4)),
        true,
    )
    .unwrap();

    let transform_after = client
        .get::<crate::domain::Transform2>(local_entity)
        .unwrap();
    assert!(
        (transform_after.x - transform_before).abs() < 0.0001,
        "duplicate authoritative tick should not re-correct x: before={} after={}",
        transform_before,
        transform_after.x
    );
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .local_correction_distance_last,
        0.0
    );
}

#[test]
fn v2_local_patch_prefers_player_authoritative_input_tick_over_cursor_tick() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(22, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let mut state_a = snapshot.players.first().cloned().unwrap();
    state_a.owner_connection_id = Some(22);
    state_a.authoritative_input_tick = Some(SimulationTick(10));
    let mut state_b = state_a.clone();
    state_b.x += 2.0;
    state_b.y += 1.5;
    state_b.authoritative_input_tick = Some(SimulationTick(10));

    let mut client = World::new();
    client.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(22)),
        connected: true,
        ..Default::default()
    });
    client.insert_resource(LocalPlayerRef {
        player_id: Some(1),
        entity: None,
    });
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(CavernPredictionState {
        pending_frames: Vec::new(),
        corrections_applied: 0,
        last_authoritative_tick: SimulationTick::default(),
    });
    client.insert_resource(SimulationTick(90));

    let entity_id = NetworkEntityId(0x1000_0001);
    apply_player_patch_ops_v2(
        &mut client,
        vec![CavernPlayerPatchOpV2::Spawn {
            entity_id,
            priority: CavernPatchPriorityV2::Critical,
            state: state_a,
        }],
        Some(SimulationTick(10)),
        true,
    )
    .unwrap();
    let local_entity = client.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
    let transform_before = client
        .get::<crate::domain::Transform2>(local_entity)
        .unwrap()
        .x;

    apply_player_patch_ops_v2(
        &mut client,
        vec![CavernPlayerPatchOpV2::Patch {
            entity_id,
            priority: CavernPatchPriorityV2::High,
            state: state_b,
        }],
        Some(SimulationTick(120)),
        true,
    )
    .unwrap();

    let transform_after = client
        .get::<crate::domain::Transform2>(local_entity)
        .unwrap();
    assert!(
        (transform_after.x - transform_before).abs() < 0.0001,
        "player authoritative input tick should gate corrections even when cursor advances: before={} after={}",
        transform_before,
        transform_after.x
    );
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .local_correction_distance_last,
        0.0
    );
    assert_eq!(
        client
            .resource::<CavernPredictionState>()
            .unwrap()
            .last_authoritative_tick,
        SimulationTick(10)
    );
}
