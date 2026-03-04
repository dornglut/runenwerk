use crate::domain::{
    CavernAimState, CavernCameraState, CavernControlState, CavernLayout, CavernMetaProfile,
    CavernPlayerOwnershipState, CavernPredictionState, CavernRunConfig, CavernRunState,
    CavernSdfWorldFrame, CavernServerControlMap, LocalPlayerRef, LootTableRegistry, PlayerActive,
    PlayerId, SpawnDirector,
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
    if session.max_players > 0 {
        config.max_players = session.max_players.max(1);
    }
    Ok(())
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
    let max_players = world
        .resource::<CavernRunConfig>()
        .map(|config| config.max_players.max(1))
        .unwrap_or(1);
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
        }
    }

    let player_entities = world
        .query::<(engine::prelude::Entity, &PlayerId)>()
        .iter()
        .map(|(entity, player_id)| (entity, player_id.0))
        .collect::<Vec<_>>();
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
        CavernAimState, CavernCameraState, CavernLayout, CavernMetaProfile,
        CavernPlayerOwnershipState, CavernRunConfig, CavernRunState, CavernSdfWorldFrame,
        CavernServerControlMap, LocalPlayerRef, LootTableRegistry, PlayerActive, PlayerId,
        SpawnDirector,
    };
    use crate::plugins::worldgen;
    use engine::plugins::ui::domain::UiWorldHudStats;
    use engine::prelude::{
        AuthorityRole, DeterminismLevel, SimulationProfile, SimulationProfileConfig, World,
    };

    #[test]
    fn server_activation_follows_owned_connections() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
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

        sync_active_player_slots(&mut world).unwrap();

        let active_ids = world
            .query::<(engine::prelude::Entity, &PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                world
                    .get::<PlayerActive>(entity)
                    .is_some()
                    .then_some(player_id.0)
            })
            .collect::<Vec<_>>();
        assert_eq!(active_ids, vec![1, 2]);
        assert_eq!(
            world
                .resource::<CavernRunState>()
                .unwrap()
                .party_alive_count,
            2
        );
    }
}
