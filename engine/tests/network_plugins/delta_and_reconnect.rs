// Owner: Engine Networking Tests - Delta and Connection-Scoped Replication
#[derive(Debug, Copy, Clone, Default, runen_ecs::Resource)]
struct NetworkTestFrameDelta(f32);

fn apply_network_test_frame_delta(
    frame_delta: Res<NetworkTestFrameDelta>,
    mut time: ResMut<Time>,
) {
    time.delta_seconds = frame_delta.0;
}

fn install_network_test_clock(app: &mut App) {
    app.init_resource::<NetworkTestFrameDelta>();
    app.add_systems(
        PreUpdate,
        apply_network_test_frame_delta.after(CoreSet::Time),
    );
}

fn run_network_protocol_frame(mut app: App, context: &str) -> App {
    app.world_mut()
        .resource_mut::<NetworkTestFrameDelta>()
        .expect("network test clock should be installed")
        .0 = 0.0;
    app.run_for_frames(1)
        .unwrap_or_else(|error| panic!("{context}: {error:#}"))
}

fn run_network_fixed_step(mut app: App, context: &str) -> App {
    let step_seconds = app
        .world()
        .resource::<FixedTimeConfig>()
        .expect("fixed-time config should be installed")
        .step_seconds;
    app.world_mut()
        .resource_mut::<NetworkTestFrameDelta>()
        .expect("network test clock should be installed")
        .0 = step_seconds;
    app.run_for_frames(1)
        .unwrap_or_else(|error| panic!("{context}: {error:#}"))
}

#[test]
fn server_delta_snapshot_applies_cleanly_on_client() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut server, &[(connection, ParticipantId::new(1))]);

    let server = server
        .run_for_fixed_steps(1)
        .expect("first server replication fixed step should run");
    let full_snapshot = server
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
        .expect("server should emit a full snapshot for the admitted connection");

    let mut server = server;
    enqueue_server_inbox_from(
        server.world_mut(),
        Some(connection),
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
        .run_for_fixed_steps(1)
        .expect("second server replication fixed step should run");
    assert_eq!(
        server
            .world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .acked,
        1
    );

    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    let delta_snapshot = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            OutboundServerMessage::ToConnection {
                connection: target,
                message: ServerMessage::DeltaSnapshot(snapshot),
            } if *target == connection => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a delta snapshot for the admitted connection");
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

    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::DeltaSnapshot(delta_snapshot),
    )
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
fn server_rejects_future_snapshot_ack_without_mutating_baseline() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut server, &[(connection, ParticipantId::new(1))]);

    let mut server = server
        .run_for_fixed_steps(1)
        .expect("first server replication fixed step should run");
    enqueue_server_inbox_from(
        server.world_mut(),
        Some(connection),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(99),
            last_received_tick: SimulationTick(99),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let server = server
        .run_for_frames(1)
        .expect("future ack processing frame should run");

    let diagnostics = server.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.acked, 0);
    assert_eq!(diagnostics.rejected_acks, 1);

    let replication = server.world().resource::<ServerSnapshotState>().unwrap();
    let checkpoint = replication
        .checkpoints
        .get(&connection)
        .expect("connection checkpoint should exist");
    assert_eq!(checkpoint.last_ack_cursor, SnapshotCursor::default());

    let server = server
        .run_for_fixed_steps(1)
        .expect("second server replication fixed step should run");
    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.server_messages().iter().any(|message| {
        matches!(
            message,
            OutboundServerMessage::ToConnection {
                connection: target,
                message: ServerMessage::Snapshot(snapshot),
            } if *target == connection
                && snapshot.cursor == SnapshotCursor(2)
                && snapshot.last_applied == SnapshotCursor::default()
        )
    }));
    assert!(
        !outbound.server_messages().iter().any(|message| {
            matches!(
                message,
                OutboundServerMessage::ToConnection {
                    connection: target,
                    message: ServerMessage::DeltaSnapshot(snapshot),
                } if *target == connection && snapshot.base == SnapshotCursor(99)
            )
        }),
        "rejected future ACK must not become a delta baseline"
    );
}

