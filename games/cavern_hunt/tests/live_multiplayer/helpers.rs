// Owner: Cavern Hunt Live Multiplayer Tests - Shared Helpers
#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct LiveSnapshot;

#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct LiveDelta;

#[derive(Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct LiveInput;

struct LiveRuntimeDriver;

impl engine_net::replication::ReplicationDriver for LiveRuntimeDriver {
    type Snapshot = LiveSnapshot;
    type Delta = LiveDelta;
    type Input = LiveInput;
    type Error = std::io::Error;

    fn capture_snapshot(_world: &ecs::World) -> Result<Option<Self::Snapshot>, Self::Error> {
        Ok(None)
    }

    fn build_delta(_previous: &Self::Snapshot, _current: &Self::Snapshot) -> Self::Delta {
        LiveDelta
    }

    fn apply_delta_to_snapshot(_base: &Self::Snapshot, _delta: &Self::Delta) -> Self::Snapshot {
        LiveSnapshot
    }

    fn receive_remote_input(
        _world: &mut ecs::World,
        _tick: engine_net::SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply_snapshot(
        _world: &mut ecs::World,
        _tick: engine_net::SimulationTick,
        _snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn apply_delta(
        _world: &mut ecs::World,
        _tick: engine_net::SimulationTick,
        _delta: Self::Delta,
    ) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn take_local_input(_world: &mut ecs::World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(Vec::new())
    }

    fn apply_input(_world: &mut ecs::World, _input: &[Self::Input]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
    }
}

fn build_server_app(
    handle: NetworkRuntimeHandle,
    session_config: engine_net::ServerSessionConfig,
) -> App {
    let mut app = App::headless();
    app.set_title("Cavern Hunt Live Server Test");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Server);
    app.add_plugins(default_plugins());
    app.add_plugins((
        ScenePlugin,
        RenderPlugin,
        NetworkServerPlugin,
        NetworkReplicationRuntimePlugin::<LiveRuntimeDriver>::default(),
        CavernHuntPlugin,
        CavernHuntServerPlugin,
    ));
    app.world_mut().insert_resource(session_config);
    app.world_mut().insert_resource(handle);
    app.world_mut()
        .insert_resource(engine::plugins::NetworkClientInbox::default());
    app
}

fn build_client_app(handle: NetworkRuntimeHandle) -> App {
    let mut app = App::headless();
    app.set_title("Cavern Hunt Live Client Test");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Client);
    app.add_plugins(default_plugins());
    app.add_plugins((
        ScenePlugin,
        RenderPlugin,
        NetworkClientPlugin,
        NetworkReplicationRuntimePlugin::<LiveRuntimeDriver>::default(),
        CavernHuntPlugin,
        CavernHuntClientPlugin,
    ));
    app.world_mut().insert_resource(handle);
    app.world_mut()
        .insert_resource(engine::plugins::NetworkServerInbox::default());
    app.world_mut()
        .insert_resource(engine::plugins::NetworkServerOutbox::default());
    app.world_mut()
        .insert_resource(engine_net::ServerSessionConfig::default());
    app.world_mut()
        .insert_resource(engine_net::ServerSessionState::default());
    app.world_mut()
        .insert_resource(CavernMetaPersistenceConfig { enabled: false });
    app
}

fn clear_client_input(app: &mut App) -> Result<()> {
    let input = &mut *app
        .world_mut()
        .resource_mut::<engine::prelude::InputState>()?;
    input.world_move_left = false;
    input.world_move_right = false;
    input.world_move_up = false;
    input.world_move_down = false;
    Ok(())
}

fn step_tick(mut app: App) -> Result<App> {
    let next_tick = app.current_tick().saturating_add(1);
    app = app.run_for_ticks(next_tick)?;
    Ok(app)
}

async fn pump_round(server: App, client_a: App, client_b: App) -> Result<(App, App, App)> {
    let client_a = step_tick(client_a)?;
    let client_b = step_tick(client_b)?;
    let server = step_tick(server)?;
    tokio::time::sleep(Duration::from_millis(10)).await;
    Ok((server, client_a, client_b))
}

async fn pump_round_many(server: App, clients: Vec<App>) -> Result<(App, Vec<App>)> {
    let mut stepped_clients = Vec::with_capacity(clients.len());
    for client in clients {
        stepped_clients.push(step_tick(client)?);
    }
    let server = step_tick(server)?;
    tokio::time::sleep(Duration::from_millis(10)).await;
    Ok((server, stepped_clients))
}

fn make_client_handle(
    transport: &QuicTransport,
    server_id: &str,
    server_addr: std::net::SocketAddr,
    server_name: &str,
    protocol: ProtocolVersion,
    fingerprint: &str,
    trusted_certificate: &CertificateDer<'static>,
) -> Result<NetworkRuntimeHandle> {
    let target = ClientSessionTarget {
        server_id: server_id.to_string(),
        server_endpoint: server_addr.to_string(),
        transport: TransportKind::Quic,
        protocol,
        server_cert_fingerprint_sha256: fingerprint.to_string(),
        ticket: "ticket-live".to_string(),
    };
    let trust_policy = QuicTrustPolicy::PinnedServer {
        expected_fingerprint_sha256: fingerprint.to_string(),
        trusted_certificates: vec![trusted_certificate.clone()],
    };
    let runtime = transport.spawn_client_runtime(
        default_client_bind_addr(),
        server_name,
        target,
        trust_policy,
    )?;
    let (command_tx, event_rx) = runtime.into_channels();
    Ok(NetworkRuntimeHandle::new(command_tx, event_rx))
}

fn shutdown_app(app: &App) {
    if let Ok(handle) = app.world().resource::<NetworkRuntimeHandle>() {
        let _ = handle.send(SessionRuntimeCommand::Shutdown);
    }
}
