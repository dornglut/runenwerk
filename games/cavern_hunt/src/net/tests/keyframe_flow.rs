use super::*;

#[test]
fn v2_followup_keyframe_preserves_owned_pose_without_pending_prediction() {
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
    client.insert_resource(SimulationTick(120));

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(60),
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
    let predicted_transform = crate::Transform2::new(9.5, -4.25, 0.45);
    let predicted_velocity = crate::Velocity2 { x: 5.5, y: -2.0 };
    let _ = client.insert(local_entity, predicted_transform);
    let _ = client.insert(local_entity, predicted_velocity);
    assert!(
        client
            .resource::<ClientReplicationStateV2>()
            .unwrap()
            .has_keyframe
    );

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(120),
                stream_cursor: 2,
                base_cursor: 1,
            },
            snapshot,
        })
        .unwrap();
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_KEYFRAME_V2.to_string(),
            payload,
        }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let local_entity_after = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
        .unwrap();
    let transform_after = client
        .get::<crate::Transform2>(local_entity_after)
        .copied()
        .unwrap();
    let velocity_after = client
        .get::<crate::Velocity2>(local_entity_after)
        .copied()
        .unwrap();
    assert!(
        (transform_after.x - predicted_transform.x).abs() < 0.001,
        "x after={} predicted={}",
        transform_after.x,
        predicted_transform.x
    );
    assert!(
        (transform_after.y - predicted_transform.y).abs() < 0.001,
        "y after={} predicted={}",
        transform_after.y,
        predicted_transform.y
    );
    assert!(
        (transform_after.yaw - predicted_transform.yaw).abs() < 0.001,
        "yaw after={} predicted={}",
        transform_after.yaw,
        predicted_transform.yaw
    );
    assert_eq!(velocity_after, predicted_velocity);
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .full_world_restores,
        1
    );
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .local_correction_hard_snaps_total,
        0
    );
}

#[test]
fn v2_followup_keyframe_replays_from_player_authoritative_input_tick() {
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

    let mut followup_snapshot = snapshot;
    if let Some(local_player) = followup_snapshot
        .players
        .iter_mut()
        .find(|player| player.player_id == 1)
    {
        local_player.authoritative_input_tick = Some(SimulationTick(2));
    }
    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(120),
                stream_cursor: 120,
                base_cursor: 119,
            },
            snapshot: followup_snapshot,
        })
        .unwrap();
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_KEYFRAME_V2.to_string(),
            payload,
        }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let prediction = client.resource::<CavernPredictionState>().unwrap();
    assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
    assert_eq!(prediction.corrections_applied, 1);
    assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .full_world_restores,
        2
    );
}

#[test]
fn v2_followup_keyframe_preserves_remote_pose_for_smoothing() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(11, 1), (22, 2)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(11)),
        connected: true,
        ..Default::default()
    });
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
    client.insert_resource(SimulationTick(120));

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(60),
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

    let remote_entity = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 2).then_some(entity))
        .unwrap();
    let remote_predicted = crate::Transform2::new(-7.25, 4.0, -0.35);
    let _ = client.insert(remote_entity, remote_predicted);

    {
        let payload = postcard::to_allocvec(&crate::CavernKeyframeEventV2 {
            cursor: crate::ReplicationCursor {
                server_tick: SimulationTick(120),
                stream_cursor: 2,
                base_cursor: 1,
            },
            snapshot,
        })
        .unwrap();
        let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
        inbound.clear();
        inbound.push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_KEYFRAME_V2.to_string(),
            payload,
        }));
    }
    client_apply_replication_events_v2(&mut client).unwrap();

    let remote_entity_after = client
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, player_id)| (player_id.0 == 2).then_some(entity))
        .unwrap();
    let remote_after = client
        .get::<crate::Transform2>(remote_entity_after)
        .copied()
        .unwrap();
    assert!((remote_after.x - remote_predicted.x).abs() < 0.001);
    assert!((remote_after.y - remote_predicted.y).abs() < 0.001);
    assert!((remote_after.yaw - remote_predicted.yaw).abs() < 0.001);
    assert_eq!(
        client
            .resource::<ReplicationRuntimeMetrics>()
            .unwrap()
            .full_world_restores,
        1
    );
}
