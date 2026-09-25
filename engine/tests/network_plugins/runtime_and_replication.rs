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
    let payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("baseline snapshot should encode");
    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload,
        }),
    )
    .expect("baseline should stage");
    let app = app.run_for_frames(1).expect("baseline should activate prediction");

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
fn server_replication_prepares_snapshot_until_delivery_is_accepted() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("server replication fixed step should run");
    assert!(
        app.world()
            .resource::<NetworkOutboundQueue>()
            .unwrap()
            .server_messages()
            .is_empty(),
        "preparing authority replication must not imply transport emission"
    );

    let submissions =
        authority_replication_submissions(app.world()).expect("prepared authority submission");
    assert_eq!(submissions.len(), 1);
    let submission = submissions.into_iter().next().unwrap();
    assert_eq!(submission.connection(), connection);
    let message = match submission.message() {
        ServerMessage::Snapshot(snapshot) => snapshot.clone(),
        other => panic!("initial authority candidate must be a full snapshot, got {other:?}"),
    };
    let snapshot: TestSnapshot =
        postcard::from_bytes(&message.payload).expect("snapshot payload should decode");
    assert_eq!(message.cursor, SnapshotCursor(1));
    assert_eq!(snapshot.context.world_scene_label, "gameplay_stub");
    let prepared_diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(prepared_diagnostics.emitted_snapshots, 0);
    assert_eq!(
        prepared_diagnostics.last_snapshot_cursor, 0,
        "prepared authority submission must not advance emitted-cursor diagnostics"
    );

    let emitted = record_authority_replication_delivery_acceptance(
        app.world_mut(),
        submission.token(),
        DeliveryAcceptance::Accepted,
    )
    .expect("accepted delivery feedback should be admitted")
    .expect("accepted delivery should emit the pending snapshot");
    assert_eq!(emitted.target_cursor.get(), 1);
    let emitted_diagnostics = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(emitted_diagnostics.emitted_snapshots, 1);
    assert_eq!(emitted_diagnostics.last_snapshot_cursor, 1);
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
    let mut server = server
        .run_for_fixed_steps(1)
        .expect("server fixed step should run");
    let submission = authority_replication_submissions(server.world())
        .expect("server authority submission should project")
        .into_iter()
        .find(|submission| submission.connection() == connection)
        .expect("server should prepare a snapshot");
    let authoritative_snapshot = match submission.message() {
        ServerMessage::Snapshot(snapshot) => snapshot.clone(),
        other => panic!("initial server authority submission must be a snapshot, got {other:?}"),
    };
    let authoritative_tick = authoritative_snapshot.tick;
    record_authority_replication_delivery_acceptance(
        server.world_mut(),
        submission.token(),
        DeliveryAcceptance::Accepted,
    )
    .expect("server delivery acceptance should succeed")
    .expect("accepted server submission should become emitted");

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
        client_prediction_pending_count(client.world()),
        Some(0),
        "prediction remains inactive until an authoritative baseline is realized"
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
    assert_eq!(client_prediction_pending_count(client.world()), Some(0));
}

#[test]
fn prediction_replay_preserves_runennet_target_tick_and_updates_diagnostics() {
    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));

    let baseline_payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("baseline snapshot payload should encode");
    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload: baseline_payload,
        }),
    )
    .expect("baseline should stage");
    let mut client = client
        .run_for_frames(1)
        .expect("baseline should activate prediction");

    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 }));
    client = client
        .run_for_fixed_steps(1)
        .expect("predicted fixed step should run");
    assert_eq!(client_prediction_pending_count(client.world()), Some(1));

    let correction_payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("correction snapshot payload should encode");
    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(2),
            last_applied: SnapshotCursor(1),
            payload: correction_payload,
        }),
    )
    .expect("correction should stage");
    let client = client
        .run_for_frames(1)
        .expect("authoritative correction frame should run");

    let diagnostics = client.world().resource::<PredictionDiagnostics>().unwrap();
    assert_eq!(diagnostics.replayed, 1);
    assert_eq!(client_prediction_pending_count(client.world()), Some(1));
    assert!(
        client
            .world()
            .resource::<AppliedInputLog>()
            .unwrap()
            .ticks
            .iter()
            .all(|tick| *tick == SimulationTick(1)),
        "ordinary prediction and RunenNet replay must preserve the semantic target tick"
    );
}