#[test]
fn server_tracks_lagged_input_frames_against_simulation_tick() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);
    *app.world_mut().resource_mut::<SimulationTick>().unwrap() = SimulationTick(5);

    let payload =
        TestReplicationDriver::encode_input(&[ClientCommandEnvelope::Move(MoveCommand {
            x: 0.0,
            y: 1.0,
        })])
        .expect("input payload should encode");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(4),
            payload,
        }),
    )
    .expect("server inbox enqueue should succeed");

    let app = run_network_protocol_frame(app, "server lagged input frame should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(5),
        "protocol-only frame must not advance fixed time"
    );
    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.lagged, 1);
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn server_rejects_equal_current_remote_input_as_already_simulated() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);
    *app.world_mut().resource_mut::<SimulationTick>().unwrap() = SimulationTick(5);

    let payload = TestReplicationDriver::encode_input(&[ClientCommandEnvelope::Ability(
        AbilityCommand { slot: 5 },
    )])
    .expect("input payload should encode");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(5),
            payload,
        }),
    )
    .expect("server inbox enqueue should succeed");

    let app = run_network_protocol_frame(app, "equal-current remote input frame should run");
    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.lagged, 1);
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(5)
    );
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn future_remote_input_precedes_local_input_for_same_fixed_tick() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let remote = ClientCommandEnvelope::Ability(AbilityCommand { slot: 7 });
    let local = ClientCommandEnvelope::Ability(AbilityCommand { slot: 8 });
    let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&remote))
        .expect("input payload should encode");
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(1),
            payload,
        }),
    )
    .expect("future remote input should enqueue");
    app.world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(local.clone());

    let app = run_network_fixed_step(app, "same-step remote/local input should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(1)
    );
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![remote, local]
    );
    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .lagged,
        0
    );
}

#[test]
fn server_tracks_per_connection_baselines_for_runennet_connections() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection_a = ConnectionHandle::new(1);
    let connection_b = ConnectionHandle::new(2);
    install_runennet_connections(
        &mut app,
        &[
            (connection_a, ParticipantId::new(1)),
            (connection_b, ParticipantId::new(2)),
        ],
    );

    let app = run_network_protocol_frame(app, "RunenNet projection frame should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(0),
        "session projection frame must not advance fixed time"
    );
    let mut app = run_network_fixed_step(app, "first replication fixed step should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(1)
    );

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection_a),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .expect("server inbox enqueue should succeed");
    let app = run_network_protocol_frame(app, "ack frame should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(1),
        "protocol-only ACK frame must not advance fixed time"
    );

    let app = run_network_fixed_step(app, "second replication fixed step should run");
    assert_eq!(
        *app.world().resource::<SimulationTick>().unwrap(),
        SimulationTick(2)
    );
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    assert!(outbound.server_messages().iter().any(|message| {
        matches!(
            message,
            OutboundServerMessage::ToConnection {
                connection,
                message: ServerMessage::DeltaSnapshot(snapshot),
            } if *connection == connection_a
                && snapshot.base == SnapshotCursor(1)
                && snapshot.cursor == SnapshotCursor(2)
        )
    }));
    assert!(outbound.server_messages().iter().any(|message| {
        matches!(
            message,
            OutboundServerMessage::ToConnection {
                connection,
                message: ServerMessage::Snapshot(snapshot),
            } if *connection == connection_b && snapshot.cursor == SnapshotCursor(2)
        )
    }));

    let replication = app.world().resource::<ServerSnapshotState>().unwrap();
    let checkpoint_a = replication
        .checkpoints
        .get(&connection_a)
        .expect("connection 1 checkpoint should exist");
    let checkpoint_b = replication
        .checkpoints
        .get(&connection_b)
        .expect("connection 2 checkpoint should exist");
    assert_eq!(checkpoint_a.last_ack_cursor, SnapshotCursor(1));
    assert_eq!(checkpoint_b.last_full_snapshot_cursor, SnapshotCursor(2));
}


