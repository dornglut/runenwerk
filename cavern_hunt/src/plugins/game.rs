use crate::domain::{
    CavernAimState, CavernCameraState, CavernCollisionField, CavernControlState,
    CavernGeometryGraph, CavernGeometryRuntimeState, CavernHudState, CavernLayout,
    CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState, CavernObjectiveKind,
    CavernObjectiveState, CavernPlayerOwnershipState, CavernPredictionState, CavernRunConfig,
    CavernRunState, CavernSdfWorldFrame, CavernServerControlMap, CavernSessionSettings,
    CavernTopology, EnemyCombatTuning, ExtractionState, LocalPlayerRef, LootTableRegistry,
    PlayerActive, PlayerCombatTuning, PlayerId, PlayerSpawnProfile, RoomEncounterRegistry,
    RoomEncounterState, RoomEncounterStatus, RoomRole, RunDifficultyProfile, SessionSpawnPolicy,
    SpawnDirector,
};
use crate::plugins::{ai, combat, hud, loot, meta, net_sync, render_sdf, worldgen};
use anyhow::Result;
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Plugin, PreUpdate, Res, ResMut, SimulationProfileConfig, Startup,
    SystemConfigExt, Update, World, WorldMut,
};
use engine::state::SessionRuntimeState;
use engine_net::ServerSessionState;
use std::collections::BTreeSet;

pub struct CavernHuntPlugin;
pub struct CavernHuntClientPlugin;
pub struct CavernHuntServerPlugin;

impl Plugin for CavernHuntPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernRunConfig>();
        app.init_resource::<CavernRunState>();
        app.init_resource::<CavernLayout>();
        app.init_resource::<CavernTopology>();
        app.init_resource::<CavernGeometryGraph>();
        app.init_resource::<CavernCollisionField>();
        app.init_resource::<SpawnDirector>();
        app.init_resource::<LootTableRegistry>();
        app.init_resource::<CavernMetaProfile>();
        app.init_resource::<CavernMetaPersistenceConfig>();
        app.init_resource::<CavernMetaRewardState>();
        app.init_resource::<LocalPlayerRef>();
        app.init_resource::<CavernCameraState>();
        app.init_resource::<CavernAimState>();
        app.init_resource::<CavernControlState>();
        app.init_resource::<CavernPredictionState>();
        app.init_resource::<CavernServerControlMap>();
        app.init_resource::<CavernPlayerOwnershipState>();
        app.init_resource::<CavernSdfWorldFrame>();
        app.init_resource::<CavernGeometryRuntimeState>();
        app.init_resource::<SessionSpawnPolicy>();
        app.init_resource::<RoomEncounterRegistry>();
        app.init_resource::<CavernObjectiveState>();
        app.init_resource::<ExtractionState>();
        app.init_resource::<CavernHudState>();
        app.init_resource::<PlayerCombatTuning>();
        app.init_resource::<EnemyCombatTuning>();
        app.init_resource::<UiWorldHudStats>();
        app.add_plugins((
            combat::CavernHuntCombatPlugin,
            ai::CavernHuntAiPlugin,
            hud::CavernHuntHudPlugin,
            loot::CavernHuntLootPlugin,
            net_sync::CavernHuntNetSyncPlugin,
        ));
        app.add_systems(
            PreUpdate,
            (
                sync_session_runtime_config_system.after(CoreSet::NetReceive),
                sync_session_spawn_policy_system.after(CoreSet::NetReceive),
                sync_active_player_slots_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(Update, sync_run_presentation_state_system);
        app.add_systems(Update, meta::apply_run_meta_rewards_system);
    }
}

impl Plugin for CavernHuntClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, client_setup_system);
        app.add_systems(
            Update,
            (
                render_sdf::update_camera_and_hud_system,
                render_sdf::build_sdf_world_frame_system,
            ),
        );
    }
}

impl Plugin for CavernHuntServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, server_setup_system);
    }
}

fn sync_session_runtime_config_system(
    session: Res<SessionRuntimeState>,
    mut config: ResMut<CavernRunConfig>,
) -> Result<()> {
    sync_session_runtime_config(&session, &mut config);
    Ok(())
}

fn sync_session_spawn_policy_system(
    session: Res<SessionRuntimeState>,
    config: Res<CavernRunConfig>,
    mut policy: ResMut<SessionSpawnPolicy>,
) -> Result<()> {
    sync_session_spawn_policy(&session, &config, &mut policy);
    Ok(())
}

