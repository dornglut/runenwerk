use crate::{
    CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState, CavernRunPhase,
    CavernRunState, InventoryRunState, LocalPlayerRef, PlayerId,
};
use anyhow::{Context, Result};
use engine::prelude::{AuthorityRole, SimulationProfileConfig, World, WorldMut};
use std::path::Path;

pub const META_PROFILE_PATH: &str = "var/cavern_hunt/meta_profile.json";

pub fn load_meta_profile(world: &mut World) -> Result<()> {
    let path = Path::new(META_PROFILE_PATH);
    let profile = match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<CavernMetaProfile>(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => CavernMetaProfile::default(),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };
    world.insert_resource(profile);
    Ok(())
}

pub fn save_meta_profile(profile: &CavernMetaProfile) -> Result<()> {
    let path = Path::new(META_PROFILE_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(profile).context("failed to serialize meta profile")?;
    std::fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))
}

pub fn apply_run_meta_rewards_system(mut world: WorldMut) -> Result<()> {
    apply_run_meta_rewards(&mut world)
}

pub(crate) fn apply_run_meta_rewards(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }

    let run_state = match world.resource::<CavernRunState>() {
        Ok(run_state) => run_state.clone(),
        Err(_) => return Ok(()),
    };
    if !matches!(run_state.phase, CavernRunPhase::Success) {
        return Ok(());
    }

    let already_awarded = world
        .resource::<CavernMetaRewardState>()
        .ok()
        .and_then(|state| state.last_awarded_run_id)
        == Some(run_state.run_id);
    if already_awarded {
        return Ok(());
    }

    let Some(reward) = resolve_local_player_scrap_reward(world) else {
        return Ok(());
    };
    let should_persist = world
        .resource::<CavernMetaPersistenceConfig>()
        .map(|config| config.enabled)
        .unwrap_or(true);
    {
        let mut profile = world.resource_mut::<CavernMetaProfile>()?;
        profile.cavern_marks = profile.cavern_marks.saturating_add(reward);
        let snapshot = profile.clone();
        drop(profile);
        if should_persist {
            save_meta_profile(&snapshot)?;
        }
    }

    if let Ok(mut reward_state) = world.resource_mut::<CavernMetaRewardState>() {
        reward_state.last_awarded_run_id = Some(run_state.run_id);
    }

    Ok(())
}

fn resolve_local_player_scrap_reward(world: &World) -> Option<u32> {
    let local_player = match world.resource::<LocalPlayerRef>() {
        Ok(local) => local.clone(),
        Err(_) => return None,
    };
    if let Some(entity) = local_player.entity
        && let Some(inventory) = world.get::<InventoryRunState>(entity)
    {
        return Some(inventory.scrap);
    }

    let Some(player_id) = local_player.player_id else {
        return None;
    };
    let query = world.query_state::<(engine::prelude::Entity, &PlayerId), ()>();
    query.iter(world).find_map(|(entity, current_player_id)| {
        (current_player_id.0 == player_id)
            .then(|| {
                world
                    .get::<InventoryRunState>(entity)
                    .map(|inventory| inventory.scrap)
            })
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::apply_run_meta_rewards;
    use crate::{
        CavernMetaPersistenceConfig, CavernMetaProfile, CavernMetaRewardState, CavernRunPhase,
        CavernRunState, InventoryRunState, LocalPlayerRef, Player, PlayerActive, PlayerId,
        Transform2,
    };
    use engine::prelude::{
        AuthorityRole, DeterminismLevel, SimulationProfile, SimulationProfileConfig, World,
    };

    #[test]
    fn client_reward_is_applied_once_for_successful_run() {
        let mut world = World::new();
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(CavernRunState {
            run_id: 42,
            phase: CavernRunPhase::Success,
            ..CavernRunState::default()
        });
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Client,
            determinism: DeterminismLevel::Validated,
        });
        let entity = world.spawn((
            Player,
            PlayerId(1),
            PlayerActive,
            Transform2::new(0.0, 0.0, 0.0),
            InventoryRunState {
                scrap: 17,
                weapon_mods: Vec::new(),
                relics: Vec::new(),
            },
        ));
        world.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: Some(entity),
        });

        apply_run_meta_rewards(&mut world).unwrap();
        apply_run_meta_rewards(&mut world).unwrap();

        assert_eq!(
            world.resource::<CavernMetaProfile>().unwrap().cavern_marks,
            17
        );
        assert_eq!(
            world
                .resource::<CavernMetaRewardState>()
                .unwrap()
                .last_awarded_run_id,
            Some(42)
        );
    }

    #[test]
    fn server_does_not_apply_local_meta_rewards() {
        let mut world = World::new();
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(CavernMetaPersistenceConfig { enabled: false });
        world.insert_resource(CavernMetaRewardState::default());
        world.insert_resource(CavernRunState {
            run_id: 7,
            phase: CavernRunPhase::Success,
            ..CavernRunState::default()
        });
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: DeterminismLevel::Validated,
        });

        apply_run_meta_rewards(&mut world).unwrap();

        assert_eq!(
            world.resource::<CavernMetaProfile>().unwrap().cavern_marks,
            0
        );
        assert_eq!(
            world
                .resource::<CavernMetaRewardState>()
                .unwrap()
                .last_awarded_run_id,
            None
        );
    }
}