#[test]
fn authority_input_rejects_future_tick_outside_explicit_window() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let payload = TestReplicationDriver::encode_input(&[ClientCommandEnvelope::Ability(
        AbilityCommand { slot: 9 },
    )])
    .expect("input payload should encode");
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(10),
            payload,
        }),
    )
    .expect("future input should enqueue");

    let app = run_network_protocol_frame(app, "future-window rejection should run");
    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.future_inputs, 1);
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn authority_input_duplicate_and_conflict_do_not_execute_twice() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let accepted = ClientCommandEnvelope::Ability(AbilityCommand { slot: 21 });
    let accepted_payload =
        TestReplicationDriver::encode_input(std::slice::from_ref(&accepted))
            .expect("accepted input payload should encode");
    let conflicting_payload = TestReplicationDriver::encode_input(&[
        ClientCommandEnvelope::Ability(AbilityCommand { slot: 22 }),
    ])
    .expect("conflicting input payload should encode");

    for payload in [
        accepted_payload.clone(),
        accepted_payload,
        conflicting_payload,
    ] {
        enqueue_server_inbox_from(
            app.world_mut(),
            Some(connection),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(1),
                payload,
            }),
        )
        .expect("authority input should enqueue");
        app = run_network_protocol_frame(app, "authority-input classification frame should run");
    }

    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.duplicate_inputs, 1);
    assert_eq!(diagnostics.conflicting_inputs, 1);

    let app = run_network_fixed_step(app, "accepted authority input should execute once");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![accepted]
    );
}

#[test]
fn authority_input_rejects_unbound_connection() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let admitted = ConnectionHandle::new(1);
    let unbound = ConnectionHandle::new(2);
    install_runennet_connections(&mut app, &[(admitted, ParticipantId::new(1))]);

    let payload = TestReplicationDriver::encode_input(&[
        ClientCommandEnvelope::Ability(AbilityCommand { slot: 22 }),
    ])
    .expect("input payload should encode");
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(unbound),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(1),
            payload,
        }),
    )
    .expect("unbound input should enter bounded inbox");

    let app = run_network_protocol_frame(app, "unauthorized authority input should run");
    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.unauthorized_inputs, 1);
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn authority_input_requires_explicit_session_policy() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    let mut core = test_runennet_session_core_without_authority_input();
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(&mut core, &mut projection, participant, connection);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let payload = TestReplicationDriver::encode_input(&[
        ClientCommandEnvelope::Ability(AbilityCommand { slot: 23 }),
    ])
    .expect("input payload should encode");
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(1),
            payload,
        }),
    )
    .expect("input should enter bounded server inbox");

    let error = match app.run_for_frames(1) {
        Ok(_) => panic!("remote authority input without explicit policy must fail closed"),
        Err(error) => error,
    };
    assert!(
        format!("{error:#}").contains("with_authority_input_policy"),
        "configuration failure should name the required explicit session policy: {error:#}"
    );
}

#[test]
fn authority_input_resource_limits_reject_without_execution() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let participant_limits = AuthorityInputLimits::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        8,
    )
    .unwrap();
    let aggregate_limits = AuthorityInputAggregateLimits::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    );
    let policy = AuthorityInputPolicy::new(participant_limits, aggregate_limits);

    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    let mut core =
        test_runennet_session_core_without_authority_input().with_authority_input_policy(policy);
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(&mut core, &mut projection, participant, connection);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let payload = TestReplicationDriver::encode_input(&[
        ClientCommandEnvelope::Ability(AbilityCommand { slot: 24 }),
    ])
    .expect("input payload should encode");
    assert!(payload.len() > 1);
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(1),
            payload,
        }),
    )
    .expect("resource-rejected input should enter bounded inbox");

    let app = run_network_protocol_frame(app, "authority-input resource rejection should run");
    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .input_resource_rejections,
        1
    );
    let app = run_network_fixed_step(app, "resource-rejected input must not execute");
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn retained_replacement_preserves_authority_input_evidence_and_execution() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let old_connection = ConnectionHandle::new(1);
    let new_connection = ConnectionHandle::new(2);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(old_connection, participant)]);

    let accepted = ClientCommandEnvelope::Ability(AbilityCommand { slot: 31 });
    let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&accepted))
        .expect("input payload should encode");
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(old_connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload: payload.clone(),
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "initial authority input should be accepted");

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    let duration = RecoveryDuration::new(NonZeroU64::new(4).unwrap());
    core.connection_lost(
        &mut projection,
        participant,
        old_connection,
        RetentionPolicy::RetainForRecovery { duration },
    )
    .unwrap();
    establish_runennet_negotiation(&mut core, new_connection);
    core.bind_replacement(&mut projection, participant, new_connection)
        .unwrap();
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(old_connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload: payload.clone(),
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "old replacement connection should be unauthorized");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(new_connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "replacement duplicate should be classified");

    let diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(diagnostics.unauthorized_inputs, 1);
    assert_eq!(diagnostics.duplicate_inputs, 1);

    app = run_network_fixed_step(app, "tick one should run");
    app = run_network_fixed_step(app, "tick two should execute retained authority input");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![accepted]
    );
}

