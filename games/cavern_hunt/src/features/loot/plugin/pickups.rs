use super::spatial::distance_squared;
use super::*;

pub(super) fn collect_pickups(world: &mut World) -> Result<()> {
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
                if world.get::<Chest>(pickup_entity).is_some()
                    && !player_requested_interact(world, *player_entity)
                {
                    continue;
                }
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

fn player_requested_interact(world: &World, player_entity: Entity) -> bool {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    match authority {
        AuthorityRole::Server => {
            let Some(player_id) = world.get::<PlayerId>(player_entity).copied().map(|id| id.0)
            else {
                return false;
            };
            world
                .resource::<CavernServerControlMap>()
                .ok()
                .and_then(|controls| controls.by_player_id.get(&player_id).copied())
                .map(|control| control.interact_pressed)
                .unwrap_or(false)
        }
        _ => world
            .resource::<CavernControlState>()
            .map(|control| control.interact_pressed)
            .unwrap_or(false),
    }
}

pub(super) fn apply_pickup(
    world: &mut World,
    player_entity: Entity,
    pickup: PickupKind,
) -> Result<()> {
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