#[test]
fn duplicate_current_retries_failed_replay_restoration_before_ack() {
    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));

    let baseline_payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("baseline snapshot should encode");
    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload: baseline_payload,
        }),
    )
    .expect("baseline should stage");
    let mut client = client
        .run_for_frames(1)
        .expect("baseline should activate prediction");
    clear_client_outbound(client.world_mut());

    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 19 }));
    client = client
        .run_for_fixed_steps(1)
        .expect("local predicted input should apply");
    assert_eq!(client_prediction_pending_count(client.world()), Some(1));
    clear_client_outbound(client.world_mut());

    client
        .world_mut()
        .insert_resource(RejectReplayAndNextSnapshot(true));
    let correction_payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("correction snapshot should encode");
    let correction = ServerMessage::Snapshot(Snapshot {
        tick: SimulationTick(0),
        cursor: SnapshotCursor(2),
        last_applied: SnapshotCursor(1),
        payload: correction_payload,
    });
    enqueue_client_inbox(client.world_mut(), correction.clone())
        .expect("correction should stage");
    client = client
        .run_for_frames(1)
        .expect("replay/restoration failure should remain contained by receive processing");

    assert_eq!(outbound_ack(client.world()), None);
    assert_eq!(
        client_replication_acknowledgement(client.world()),
        Some((SnapshotCursor(2), SimulationTick(0))),
        "replication commit remains authoritative despite downstream replay/restoration failure"
    );
    assert!(matches!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Invalidated {
            reason: PredictionInvalidationReason::ReplayFailure,
            ..
        })
    ));

    client
        .world_mut()
        .resource_mut::<RejectSnapshotRealization>()
        .expect("failed replay should arm the restoration rejection")
        .0 = false;
    clear_client_outbound(client.world_mut());
    enqueue_client_inbox(client.world_mut(), correction)
        .expect("duplicate-current correction should stage");
    let client = client
        .run_for_frames(1)
        .expect("duplicate current should retry authoritative restoration");

    assert_eq!(
        outbound_ack(client.world()).map(|ack| ack.cursor),
        Some(SnapshotCursor(2)),
        "ACK is allowed only after duplicate-current restores host state"
    );
    assert_eq!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Active {
            frontier: runen_net::identity::SimulationTick::new(0),
        })
    );
}

#[test]
fn client_outbox_backpressure_does_not_roll_back_admitted_prediction() {
    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((ScenePlugin, NetworkClientPlugin));
    install_backpressure_test_clock(&mut client);
    let baseline_payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("baseline should encode");
    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload: baseline_payload,
        }),
    )
    .expect("baseline should stage");
    client = run_backpressure_protocol_frame(
        client,
        "baseline should activate tracked prediction",
    );
    let expected_outbound = (0..4_096usize)
        .map(|index| client_probe((index % 251) as u8))
        .collect::<Vec<_>>();
    for message in expected_outbound.iter().cloned() {
        enqueue_client_outbox(client.world_mut(), message)
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
        client_prediction_pending_count(client.world()),
        Some(1),
        "RunenNet prediction admission remains authoritative despite queue backpressure"
    );
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![command],
        "an admitted prediction still applies locally when the current delivery submission backpressures"
    );
    let outbound = client.world().resource::<NetworkOutboundQueue>().unwrap();
    assert_eq!(
        outbound.client_messages(),
        expected_outbound.as_slice(),
        "backpressure must leave the saturated client outbox unchanged"
    );
}

#[test]
fn server_outbox_backpressure_is_orthogonal_to_authority_delivery_acceptance() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((ScenePlugin, NetworkServerPlugin));
    let expected_outbound = (0..4_096usize)
        .map(|index| {
            OutboundServerMessage::Broadcast(server_probe((index % 251) as u8))
        })
        .collect::<Vec<_>>();
    server.add_systems(
        FixedUpdate,
        saturate_server_outbox_before_replication
            .on_invoker_thread()
            .after_if_present(CoreSet::Simulation)
            .before(engine::plugins::net::NetFixedSet::Replication),
    );
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut server, &[(connection, ParticipantId::new(1))]);

    let mut server = server
        .run_for_fixed_steps(1)
        .expect("server replication fixed step should survive outbox backpressure");

    let first = authority_replication_submissions(server.world())
        .expect("pending authority submission should project")
        .into_iter()
        .find(|submission| submission.connection() == connection)
        .expect("replication should retain a pending candidate despite queue saturation");
    assert_eq!(
        server
            .world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .emitted_snapshots,
        0,
        "queue state must not become delivery evidence"
    );

    let not_emitted = record_authority_replication_delivery_acceptance(
        server.world_mut(),
        first.token(),
        DeliveryAcceptance::NotAccepted,
    )
    .expect("not-accepted delivery feedback should be valid");
    assert!(not_emitted.is_none());

    let retry = authority_replication_submissions(server.world())
        .expect("pending authority retry should project")
        .into_iter()
        .find(|submission| submission.connection() == connection)
        .expect("not-accepted candidate must remain pending for explicit retry");
    assert_eq!(retry.token(), first.token());
    assert_eq!(retry.message(), first.message());

    assert!(
        cancel_authority_replication_submission(server.world_mut(), retry.token())
            .expect("explicit pending cancellation should succeed")
    );
    assert!(
        authority_replication_submissions(server.world())
            .expect("post-cancellation projection should succeed")
            .is_empty()
    );

    let outbound = server.world().resource::<NetworkOutboundQueue>().unwrap();
    assert_eq!(
        outbound.server_messages(),
        expected_outbound.as_slice(),
        "backpressure must leave the saturated server outbox unchanged"
    );
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
        client_prediction_pending_count(host.world()),
        Some(0),
        "peer-host local authority input is not client prediction"
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


