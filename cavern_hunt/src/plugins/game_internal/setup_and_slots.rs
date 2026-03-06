// Owner: Cavern Hunt Gameplay Plugin - Setup and Active Slots
fn client_setup_system(mut world: WorldMut) -> Result<()> {
    if let Err(err) = meta::load_meta_profile(&mut world) {
        tracing::warn!(
            ?err,
            "failed to load Cavern Hunt meta profile; using defaults"
        );
        world.insert_resource(CavernMetaProfile::default());
    }
    worldgen::initialize_run_world(&mut world, true)?;
    render_sdf::setup_render_resources(&mut world)?;
    Ok(())
}

fn server_setup_system(mut world: WorldMut) -> Result<()> {
    worldgen::initialize_run_world(&mut world, false)
}

fn sync_active_player_slots_system(mut world: WorldMut) -> Result<()> {
    sync_active_player_slots(&mut world)
}

pub(crate) fn sync_active_player_slots(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let local_player = world
        .resource::<LocalPlayerRef>()
        .cloned()
        .unwrap_or_default();
    let mut ownership = world
        .resource::<CavernPlayerOwnershipState>()
        .cloned()
        .unwrap_or_default();
    let session = world
        .resource::<SessionRuntimeState>()
        .cloned()
        .unwrap_or_default();
    let max_players = world
        .resource::<CavernRunConfig>()
        .map(|config| config.max_players.max(1))
        .unwrap_or(1);
    let spawn_policy = world
        .resource::<SessionSpawnPolicy>()
        .cloned()
        .unwrap_or_default();
    let meta_profile = world
        .resource::<CavernMetaProfile>()
        .cloned()
        .unwrap_or_default();
    let mut active_player_ids = BTreeSet::new();
    match authority {
        AuthorityRole::Local => {
            active_player_ids.insert(local_player.player_id.unwrap_or(1));
        }
        AuthorityRole::Client | AuthorityRole::Peer => {
            for (_, player_id) in world.query::<(engine::prelude::Entity, &PlayerId)>().iter() {
                active_player_ids.insert(player_id.0);
            }
            if let Some(player_id) = local_player.player_id {
                active_player_ids.insert(player_id);
            }
        }
        AuthorityRole::Server => {
            if let Ok(session_state) = world.resource::<ServerSessionState>() {
                if !session_state.active_connections.is_empty() {
                    let live_connections = session_state
                        .active_connections
                        .iter()
                        .map(|connection_id| connection_id.0)
                        .collect::<Vec<_>>();
                    ownership.retain_active_connections(live_connections.clone());
                    let mut assigned_ids = ownership
                        .by_connection_id
                        .values()
                        .copied()
                        .filter(|player_id| *player_id >= 1 && *player_id <= u32::from(max_players))
                        .collect::<std::collections::BTreeSet<_>>();
                    for connection_id in live_connections {
                        if ownership.by_connection_id.contains_key(&connection_id) {
                            continue;
                        }
                        if let Some(player_id) =
                            (1..=u32::from(max_players)).find(|id| !assigned_ids.contains(id))
                        {
                            ownership.by_connection_id.insert(connection_id, player_id);
                            assigned_ids.insert(player_id);
                        }
                    }
                    world.insert_resource(ownership.clone());
                }
            }
            for player_id in ownership.by_connection_id.values().copied() {
                if player_id >= 1 && player_id <= u32::from(max_players) {
                    active_player_ids.insert(player_id);
                }
            }
            if session.admitted {
                let desired_total = session
                    .ai_fill_target
                    .max(active_player_ids.len().clamp(1, u8::MAX as usize) as u8)
                    .min(max_players);
                let mut next_player_id = 1_u32;
                while active_player_ids.len() < usize::from(desired_total) {
                    if next_player_id <= u32::from(max_players)
                        && !active_player_ids.contains(&next_player_id)
                    {
                        active_player_ids.insert(next_player_id);
                    }
                    next_player_id = next_player_id.saturating_add(1);
                    if next_player_id > u32::from(max_players).saturating_add(1) {
                        break;
                    }
                }
            }
        }
    }

    let mut player_entities = world
        .query::<(engine::prelude::Entity, &PlayerId)>()
        .iter()
        .map(|(entity, player_id)| (entity, player_id.0))
        .collect::<Vec<_>>();
    let existing_ids = player_entities
        .iter()
        .map(|(_, player_id)| *player_id)
        .collect::<BTreeSet<_>>();
    for (spawn_index, player_id) in active_player_ids.iter().copied().enumerate() {
        if !existing_ids.contains(&player_id) {
            let roster_index = player_id.saturating_sub(1) as usize;
            let is_companion = !ownership
                .by_connection_id
                .values()
                .any(|owned| *owned == player_id);
            let companion_slot = active_player_ids
                .iter()
                .copied()
                .filter(|candidate| {
                    !ownership
                        .by_connection_id
                        .values()
                        .any(|owned| *owned == *candidate)
                        && *candidate <= player_id
                })
                .count()
                .saturating_sub(1) as u8;
            let player_code = session
                .roster_player_codes
                .get(roster_index)
                .cloned()
                .unwrap_or_else(|| {
                    if is_companion {
                        format!("companion_{player_id}")
                    } else {
                        format!("hunter_{player_id}")
                    }
                });
            let spawn_profile = if is_companion {
                PlayerSpawnProfile {
                    is_human: false,
                    role: Some(match companion_slot % 2 {
                        0 => crate::domain::CompanionBehaviorRole::Skirmisher,
                        _ => crate::domain::CompanionBehaviorRole::SupportShooter,
                    }),
                    spawn_radius: spawn_policy.spawn_radius
                        + spawn_policy.companion_spacing * companion_slot as f32 * 0.15,
                    weapon_cooldown_scale: if companion_slot % 2 == 0 { 0.95 } else { 1.1 },
                    projectile_speed_scale: if companion_slot % 2 == 0 { 1.05 } else { 1.15 },
                    bonus_health: if companion_slot % 2 == 0 { 1.0 } else { 0.0 },
                }
            } else {
                PlayerSpawnProfile {
                    is_human: true,
                    role: None,
                    spawn_radius: spawn_policy.spawn_radius,
                    weapon_cooldown_scale: 1.0,
                    projectile_speed_scale: 1.0,
                    bonus_health: 0.0,
                }
            };
            let entity = worldgen::spawn_player_entity(
                world,
                player_id,
                spawn_index,
                true,
                &meta_profile,
                &spawn_profile,
                player_code,
                roster_index as u8,
                is_companion,
            );
            player_entities.push((entity, player_id));
        }
    }
    player_entities.sort_by_key(|(_, player_id)| *player_id);
    let mut resolved_local_entity = None;
    let mut living_active_players = 0_u8;

    for (entity, player_id) in player_entities {
        let should_be_active = active_player_ids.contains(&player_id);
        let is_active = world.get::<PlayerActive>(entity).is_some();
        if should_be_active && !is_active {
            let _ = world.insert(entity, PlayerActive);
        } else if !should_be_active && is_active {
            let _ = world.remove::<PlayerActive>(entity);
        }

        if should_be_active {
            if local_player.player_id == Some(player_id) {
                resolved_local_entity = Some(entity);
            }
            if world
                .get::<crate::domain::Health>(entity)
                .map(|health| health.current > 0.0)
                .unwrap_or(false)
            {
                living_active_players = living_active_players.saturating_add(1);
            }
        }
    }

    if let Ok(mut run_state) = world.resource_mut::<CavernRunState>() {
        run_state.party_alive_count = living_active_players;
    }
    if let Ok(mut local_ref) = world.resource_mut::<LocalPlayerRef>() {
        local_ref.entity = resolved_local_entity;
    }

    Ok(())
}
