// Owner: Engine Networking Tests - RunenNet Authority Replication Cutover

fn install_runennet_connections_with_core(
    app: &mut App,
    mut core: RunenNetSessionCore,
    bindings: &[(ConnectionHandle, ParticipantId)],
) {
    let mut projection = RunenNetSessionProjection::default();
    for (connection, participant) in bindings.iter().copied() {
        establish_runennet_connection(&mut core, &mut projection, participant, connection);
    }
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());
}

fn tiny_authority_replication_policy() -> AuthorityReplicationPolicy {
    let one = NonZeroUsize::new(1).unwrap();
    AuthorityReplicationPolicy::new(
        AuthorityAggregateLimits::new(one, one, one, one, one),
        ReplicationRetentionLimits::new(one, one, one, one, one)
            .expect("one-byte authority retention limits are structurally valid"),
    )
}

fn eviction_authority_replication_policy() -> AuthorityReplicationPolicy {
    let state_image =
        NonZeroUsize::new(64 * 1024).expect("test state-image limit must be non-zero");
    let retention = ReplicationRetentionLimits::new(
        state_image,
        NonZeroUsize::new(2).expect("test retained-image limit must be non-zero"),
        NonZeroUsize::new(128 * 1024).expect("test retained-byte limit must be non-zero"),
        NonZeroUsize::new(128 * 1024).expect("test candidate-byte limit must be non-zero"),
        NonZeroUsize::new(8).expect("test emission-evidence limit must be non-zero"),
    )
    .expect("eviction-test authority retention limits must be valid");
    let aggregate = AuthorityAggregateLimits::new(
        NonZeroUsize::new(4).expect("test lineage limit must be non-zero"),
        NonZeroUsize::new(512 * 1024).expect("test state-byte limit must be non-zero"),
        NonZeroUsize::new(8).expect("test retained-image aggregate must be non-zero"),
        NonZeroUsize::new(512 * 1024).expect("test retained-byte aggregate must be non-zero"),
        NonZeroUsize::new(32).expect("test evidence aggregate must be non-zero"),
    );
    AuthorityReplicationPolicy::new(aggregate, retention)
}

#[test]
fn authority_replication_requires_explicit_policy_when_work_exists() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    let core = test_runennet_session_core_without_authority_input()
        .with_authority_input_policy(test_authority_input_policy());
    install_runennet_connections_with_core(
        &mut app,
        core,
        &[(connection, ParticipantId::new(1))],
    );

    let error = match app.run_for_fixed_steps(1) {
        Ok(_) => panic!("active authority replication without explicit policy must fail"),
        Err(error) => error,
    };
    assert!(
        format!("{error:#}").contains("with_authority_replication_policy"),
        "configuration failure should identify the missing explicit policy: {error:#}"
    );
}

#[test]
fn stale_authority_delivery_token_cannot_finalize_new_candidate() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("initial authority candidate should prepare");
    let first = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("first candidate should exist");
    assert!(
        cancel_authority_replication_submission(app.world_mut(), first.token())
            .expect("first candidate cancellation should succeed")
    );

    app = app
        .run_for_fixed_steps(1)
        .expect("replacement candidate should prepare");
    let second = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("second candidate should exist");
    assert_ne!(first.token(), second.token());

    let error = record_authority_replication_delivery_acceptance(
        app.world_mut(),
        first.token(),
        DeliveryAcceptance::Accepted,
    )
    .expect_err("stale feedback token must not finalize a newer candidate");
    assert!(format!("{error:#}").contains("stale authority replication submission token"));

    assert_eq!(
        authority_replication_submissions(app.world())
            .unwrap()
            .into_iter()
            .next()
            .expect("new candidate must remain pending")
            .token(),
        second.token()
    );
}

