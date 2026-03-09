use super::*;

#[test]
fn v2_patch_flow_avoids_full_restore_after_initial_keyframe() {
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
    game::sync_active_player_slots(&mut server).unwrap();

    server_emit_replication_v2(&mut server).unwrap();
    let initial_messages = server
        .resource_mut::<NetworkServerOutbox>()
        .unwrap()
        .drain();
    assert!(initial_messages.iter().any(|message| {
        matches!(
            message,
            ServerMessage::RunEvent(RunEvent { code, .. }) if code == RUN_EVENT_KEYFRAME_V2
        )
    }));

    let player_entity = server
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();
    if let Some(mut transform) = server.get_mut::<crate::Transform2>(player_entity) {
        transform.x += 0.75;
    }
    if let Ok(mut tick) = server.resource_mut::<SimulationTick>() {
        tick.0 = 2;
    }

    server_emit_replication_v2(&mut server).unwrap();
    let patch_messages = server
        .resource_mut::<NetworkServerOutbox>()
        .unwrap()
        .drain();
    assert!(patch_messages.iter().any(|message| {
        matches!(
            message,
            ServerMessage::RunEvent(RunEvent { code, .. }) if code == RUN_EVENT_PATCH_V2
        )
    }));

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef::default());
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick::default());

    {
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        for message in initial_messages {
            inbound.push_server(message);
        }
    }
    client_apply_replication_events_v2(&mut client).unwrap();
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .full_world_restores,
        1
    );
    let player_entity_after_keyframe = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();

    {
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        for message in patch_messages {
            inbound.push_server(message);
        }
    }
    client_apply_replication_events_v2(&mut client).unwrap();
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .full_world_restores,
        1
    );
    assert!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .patches_applied
            >= 1
    );
    let player_entity_after_patch = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();
    assert_eq!(player_entity_after_patch, player_entity_after_keyframe);
}

#[test]
fn v2_applies_contiguous_patch_sequence_from_single_inbound_batch() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let mut player_step_1 = snapshot
        .players
        .iter()
        .find(|player| player.player_id == 1)
        .cloned()
        .unwrap();
    player_step_1.x += 0.75;
    let mut player_step_2 = player_step_1.clone();
    player_step_2.x += 0.75;

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef::default());
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick(120));

    let keyframe_payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
        cursor: crate::ReplicationCursor {
            server_tick: SimulationTick(60),
            stream_cursor: 1,
            base_cursor: 0,
        },
        snapshot,
    })
    .unwrap();
    let patch_1_payload = postcard::to_allocvec(&CavernPatchEventV2 {
        cursor: crate::ReplicationCursor {
            server_tick: SimulationTick(61),
            stream_cursor: 2,
            base_cursor: 1,
        },
        run_state: None,
        player_ops: vec![crate::CavernPlayerPatchOpV2::Patch {
            entity_id: NetworkEntityId(0x1000_0001),
            priority: crate::CavernPatchPriorityV2::High,
            state: player_step_1,
        }],
        enemy_ops: Vec::new(),
        projectile_ops: Vec::new(),
        pickup_ops: Vec::new(),
        extraction_ops: Vec::new(),
    })
    .unwrap();
    let patch_2_payload = postcard::to_allocvec(&CavernPatchEventV2 {
        cursor: crate::ReplicationCursor {
            server_tick: SimulationTick(62),
            stream_cursor: 3,
            base_cursor: 2,
        },
        run_state: None,
        player_ops: vec![crate::CavernPlayerPatchOpV2::Patch {
            entity_id: NetworkEntityId(0x1000_0001),
            priority: crate::CavernPatchPriorityV2::High,
            state: player_step_2.clone(),
        }],
        enemy_ops: Vec::new(),
        projectile_ops: Vec::new(),
        pickup_ops: Vec::new(),
        extraction_ops: Vec::new(),
    })
    .unwrap();

    {
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_KEYFRAME_V2.to_string(),
            payload: keyframe_payload,
        }));
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_PATCH_V2.to_string(),
            payload: patch_1_payload,
        }));
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_PATCH_V2.to_string(),
            payload: patch_2_payload,
        }));
    }

    client_apply_replication_events_v2(&mut client).unwrap();

    let state = client.resource::<ClientReplicationStateV2>().unwrap();
    assert!(state.has_keyframe);
    assert_eq!(state.last_cursor.stream_cursor, 3);

    let metrics = client.resource::<ReplicationRuntimeMetrics>().unwrap();
    assert_eq!(metrics.keyframes_applied, 1);
    assert_eq!(metrics.patches_applied, 2);
    assert_eq!(metrics.patches_applied_last_frame, 2);
    assert_eq!(metrics.patches_skipped_base_mismatch_last_frame, 0);
    assert_eq!(metrics.patches_stale_ignored_last_frame, 0);
}