#[test]
fn participant_removal_purges_pending_authority_input_before_readmission() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);
    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(connection, participant)]);

    let first = ClientCommandEnvelope::Ability(AbilityCommand { slot: 41 });
    let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&first)).unwrap();
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "first participant lifetime input should be accepted");

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    core.remove_participant(&mut projection, participant).unwrap();
    core.admit_established(&mut projection, participant, connection)
        .expect("participant should be able to begin a fresh membership lifetime");
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let replacement = ClientCommandEnvelope::Ability(AbilityCommand { slot: 42 });
    let replacement_payload =
        TestReplicationDriver::encode_input(std::slice::from_ref(&replacement)).unwrap();
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload: replacement_payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(
        app,
        "fresh membership input should not conflict with old lifetime",
    );

    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .duplicate_inputs,
        0
    );
    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .conflicting_inputs,
        0
    );

    app = run_network_fixed_step(app, "tick one should run");
    app = run_network_fixed_step(app, "tick two should execute only fresh-lifetime input");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![replacement]
    );
}

#[test]
fn authority_input_participant_retained_key_limit_rejects_second_pending_tick() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let participant_limits = AuthorityInputLimits::new(
        NonZeroUsize::new(64 * 1024).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(512 * 1024).unwrap(),
        8,
    )
    .unwrap();
    let aggregate_limits = AuthorityInputAggregateLimits::new(
        NonZeroUsize::new(16).unwrap(),
        NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
    );
    let policy = AuthorityInputPolicy::new(participant_limits, aggregate_limits);

    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    let mut core =
        test_runennet_session_core_without_authority_input().with_authority_input_policy(policy);
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(&mut core, &mut projection, participant, connection);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let first = ClientCommandEnvelope::Ability(AbilityCommand { slot: 51 });
    let second = ClientCommandEnvelope::Ability(AbilityCommand { slot: 52 });
    for (tick, command) in [(1, first.clone()), (2, second)] {
        let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&command)).unwrap();
        enqueue_server_inbox_from(
            app.world_mut(),
            Some(connection),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(tick),
                payload,
            }),
        )
        .unwrap();
        app = run_network_protocol_frame(app, "participant input limit frame should run");
    }

    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .input_resource_rejections,
        1
    );

    app = run_network_fixed_step(app, "first accepted authority input should execute");
    app = run_network_fixed_step(app, "rejected second authority input must not execute");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![first]
    );
}

#[test]
fn authority_input_aggregate_retained_key_limit_is_shared_across_participants() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let participant_limits = AuthorityInputLimits::new(
        NonZeroUsize::new(64 * 1024).unwrap(),
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(512 * 1024).unwrap(),
        8,
    )
    .unwrap();
    let aggregate_limits = AuthorityInputAggregateLimits::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
    );
    let policy = AuthorityInputPolicy::new(participant_limits, aggregate_limits);

    let first_connection = ConnectionHandle::new(1);
    let second_connection = ConnectionHandle::new(2);
    let first_participant = ParticipantId::new(1);
    let second_participant = ParticipantId::new(2);
    let mut core =
        test_runennet_session_core_without_authority_input().with_authority_input_policy(policy);
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(
        &mut core,
        &mut projection,
        first_participant,
        first_connection,
    );
    establish_runennet_connection(
        &mut core,
        &mut projection,
        second_participant,
        second_connection,
    );
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let accepted = ClientCommandEnvelope::Ability(AbilityCommand { slot: 61 });
    let rejected = ClientCommandEnvelope::Ability(AbilityCommand { slot: 62 });
    for (connection, command) in [
        (first_connection, accepted.clone()),
        (second_connection, rejected),
    ] {
        let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&command)).unwrap();
        enqueue_server_inbox_from(
            app.world_mut(),
            Some(connection),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(1),
                payload,
            }),
        )
        .unwrap();
    }

    app = run_network_protocol_frame(app, "aggregate input limit frame should run");
    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .input_resource_rejections,
        1
    );

    app = run_network_fixed_step(app, "only aggregate-admitted input should execute");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![accepted]
    );
}

