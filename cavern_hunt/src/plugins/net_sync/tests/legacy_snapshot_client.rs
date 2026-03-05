use super::*;

#[test]
#[ignore = "legacy V1 snapshot/delta protocol removed"]
fn client_applies_snapshot_and_delta_events() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let snapshot_event = RunEvent {
        code: RUN_EVENT_SNAPSHOT.to_string(),
        payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
            tick: SimulationTick(2),
            cursor: 1,
            snapshot: snapshot.clone(),
        })
        .unwrap(),
    };

    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick::default());
    client
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(snapshot_event));
    client_apply_replication_events(&mut client).unwrap();
    let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
    assert_eq!(rebuilt, snapshot);

    let local = server
        .query::<(engine::prelude::Entity, &crate::domain::Player)>()
        .iter()
        .map(|(entity, _)| entity)
        .next()
        .unwrap();
    if let Some(mut transform) = server.get_mut::<crate::domain::Transform2>(local) {
        transform.x += 2.0;
    }
    combat::spawn_projectile(
        &mut server,
        [1.0, 1.0],
        [1.0, 0.0],
        6.0,
        1.5,
        crate::domain::Faction::Hunters,
    );
    let current = capture_cavern_run_snapshot(&server).unwrap();
    let delta = crate::domain::build_cavern_run_delta(&snapshot, &current);
    client
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .clear();
    client
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_DELTA.to_string(),
            payload: postcard::to_allocvec(&CavernDeltaEventV1 {
                tick: SimulationTick(3),
                base_cursor: 1,
                cursor: 2,
                delta,
            })
            .unwrap(),
        }));
    client_apply_replication_events(&mut client).unwrap();
    let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
    assert_eq!(rebuilt, current);
}

#[test]
#[ignore = "legacy V1 geometry path removed"]
fn client_applies_geometry_edit_event_incrementally() {
    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState::default());
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick::default());
    client.insert_resource(CavernRunConfig::default());
    client.insert_resource(CavernRunState::default());
    client.insert_resource(crate::domain::CavernLayout::default());
    client.insert_resource(SpawnDirector::default());
    client.insert_resource(LootTableRegistry::default());
    client.insert_resource(CavernMetaProfile::default());
    client.insert_resource(CavernCameraState::default());
    client.insert_resource(CavernAimState::default());
    crate::plugins::worldgen::initialize_run_world(&mut client, true).unwrap();
    let baseline_runtime = client
        .resource::<crate::domain::CavernGeometryRuntimeState>()
        .unwrap()
        .clone();
    let baseline_edit_count = baseline_runtime.edit_events.len();
    let baseline_blocker_count = client
        .resource::<crate::domain::CavernGeometryGraph>()
        .unwrap()
        .primitives
        .iter()
        .filter(|primitive| primitive.op == GeometryOp::Blocker)
        .count();
    let next_revision = client
        .resource::<crate::domain::CavernGeometryGraph>()
        .unwrap()
        .revision
        .0
        .saturating_add(1);

    let edits = vec![crate::domain::GeometryEditEvent {
        revision: crate::domain::GeometryRevision(next_revision),
        edit: GeometryEdit {
            kind: GeometryEditKind::AddBlocker(GeometryPrimitiveShape3::Cylinder {
                center: [0.0, crate::domain::CAVERN_GAMEPLAY_HEIGHT, 0.0],
                radius: 1.2,
                half_height: 1.4,
            }),
        },
    }];
    client
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_GEOMETRY_EDITS.to_string(),
            payload: postcard::to_allocvec(&CavernGeometryEditsEventV1 {
                tick: SimulationTick(5),
                from_index: baseline_edit_count,
                to_index: baseline_edit_count + edits.len(),
                extraction_seal_primitive: None,
                edits: edits.clone(),
            })
            .unwrap(),
        }));

    client_apply_replication_events(&mut client).unwrap();

    let runtime = client
        .resource::<crate::domain::CavernGeometryRuntimeState>()
        .unwrap();
    assert_eq!(runtime.edit_events.len(), baseline_edit_count + edits.len());
    assert_eq!(runtime.edit_events.last(), edits.last());
    let graph = client
        .resource::<crate::domain::CavernGeometryGraph>()
        .unwrap();
    let blocker_count = graph
        .primitives
        .iter()
        .filter(|primitive| primitive.op == GeometryOp::Blocker)
        .count();
    assert_eq!(blocker_count, baseline_blocker_count + 1);
}

