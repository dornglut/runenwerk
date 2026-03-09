use crate::{
    ClientNetworkConfigAssetV1, NetConfigHotReloadState, ServerNetworkConfigAssetV1,
    load_client_network_config_from_path, load_server_network_config_from_path,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, Plugin, PreUpdate, SimulationProfileConfig, Startup, Time, World, WorldMut,
};

pub struct CavernHuntNetConfigPlugin;

impl Plugin for CavernHuntNetConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_loaded_network_config_system);
        app.add_systems(PreUpdate, network_config_hot_reload_system);
    }
}

fn apply_loaded_network_config_system(mut world: WorldMut) -> Result<()> {
    apply_loaded_network_config(&mut world)
}

fn network_config_hot_reload_system(mut world: WorldMut) -> Result<()> {
    let mut state = match world.resource::<NetConfigHotReloadState>() {
        Ok(state) => state.clone(),
        Err(_) => return Ok(()),
    };
    if !state.enabled {
        return Ok(());
    }

    let delta = world
        .resource::<Time>()
        .map(|time| time.delta_seconds.max(0.0))
        .unwrap_or(0.0);
    state.accumulator_seconds += delta;
    if state.accumulator_seconds < state.poll_interval_seconds {
        world.insert_resource(state);
        return Ok(());
    }
    state.accumulator_seconds = 0.0;

    let modified = std::fs::metadata(&state.path)
        .ok()
        .and_then(|meta| meta.modified().ok());
    if modified.is_some() && modified == state.last_modified {
        world.insert_resource(state);
        return Ok(());
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let path = state.path.clone();
    match authority {
        AuthorityRole::Server => match load_server_network_config_from_path(&path) {
            Ok(config) => {
                world.insert_resource(config.clone());
                apply_server_network_config(&mut world, &config);
                state.last_modified = modified;
                tracing::info!(path = %path.display(), "reloaded server network config");
            }
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    ?error,
                    "failed to hot reload server network config"
                );
            }
        },
        _ => match load_client_network_config_from_path(&path) {
            Ok(config) => {
                world.insert_resource(config.clone());
                apply_client_network_config(&mut world, &config);
                state.last_modified = modified;
                tracing::info!(path = %path.display(), "reloaded client network config");
            }
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    ?error,
                    "failed to hot reload client network config"
                );
            }
        },
    }

    world.insert_resource(state);
    Ok(())
}

pub(crate) fn apply_loaded_network_config(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    match authority {
        AuthorityRole::Server => {
            if let Ok(config) = world.resource::<ServerNetworkConfigAssetV1>() {
                apply_server_network_config(world, &config.clone());
            }
        }
        AuthorityRole::Client | AuthorityRole::Peer => {
            if let Ok(config) = world.resource::<ClientNetworkConfigAssetV1>() {
                apply_client_network_config(world, &config.clone());
            }
        }
        AuthorityRole::Local => {
            if let Ok(config) = world.resource::<ClientNetworkConfigAssetV1>() {
                apply_client_network_config(world, &config.clone());
            } else if let Ok(config) = world.resource::<ServerNetworkConfigAssetV1>() {
                apply_server_network_config(world, &config.clone());
            }
        }
    }
    Ok(())
}

fn apply_client_network_config(world: &mut World, config: &ClientNetworkConfigAssetV1) {
    world.insert_resource(config.interpolation);
    world.insert_resource(config.diagnostics);
}

fn apply_server_network_config(world: &mut World, config: &ServerNetworkConfigAssetV1) {
    world.insert_resource(config.replication_budget);
    world.insert_resource(config.replication_cadence);
    world.insert_resource(config.load_shed);
    world.insert_resource(config.keyframe);
    world.insert_resource(config.diagnostics);
}
