// Owner: Engine Networking Tests - Delta and Reconnect
#[test]
fn server_delta_snapshot_applies_cleanly_on_client() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));

    enqueue_server_inbox(
        server.world_mut(),
        ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }),
    )
    .expect("server inbox enqueue should succeed");
    enqueue_server_inbox(
        server.world_mut(),
        ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-1".to_string(),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let server = server.run_for_frames(1).expect("join handshake should run");

    let server = server
        .run_for_ticks(1)
        .expect("first server replication tick should run");
    let full_snapshot = server
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                connection_id,
                message: ServerMessage::Snapshot(snapshot),
            } if *connection_id == ConnectionId(1) => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a full snapshot for connection 1");

    let mut server = server;
    enqueue_server_inbox_from(
        server.world_mut(),
        Some(ConnectionId(1)),
        ClientMessage::Ack(Ack {
            cursor: full_snapshot.cursor,
            last_received_tick: full_snapshot.tick,
        }),
    )
    .expect("server inbox enqueue should succeed");
    server
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand { x: -0.5, y: 0.25 }));

    let server = server
        .run_for_frames(1)
        .expect("ack processing frame should run")
        .run_for_ticks(2)
        .expect("second server replication tick should run");

    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    let delta_snapshot = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                connection_id,
                message: ServerMessage::DeltaSnapshot(snapshot),
            } if *connection_id == ConnectionId(1) => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a delta snapshot for connection 1");
    let authoritative_second_snapshot = server
        .world()
        .resource::<ServerSnapshotState>()
        .unwrap()
        .latest_snapshot
        .clone()
        .expect("server should retain the latest authoritative snapshot");
    let decoded_delta: TestDelta =
        postcard::from_bytes(&delta_snapshot.payload).expect("delta payload should decode");
    let delta_tick = delta_snapshot.tick;
    assert_eq!(delta_snapshot.base, SnapshotCursor(1));
    assert_eq!(delta_snapshot.cursor, SnapshotCursor(2));
    assert!(!full_snapshot.payload.is_empty());
    assert!(!decoded_delta.changed);

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));

    enqueue_client_inbox(client.world_mut(), ServerMessage::Snapshot(full_snapshot))
        .expect("client inbox enqueue should succeed");
    let mut client = client
        .run_for_frames(1)
        .expect("client should accept the full snapshot");

    enqueue_client_inbox(client.world_mut(), ServerMessage::DeltaSnapshot(delta_snapshot))
        .expect("client inbox enqueue should succeed");
    let client = client
        .run_for_frames(1)
        .expect("client should apply the delta snapshot");

    let replication = client.world().resource::<ClientSnapshotState>().unwrap();
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
fn server_tracks_per_connection_baselines_across_reconnects() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    app.world_mut().insert_resource(ServerSessionConfig {
        server_id: "srv-local".to_string(),
        protocol: ProtocolVersion::new(1, 1, 1),
        tick_rate_hz: 60,
    });

    enqueue_server_inbox(
        app.world_mut(),
        ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }),
    )
    .expect("server inbox enqueue should succeed");
    enqueue_server_inbox(
        app.world_mut(),
        ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-1".to_string(),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let app = app.run_for_frames(1).expect("first join should run");
    let mut app = app.run_for_ticks(1).expect("first replication tick should run");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(ConnectionId(1)),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let mut app = app.run_for_frames(1).expect("ack frame should run");

    enqueue_server_inbox(
        app.world_mut(),
        ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }),
    )
    .expect("server inbox enqueue should succeed");
    enqueue_server_inbox(
        app.world_mut(),
        ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-2".to_string(),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let app = app.run_for_frames(1).expect("second join should run");

    let session = app.world().resource::<engine_net::ServerSessionState>().unwrap();
    assert!(session.active_connections.contains(&ConnectionId(1)));
    assert!(session.active_connections.contains(&ConnectionId(2)));

    let app = app
        .run_for_ticks(2)
        .expect("second replication tick should run");
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.server_messages().iter().any(|message| {
        matches!(
            message,
            OutboundServerMessage::ToConnection {
                connection_id,
                message: ServerMessage::DeltaSnapshot(snapshot),
            } if *connection_id == ConnectionId(1)
                && snapshot.base == SnapshotCursor(1)
                && snapshot.cursor == SnapshotCursor(2)
        )
    }));
    assert!(outbound.server_messages().iter().any(|message| {
        matches!(
            message,
            OutboundServerMessage::ToConnection {
                connection_id,
                message: ServerMessage::Snapshot(snapshot),
            } if *connection_id == ConnectionId(2) && snapshot.cursor == SnapshotCursor(2)
        )
    }));

    let replication = app.world().resource::<ServerSnapshotState>().unwrap();
    let checkpoint_a = replication
        .checkpoints
        .get(&ConnectionId(1))
        .expect("connection 1 checkpoint should exist");
    let checkpoint_b = replication
        .checkpoints
        .get(&ConnectionId(2))
        .expect("connection 2 checkpoint should exist");
    assert_eq!(checkpoint_a.last_ack_cursor, SnapshotCursor(1));
    assert_eq!(checkpoint_b.last_full_snapshot_cursor, SnapshotCursor(2));
}
