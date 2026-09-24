// Owner: Engine Networking Tests - Runtime and Replication
#[derive(Debug, Copy, Clone, Default, runen_ecs::Resource)]
struct BackpressureFrameDelta(f32);

fn apply_backpressure_frame_delta(
    frame_delta: Res<BackpressureFrameDelta>,
    mut time: ResMut<Time>,
) {
    time.delta_seconds = frame_delta.0;
}

fn install_backpressure_test_clock(app: &mut App) {
    app.init_resource::<BackpressureFrameDelta>();
    app.add_systems(
        PreUpdate,
        apply_backpressure_frame_delta.after(CoreSet::Time),
    );
}

fn run_backpressure_protocol_frame(mut app: App, context: &str) -> App {
    app.world_mut()
        .resource_mut::<BackpressureFrameDelta>()
        .expect("backpressure test clock should be installed")
        .0 = 0.0;
    app.run_for_frames(1)
        .unwrap_or_else(|error| panic!("{context}: {error:#}"))
}

fn run_backpressure_fixed_step(mut app: App, context: &str) -> App {
    let step_seconds = app
        .world()
        .resource::<FixedTimeConfig>()
        .expect("fixed-time config should be installed")
        .step_seconds;
    app.world_mut()
        .resource_mut::<BackpressureFrameDelta>()
        .expect("backpressure test clock should be installed")
        .0 = step_seconds;
    app.run_for_frames(1)
        .unwrap_or_else(|error| panic!("{context}: {error:#}"))
}

fn saturate_server_outbox_before_replication(mut world: WorldMut) {
    while server_outbox_len(&world) < 4_096 {
        let index = server_outbox_len(&world);
        enqueue_server_outbox_broadcast(&mut world, server_probe((index % 251) as u8))
            .expect("server outbox should fill through its configured capacity");
    }
    assert_eq!(
        server_outbox_len(&world),
        4_096,
        "server outbox must be saturated at the replication boundary"
    );
}

fn produce_simulation_input(mut commands: ResMut<PlayerCommandBuffer>) {
    commands.push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 73 }));
}

#[test]
fn prediction_waits_for_later_registered_simulation_input() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugin(NetworkClientPlugin);
    app.add_systems(FixedUpdate, produce_simulation_input.in_set(CoreSet::Simulation));

    let app = app
        .run_for_fixed_steps(1)
        .expect("prediction should run after the present simulation producer");

    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![ClientCommandEnvelope::Ability(AbilityCommand { slot: 73 })],
        "present Simulation ordering must apply input during the same fixed step"
    );
}

#[test]
fn server_replication_emits_scene_snapshot_payloads_for_runennet_connection() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let app = app
        .run_for_fixed_steps(1)
        .expect("server replication fixed step should run");
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
    let server = server
        .run_for_fixed_steps(1)
        .expect("server fixed step should run");
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
    let authoritative_tick = authoritative_snapshot.tick;

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 }));
    let mut client = client
        .run_for_fixed_steps(1)
        .expect("client prediction fixed step should run");
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
        client_replication_acknowledgement(client.world()),
        Some((SnapshotCursor(1), authoritative_tick))
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
        .run_for_fixed_steps(1)
        .expect("first prediction fixed step should run");

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

#[test]
fn client_outbox_backpressure_does_not_record_unsent_prediction_frame() {
    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));
    install_backpressure_test_clock(&mut client);
    for index in 0..4_096usize {
        enqueue_client_outbox(client.world_mut(), client_probe((index % 251) as u8))
            .expect("client outbox should fill through its configured capacity");
    }

    let command = ClientCommandEnvelope::Ability(AbilityCommand { slot: 41 });
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(command.clone());

    let client = run_backpressure_fixed_step(
        client,
        "client prediction fixed step should survive outbox backpressure",
    );

    assert_eq!(
        client
            .world()
            .resource::<PredictionState>()
            .unwrap()
            .pending_frames_len(),
        0,
        "a frame rejected by the client outbox must not enter prediction replay history"
    );
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![command],
        "staging-accepted local input still applies locally"
    );
    let outbound = client.world().resource::<NetworkOutboundQueue>().unwrap();
    assert_eq!(outbound.client_messages().len(), 4_096);
    assert!(
        outbound
            .client_messages()
            .iter()
            .all(|message| !matches!(message, ClientMessage::InputFrame(_)))
    );
}

#[test]
fn server_outbox_backpressure_does_not_mark_rejected_snapshot_as_sent() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));
    server.add_systems(
        FixedUpdate,
        saturate_server_outbox_before_replication
            .on_invoker_thread()
            .after_if_present(CoreSet::Simulation)
            .before(engine::plugins::net::NetFixedSet::Replication),
    );
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut server, &[(connection, ParticipantId::new(1))]);

    let server = server
        .run_for_fixed_steps(1)
        .expect("server replication fixed step should survive outbox backpressure");

    let state = server.world().resource::<ServerSnapshotState>().unwrap();
    let checkpoint = state
        .checkpoints
        .get(&connection)
        .expect("replication should establish connection checkpoint state");
    assert_eq!(checkpoint.last_sent_cursor, SnapshotCursor::default());
    assert!(checkpoint.sent_cursors.is_empty());
    assert!(checkpoint.needs_full_resync);

    let streaming = server
        .world()
        .resource::<engine::plugins::net::NetStreamingStateResource>()
        .unwrap();
    let streaming_state = streaming
        .per_connection
        .get(&connection)
        .expect("streaming state should exist for admitted connection");
    assert_eq!(streaming_state.last_sent_cursor.0, 0);
    assert!(streaming_state.pending_cursor_markers.is_empty());
    assert!(streaming_state.needs_full_resync);

    let diagnostics = server.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.last_snapshot_cursor, 1);
    assert_eq!(diagnostics.emitted_snapshots, 0);

    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    assert_eq!(outbound.server_messages().len(), 4_096);
    assert!(outbound.server_messages().iter().all(
        |message| matches!(message, OutboundServerMessage::Broadcast(ServerMessage::TypedPayload(_)))
    ));
}

#[test]
fn remote_authority_input_does_not_consume_local_prediction_staging() {
    let mut host = App::headless();
    host.add_plugins(default_plugins());
    host.add_plugins((ScenePlugin, NetworkHostPlugin));
    install_backpressure_test_clock(&mut host);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut host, &[(connection, ParticipantId::new(1))]);

    let remote_inputs = (0..4_096usize)
        .map(|index| {
            ClientCommandEnvelope::Ability(AbilityCommand {
                slot: (index % 251) as u8,
            })
        })
        .collect::<Vec<_>>();
    let payload = TestReplicationDriver::encode_input(&remote_inputs)
        .expect("remote authority-input payload should encode");
    enqueue_server_inbox_from(
        host.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .expect("future remote authority input should enqueue");

    let mut host = run_backpressure_protocol_frame(
        host,
        "remote authority input should be retained outside local prediction staging",
    );
    assert_eq!(
        *host.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(0)
    );

    let local = ClientCommandEnvelope::Ability(AbilityCommand { slot: 252 });
    host.world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(local.clone());
    let host = run_backpressure_fixed_step(
        host,
        "local input should remain independent of retained remote authority input",
    );

    assert_eq!(
        *host.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(1)
    );
    assert_eq!(
        host.world()
            .resource::<PredictionState>()
            .unwrap()
            .pending_frames_len(),
        1
    );
    assert_eq!(
        host.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![local]
    );
    let outbound = host.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(
        outbound
            .client_messages()
            .iter()
            .any(|message| matches!(message, ClientMessage::InputFrame(_)))
    );
}
