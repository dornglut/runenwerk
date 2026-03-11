// Owner: Cavern Hunt Live Multiplayer Tests - Two Clients
#[tokio::test(flavor = "multi_thread")]
async fn two_live_clients_share_one_cavern_hunt_run() -> Result<()> {
    let transport = QuicTransport::default();
    let protocol = ProtocolVersion::new(1, 1, 1);
    let session_config = ServerSessionConfig {
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
            .resource::<engine::plugins::net::NetworkSessionStatus>()?;
        let client_a_status = client_a
            .world()
            .resource::<engine::plugins::net::NetworkSessionStatus>()?;
        let client_b_status = client_b
            .world()
            .resource::<engine::plugins::net::NetworkSessionStatus>()?;
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