fn sync_session_runtime_config(session: &SessionRuntimeState, config: &mut CavernRunConfig) {
    if session.max_players > 0 {
        // Local/dev dedicated-authority sessions can admit with a fallback join state
        // that carries `max_players = 1`. Do not collapse the game-configured run
        // capacity in that case; only widen capacity here until real lobby metadata
        // becomes mandatory for all admissions.
        config.max_players = config.max_players.max(session.max_players.max(1));
    }
    if let Some(settings) = parse_cavern_session_settings(session) {
        if let Some(seed) = settings.seed {
            config.seed = seed;
        }
        if let Some(enemy_density) = settings.enemy_density {
            config.enemy_density = enemy_density.max(0.1);
        }
        if let Some(extract_countdown_seconds) = settings.extract_countdown_seconds {
            config.extract_countdown_seconds = extract_countdown_seconds.max(0.0);
        }
        if let Some(base_scrap_reward) = settings.base_scrap_reward {
            config.base_scrap_reward = base_scrap_reward;
        }
    }
}

fn sync_session_spawn_policy(
    session: &SessionRuntimeState,
    config: &CavernRunConfig,
    policy: &mut SessionSpawnPolicy,
) {
    let desired_human_players = session.roster_player_codes.len().clamp(1, u8::MAX as usize) as u8;
    let desired_total_participants = if session.admitted {
        session
            .ai_fill_target
            .max(desired_human_players)
            .min(config.max_players)
    } else {
        1
    };
    let companion_target_count = desired_total_participants.saturating_sub(desired_human_players);
    let settings = parse_cavern_session_settings(session).unwrap_or_default();
    policy.desired_human_players = desired_human_players;
    policy.desired_total_participants = desired_total_participants;
    policy.companion_target_count = companion_target_count;
    policy.spawn_radius = settings.spawn_radius.unwrap_or(1.1).max(0.6);
    policy.companion_spacing = settings.companion_spacing.unwrap_or(1.25).max(0.75);
    policy.roster_display_names = session
        .roster_player_codes
        .iter()
        .enumerate()
        .map(|(index, code)| (index as u8, code.clone()))
        .collect();
    policy.difficulty = RunDifficultyProfile {
        enemy_health_scale: settings.enemy_health_scale.unwrap_or(1.0).max(0.5),
        enemy_damage_scale: settings.enemy_damage_scale.unwrap_or(1.0).max(0.5),
        elite_health_bonus: settings.elite_health_bonus.unwrap_or(0.0).max(0.0),
    };
}

fn parse_cavern_session_settings(session: &SessionRuntimeState) -> Option<CavernSessionSettings> {
    let raw = session.settings_json.as_ref()?;
    serde_json::from_str(raw).ok()
}

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
                    ownership.retain_active_connections(live_connections);
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

fn sync_run_presentation_state_system(mut world: WorldMut) -> Result<()> {
    sync_run_presentation_state(&mut world)
}

