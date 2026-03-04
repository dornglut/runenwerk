use crate::domain::{
    CavernMetaProfile, CavernRunConfig, CavernRunPhase, CavernRunState, ColliderRadius, DashState,
    EliteObjective, EnemyKind, ExtractionZone, Health, InventoryRunState, LootDrop,
    LootTableRegistry, Pickup, PickupKind, RelicKind, Transform2, WeaponModKind, WeaponState,
    is_active_player_entity,
};
use crate::plugins::meta;
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, Entity, FixedUpdate, Plugin, SimulationProfileConfig, SimulationRng,
    SimulationTick, World, WorldMut,
};

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

fn resolve_enemy_deaths(world: &mut World) -> Result<()> {
    let enemy_entities = world
        .query::<(Entity, &EnemyKind)>()
        .iter()
        .filter_map(|(entity, kind)| {
            let health = world.get::<Health>(entity).copied()?;
            (health.current <= 0.0).then_some((entity, *kind))
        })
        .collect::<Vec<_>>();
    if enemy_entities.is_empty() {
        return Ok(());
    }

    for (entity, kind) in enemy_entities {
        let transform = world
            .get::<Transform2>(entity)
            .copied()
            .unwrap_or_else(|| Transform2::new(0.0, 0.0, 0.0));
        let drops = build_enemy_drops(world, kind)?;
        for (index, pickup) in drops.into_iter().enumerate() {
            let angle = index as f32 * 1.7;
            world.spawn((
                LootDrop,
                Pickup { kind: pickup },
                Transform2::new(
                    transform.x + angle.cos() * 0.55,
                    transform.y + angle.sin() * 0.55,
                    0.0,
                ),
                ColliderRadius(0.38),
            ));
        }

        if world.get::<EliteObjective>(entity).is_some() {
            let mut run_state = world.resource_mut::<CavernRunState>()?;
            run_state.elite_defeated = true;
            run_state.extraction_active = true;
            run_state.phase = CavernRunPhase::Extraction;
        }
        {
            let mut run_state = world.resource_mut::<CavernRunState>()?;
            run_state.enemy_kills = run_state.enemy_kills.saturating_add(1);
        }
        let _ = world.despawn(entity);
    }

    Ok(())
}

fn collect_pickups(world: &mut World) -> Result<()> {
    let living_players = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter_map(|(entity, transform)| {
            if !is_active_player_entity(world, entity) {
                return None;
            }
            let health = world.get::<Health>(entity).copied()?;
            if health.current <= 0.0 {
                return None;
            }
            let radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.55))
                .0;
            Some((entity, [transform.x, transform.y], radius))
        })
        .collect::<Vec<_>>();
    if living_players.is_empty() {
        return Ok(());
    }

    let pickup_entities = world
        .query::<(Entity, &Pickup)>()
        .iter()
        .map(|(entity, pickup)| (entity, pickup.kind))
        .collect::<Vec<_>>();
    let mut picked = Vec::new();
    for (pickup_entity, pickup_kind) in pickup_entities {
        let Some(transform) = world.get::<Transform2>(pickup_entity).copied() else {
            continue;
        };
        let pickup_radius = world
            .get::<ColliderRadius>(pickup_entity)
            .copied()
            .unwrap_or(ColliderRadius(0.35))
            .0;
        let mut receiver = None;
        for (player_entity, player_pos, player_radius) in &living_players {
            if distance_squared([transform.x, transform.y], *player_pos)
                <= (pickup_radius + *player_radius).powi(2)
            {
                receiver = Some(*player_entity);
                break;
            }
        }
        if let Some(player_entity) = receiver {
            apply_pickup(world, player_entity, pickup_kind)?;
            picked.push(pickup_entity);
        }
    }

    for entity in picked {
        let _ = world.despawn(entity);
    }

    Ok(())
}