#[test]
fn v2_local_patch_replays_pending_prediction_frames() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef {
        player_id: Some(1),
        entity: None,
    });
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick(3));

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(1),
                stream_cursor: 1,
                base_cursor: 0,
            },
            snapshot: snapshot.clone(),
        })
        .unwrap();
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload,
            }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let local_entity = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();
    if let Some(mut transform) = client.get_mut::<crate::Transform2>(local_entity) {
        transform.x += 2.0;
    }

    client.insert_resource(CavernPredictionState {
        pending_frames: vec![crate::CavernPredictedFrame {
            tick: SimulationTick(3),
            control: CavernControlState {
                movement: [1.0, 0.0],
                aim_world: [100.0, 0.0],
                fire_pressed: false,
                dash_pressed: false,
                interact_pressed: false,
                source_tick: SimulationTick(3),
            },
        }],
        corrections_applied: 0,
        last_authoritative_tick: SimulationTick(1),
    });
    client.insert_resource(SimulationTick(3));

    let local_state = snapshot
        .players
        .iter()
        .find(|player| player.player_id == 1)
        .cloned()
        .unwrap();
    {
        let payload = postcard::to_allocvec(&CavernPatchEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(2),
                stream_cursor: 2,
                base_cursor: 1,
            },
            run_state: None,
            player_ops: vec![crate::CavernPlayerPatchOpV2::Patch {
                entity_id: NetworkEntityId(0x1000_0001),
                priority: crate::CavernPatchPriorityV2::High,
                state: local_state,
            }],
            enemy_ops: Vec::new(),
            projectile_ops: Vec::new(),
            pickup_ops: Vec::new(),
            extraction_ops: Vec::new(),
        })
        .unwrap();
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_PATCH_V2.to_string(),
            payload,
        }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let prediction = client.resource::<CavernPredictionState>().unwrap();
    assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
    assert_eq!(prediction.corrections_applied, 1);
    assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
}

#[test]
fn v2_local_patch_replays_from_player_authoritative_input_tick() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef {
        player_id: Some(1),
        entity: None,
    });
    client.insert_resource(ClientReplicationStateV2::default());
    client.insert_resource(ClientReplicationMap::default());
    client.insert_resource(InterpolationConfig::default());
    client.insert_resource(AdaptiveSmoothingState::default());
    client.insert_resource(CorrectionStats::default());
    client.insert_resource(ReplicationRuntimeMetrics::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick(3));

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(1),
                stream_cursor: 1,
                base_cursor: 0,
            },
            snapshot: snapshot.clone(),
        })
        .unwrap();
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload,
            }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let local_entity = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();
    if let Some(mut transform) = client.get_mut::<crate::Transform2>(local_entity) {
        transform.x += 2.0;
    }

    client.insert_resource(CavernPredictionState {
        pending_frames: vec![crate::CavernPredictedFrame {
            tick: SimulationTick(3),
            control: CavernControlState {
                movement: [1.0, 0.0],
                aim_world: [100.0, 0.0],
                fire_pressed: false,
                dash_pressed: false,
                interact_pressed: false,
                source_tick: SimulationTick(3),
            },
        }],
        corrections_applied: 0,
        last_authoritative_tick: SimulationTick(1),
    });
    client.insert_resource(SimulationTick(3));

    let mut local_state = snapshot
        .players
        .iter()
        .find(|player| player.player_id == 1)
        .cloned()
        .unwrap();
    local_state.authoritative_input_tick = Some(SimulationTick(2));
    {
        let payload = postcard::to_allocvec(&CavernPatchEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(120),
                stream_cursor: 2,
                base_cursor: 1,
            },
            run_state: None,
            player_ops: vec![crate::CavernPlayerPatchOpV2::Patch {
                entity_id: NetworkEntityId(0x1000_0001),
                priority: crate::CavernPatchPriorityV2::High,
                state: local_state,
            }],
            enemy_ops: Vec::new(),
            projectile_ops: Vec::new(),
            pickup_ops: Vec::new(),
            extraction_ops: Vec::new(),
        })
        .unwrap();
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_PATCH_V2.to_string(),
            payload,
        }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let prediction = client.resource::<CavernPredictionState>().unwrap();
    assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
    assert_eq!(prediction.corrections_applied, 1);
    assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
}