#[test]
fn authority_ack_requires_live_authorized_connection() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    install_runennet_connections(&mut app, &[(connection, ParticipantId::new(1))]);

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("initial authority candidate should prepare");
    let submission = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("initial candidate should exist");
    record_authority_replication_delivery_acceptance(
        app.world_mut(),
        submission.token(),
        DeliveryAcceptance::Accepted,
    )
    .expect("delivery acceptance should succeed")
    .expect("snapshot should become emitted");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(ConnectionHandle::new(99)),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .expect("unrelated ACK should stage");
    app = app
        .run_for_frames(1)
        .expect("unrelated ACK should be rejected without failing the frame");
    assert_eq!(
        app.world()
            .resource::<ReplicationDiagnostics>()
            .unwrap()
            .rejected_acks,
        1
    );

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .expect("authorized ACK should stage");
    let app = app
        .run_for_frames(1)
        .expect("authorized ACK should process");
    assert_eq!(
        app.world().resource::<ReplicationDiagnostics>().unwrap().acked,
        1
    );
}

#[test]
fn retained_loss_cancels_pending_and_replacement_requires_full_recovery() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let old_connection = ConnectionHandle::new(1);
    let new_connection = ConnectionHandle::new(2);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(old_connection, participant)]);

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("initial authority candidate should prepare");
    let initial = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("initial full candidate should exist");
    record_authority_replication_delivery_acceptance(
        app.world_mut(),
        initial.token(),
        DeliveryAcceptance::Accepted,
    )
    .unwrap()
    .expect("initial full should become emitted");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(old_connection),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .unwrap();
    app = app.run_for_frames(1).expect("baseline ACK should process");
    app = app
        .run_for_fixed_steps(1)
        .expect("delta candidate should prepare");
    assert!(matches!(
        authority_replication_submissions(app.world())
            .unwrap()
            .into_iter()
            .next()
            .expect("pending delta should exist")
            .message(),
        ServerMessage::DeltaSnapshot(_)
    ));

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    let duration = RecoveryDuration::new(NonZeroU64::new(8).unwrap());
    core.connection_lost(
        &mut projection,
        participant,
        old_connection,
        RetentionPolicy::RetainForRecovery { duration },
    )
    .expect("retained connection loss should succeed");
    establish_runennet_negotiation(&mut core, new_connection);
    core.bind_replacement(&mut projection, participant, new_connection)
        .expect("replacement binding should succeed");
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());

    assert!(
        authority_replication_submissions(app.world())
            .unwrap()
            .is_empty(),
        "lost-route pending candidate must not survive replacement"
    );

    app = app
        .run_for_fixed_steps(1)
        .expect("replacement recovery candidate should prepare");
    let replacement = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .find(|submission| submission.connection() == new_connection)
        .expect("replacement connection should receive a recovery candidate");
    assert!(
        matches!(replacement.message(), ServerMessage::Snapshot(_)),
        "connection replacement must force full recovery"
    );
}

#[test]
fn authority_replication_limits_account_encoded_snapshot_bytes() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    let core = test_runennet_session_core_without_authority_input()
        .with_authority_input_policy(test_authority_input_policy())
        .with_authority_replication_policy(tiny_authority_replication_policy());
    install_runennet_connections_with_core(
        &mut app,
        core,
        &[(connection, ParticipantId::new(1))],
    );

    let error = match app.run_for_fixed_steps(1) {
        Ok(_) => panic!("encoded snapshot larger than the explicit byte budget must be rejected"),
        Err(error) => error,
    };
    let text = format!("{error:#}");
    assert!(
        text.contains("CandidateTooLarge") || text.contains("AggregateResourceLimitExceeded"),
        "resource rejection should come from RunenNet exact byte accounting: {text}"
    );
}