fn resolve_run_state(world: &mut World) -> Result<()> {
    let alive_players = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter_map(|(entity, transform)| {
            if !is_active_player_entity(world, entity) {
                return None;
            }
            let health = world.get::<Health>(entity).copied()?;
            (health.current > 0.0).then_some((entity, [transform.x, transform.y]))
        })
        .collect::<Vec<_>>();
    {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        run_state.party_alive_count = alive_players.len() as u8;
    }

    let current_phase = world.resource::<CavernRunState>()?.phase;
    if matches!(
        current_phase,
        CavernRunPhase::Success | CavernRunPhase::Failure
    ) {
        return Ok(());
    }

    let active_player_count = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter(|(entity, _)| is_active_player_entity(world, *entity))
        .count();
    if active_player_count == 0 {
        return Ok(());
    }

    if alive_players.is_empty() {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        run_state.phase = CavernRunPhase::Failure;
        run_state.extraction_started_at_tick = None;
        return Ok(());
    }

    let extraction_active = world.resource::<CavernRunState>()?.extraction_active;
    if !extraction_active {
        return Ok(());
    }

    let extraction_zones = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter_map(|(entity, transform)| {
            world.get::<ExtractionZone>(entity).map(|_| {
                let radius = world
                    .get::<ColliderRadius>(entity)
                    .copied()
                    .unwrap_or(ColliderRadius(1.25))
                    .0;
                ([transform.x, transform.y], radius)
            })
        })
        .collect::<Vec<_>>();
    if extraction_zones.is_empty() {
        return Ok(());
    }

    let mut any_inside = false;
    for (player_entity, player_pos) in &alive_players {
        let player_radius = world
            .get::<ColliderRadius>(*player_entity)
            .copied()
            .unwrap_or(ColliderRadius(0.55))
            .0;
        if extraction_zones.iter().any(|(zone_pos, zone_radius)| {
            distance_squared(*player_pos, *zone_pos) <= (player_radius + *zone_radius).powi(2)
        }) {
            any_inside = true;
            break;
        }
    }

    if !any_inside {
        world
            .resource_mut::<CavernRunState>()?
            .extraction_started_at_tick = None;
        return Ok(());
    }

    let current_tick = world.resource::<SimulationTick>()?.0;
    let countdown_seconds = world
        .resource::<CavernRunConfig>()?
        .extract_countdown_seconds;
    let fixed_dt = world
        .resource::<engine::prelude::Time>()?
        .delta_seconds
        .max(1.0 / 60.0);
    let started_at = {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        *run_state
            .extraction_started_at_tick
            .get_or_insert(SimulationTick(current_tick))
    };

    let elapsed = (current_tick.saturating_sub(started_at.0)) as f32 * fixed_dt;
    if elapsed >= countdown_seconds {
        let total_scrap = total_party_scrap(world);
        {
            let mut run_state = world.resource_mut::<CavernRunState>()?;
            run_state.phase = CavernRunPhase::Success;
        }
        if total_scrap > 0 {
            let updated_profile = {
                let mut profile = world
                    .resource_mut::<CavernMetaProfile>()
                    .unwrap_or_else(|_| {
                        panic!("meta profile should be initialized on game startup")
                    });
                profile.cavern_marks = profile.cavern_marks.saturating_add(total_scrap);
                profile.clone()
            };
            if let Err(err) = meta::save_meta_profile(&updated_profile) {
                tracing::warn!(?err, "failed to persist Cavern Hunt meta profile");
            }
        }
    }

    Ok(())
}

fn build_enemy_drops(world: &mut World, kind: EnemyKind) -> Result<Vec<PickupKind>> {
    let table = {
        let registry = world.resource::<LootTableRegistry>()?;
        match kind {
            EnemyKind::Swarmer => registry.swarmer.clone(),
            EnemyKind::Bruiser => registry.bruiser.clone(),
            EnemyKind::Spitter => registry.spitter.clone(),
            EnemyKind::NestGuardian => registry.elite.clone(),
        }
    };

    let mut drops = Vec::new();
    if table.guaranteed_scrap > 0 {
        drops.push(PickupKind::Scrap(table.guaranteed_scrap));
    }

    let (weapon_roll, healing_roll, relic_roll) = {
        let mut rng = world.resource_mut::<SimulationRng>()?;
        (rng.next_f32(), rng.next_f32(), rng.next_f32())
    };

    if weapon_roll <= table.weapon_mod_chance {
        drops.push(PickupKind::WeaponMod(match kind {
            EnemyKind::Swarmer => WeaponModKind::FireRateUp,
            EnemyKind::Bruiser => WeaponModKind::DamageUp,
            EnemyKind::Spitter => WeaponModKind::ProjectileSpeedUp,
            EnemyKind::NestGuardian => WeaponModKind::PierceOne,
        }));
    }
    if healing_roll <= table.healing_charge_chance {
        drops.push(PickupKind::HealingCharge(match kind {
            EnemyKind::Swarmer => 1,
            EnemyKind::Bruiser | EnemyKind::Spitter => 2,
            EnemyKind::NestGuardian => 3,
        }));
    }
    if relic_roll <= table.relic_chance {
        drops.push(PickupKind::Relic(match kind {
            EnemyKind::NestGuardian => RelicKind::DashCooldownDown,
            EnemyKind::Spitter => RelicKind::CritChanceUp,
            _ => RelicKind::MaxHealthUp,
        }));
    }

    Ok(drops)
}

