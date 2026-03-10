// Owner: Engine Networking Tests - Basic Flow
#[test]
fn network_client_plugin_drains_server_messages_and_flushes_client_messages() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    app.world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    app.world_mut()
        .resource_mut::<NetworkClientOutbox>()
        .unwrap()
        .push(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));

    let app = app
        .run_for_frames(1)
        .expect("client network frame should run");

    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.processed_server_messages_last_frame, 1);
    assert_eq!(diagnostics.flushed_client_messages_last_frame, 1);
    assert_eq!(diagnostics.flush_count, 1);
    assert!(
        app.world()
            .resource::<NetworkClientInbox>()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        app.world().resource::<NetworkClientOutbox>().unwrap().len(),
        0
    );
}

#[test]
fn network_server_plugin_drains_client_messages_and_flushes_server_messages() {
    let mut app = App::headless();
    app.add_plugin(NetworkServerPlugin);
    app.world_mut()
        .resource_mut::<NetworkServerInbox>()
        .unwrap()
        .push(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    app.world_mut()
        .resource_mut::<NetworkServerOutbox>()
        .unwrap()
        .push(ServerMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));

    let app = app
        .run_for_frames(1)
        .expect("server network frame should run");

    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.processed_client_messages_last_frame, 1);
    assert!(diagnostics.flushed_server_messages_last_frame >= 2);
    assert_eq!(diagnostics.flush_count, 1);
    assert!(
        app.world()
            .resource::<NetworkServerInbox>()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        app.world().resource::<NetworkServerOutbox>().unwrap().len(),
        0
    );
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
fn client_bootstrap_session_queues_join_handshake() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    let messages = {
        let mut session = app
            .world_mut()
            .resource_mut::<ClientSessionState>()
            .unwrap();
        begin_client_session(
            &mut session,
            ClientSessionTarget {
                server_id: "srv-local".to_string(),
                server_endpoint: "127.0.0.1:7000".to_string(),
                transport: TransportKind::Quic,
                protocol: ProtocolVersion::new(1, 1, 1),
                server_cert_fingerprint_sha256: "a".repeat(64),
                ticket: "ticket-1".to_string(),
            },
        )
    };
    {
        let mut outbox = app
            .world_mut()
            .resource_mut::<NetworkClientOutbox>()
            .unwrap();
        for message in messages {
            outbox.push(message);
        }
    }

    let app = app
        .run_for_frames(1)
        .expect("client bootstrap frame should run");
    let session = app.world().resource::<ClientSessionState>().unwrap();
    assert_eq!(session.phase, SessionPhase::Handshaking);
    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.flushed_client_messages_last_frame, 2);
}

#[test]
fn server_plugin_accepts_valid_join_requests() {
    let mut app = App::headless();
    app.add_plugin(NetworkServerPlugin);
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

    let app = app
        .run_for_frames(1)
        .expect("server bootstrap frame should run");
    let session = app
        .world()
        .resource::<engine_net::ServerSessionState>()
        .unwrap();
    assert_eq!(session.phase, SessionPhase::Active);
    assert_eq!(session.active_connection, Some(engine_net::ConnectionId(1)));
    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.processed_client_messages_last_frame, 2);
    assert_eq!(diagnostics.accepted_connections, 1);
    assert!(diagnostics.flushed_server_messages_last_frame >= 2);
}

#[test]
fn client_plugin_marks_session_active_on_join_accept() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    app.world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));
    app.world_mut()
        .resource_mut::<NetworkClientInbox>()
        .unwrap()
        .push(ServerMessage::JoinAccepted(engine_net::JoinAccepted {
            connection_id: 7,
            tick_rate_hz: 60,
            join_state: engine_net::AuthoritativeJoinState {
                lobby_id: Some("lobby-1".to_string()),
                roster_player_codes: vec!["P1".to_string(), "P2".to_string()],
                max_players: 4,
                ai_fill_target: 4,
                settings_json: Some("{\"difficulty\":\"normal\"}".to_string()),
            },
        }));

    let app = app
        .run_for_frames(1)
        .expect("client receive frame should run");
    let session = app.world().resource::<ClientSessionState>().unwrap();
    assert_eq!(session.phase, SessionPhase::Active);
    assert_eq!(session.connection_id, Some(engine_net::ConnectionId(7)));
    let admission = app.world().resource::<NetworkAdmissionState>().unwrap();
    assert_eq!(
        admission
            .authoritative_join
            .as_ref()
            .and_then(|state| state.lobby_id.as_deref()),
        Some("lobby-1")
    );
    let session = app
        .world()
        .resource::<engine::SessionRuntimeState>()
        .unwrap();
    assert!(session.admitted);
    assert_eq!(session.lobby_id.as_deref(), Some("lobby-1"));
    assert_eq!(session.roster_player_codes, vec!["P1", "P2"]);
    assert_eq!(session.max_players, 4);
    assert_eq!(session.ai_fill_target, 4);
    assert_eq!(
        session.settings_json.as_deref(),
        Some("{\"difficulty\":\"normal\"}")
    );
    let diagnostics = app.world().resource::<NetworkDiagnostics>().unwrap();
    assert_eq!(diagnostics.accepted_connections, 1);
    let runtime_status = app.world().resource::<NetworkSessionStatus>().unwrap();
    assert_eq!(runtime_status.phase, SessionPhase::Active);
    assert_eq!(
        runtime_status.connection_id,
        Some(engine_net::ConnectionId(7))
    );
}
