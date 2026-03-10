// Owner: Engine Networking Tests - Delta and Reconnect
#[test]
fn server_delta_snapshot_applies_cleanly_on_client() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));

    let server = server
        .run_for_ticks(1)
        .expect("first server tick should run");
    let full_snapshot = server
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .server_messages()
        .iter()
        .find_map(|message| match message {
            ServerMessage::Snapshot(snapshot) => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a full snapshot first");

    let mut server = server;
    server
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand {
            x: -0.5,
            y: 0.25,
        }));
    let server = server
        .run_for_ticks(2)
        .expect("second server tick should run");
    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    let delta_snapshot = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            ServerMessage::DeltaSnapshot(snapshot) => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a delta snapshot on the second tick");
    let authoritative_second_snapshot = server
        .world()
        .resource::<SnapshotReplicationState>()
        .unwrap()
        .last_sent_snapshot
        .clone()
        .expect("server should retain the second authoritative snapshot");
    let decoded_delta: TestDelta =
        postcard::from_bytes(&delta_snapshot.payload).expect("delta payload should decode");
    let delta_tick = delta_snapshot.tick;
    assert_eq!(delta_snapshot.base, SnapshotCursor(0));
    assert_eq!(delta_snapshot.cursor, SnapshotCursor(2));
    assert!(!full_snapshot.payload.is_empty());
    assert!(!decoded_delta.changed);

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));

    client
        .world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::Snapshot(full_snapshot));
    let mut client = client
        .run_for_frames(1)
        .expect("client should accept the full snapshot");

    client
        .world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::DeltaSnapshot(delta_snapshot));
    let client = client
        .run_for_frames(1)
        .expect("client should apply the delta snapshot");

    let replication = client
        .world()
        .resource::<SnapshotReplicationState>()
        .unwrap();
    assert_eq!(replication.last_acknowledged_cursor, SnapshotCursor(2));
    assert_eq!(replication.last_received_tick, delta_tick);
    let last_snapshot = replication
        .last_received_snapshot
        .clone()
        .expect("client should retain the latest applied snapshot");
    assert_eq!(last_snapshot, authoritative_second_snapshot);

    let outbound = client.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.client_messages().iter().any(
        |message| matches!(message, ClientMessage::Ack(ack) if ack.cursor == SnapshotCursor(2))
    ));
}

#[test]
fn server_reconnect_resets_initial_snapshot_state() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    app.world_mut().insert_resource(ServerSessionConfig {
        server_id: "srv-local".to_string(),
        protocol: ProtocolVersion::new(1, 1, 1),
        tick_rate_hz: 60,
    });

    app.world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    app.world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-1".to_string(),
        }));
    let app = app.run_for_frames(1).expect("first join should run");
    let mut app = app
        .run_for_ticks(1)
        .expect("first replication tick should run");
    assert!(
        app.world()
            .resource::<SnapshotReplicationState>()
            .unwrap()
            .initial_snapshot_sent
    );

    app.world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    app.world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-2".to_string(),
        }));
    let app = app.run_for_frames(1).expect("second join should run");
    assert_eq!(
        app.world()
            .resource::<SnapshotReplicationState>()
            .unwrap()
            .active_connection,
        Some(engine_net::ConnectionId(2))
    );
    assert!(
        !app.world()
            .resource::<SnapshotReplicationState>()
            .unwrap()
            .initial_snapshot_sent
    );
    assert_eq!(
        app.world()
            .resource::<SnapshotReplicationState>()
            .unwrap()
            .last_acknowledged_cursor,
        SnapshotCursor(0)
    );

    let app = app
        .run_for_ticks(2)
        .expect("second replication tick should run");
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.server_messages().iter().any(
        |message| matches!(message, ServerMessage::Snapshot(snapshot) if snapshot.cursor == SnapshotCursor(2))
    ));
}