fn client_app_with_prediction_policy(policy: ClientPredictionPolicy) -> App {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.init_resource::<PlayerCommandBuffer>();
    app.add_plugin(SimulationPlugin);
    app.add_plugin(
        NetPlugin::<TestReplicationDriver>::new(NetRole::Client).with_config(
            NetPluginConfig::default()
                .with_client_replication_policy(test_client_replication_policy())
                .with_client_prediction_policy(policy),
        ),
    );
    app
}

fn activate_client_prediction_baseline(mut app: App) -> App {
    let payload = TestReplicationDriver::encode_snapshot(&TestSnapshot::default())
        .expect("baseline snapshot should encode");
    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(0),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload,
        }),
    )
    .expect("baseline should stage");
    let mut app = app
        .run_for_frames(1)
        .expect("baseline should activate client prediction");
    clear_client_outbound(app.world_mut());
    app
}

#[test]
fn client_prediction_policy_requires_explicit_client_replication_policy() {
    let result = std::panic::catch_unwind(|| {
        let mut app = App::headless();
        app.add_plugins(default_plugins());
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(SimulationPlugin);
        app.add_plugin(
            NetPlugin::<TestReplicationDriver>::new(NetRole::Client).with_config(
                NetPluginConfig::default()
                    .with_client_prediction_policy(test_client_prediction_policy()),
            ),
        );
    });

    assert!(
        result.is_err(),
        "tracked prediction without an explicit client replication lineage must fail setup"
    );
}

#[test]
fn client_prediction_retains_exact_encoded_staged_batch_and_applies_once() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    let command = ClientCommandEnvelope::Ability(AbilityCommand { slot: 11 });
    let expected_payload = TestReplicationDriver::encode_input(std::slice::from_ref(&command))
        .expect("prediction batch should encode");
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(command.clone());

    let client = client
        .run_for_fixed_steps(1)
        .expect("new prediction batch should apply");

    assert_eq!(client_prediction_pending_count(client.world()), Some(1));
    assert_eq!(
        client_prediction_pending_bytes(client.world()),
        Some(expected_payload.len()),
        "RunenNet prediction accounting must use the exact retained encoded batch length"
    );
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![command]
    );
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().ticks,
        vec![SimulationTick(1)]
    );
    assert!(client
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .client_messages()
        .iter()
        .any(|message| matches!(
            message,
            ClientMessage::InputFrame(frame)
                if frame.tick == SimulationTick(1) && frame.payload == expected_payload
        )));
}

#[test]
fn client_prediction_duplicate_and_conflicting_same_tick_batches_are_classified_before_mutation() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    let first = ClientCommandEnvelope::Ability(AbilityCommand { slot: 21 });
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(first.clone());
    client = client
        .run_for_fixed_steps(1)
        .expect("first predicted batch should apply");
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![first.clone()]
    );

    *client.world_mut().resource_mut::<SimulationTick>().unwrap() = SimulationTick(0);
    clear_client_outbound(client.world_mut());
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(first.clone());
    client = client
        .run_for_fixed_steps(1)
        .expect("duplicate predicted batch should classify");
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![first.clone()],
        "duplicate same-tick prediction must not be applied twice"
    );
    assert!(client
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .client_messages()
        .iter()
        .any(|message| matches!(message, ClientMessage::InputFrame(_))),
        "duplicate batch remains eligible for delivery retry"
    );

    *client.world_mut().resource_mut::<SimulationTick>().unwrap() = SimulationTick(0);
    clear_client_outbound(client.world_mut());
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 22 }));
    let client = client
        .run_for_fixed_steps(1)
        .expect("conflicting same-tick prediction should classify");
    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs,
        vec![first],
        "conflicting same-tick content must not mutate speculative gameplay"
    );
    assert!(client
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .client_messages()
        .iter()
        .all(|message| !matches!(message, ClientMessage::InputFrame(_))),
        "conflicting same-tick content must not be submitted"
    );
}

