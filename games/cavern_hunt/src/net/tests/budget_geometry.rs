use super::*;

#[test]
fn v2_patch_budget_caps_projectile_ops_and_tracks_drops() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    server.insert_resource(engine_net::ServerSessionState {
        phase: engine_net::SessionPhase::Active,
        active_connection: Some(ConnectionId(7)),
        active_connections: [ConnectionId(7)].into_iter().collect(),
        ..Default::default()
    });
    server.insert_resource(ServerReplicationMap::default());
    server.insert_resource(ServerReplicationStateByConnection::default());
    server.insert_resource(ReplicationRuntimeMetrics::default());
    server.insert_resource(ReplicationBudgetConfig {
        enemy_ops_per_patch_level0: 64,
        enemy_ops_per_patch_level1: 64,
        enemy_ops_per_patch_level2: 64,
        projectile_ops_per_patch_level0: 2,
        projectile_ops_per_patch_level1: 2,
        projectile_ops_per_patch_level2: 2,
        pickup_ops_per_patch_level0: 64,
        pickup_ops_per_patch_level1: 64,
        pickup_ops_per_patch_level2: 64,
        extraction_ops_per_patch_level0: 64,
        extraction_ops_per_patch_level1: 64,
        extraction_ops_per_patch_level2: 64,
    });
    game::sync_active_player_slots(&mut server).unwrap();

    server_emit_replication_v2(&mut server).unwrap();
    server
        .resource_mut::<NetworkServerOutbox>()
        .unwrap()
        .drain();

    for i in 0..10 {
        combat::spawn_projectile(
            &mut server,
            [i as f32 * 0.1, 0.0],
            [1.0, 0.0],
            8.0,
            1.2,
            crate::Faction::Hunters,
        );
    }
    if let Ok(mut tick) = server.resource_mut::<SimulationTick>() {
        tick.0 = 2;
    }

    server_emit_replication_v2(&mut server).unwrap();
    let messages = server
        .resource_mut::<NetworkServerOutbox>()
        .unwrap()
        .drain();
    let patch = messages
        .iter()
        .find_map(|message| match message {
            ServerMessage::RunEvent(RunEvent { code, payload }) if code == RUN_EVENT_PATCH_V2 => {
                postcard::from_bytes::<CavernPatchEventV2>(payload).ok()
            }
            _ => None,
        })
        .expect("expected v2 patch event");
    assert!(patch.projectile_ops.len() <= 2);
    let metrics = server.resource::<ReplicationRuntimeMetrics>().unwrap();
    assert!(metrics.dropped_projectile_ops_last_tick > 0);
    assert!(metrics.patch_projectile_ops_last_tick <= 2);
}
