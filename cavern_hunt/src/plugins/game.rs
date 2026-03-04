use crate::domain::{
    CavernAimState, CavernCameraState, CavernControlState, CavernLayout, CavernMetaProfile,
    CavernPredictionState, CavernRunConfig, CavernRunState, CavernSdfWorldFrame, LocalPlayerRef,
    LootTableRegistry, SpawnDirector,
};
use crate::plugins::{ai, combat, loot, meta, net_sync, render_sdf, worldgen};
use anyhow::Result;
use engine::plugins::ui::domain::UiWorldHudStats;
use engine::prelude::{
    App, CoreSet, Plugin, PreUpdate, RenderPrepare, Res, ResMut, Startup, SystemConfigExt, Update,
    WorldMut,
};
use engine::state::SessionRuntimeState;

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
            sync_session_runtime_config_system.after(CoreSet::NetReceive),
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
