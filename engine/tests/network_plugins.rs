use engine::plugins::scene::SceneSimulationDeltaV1;
use engine::plugins::{
    NetworkAdmissionState, NetworkClientInbox, NetworkClientOutbox, NetworkClientPlugin,
    NetworkDiagnostics, NetworkOutboundQueue, NetworkRuntimeHandle, NetworkServerInbox,
    NetworkServerOutbox, NetworkServerPlugin, NetworkSessionStatus, PredictionDiagnostics,
    PredictionPlugin, ReplicationDiagnostics, ReplicationPlugin, ScenePlugin, default_plugins,
};
use engine::prelude::*;
use engine_net::{
    ClientCommandEnvelope, ClientMessage, ClientSessionState, ClientSessionTarget, Hello,
    MoveCommand, PlayerCommandBuffer, ProtocolVersion, ServerMessage, ServerSessionConfig,
    SessionPhase, SnapshotCursor, TransportKind, begin_client_session,
};

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
    assert_eq!(diagnostics.flushed_server_messages_last_frame, 2);
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
    app.add_plugins((NetworkServerPlugin, ReplicationPlugin, PredictionPlugin));
    app.world_mut()
        .resource_mut::<PlayerCommandBuffer>()
        .unwrap()
        .push(engine_net::ClientCommandEnvelope::Ability(
            engine_net::AbilityCommand { slot: 2 },
        ));

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
    assert_eq!(diagnostics.flushed_server_messages_last_frame, 2);
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
}

#[test]
fn network_runtime_handle_events_flow_into_engine_state() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    let (command_tx, _command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    event_tx
        .send(engine_net::SessionRuntimeEvent::Connected {
            connection_id: Some(engine_net::ConnectionId(9)),
        })
        .unwrap();
    event_tx
        .send(engine_net::SessionRuntimeEvent::ServerMessage(
            ServerMessage::JoinAccepted(engine_net::JoinAccepted {
                connection_id: 9,
                tick_rate_hz: 60,
                join_state: engine_net::AuthoritativeJoinState::default(),
            }),
        ))
        .unwrap();
    event_tx
        .send(engine_net::SessionRuntimeEvent::RttUpdated { millis: 14 })
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
            .resource::<engine::RoundTripMetrics>()
            .unwrap()
            .last_rtt_millis,
        Some(14)
    );
}

#[test]
fn reconnecting_event_updates_client_runtime_status() {
    let mut app = App::headless();
    app.add_plugin(NetworkClientPlugin);
    let (command_tx, _command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    event_tx
        .send(engine_net::SessionRuntimeEvent::Reconnecting { attempt: 2 })
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
    app.add_plugins((ScenePlugin, NetworkServerPlugin, ReplicationPlugin));

    let app = app.run_for_ticks(1).expect("server tick should run");
    let outbound = app.world().resource::<NetworkOutboundQueue>().unwrap();
    let message = outbound
        .server_messages()
        .iter()
        .find_map(|message| match message {
            ServerMessage::Snapshot(snapshot) => Some(snapshot),
            _ => None,
        })
        .expect("server should emit an initial full snapshot");
    let snapshot: engine::SceneSimulationSnapshotV1 =
        postcard::from_bytes(&message.payload).expect("snapshot payload should decode");
    assert_eq!(message.cursor, SnapshotCursor(1));
    assert_eq!(snapshot.context.world_scene_label, "gameplay_stub");
}

#[test]
fn client_snapshot_application_sends_ack_and_reconciles_prediction() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((
        ScenePlugin,
        NetworkServerPlugin,
        ReplicationPlugin,
        PredictionPlugin,
    ));
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
            ServerMessage::Snapshot(snapshot) => Some(snapshot.clone()),
            _ => None,
        })
        .expect("server should emit a snapshot");

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((
        ScenePlugin,
        NetworkClientPlugin,
        ReplicationPlugin,
        PredictionPlugin,
    ));
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
            .resource::<engine::SnapshotReplicationState>()
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
fn server_delta_snapshot_applies_cleanly_on_client() {
    let mut server = App::headless();
    server.add_plugins(default_plugins());
    server.add_plugins((
        ScenePlugin,
        NetworkServerPlugin,
        ReplicationPlugin,
        PredictionPlugin,
    ));

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
        .resource::<engine::SnapshotReplicationState>()
        .unwrap()
        .last_sent_snapshot
        .clone()
        .expect("server should retain the second authoritative snapshot");
    let decoded_delta: SceneSimulationDeltaV1 =
        postcard::from_bytes(&delta_snapshot.payload).expect("delta payload should decode");
    let delta_tick = delta_snapshot.tick;
    assert_eq!(delta_snapshot.base, SnapshotCursor(0));
    assert_eq!(delta_snapshot.cursor, SnapshotCursor(2));
    assert!(delta_snapshot.payload.len() < full_snapshot.payload.len());
    assert_ne!(decoded_delta, SceneSimulationDeltaV1::default());
    assert_eq!(decoded_delta.context.world_scene_label, None);

    let mut client = App::headless();
    client.add_plugins(default_plugins());
    client.add_plugins((
        ScenePlugin,
        NetworkClientPlugin,
        ReplicationPlugin,
        PredictionPlugin,
    ));

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
        .resource::<engine::SnapshotReplicationState>()
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
    app.add_plugins((ScenePlugin, NetworkServerPlugin, ReplicationPlugin));
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
            .resource::<engine::SnapshotReplicationState>()
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
            .resource::<engine::SnapshotReplicationState>()
            .unwrap()
            .active_connection,
        Some(engine_net::ConnectionId(2))
    );
    assert!(
        !app.world()
            .resource::<engine::SnapshotReplicationState>()
            .unwrap()
            .initial_snapshot_sent
    );
    assert_eq!(
        app.world()
            .resource::<engine::SnapshotReplicationState>()
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
