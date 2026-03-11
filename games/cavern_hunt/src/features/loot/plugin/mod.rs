use crate::features::timing::fixed_step_seconds;
use crate::features::worldgen::plugin::apply_runtime_geometry_edit;
use crate::{
    CavernControlState, CavernGeometryRuntimeState, CavernRunConfig, CavernRunPhase,
    CavernRunState, CavernServerControlMap, Chest, ColliderRadius, DashState, EliteObjective,
    EnemyKind, ExtractionZone, GeometryEdit, GeometryEditKind, Health, InventoryRunState, LootDrop,
    LootTableRegistry, Pickup, PickupKind, PlayerId, PlayerSpectator, RelicKind, Transform2,
    WeaponModKind, WeaponState, is_active_player_entity,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, Entity, FixedUpdate, Plugin, SimulationProfileConfig, SimulationRng,
    SimulationTick, World, WorldMut,
};

mod enemy_resolution;
mod pickups;
mod run_state;
mod spatial;

#[cfg(test)]
mod tests;

use enemy_resolution::resolve_enemy_deaths;
use pickups::collect_pickups;
use run_state::resolve_run_state;

pub struct CavernHuntLootPlugin;

impl Plugin for CavernHuntLootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, resolve_loot_and_run_state_system);
    }
}

fn resolve_loot_and_run_state_system(mut world: WorldMut) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }

    resolve_enemy_deaths(&mut world)?;
    collect_pickups(&mut world)?;
    resolve_run_state(&mut world)?;
    Ok(())
}
