// Owner: Engine Networking Tests - Runtime and Replication
#[test]
fn server_replication_emits_scene_snapshot_payloads_for_runennet_connection() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let app = app
        .run_for_ticks(1)
        .expect("server replication tick should run");
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    let message = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                connection: target,
                message: ServerMessage::Snapshot(snapshot),
            } if *target == connection => Some(snapshot),
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
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut server, &[(connection, ParticipantId::new(1))]);
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
                connection: target,
                message: ServerMessage::Snapshot(snapshot),
            } if *target == connection => Some(snapshot.clone()),
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

    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(authoritative_snapshot),
    )
    .expect("client inbox enqueue should succeed");
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
            .corrected,
        1
    );
    assert_eq!(
        client
            .world()
            .resource::<PredictionDiagnostics>()
            .unwrap()
            .replayed,
        0
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

#[test]
fn prediction_replay_updates_prediction_diagnostics_counter() {
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
        .expect("first prediction tick should run");

    let payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("snapshot payload encoding should succeed");
    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            entity_ids: Vec::new(),
            payload,
        }),
    )
    .expect("client inbox enqueue should succeed");
    let client = client
        .run_for_frames(1)
        .expect("authoritative snapshot frame should run");

    let diagnostics = client.world().resource::<PredictionDiagnostics>().unwrap();
    assert_eq!(diagnostics.corrected, 1);
    assert_eq!(diagnostics.replayed, 1);
}