fn apply_pickup(world: &mut World, player_entity: Entity, pickup: PickupKind) -> Result<()> {
    if let Some(mut inventory) = world.get_mut::<InventoryRunState>(player_entity) {
        match pickup {
            PickupKind::Scrap(amount) => {
                inventory.scrap = inventory.scrap.saturating_add(amount);
            }
            PickupKind::WeaponMod(kind) => {
                inventory.weapon_mods.push(kind);
                drop(inventory);
                if let Some(mut weapon) = world.get_mut::<WeaponState>(player_entity) {
                    match kind {
                        WeaponModKind::DamageUp => weapon.damage += 0.75,
                        WeaponModKind::FireRateUp => {
                            weapon.fire_interval_seconds =
                                (weapon.fire_interval_seconds * 0.9).max(0.12);
                        }
                        WeaponModKind::PierceOne => weapon.damage += 0.35,
                        WeaponModKind::ProjectileSpeedUp => weapon.projectile_speed += 2.0,
                    }
                }
                return Ok(());
            }
            PickupKind::Relic(kind) => {
                inventory.relics.push(kind);
                drop(inventory);
                match kind {
                    RelicKind::MaxHealthUp => {
                        if let Some(mut health) = world.get_mut::<Health>(player_entity) {
                            health.max += 2.0;
                            health.current = (health.current + 2.0).min(health.max);
                        }
                    }
                    RelicKind::DashCooldownDown => {
                        if let Some(mut dash) = world.get_mut::<DashState>(player_entity) {
                            dash.cooldown_seconds = (dash.cooldown_seconds - 0.25).max(0.8);
                        }
                    }
                    RelicKind::CritChanceUp => {
                        if let Some(mut weapon) = world.get_mut::<WeaponState>(player_entity) {
                            weapon.damage += 0.4;
                        }
                    }
                }
                return Ok(());
            }
            PickupKind::HealingCharge(amount) => {
                drop(inventory);
                if let Some(mut health) = world.get_mut::<Health>(player_entity) {
                    health.current = (health.current + amount as f32).min(health.max);
                }
                return Ok(());
            }
        }
    }
    Ok(())
}

fn total_party_scrap(world: &World) -> u32 {
    world
        .query::<(Entity, &InventoryRunState)>()
        .iter()
        .filter_map(|(entity, inventory)| {
            is_active_player_entity(world, entity).then_some(inventory.scrap)
        })
        .sum()
}

fn distance_squared(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::{apply_pickup, resolve_enemy_deaths};
    use crate::domain::{
        CavernMetaProfile, CavernRunConfig, CavernRunState, Health, InventoryRunState,
        LootTableRegistry, PickupKind, SpawnDirector,
    };
    use crate::plugins::worldgen;
    use engine::prelude::{Entity, SimulationRng, SimulationSeed, World};

    #[test]
    fn elite_death_activates_extraction() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(crate::domain::LocalPlayerRef::default());
        world.insert_resource(crate::domain::CavernCameraState::default());
        world.insert_resource(SimulationRng::from_seed(SimulationSeed(7)));
        worldgen::initialize_run_world(&mut world, true).unwrap();

        let elite = world
            .query::<(Entity, &crate::domain::EnemyKind)>()
            .iter()
            .find(|(_, kind)| **kind == crate::domain::EnemyKind::NestGuardian)
            .map(|(entity, _)| entity)
            .unwrap();
        world.get_mut::<Health>(elite).unwrap().current = 0.0;
        resolve_enemy_deaths(&mut world).unwrap();

        let run_state = world.resource::<CavernRunState>().unwrap();
        assert!(run_state.elite_defeated);
        assert!(run_state.extraction_active);
    }

    #[test]
    fn scrap_pickup_adds_to_inventory() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(crate::domain::LocalPlayerRef::default());
        world.insert_resource(crate::domain::CavernCameraState::default());
        world.insert_resource(SimulationRng::from_seed(SimulationSeed(9)));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        let player = world
            .resource::<crate::domain::LocalPlayerRef>()
            .unwrap()
            .entity
            .unwrap();

        apply_pickup(&mut world, player, PickupKind::Scrap(5)).unwrap();

        let inventory = world.get::<InventoryRunState>(player).unwrap();
        assert_eq!(inventory.scrap, 5);
    }
}