#[test]
fn recovery_expiry_purges_pending_authority_input_before_new_membership() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let old_connection = ConnectionHandle::new(1);
    let new_connection = ConnectionHandle::new(2);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(old_connection, participant)]);

    let stale_lifetime = ClientCommandEnvelope::Ability(AbilityCommand { slot: 71 });
    let payload =
        TestReplicationDriver::encode_input(std::slice::from_ref(&stale_lifetime)).unwrap();
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(old_connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "pre-expiry authority input should be accepted");

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    let duration = RecoveryDuration::new(NonZeroU64::new(1).unwrap());
    core.connection_lost(
        &mut projection,
        participant,
        old_connection,
        RetentionPolicy::RetainForRecovery { duration },
    )
    .unwrap();
    core.advance_recovery_clock(runen_net::session::RecoveryTime::new(1))
        .unwrap();
    establish_runennet_negotiation(&mut core, new_connection);
    core.admit_established(&mut projection, participant, new_connection)
        .unwrap();
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    let fresh_lifetime = ClientCommandEnvelope::Ability(AbilityCommand { slot: 72 });
    let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&fresh_lifetime)).unwrap();
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(new_connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "post-expiry authority input should be fresh");

    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .conflicting_inputs,
        0
    );
    app = run_network_fixed_step(app, "tick one should run after recovery expiry");
    app = run_network_fixed_step(app, "tick two should execute only new-lifetime input");
    assert_eq!(
        app.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![fresh_lifetime]
    );
}

#[test]
fn session_close_purges_pending_authority_input_execution() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(connection, participant)]);

    let pending = ClientCommandEnvelope::Ability(AbilityCommand { slot: 81 });
    let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&pending)).unwrap();
    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::InputFrame(InputFrame {
            tick: SimulationTick(2),
            payload,
        }),
    )
    .unwrap();
    app = run_network_protocol_frame(app, "pre-close authority input should be accepted");

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    core.close(&mut projection);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    app = run_network_fixed_step(app, "tick one should run after session close");
    app = run_network_fixed_step(app, "tick two must not execute closed-session input");
    assert!(app.world().resource::<AppliedInputLog>().is_err());
}

#[test]
fn authority_input_keeps_same_tick_participants_independent() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    install_network_test_clock(&mut app);

    let first_connection = ConnectionHandle::new(1);
    let second_connection = ConnectionHandle::new(2);
    install_runennet_connections(
        &mut app,
        &[
            (first_connection, ParticipantId::new(1)),
            (second_connection, ParticipantId::new(2)),
        ],
    );

    let first = ClientCommandEnvelope::Ability(AbilityCommand { slot: 91 });
    let second = ClientCommandEnvelope::Ability(AbilityCommand { slot: 92 });
    for (connection, command) in [
        (first_connection, first.clone()),
        (second_connection, second.clone()),
    ] {
        let payload = TestReplicationDriver::encode_input(std::slice::from_ref(&command)).unwrap();
        enqueue_server_inbox_from(
            app.world_mut(),
            Some(connection),
            ClientMessage::InputFrame(InputFrame {
                tick: SimulationTick(1),
                payload,
            }),
        )
        .unwrap();
    }

    app = run_network_protocol_frame(app, "same-tick participant inputs should both be admitted");
    app = run_network_fixed_step(app, "same-tick participant inputs should both execute");

    let applied = &app.world().resource::<AppliedInputLog>().unwrap().inputs;
    assert_eq!(applied.len(), 2);
    assert!(applied.contains(&first));
    assert!(applied.contains(&second));
}

