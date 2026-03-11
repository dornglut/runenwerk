// Owner: Cavern Hunt Live Multiplayer Tests - Four Clients and Reconnect
#[tokio::test(flavor = "multi_thread")]
async fn four_live_clients_complete_run_and_reconnect_one_client() -> Result<()> {
    let transport = QuicTransport::default();
    let protocol = ProtocolVersion::new(1, 1, 1);
    let session_config = ServerSessionConfig {
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
