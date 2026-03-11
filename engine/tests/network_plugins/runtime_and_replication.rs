// Owner: Engine Networking Tests - Runtime and Replication
#[test]
fn network_runtime_handle_events_flow_into_engine_state() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    let (command_tx, _command_rx) = tokio::sync::mpsc::channel(16);
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(16);
    event_tx
        .try_send(engine_net::SessionRuntimeEvent::Connected {
            connection_id: Some(engine_net::ConnectionId(9)),
        })
        .unwrap();
    event_tx
        .try_send(engine_net::SessionRuntimeEvent::ServerMessage(
            ServerMessage::JoinAccepted(engine_net::JoinAccepted {
                connection_id: 9,
                tick_rate_hz: 60,
                join_state: engine_net::AuthoritativeJoinState::default(),
            }),
        ))
        .unwrap();
    event_tx
        .try_send(engine_net::SessionRuntimeEvent::RttUpdated { millis: 14 })
        .unwrap();
    app.world_mut()
        .insert_resource(NetworkRuntimeHandle::new(command_tx, event_rx));

    let app = app
        .run_for_frames(1)
        .expect("runtime bridge frame should run");
    let status = app.world().resource::<NetworkSessionStatus>().unwrap();
    assert!(status.connected);
    assert_eq!(status.connection_id, Some(engine_net::ConnectionId(9)));
    assert_eq!(status.phase, SessionPhase::Active);
    assert_eq!(
        app.world()
            .resource::<RoundTripMetrics>()
            .unwrap()
            .last_rtt_millis,
        Some(14)
    );
}

#[test]
fn reconnecting_event_updates_client_runtime_status() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    let (command_tx, _command_rx) = tokio::sync::mpsc::channel(16);
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(16);
    event_tx
        .try_send(engine_net::SessionRuntimeEvent::Reconnecting { attempt: 2 })
        .unwrap();
    app.world_mut()
        .insert_resource(NetworkRuntimeHandle::new(command_tx, event_rx));

    let app = app
        .run_for_frames(1)
        .expect("runtime reconnect frame should run");
    let status = app.world().resource::<NetworkSessionStatus>().unwrap();
    assert!(!status.connected);
    assert_eq!(status.reconnect_attempt, Some(2));
    assert_eq!(status.phase, SessionPhase::Handshaking);
    assert_eq!(
        app.world()
            .resource::<NetworkDiagnostics>()
            .unwrap()
            .reconnect_attempts,
        1
    );
}

#[test]
fn server_replication_emits_scene_snapshot_payloads() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
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

    let app = app
        .run_for_frames(1)
        .expect("server join frame should run")
        .run_for_ticks(1)
        .expect("server tick should run");
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    let message = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                message: ServerMessage::Snapshot(snapshot),
                ..
            }
            | OutboundServerMessage::Broadcast(ServerMessage::Snapshot(snapshot)) => Some(snapshot),
            _ => None,
        })
        .expect("server should emit an initial full snapshot");
    let snapshot: TestSnapshot =
        postcard::from_bytes(&message.payload).expect("snapshot payload should decode");
    assert_eq!(message.cursor, SnapshotCursor(1));
    assert_eq!(snapshot.context.world_scene_label, "gameplay_stub");
}

#[test]
fn client_snapshot_application_sends_ack_and_reconciles_prediction() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));
    server
        .world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    server
        .world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::JoinRequest(engine_net::JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-local".to_string(),
            ticket: "ticket-1".to_string(),
        }));
    let server = server
        .run_for_frames(1)
        .expect("server join frame should run");
    let mut server = server;
    server
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand {
            x: -0.75,
            y: 0.5,
        }));
    let server = server.run_for_ticks(1).expect("server tick should run");
    let authoritative_snapshot = server
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                message: ServerMessage::Snapshot(snapshot),
                ..
            }
            | OutboundServerMessage::Broadcast(ServerMessage::Snapshot(snapshot)) => {
                Some(snapshot.clone())
            }
            _ => None,
        })
        .expect("server should emit a snapshot");

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 }));
    let mut client = client
        .run_for_ticks(1)
        .expect("client prediction tick should run");
    assert_eq!(
        client
            .world()
            .resource::<PredictionState>()
            .unwrap()
            .pending_frames_len(),
        1
    );

    client
        .world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::Snapshot(authoritative_snapshot));
    let client = client
        .run_for_frames(1)
        .expect("client receive frame should run");

    let outbound = client.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.client_messages().iter().any(
        |message| matches!(message, ClientMessage::Ack(ack) if ack.cursor == SnapshotCursor(1))
    ));
    assert_eq!(
        client
            .world()
            .resource::<PredictionDiagnostics>()
            .unwrap()
            .corrections_applied,
        1
    );
    assert_eq!(
        client
            .world()
            .resource::<ClientSnapshotState>()
            .unwrap()
            .last_acknowledged_cursor,
        SnapshotCursor(1)
    );
    assert_eq!(
        client
            .world()
            .resource::<PredictionState>()
            .unwrap()
            .pending_frames_len(),
        0
    );
}
