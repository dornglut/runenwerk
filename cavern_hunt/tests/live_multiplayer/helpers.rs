// Owner: Cavern Hunt Live Multiplayer Tests - Shared Helpers
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
        NetworkServerPlugin,
        CavernHuntPlugin,
        CavernHuntServerPlugin,
    ));
    app.world_mut().insert_resource(session_config);
    app.world_mut().insert_resource(handle);
    app
}

fn build_client_app(handle: NetworkRuntimeHandle) -> App {
    let mut app = App::headless();
    app.set_title("Cavern Hunt Live Client Test");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Client);
    app.add_plugins(default_plugins());
    app.add_plugins((NetworkClientPlugin, CavernHuntPlugin));
    app.world_mut().insert_resource(handle);
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