#[test]
fn evicted_emitted_cursor_ack_forces_full_recovery() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    let core = test_runennet_session_core_without_authority_input()
        .with_authority_input_policy(test_authority_input_policy())
        .with_authority_replication_policy(eviction_authority_replication_policy());
    install_runennet_connections_with_core(
        &mut app,
        core,
        &[(connection, ParticipantId::new(1))],
    );

    let mut app = app
        .run_for_fixed_steps(1)
        .expect("initial full should prepare");
    let first = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("initial full should exist");
    record_authority_replication_delivery_acceptance(
        app.world_mut(),
        first.token(),
        DeliveryAcceptance::Accepted,
    )
    .unwrap()
    .expect("initial full should emit");

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(1),
            last_received_tick: SimulationTick(1),
        }),
    )
    .unwrap();
    app = app.run_for_frames(1).expect("initial ACK should process");

    let mut emitted = Vec::new();
    for _ in 0..3 {
        app = app
            .run_for_fixed_steps(1)
            .expect("delta candidate should prepare");
        let submission = authority_replication_submissions(app.world())
            .unwrap()
            .into_iter()
            .next()
            .expect("delta candidate should exist");
        let cursor = match submission.message() {
            ServerMessage::DeltaSnapshot(snapshot) => snapshot.cursor,
            other => panic!("expected delta while cursor 1 remains confirmed, got {other:?}"),
        };
        record_authority_replication_delivery_acceptance(
            app.world_mut(),
            submission.token(),
            DeliveryAcceptance::Accepted,
        )
        .unwrap()
        .expect("delta should emit");
        emitted.push(cursor);
    }
    assert_eq!(
        emitted,
        vec![SnapshotCursor(2), SnapshotCursor(3), SnapshotCursor(4)]
    );

    enqueue_server_inbox_from(
        app.world_mut(),
        Some(connection),
        ClientMessage::Ack(Ack {
            cursor: SnapshotCursor(3),
            last_received_tick: SimulationTick(3),
        }),
    )
    .unwrap();
    app = app
        .run_for_frames(1)
        .expect("ACK of evicted emitted cursor should classify");
    assert_eq!(
        app.world().resource::<ReplicationDiagnostics>().unwrap().acked,
        2,
        "emission evidence keeps the ACK classifiable even after retained image eviction"
    );

    app = app
        .run_for_fixed_steps(1)
        .expect("full recovery candidate should prepare");
    let recovery = authority_replication_submissions(app.world())
        .unwrap()
        .into_iter()
        .next()
        .expect("recovery candidate should exist");
    assert!(
        matches!(recovery.message(), ServerMessage::Snapshot(_)),
        "ACK of an emitted cursor whose retained image was evicted must force full recovery"
    );
}

#[test]
fn terminal_membership_expiry_and_session_close_remove_authority_continuity() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugins((ScenePlugin, NetworkServerPlugin));
    let connection = ConnectionHandle::new(1);
    let participant = ParticipantId::new(1);
    install_runennet_connections(&mut app, &[(connection, participant)]);

    app = app
        .run_for_fixed_steps(1)
        .expect("authority candidate should prepare");
    assert_eq!(authority_replication_submissions(app.world()).unwrap().len(), 1);

    let mut core = app
        .world_mut()
        .remove_resource::<RunenNetSessionCore>()
        .unwrap();
    let mut projection = app
        .world_mut()
        .remove_resource::<RunenNetSessionProjection>()
        .unwrap();
    core.remove_participant(&mut projection, participant)
        .expect("terminal membership removal should succeed");
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());
    assert!(
        authority_replication_submissions(app.world()).unwrap().is_empty(),
        "terminal participant removal must remove authority lineage and pending continuity"
    );

    let mut core = test_runennet_session_core();
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(
        &mut core,
        &mut projection,
        participant,
        ConnectionHandle::new(2),
    );
    let duration = RecoveryDuration::new(NonZeroU64::new(1).unwrap());
    core.connection_lost(
        &mut projection,
        participant,
        ConnectionHandle::new(2),
        RetentionPolicy::RetainForRecovery { duration },
    )
    .expect("retained loss should succeed");
    let expired = core
        .advance_recovery_clock(runen_net::session::RecoveryTime::new(1))
        .expect("recovery clock should expire retained membership");
    assert_eq!(expired, vec![participant]);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());
    assert!(authority_replication_submissions(app.world()).unwrap().is_empty());

    let mut core = test_runennet_session_core();
    let mut projection = RunenNetSessionProjection::default();
    establish_runennet_connection(
        &mut core,
        &mut projection,
        ParticipantId::new(3),
        ConnectionHandle::new(3),
    );
    core.close(&mut projection);
    app.world_mut().insert_resource(core);
    app.world_mut().insert_resource(projection);
    sync_runennet_session_projection(app.world_mut());
    assert!(
        authority_replication_submissions(app.world()).unwrap().is_empty(),
        "session close must clear all authority replication continuity"
    );
}
