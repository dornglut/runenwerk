use crate::domain::{
    CavernAimState, CavernCameraState, CavernControlState, CavernLayout,
    CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState,
    CavernPlayerOwnershipState, CavernPredictionState, CavernRunConfig, CavernRunState,
    CavernSdfWorldFrame, CavernServerControlMap, CavernSessionSettings, LocalPlayerRef,
    LootTableRegistry, PlayerActive, PlayerId, SpawnDirector,
};
use crate::plugins::{ai, combat, loot, meta, net_sync, render_sdf, worldgen};
use anyhow::Result;
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Plugin, PreUpdate, RenderPrepare, Res, ResMut,
    SimulationProfileConfig, Startup, SystemConfigExt, Update, World, WorldMut,
};
use engine::state::SessionRuntimeState;
use std::collections::BTreeSet;

pub struct CavernHuntPlugin;
pub struct CavernHuntClientPlugin;
pub struct CavernHuntServerPlugin;

impl Plugin for CavernHuntPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernRunConfig>();
        app.init_resource::<CavernRunState>();
        app.init_resource::<CavernLayout>();
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
        app.init_resource::<UiWorldHudStats>();
        app.add_plugins((
            combat::CavernHuntCombatPlugin,
            ai::CavernHuntAiPlugin,
            loot::CavernHuntLootPlugin,
            net_sync::CavernHuntNetSyncPlugin,
        ));
        app.add_systems(
            PreUpdate,
            (
                sync_session_runtime_config_system.after(CoreSet::NetReceive),
                sync_active_player_slots_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(PreUpdate, meta::apply_run_meta_rewards_system);
    }
}

impl Plugin for CavernHuntClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, client_setup_system);
        app.add_systems(Update, render_sdf::update_camera_and_hud_system);
        app.add_systems(RenderPrepare, render_sdf::build_sdf_world_frame_system);
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
    let ownership = world
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
            let entity = worldgen::spawn_player_entity(
                world,
                player_id,
                spawn_index,
                true,
                &meta_profile,
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