fn sync_run_presentation_state(world: &mut World) -> Result<()> {
    let layout = world.resource::<CavernLayout>()?.clone();
    let run_state = world.resource::<CavernRunState>()?.clone();

    let mut encounters = world
        .resource::<RoomEncounterRegistry>()
        .cloned()
        .unwrap_or_default();
    if encounters.by_room_id.is_empty() {
        encounters.by_room_id = layout
            .rooms
            .iter()
            .map(|room| {
                (
                    room.id,
                    RoomEncounterStatus {
                        room_id: room.id,
                        role: room.role,
                        state: if room.role == RoomRole::Start {
                            RoomEncounterState::Cleared
                        } else {
                            RoomEncounterState::Dormant
                        },
                        has_reward: matches!(room.role, RoomRole::Loot | RoomRole::Elite),
                    },
                )
            })
            .collect();
    }

    let living_player_positions = world
        .query::<(engine::prelude::Entity, &crate::domain::Transform2)>()
        .iter()
        .filter_map(|(entity, transform)| {
            crate::domain::is_active_player_entity(world, entity)
                .then(|| world.get::<crate::domain::Health>(entity).copied())
                .flatten()
                .filter(|health| health.current > 0.0)
                .map(|_| [transform.x, transform.y])
        })
        .collect::<Vec<_>>();
    let occupied_rooms = living_player_positions
        .iter()
        .filter_map(|position| room_containing_point(&layout, *position))
        .collect::<BTreeSet<_>>();
    let living_enemies_by_room = world
        .query::<(engine::prelude::Entity, &crate::domain::EnemyKind)>()
        .iter()
        .filter_map(|(entity, _)| {
            let health = world.get::<crate::domain::Health>(entity).copied()?;
            if health.current <= 0.0 {
                return None;
            }
            world
                .get::<crate::domain::RoomAnchor>(entity)
                .map(|room| room.room_id)
        })
        .fold(BTreeSet::new(), |mut set, room_id| {
            set.insert(room_id);
            set
        });

    for (room_id, status) in &mut encounters.by_room_id {
        status.state =
            if occupied_rooms.contains(room_id) && living_enemies_by_room.contains(room_id) {
                RoomEncounterState::Active
            } else if !living_enemies_by_room.contains(room_id)
                && !matches!(status.role, RoomRole::Start | RoomRole::Fork)
            {
                RoomEncounterState::Cleared
            } else {
                status.state
            };
    }

    let extraction_remaining = if run_state.extraction_active {
        let fixed_dt = world
            .resource::<engine::prelude::Time>()
            .map(|time| time.delta_seconds.max(1.0 / 60.0))
            .unwrap_or(1.0 / 60.0);
        run_state
            .extraction_started_at_tick
            .map(|started| {
                let current_tick = world
                    .resource::<engine::prelude::SimulationTick>()
                    .copied()
                    .unwrap_or_default();
                let elapsed = current_tick.0.saturating_sub(started.0) as f32 * fixed_dt;
                (world
                    .resource::<CavernRunConfig>()
                    .map(|config| config.extract_countdown_seconds)
                    .unwrap_or(0.0)
                    - elapsed)
                    .max(0.0)
            })
            .unwrap_or_else(|| {
                world
                    .resource::<CavernRunConfig>()
                    .map(|config| config.extract_countdown_seconds)
                    .unwrap_or(0.0)
            })
    } else {
        0.0
    };

    let objective = match run_state.phase {
        crate::domain::CavernRunPhase::Success => CavernObjectiveState {
            kind: CavernObjectiveKind::Success,
            title: "Extraction successful".to_string(),
            detail: "Cash out and run it back".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::domain::CavernRunPhase::Failure => CavernObjectiveState {
            kind: CavernObjectiveKind::Failure,
            title: "Run failed".to_string(),
            detail: "The hunt is over".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::domain::CavernRunPhase::Extraction
            if run_state.extraction_started_at_tick.is_some() =>
        {
            CavernObjectiveState {
                kind: CavernObjectiveKind::ExtractionCountdown,
                title: "Reach extraction".to_string(),
                detail: format!("Hold for {:.1}s", extraction_remaining),
                elite_room: Some(layout.elite_room),
                extraction_room: Some(layout.extraction_room),
            }
        }
        crate::domain::CavernRunPhase::Extraction => CavernObjectiveState {
            kind: CavernObjectiveKind::ReachExtraction,
            title: "Reach extraction".to_string(),
            detail: "The exit is live".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::domain::CavernRunPhase::EliteAvailable => CavernObjectiveState {
            kind: CavernObjectiveKind::HuntElite,
            title: "Defeat the Nest Guardian".to_string(),
            detail: "Push into the elite room".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::domain::CavernRunPhase::Exploring => {
            let combats_cleared = encounters
                .by_room_id
                .values()
                .filter(|room| {
                    room.role == RoomRole::Combat && room.state == RoomEncounterState::Cleared
                })
                .count();
            if combats_cleared >= 1 {
                CavernObjectiveState {
                    kind: CavernObjectiveKind::HuntElite,
                    title: "Defeat the Nest Guardian".to_string(),
                    detail: "Follow the deeper path".to_string(),
                    elite_room: Some(layout.elite_room),
                    extraction_room: Some(layout.extraction_room),
                }
            } else {
                CavernObjectiveState {
                    kind: CavernObjectiveKind::Explore,
                    title: "Explore the caverns".to_string(),
                    detail: "Find the Nest Guardian".to_string(),
                    elite_room: Some(layout.elite_room),
                    extraction_room: Some(layout.extraction_room),
                }
            }
        }
    };

    let extraction = ExtractionState {
        active: run_state.extraction_active,
        room_id: Some(layout.extraction_room),
        countdown_started_at_tick: run_state.extraction_started_at_tick,
        countdown_remaining_seconds: extraction_remaining,
        occupied_by_alive_player: occupied_rooms.contains(&layout.extraction_room),
    };

    world.insert_resource(encounters);
    world.insert_resource(objective);
    world.insert_resource(extraction);
    Ok(())
}

fn room_containing_point(layout: &CavernLayout, point: [f32; 2]) -> Option<crate::domain::RoomId> {
    layout.rooms.iter().find_map(|room| {
        let dx = (point[0] - room.center[0]) / room.radii[0].max(0.1);
        let dy = (point[1] - room.center[1]) / room.radii[1].max(0.1);
        ((dx * dx) + (dy * dy) <= 1.0).then_some(room.id)
    })
}

#[cfg(test)]
mod tests {
    use super::sync_active_player_slots;
    use crate::domain::{
        CavernAimState, CavernCameraState, CavernLayout, CavernMetaPersistenceConfig,
        CavernMetaProfile, CavernMetaRewardState, CavernPlayerOwnershipState, CavernRunConfig,
        CavernRunState, CavernSdfWorldFrame, CavernServerControlMap, LocalPlayerRef,
        LootTableRegistry, PlayerActive, PlayerCompanion, PlayerId, PlayerRosterIdentity,
        SpawnDirector,
    };
    use crate::plugins::worldgen;
    use engine::plugins::ui::domain::UiWorldHudStats;
    use engine::prelude::{
        AuthorityRole, DeterminismLevel, SimulationProfile, SimulationProfileConfig, World,
    };
    use engine::state::SessionRuntimeState;

    #[test]
    fn session_sync_does_not_shrink_default_party_capacity_for_dev_join_state() {
        let session = SessionRuntimeState {
            admitted: true,
            max_players: 1,
            ..SessionRuntimeState::default()
        };
        let mut config = CavernRunConfig {
            max_players: 4,
            ..CavernRunConfig::default()
        };

        super::sync_session_runtime_config(&session, &mut config);

        assert_eq!(config.max_players, 4);
    }

    #[test]
    fn session_settings_json_overrides_game_run_config() {
        let session = SessionRuntimeState {
            admitted: true,
            settings_json: Some(
                r#"{"seed":4242,"enemy_density":1.7,"extract_countdown_seconds":3.5,"base_scrap_reward":19}"#
                    .to_string(),
            ),
            ..SessionRuntimeState::default()
        };
        let mut config = CavernRunConfig::default();

        super::sync_session_runtime_config(&session, &mut config);

        assert_eq!(config.seed.0, 4242);
        assert_eq!(config.enemy_density, 1.7);
        assert_eq!(config.extract_countdown_seconds, 3.5);
        assert_eq!(config.base_scrap_reward, 19);
    }

    #[test]
    fn server_activation_follows_owned_connections() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-test".into()),
            roster_player_codes: vec!["alpha".into(), "beta".into()],
            max_players: 2,
            ai_fill_target: 2,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1), (22, 2)].into_iter().collect(),
        });
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();
        assert_eq!(
            world
                .query::<(engine::prelude::Entity, &PlayerId)>()
                .iter()
                .count(),
            0
        );

        sync_active_player_slots(&mut world).unwrap();

        let active_ids = world
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                world.get::<PlayerActive>(entity).is_some().then(|| {
                    (
                        player_id.0,
                        world
                            .get::<PlayerRosterIdentity>(entity)
                            .map(|identity| identity.player_code.clone())
                            .unwrap_or_default(),
                    )
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            active_ids,
            vec![(1, "alpha".to_string()), (2, "beta".to_string())]
        );
        assert_eq!(
            world
                .resource::<CavernRunState>()
                .unwrap()
                .party_alive_count,
            2
        );
    }

    #[test]
    fn server_can_fill_missing_party_slots_with_companions() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-fill".into()),
            roster_player_codes: vec!["alpha".into()],
            max_players: 4,
            ai_fill_target: 3,
            settings_json: None,
        });
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1)].into_iter().collect(),
        });
        world.insert_resource(CavernSdfWorldFrame::default());
        world.insert_resource(UiWorldHudStats::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();

        sync_active_player_slots(&mut world).unwrap();

        let active_players = world
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                world
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some((entity, player_id.0))
            })
            .collect::<Vec<_>>();
        assert_eq!(active_players.len(), 3);
        assert_eq!(
            active_players
                .iter()
                .filter(|(entity, _)| world.get::<PlayerCompanion>(*entity).is_some())
                .count(),
            2
        );
    }
}
