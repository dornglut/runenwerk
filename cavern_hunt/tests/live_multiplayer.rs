use anyhow::{Result, bail};
use cavern_hunt::domain::{
    CavernMetaPersistenceConfig, CavernMetaProfile, CavernRunConfig, CavernRunPhase,
    CavernRunState, EnemyKind, ExtractionZone, Health, InventoryRunState, LocalPlayerRef,
    PlayerActive, PlayerId, Transform2,
};
use cavern_hunt::{CavernHuntPlugin, CavernHuntServerPlugin};
use engine::plugins::{
    NetworkClientPlugin, NetworkRuntimeHandle, NetworkServerPlugin, default_plugins,
};
use engine::{App, AuthorityRole, SimulationProfile};
use engine_net::{ClientSessionTarget, ProtocolVersion, SessionRuntimeCommand, TransportKind};
use engine_net_quic::{QuicTransport, QuicTrustPolicy, default_client_bind_addr};
use rustls::pki_types::CertificateDer;
use std::time::Duration;

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

#[tokio::test(flavor = "multi_thread")]
async fn two_live_clients_share_one_cavern_hunt_run() -> Result<()> {
    let transport = QuicTransport::default();
    let protocol = ProtocolVersion::new(1, 1, 1);
    let session_config = engine_net::ServerSessionConfig {
        server_id: "srv-live-test".to_string(),
        protocol,
        tick_rate_hz: 60,
    };
    let server_runtime = transport.spawn_server_runtime(
        default_client_bind_addr(),
        "localhost",
        session_config.clone(),
    )?;
    let server_addr = server_runtime.local_addr;
    let server_name = server_runtime.server_name.clone();
    let trusted_certificate = server_runtime.certificate.clone();
    let fingerprint = server_runtime.certificate_fingerprint_sha256.clone();
    let (server_command_tx, server_event_rx) = server_runtime.into_channels();
    let mut server = build_server_app(
        NetworkRuntimeHandle::new(server_command_tx, server_event_rx),
        session_config,
    );

    let mut client_a = build_client_app(make_client_handle(
        &transport,
        "srv-live-test",
        server_addr,
        &server_name,
        protocol,
        &fingerprint,
        &trusted_certificate,
    )?);
    let mut client_b = build_client_app(make_client_handle(
        &transport,
        "srv-live-test",
        server_addr,
        &server_name,
        protocol,
        &fingerprint,
        &trusted_certificate,
    )?);

    let mut connected_round = None;
    let mut last_server_players = Vec::new();
    let mut last_client_a_local = None;
    let mut last_client_b_local = None;
    for round in 0..80_u32 {
        (server, client_a, client_b) = pump_round(server, client_a, client_b).await?;
        let server_players = server
            .world()
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter(|(entity, _)| server.world().get::<PlayerActive>(*entity).is_some())
            .map(|(_, player_id)| player_id.0)
            .collect::<Vec<_>>();
        let client_a_local = client_a.world().resource::<LocalPlayerRef>()?.player_id;
        let client_b_local = client_b.world().resource::<LocalPlayerRef>()?.player_id;
        last_server_players = server_players.clone();
        last_client_a_local = client_a_local;
        last_client_b_local = client_b_local;
        if server_players.len() == 2
            && client_a_local.is_some()
            && client_b_local.is_some()
            && client_a_local != client_b_local
        {
            connected_round = Some(round);
            break;
        }
    }

    if connected_round.is_none() {
        let server_status = server
            .world()
            .resource::<engine::plugins::NetworkSessionStatus>()?;
        let client_a_status = client_a
            .world()
            .resource::<engine::plugins::NetworkSessionStatus>()?;
        let client_b_status = client_b
            .world()
            .resource::<engine::plugins::NetworkSessionStatus>()?;
        let server_ownership = server
            .world()
            .resource::<cavern_hunt::domain::CavernPlayerOwnershipState>()?;
        bail!(
            "server and clients did not converge: server_players={last_server_players:?} client_a_local={last_client_a_local:?} client_b_local={last_client_b_local:?} server_status={server_status:?} client_a_status={client_a_status:?} client_b_status={client_b_status:?} server_ownership={server_ownership:?}"
        );
    }

    let client_a_player_id = client_a.world().resource::<LocalPlayerRef>()?.player_id;
    let client_b_player_id = client_b.world().resource::<LocalPlayerRef>()?.player_id;
    assert!(client_a_player_id.is_some(), "client A should own a player");
    assert!(client_b_player_id.is_some(), "client B should own a player");

    {
        let input = &mut *client_a
            .world_mut()
            .resource_mut::<engine::prelude::InputState>()?;
        input.world_move_right = true;
        input.handle_cursor_moved(960.0, 360.0);
    }
    {
        let input = &mut *client_b
            .world_mut()
            .resource_mut::<engine::prelude::InputState>()?;
        input.world_move_up = true;
        input.handle_cursor_moved(320.0, 240.0);
    }

    for _ in 0..20 {
        (server, client_a, client_b) = pump_round(server, client_a, client_b).await?;
    }

    let active_players = server
        .world()
        .query::<(engine::prelude::Entity, &PlayerId)>()
        .iter()
        .filter_map(|(entity, player_id)| {
            server.world().get::<PlayerActive>(entity).and_then(|_| {
                server
                    .world()
                    .get::<Transform2>(entity)
                    .copied()
                    .map(|transform| (player_id.0, transform))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(active_players.len(), 2);

    let client_a_transform = active_players
        .iter()
        .find(|(player_id, _)| Some(*player_id) == client_a_player_id)
        .map(|(_, transform)| *transform)
        .expect("client A owned player should exist on server");
    let client_b_transform = active_players
        .iter()
        .find(|(player_id, _)| Some(*player_id) == client_b_player_id)
        .map(|(_, transform)| *transform)
        .expect("client B owned player should exist on server");

    assert!(
        client_a_transform.x > 0.5,
        "client A should have moved right"
    );
    assert!(client_b_transform.y > 0.5, "client B should have moved up");

    clear_client_input(&mut client_a)?;
    clear_client_input(&mut client_b)?;

    {
        let extraction_pos = server
            .world()
            .query::<(engine::prelude::Entity, &Transform2)>()
            .iter()
            .find_map(|(entity, transform)| {
                server
                    .world()
                    .get::<ExtractionZone>(entity)
                    .map(|_| [transform.x, transform.y])
            })
            .expect("server should have an extraction zone");
        let player_entities = server
            .world()
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                server
                    .world()
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some((entity, player_id.0))
            })
            .collect::<Vec<_>>();
        for (entity, player_id) in player_entities {
            if let Some(mut inventory) = server.world_mut().get_mut::<InventoryRunState>(entity) {
                inventory.scrap = if player_id == 1 { 11 } else { 17 };
            }
            if let Some(mut transform) = server.world_mut().get_mut::<Transform2>(entity) {
                transform.x = extraction_pos[0];
                transform.y = extraction_pos[1];
            }
        }
        if let Some(elite) = server
            .world()
            .query::<(engine::prelude::Entity, &EnemyKind)>()
            .iter()
            .find_map(|(entity, kind)| (*kind == EnemyKind::NestGuardian).then_some(entity))
            && let Some(mut health) = server.world_mut().get_mut::<Health>(elite)
        {
            health.current = 0.0;
        }
        if let Ok(mut config) = server.world_mut().resource_mut::<CavernRunConfig>() {
            config.extract_countdown_seconds = 0.0;
        }
    }

    let mut saw_success = false;
    for _ in 0..30 {
        (server, client_a, client_b) = pump_round(server, client_a, client_b).await?;
        let server_phase = server.world().resource::<CavernRunState>()?.phase;
        let client_a_phase = client_a.world().resource::<CavernRunState>()?.phase;
        let client_b_phase = client_b.world().resource::<CavernRunState>()?.phase;
        if matches!(server_phase, CavernRunPhase::Success)
            && matches!(client_a_phase, CavernRunPhase::Success)
            && matches!(client_b_phase, CavernRunPhase::Success)
        {
            saw_success = true;
            break;
        }
    }

    assert!(
        saw_success,
        "server and clients should converge on run success"
    );
    let expected_client_a_marks = if client_a_player_id == Some(1) {
        11
    } else {
        17
    };
    let expected_client_b_marks = if client_b_player_id == Some(1) {
        11
    } else {
        17
    };
    assert_eq!(
        client_a
            .world()
            .resource::<CavernMetaProfile>()?
            .cavern_marks,
        expected_client_a_marks
    );
    assert_eq!(
        client_b
            .world()
            .resource::<CavernMetaProfile>()?
            .cavern_marks,
        expected_client_b_marks
    );

    shutdown_app(&server);
    shutdown_app(&client_a);
    shutdown_app(&client_b);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn four_live_clients_complete_run_and_reconnect_one_client() -> Result<()> {
    let transport = QuicTransport::default();
    let protocol = ProtocolVersion::new(1, 1, 1);
    let session_config = engine_net::ServerSessionConfig {
        server_id: "srv-live-four".to_string(),
        protocol,
        tick_rate_hz: 60,
    };
    let server_runtime = transport.spawn_server_runtime(
        default_client_bind_addr(),
        "localhost",
        session_config.clone(),
    )?;
    let server_addr = server_runtime.local_addr;
    let server_name = server_runtime.server_name.clone();
    let trusted_certificate = server_runtime.certificate.clone();
    let fingerprint = server_runtime.certificate_fingerprint_sha256.clone();
    let (server_command_tx, server_event_rx) = server_runtime.into_channels();
    let mut server = build_server_app(
        NetworkRuntimeHandle::new(server_command_tx, server_event_rx),
        session_config,
    );

    let mut clients = (0..4)
        .map(|_| {
            make_client_handle(
                &transport,
                "srv-live-four",
                server_addr,
                &server_name,
                protocol,
                &fingerprint,
                &trusted_certificate,
            )
            .map(build_client_app)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut saw_full_party = false;
    let mut owned_ids = Vec::new();
    for _ in 0..120 {
        (server, clients) = pump_round_many(server, clients).await?;
        let server_players = server
            .world()
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter(|(entity, _)| server.world().get::<PlayerActive>(*entity).is_some())
            .map(|(_, player_id)| player_id.0)
            .collect::<Vec<_>>();
        let client_ids = clients
            .iter()
            .map(|client| {
                client
                    .world()
                    .resource::<LocalPlayerRef>()
                    .map(|local| local.player_id)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let unique_client_ids = client_ids
            .iter()
            .copied()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        if server_players.len() == 4
            && client_ids.iter().all(|id| id.is_some())
            && unique_client_ids.len() == 4
        {
            owned_ids = client_ids.into_iter().flatten().collect();
            saw_full_party = true;
            break;
        }
    }
    assert!(saw_full_party, "four clients should converge into one run");
    owned_ids.sort_unstable();
    assert_eq!(owned_ids, vec![1, 2, 3, 4]);
    let mut clients_have_full_party_view = false;
    for _ in 0..120 {
        (server, clients) = pump_round_many(server, clients).await?;
        let all_clients_see_four_players = clients.iter().all(|client| {
            client
                .world()
                .query::<(engine::prelude::Entity, &PlayerId)>()
                .iter()
                .filter(|(entity, _)| client.world().get::<PlayerActive>(*entity).is_some())
                .count()
                == 4
        });
        if all_clients_see_four_players {
            clients_have_full_party_view = true;
            break;
        }
    }
    assert!(
        clients_have_full_party_view,
        "every client should render all active players in the shared run"
    );

    for (index, client) in clients.iter_mut().enumerate() {
        let input = &mut *client
            .world_mut()
            .resource_mut::<engine::prelude::InputState>()?;
        match index {
            0 => input.world_move_right = true,
            1 => input.world_move_up = true,
            2 => input.world_move_left = true,
            _ => input.world_move_down = true,
        }
        input.handle_cursor_moved(320.0 + index as f32 * 120.0, 240.0);
    }

    for _ in 0..20 {
        (server, clients) = pump_round_many(server, clients).await?;
    }

    let disconnected = clients.remove(3);
    let disconnected_player_id = disconnected
        .world()
        .resource::<LocalPlayerRef>()?
        .player_id
        .expect("disconnected client should own a player");
    shutdown_app(&disconnected);

    for _ in 0..15 {
        (server, clients) = pump_round_many(server, clients).await?;
    }

    let replacement = build_client_app(make_client_handle(
        &transport,
        "srv-live-four",
        server_addr,
        &server_name,
        protocol,
        &fingerprint,
        &trusted_certificate,
    )?);
    clients.push(replacement);

    let mut reconnected = false;
    for _ in 0..120 {
        (server, clients) = pump_round_many(server, clients).await?;
        if clients
            .last()
            .and_then(|client| client.world().resource::<LocalPlayerRef>().ok())
            .and_then(|local| local.player_id)
            == Some(disconnected_player_id)
        {
            reconnected = true;
            break;
        }
    }
    assert!(
        reconnected,
        "replacement client should recover the disconnected player slot"
    );

    for client in &mut clients {
        clear_client_input(client)?;
    }

    {
        let extraction_pos = server
            .world()
            .query::<(engine::prelude::Entity, &Transform2)>()
            .iter()
            .find_map(|(entity, transform)| {
                server
                    .world()
                    .get::<ExtractionZone>(entity)
                    .map(|_| [transform.x, transform.y])
            })
            .expect("server should have an extraction zone");
        let player_entities = server
            .world()
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                server
                    .world()
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some((entity, player_id.0))
            })
            .collect::<Vec<_>>();
        for (entity, player_id) in player_entities {
            if let Some(mut inventory) = server.world_mut().get_mut::<InventoryRunState>(entity) {
                inventory.scrap = 10 + player_id;
            }
            if let Some(mut transform) = server.world_mut().get_mut::<Transform2>(entity) {
                transform.x = extraction_pos[0];
                transform.y = extraction_pos[1];
            }
        }
        if let Some(elite) = server
            .world()
            .query::<(engine::prelude::Entity, &EnemyKind)>()
            .iter()
            .find_map(|(entity, kind)| (*kind == EnemyKind::NestGuardian).then_some(entity))
            && let Some(mut health) = server.world_mut().get_mut::<Health>(elite)
        {
            health.current = 0.0;
        }
        if let Ok(mut config) = server.world_mut().resource_mut::<CavernRunConfig>() {
            config.extract_countdown_seconds = 0.0;
        }
    }

    let mut all_success = false;
    for _ in 0..40 {
        (server, clients) = pump_round_many(server, clients).await?;
        let server_phase = server.world().resource::<CavernRunState>()?.phase;
        if matches!(server_phase, CavernRunPhase::Success)
            && clients.iter().all(|client| {
                client
                    .world()
                    .resource::<CavernRunState>()
                    .map(|state| matches!(state.phase, CavernRunPhase::Success))
                    .unwrap_or(false)
            })
        {
            all_success = true;
            break;
        }
    }
    assert!(
        all_success,
        "all four clients should converge on successful extraction"
    );

    let reward_details = clients
        .iter()
        .map(|client| {
            let local = client.world().resource::<LocalPlayerRef>().ok().cloned();
            let marks = client
                .world()
                .resource::<CavernMetaProfile>()
                .map(|profile| profile.cavern_marks)
                .unwrap_or_default();
            let local_scrap = local
                .as_ref()
                .and_then(|local| local.entity)
                .and_then(|entity| {
                    client
                        .world()
                        .get::<InventoryRunState>(entity)
                        .map(|inventory| inventory.scrap)
                })
                .unwrap_or_default();
            (local.and_then(|local| local.player_id), marks, local_scrap)
        })
        .collect::<Vec<_>>();
    for (player_id, marks, local_scrap) in &reward_details {
        assert!(
            *marks >= 11,
            "each client should receive extracted scrap; rewards={reward_details:?}, failing_player={player_id:?}, marks={marks}, local_scrap={local_scrap}"
        );
    }

    shutdown_app(&server);
    for client in &clients {
        shutdown_app(client);
    }

    Ok(())
}