#[test]
fn client_prediction_resource_rejection_can_send_without_speculative_mutation() {
    let policy = ClientPredictionPolicy::new(PredictionLimits::new(
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        8,
    ));
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(policy));
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 31 }));

    let client = client
        .run_for_fixed_steps(1)
        .expect("prediction resource rejection should fail closed");

    assert_eq!(client_prediction_pending_count(client.world()), Some(0));
    assert!(client.world().resource::<AppliedInputLog>().is_err());
    assert!(client
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .client_messages()
        .iter()
        .any(|message| matches!(message, ClientMessage::InputFrame(_))),
        "delivery remains orthogonal when product policy permits an unpredicted batch"
    );
}

#[test]
fn client_prediction_uses_bounded_staging_before_forming_prediction_batch() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    let commands = (0..4_097usize)
        .map(|index| ClientCommandEnvelope::Ability(AbilityCommand {
            slot: (index % 251) as u8,
        }))
        .collect::<Vec<_>>();
    let expected_payload = TestReplicationDriver::encode_input(&commands[..4_096])
        .expect("bounded accepted batch should encode");
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .commands
        .extend(commands);

    let client = client
        .run_for_fixed_steps(1)
        .expect("bounded prediction staging should reject only excess local input");

    assert_eq!(
        client.world().resource::<AppliedInputLog>().unwrap().inputs.len(),
        4_096
    );
    assert_eq!(
        client_prediction_pending_bytes(client.world()),
        Some(expected_payload.len())
    );
    assert!(client
        .world()
        .resource::<NetworkOutboundQueue>()
        .unwrap()
        .client_messages()
        .iter()
        .any(|message| matches!(
            message,
            ClientMessage::InputFrame(frame) if frame.payload == expected_payload
        )));
}

#[test]
fn client_prediction_recovery_invalidates_pre_recovery_pending_continuity() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 41 }));
    client = client
        .run_for_fixed_steps(1)
        .expect("prediction should become pending");
    assert_eq!(client_prediction_pending_count(client.world()), Some(1));

    enqueue_client_inbox(
        client.world_mut(),
        ServerMessage::DeltaSnapshot(DeltaSnapshot {
            tick: SimulationTick(2),
            base: SnapshotCursor(99),
            cursor: SnapshotCursor(100),
            payload: TestReplicationDriver::encode_delta(&TestDelta { changed: false })
                .expect("delta should encode"),
        }),
    )
    .expect("missing-base delta should stage");
    let client = client
        .run_for_frames(1)
        .expect("missing-base recovery should be observed by prediction");

    assert_eq!(client_prediction_pending_count(client.world()), Some(0));
    assert!(matches!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Invalidated {
            reason: PredictionInvalidationReason::ReplicationRecovery(
                ClientRecoveryReason::MissingBase
            ),
            ..
        })
    ));
}

#[test]
fn client_prediction_local_application_failure_restores_authoritative_host_before_returning_error() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    client
        .world_mut()
        .insert_resource(RejectNextInputApplication(true));
    client
        .world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 51 }));

    let error = match client.run_for_fixed_steps(1) {
        Ok(_) => panic!("local predicted application failure should surface"),
        Err(error) => error,
    };
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("apply newly admitted local prediction batch"),
        "successful authoritative restoration must return the original local-application failure: {rendered}"
    );
    assert!(
        !rendered.contains("restore authoritative host state"),
        "restoration itself must have succeeded before the original failure is surfaced: {rendered}"
    );
}

#[test]
fn client_prediction_lifecycle_hooks_project_runennet_invalidation_and_termination() {
    let mut client = activate_client_prediction_baseline(client_app_with_prediction_policy(
        test_client_prediction_policy(),
    ));
    client_prediction_connection_lost(client.world_mut()).expect("connection loss should project");
    assert!(matches!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Invalidated {
            reason: PredictionInvalidationReason::ConnectionLoss,
            ..
        })
    ));

    client_prediction_participant_membership_ended(client.world_mut())
        .expect("participant end should project");
    assert!(matches!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Invalidated {
            reason: PredictionInvalidationReason::ParticipantMembershipEnded,
            ..
        })
    ));

    client_prediction_session_closed(client.world_mut()).expect("session close should project");
    assert!(matches!(
        client_prediction_state(client.world()),
        Some(RunenNetPredictionState::Invalidated {
            reason: PredictionInvalidationReason::SessionClosed,
            ..
        })
    ));
}
