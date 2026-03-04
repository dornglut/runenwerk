use engine::plugins::{
    NetworkClientInbox, NetworkClientOutbox, NetworkClientPlugin, NetworkDiagnostics,
    NetworkServerInbox, NetworkServerOutbox, NetworkServerPlugin, PredictionDiagnostics,
    PredictionPlugin, ReplicationDiagnostics, ReplicationPlugin,
};
use engine::prelude::*;
use engine_net::{
    ClientMessage, Hello, PlayerCommandBuffer, ProtocolVersion, ServerMessage, SnapshotCursor,
    TransportKind,
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
    assert_eq!(diagnostics.flushed_server_messages_last_frame, 1);
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
