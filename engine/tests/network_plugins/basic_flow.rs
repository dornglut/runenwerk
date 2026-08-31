// Owner: Engine Networking Tests - Basic Flow
#[test]
fn network_client_plugin_drains_server_messages_and_flushes_client_messages() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::TypedPayload(TypedPayloadMessage::new(
            "test/server",
            "ServerProbe",
            1,
            vec![1],
        )),
    )
    .expect("client inbox enqueue should succeed");
    enqueue_client_outbox(
        app.world_mut(),
        ClientMessage::TypedPayload(TypedPayloadMessage::new(
            "test/client",
            "ClientProbe",
            1,
            vec![2],
        )),
    )
    .expect("client outbox enqueue should succeed");

    let app = app
        .run_for_frames(1)
        .expect("client network frame should run");

    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.processed_server_messages_last_frame, 1);
    assert_eq!(diagnostics.flushed_client_messages_last_frame, 1);
    assert_eq!(diagnostics.flush_count, 1);
    assert!(client_inbox_is_empty(app.world()));
    assert_eq!(client_outbox_len(app.world()), 0);
}

#[test]
fn network_server_plugin_drains_client_messages_and_flushes_server_messages() {
    let mut app = App::headless();
    app.add_plugin(NetworkServerPlugin);
    enqueue_server_inbox(
        app.world_mut(),
        ClientMessage::TypedPayload(TypedPayloadMessage::new(
            "test/client",
            "ClientProbe",
            1,
            vec![3],
        )),
    )
    .expect("server inbox enqueue should succeed");
    enqueue_server_outbox_broadcast(
        app.world_mut(),
        ServerMessage::TypedPayload(TypedPayloadMessage::new(
            "test/server",
            "ServerProbe",
            1,
            vec![4],
        )),
    )
    .expect("server outbox enqueue should succeed");

    let app = app
        .run_for_frames(1)
        .expect("server network frame should run");

    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.processed_client_messages_last_frame, 1);
    assert_eq!(diagnostics.flushed_server_messages_last_frame, 1);
    assert_eq!(diagnostics.flush_count, 1);
    assert!(server_inbox_is_empty(app.world()));
    assert_eq!(server_outbox_len(app.world()), 0);
}

#[test]
fn replication_and_prediction_plugins_run_on_fixed_update() {
    let mut app = App::headless();
    app.add_plugin(NetworkServerPlugin);
    app.world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 2 }));

    let app = app.run_for_ticks(2).expect("fixed ticks should run");

    let replication = app.world().resource::<ReplicationDiagnostics>().unwrap();
    assert_eq!(replication.fixed_steps_observed, 2);
    assert_eq!(replication.last_snapshot_cursor, 2);
    assert_eq!(app.world().resource::<SnapshotCursor>().unwrap().0, 2);

    let prediction = app.world().resource::<PredictionDiagnostics>().unwrap();
    assert_eq!(prediction.fixed_steps_observed, 2);
    assert_eq!(prediction.commands_applied, 1);
    assert!(
        app.world()
            .resource::<PlayerCommandBuffer>()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn runennet_admission_drives_engine_session_projection_and_owner_routing() {
    let mut app = App::headless();
    app.add_plugin(NetworkServerPlugin);
    let connection = ConnectionHandle::new(7);
    let participant = ParticipantId::new(3);

    install_runennet_connections(&mut app, &[(connection, participant)]);

    let projection = app.world().resource::<RunenNetSessionProjection>().unwrap();
    assert_eq!(
        projection.participant_for_connection(connection),
        Some(participant)
    );
    let status = app.world().resource::<NetworkSessionStatus>().unwrap();
    assert!(status.connected);
    assert_eq!(status.active_connection_count, 1);
    let routing = app.world().resource::<NetworkOwnerRouting>().unwrap();
    assert!(routing.by_connection.contains_key(&connection));
    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.accepted_connections, 1);
}

#[test]
fn reconnect_attempt_is_host_policy_not_session_authority() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);

    record_reconnect_attempt(app.world_mut(), 2);

    let status = app.world().resource::<NetworkSessionStatus>().unwrap();
    assert!(!status.connected);
    assert_eq!(status.active_connection_count, 0);
    assert_eq!(status.reconnect_attempt, Some(2));
    let projection = app.world().resource::<RunenNetSessionProjection>().unwrap();
    assert_eq!(projection.active_connection_count(), 0);
    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.reconnect_attempts, 1);
}

#[test]
fn host_plugin_composes_client_and_server_runtime_roles() {
    let mut app = App::headless();
    app.add_plugin(NetworkHostPlugin);

    let profile = app.world().resource::<SimulationProfileConfig>().unwrap();
    assert_eq!(profile.authority, AuthorityRole::Peer);
    assert_eq!(profile.profile, SimulationProfile::DedicatedAuthority);

    assert!(app.world().resource::<NetworkClientInbox>().is_ok());
    assert!(app.world().resource::<NetworkClientOutbox>().is_ok());
    assert!(app.world().resource::<NetworkServerInbox>().is_ok());
    assert!(app.world().resource::<NetworkServerOutbox>().is_ok());
}
