use super::*;

pub(super) fn resolve_enemy_deaths(world: &mut World) -> Result<()> {
    let enemy_entities = {
        let query = world.query_state::<(Entity, &EnemyKind), ()>();
        query
            .iter(world)
            .filter_map(|(entity, kind)| {
                let health = world.get::<Health>(entity).copied()?;
                (health.current <= 0.0).then_some((entity, *kind))
            })
            .collect::<Vec<_>>()
    };
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
            drop(run_state);
            let extraction_center = world.resource::<CavernLayout>().ok().and_then(|layout| {
                layout
                    .room(layout.extraction_room)
                    .map(|room| [room.center[0], room.center[1]])
            });
            if let Some([x, y]) = extraction_center {
                let _ = apply_runtime_geometry_edit(
                    world,
                    &GeometryEdit {
                        kind: GeometryEditKind::RemoveBlocker(extraction_seal_shape(x, y)),
                    },
                );
            }
        }
        {
            let mut run_state = world.resource_mut::<CavernRunState>()?;
            run_state.enemy_kills = run_state.enemy_kills.saturating_add(1);
        }
        let _ = world.despawn(entity);
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