#[test]
#[ignore = "legacy V1 snapshot path removed"]
fn client_replays_pending_predicted_frame_after_authoritative_snapshot() {
    let mut server = server_world();
    server.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: [(7, 1)].into_iter().collect(),
    });
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    let mut client = World::new();
    client.insert_resource(NetworkInboundQueue::default());
    client.insert_resource(CavernNetSyncState::default());
    client.insert_resource(CavernPredictionState {
        pending_frames: vec![crate::domain::CavernPredictedFrame {
            tick: SimulationTick(2),
            control: CavernControlState {
                movement: [1.0, 0.0],
                aim_world: [100.0, 0.0],
                fire_pressed: false,
                dash_pressed: false,
                interact_pressed: false,
                source_tick: SimulationTick(2),
            },
        }],
        corrections_applied: 0,
        last_authoritative_tick: SimulationTick::default(),
    });
    client.insert_resource(FixedTimeConfig::default());
    client.insert_resource(LocalPlayerRef::default());
    client.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client.insert_resource(SimulationTick(2));
    client
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_SNAPSHOT.to_string(),
            payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                tick: SimulationTick(1),
                cursor: 1,
                snapshot: snapshot.clone(),
            })
            .unwrap(),
        }));

    let mut expected = World::new();
    expected.insert_resource(FixedTimeConfig::default());
    expected.insert_resource(LocalPlayerRef::default());
    restore_cavern_run_snapshot(&mut expected, &snapshot).unwrap();
    combat::replay_predicted_local_frame(
        &mut expected,
        CavernControlState {
            movement: [1.0, 0.0],
            aim_world: [100.0, 0.0],
            fire_pressed: false,
            dash_pressed: false,
            interact_pressed: false,
            source_tick: SimulationTick(2),
        },
        FixedTimeConfig::default().step_seconds,
    )
    .unwrap();

    client_apply_replication_events(&mut client).unwrap();
    let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
    let expected_snapshot = capture_cavern_run_snapshot(&expected).unwrap();
    assert_eq!(rebuilt, expected_snapshot);
    let prediction = client.resource::<CavernPredictionState>().unwrap();
    assert_eq!(prediction.pending_frames.len(), 1);
    assert_eq!(prediction.last_authoritative_tick, SimulationTick(1));
    assert_eq!(prediction.corrections_applied, 1);
}

#[test]
#[ignore = "legacy V1 snapshot path removed"]
fn two_clients_restore_different_owned_players_from_same_server_run() {
    let mut server = server_world();
    {
        let mut ownership = server.resource_mut::<CavernPlayerOwnershipState>().unwrap();
        ownership.by_connection_id = [(11, 1), (22, 2)].into_iter().collect();
    }
    game::sync_active_player_slots(&mut server).unwrap();
    let snapshot = capture_cavern_run_snapshot(&server).unwrap();
    assert_eq!(snapshot.players.len(), 2);

    let mut client_a = World::new();
    client_a.insert_resource(NetworkInboundQueue::default());
    client_a.insert_resource(CavernNetSyncState::default());
    client_a.insert_resource(CavernPredictionState::default());
    client_a.insert_resource(FixedTimeConfig::default());
    client_a.insert_resource(LocalPlayerRef::default());
    client_a.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(11)),
        connected: true,
        ..Default::default()
    });
    client_a.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client_a
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_SNAPSHOT.to_string(),
            payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                tick: SimulationTick(1),
                cursor: 1,
                snapshot: snapshot.clone(),
            })
            .unwrap(),
        }));
    client_apply_replication_events(&mut client_a).unwrap();
    assert_eq!(
        client_a.resource::<LocalPlayerRef>().unwrap().player_id,
        Some(1)
    );

    let mut client_b = World::new();
    client_b.insert_resource(NetworkInboundQueue::default());
    client_b.insert_resource(CavernNetSyncState::default());
    client_b.insert_resource(CavernPredictionState::default());
    client_b.insert_resource(FixedTimeConfig::default());
    client_b.insert_resource(LocalPlayerRef::default());
    client_b.insert_resource(NetworkSessionStatus {
        connection_id: Some(ConnectionId(22)),
        connected: true,
        ..Default::default()
    });
    client_b.insert_resource(SimulationProfileConfig {
        profile: SimulationProfile::DedicatedAuthority,
        authority: engine::prelude::AuthorityRole::Client,
        determinism: engine::prelude::DeterminismLevel::Validated,
    });
    client_b
        .resource_mut::<NetworkInboundQueue>()
        .unwrap()
        .push_server(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_SNAPSHOT.to_string(),
            payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                tick: SimulationTick(1),
                cursor: 1,
                snapshot,
            })
            .unwrap(),
        }));
    client_apply_replication_events(&mut client_b).unwrap();
    assert_eq!(
        client_b.resource::<LocalPlayerRef>().unwrap().player_id,
        Some(2)
    );
}
