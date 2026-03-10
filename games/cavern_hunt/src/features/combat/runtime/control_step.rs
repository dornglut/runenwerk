use super::*;

// Owner: Cavern Hunt Combat Plugin - Control and Fixed-Step Orchestration
pub(super) fn fixed_step_combat_system(mut world: WorldMut) -> Result<()> {
    let dt = combat_fixed_step_seconds(&world);
    if dt <= f32::EPSILON {
        return Ok(());
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    match authority {
        AuthorityRole::Client | AuthorityRole::Peer => run_predicted_local_step(&mut world, dt),
        AuthorityRole::Local | AuthorityRole::Server => {
            run_authoritative_combat_step(&mut world, dt)
        }
    }
}

pub(super) fn run_authoritative_combat_step(world: &mut World, dt: f32) -> Result<()> {
    tick_cooldowns(world, dt);
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Server) {
        step_server_controlled_players(world, dt)?;
    } else {
        step_local_controlled_player(world, dt)?;
    }
    step_projectiles(world, dt, ProjectileStepMode::Authoritative)?;
    Ok(())
}

fn run_predicted_local_step(world: &mut World, dt: f32) -> Result<()> {
    tick_local_player_cooldowns(world, dt);
    step_local_controlled_player(world, dt)?;
    step_projectiles(world, dt, ProjectileStepMode::PredictedLocal)?;
    Ok(())
}

pub(crate) fn replay_predicted_local_frame(
    world: &mut World,
    control: CavernControlState,
    dt: f32,
) -> Result<()> {
    let previous = world
        .resource::<CavernControlState>()
        .copied()
        .unwrap_or_default();
    world.insert_resource(control);
    let result = run_predicted_local_step(world, dt);
    world.insert_resource(previous);
    result
}

fn tick_cooldowns(world: &mut World, dt: f32) {
    let weapon_entities = world
        .query::<(Entity, &WeaponState)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in weapon_entities {
        if let Some(mut weapon) = world.get_mut::<WeaponState>(entity) {
            weapon.cooldown_remaining = (weapon.cooldown_remaining - dt).max(0.0);
        }
    }

    let dash_entities = world
        .query::<(Entity, &DashState)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in dash_entities {
        if let Some(mut dash) = world.get_mut::<DashState>(entity) {
            dash.cooldown_remaining = (dash.cooldown_remaining - dt).max(0.0);
            dash.invulnerability_remaining = (dash.invulnerability_remaining - dt).max(0.0);
        }
    }

    let flashed_entities = world
        .query::<(Entity, &HitFlashState)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    for entity in flashed_entities {
        let mut clear = false;
        if let Some(mut flash) = world.get_mut::<HitFlashState>(entity) {
            flash.remaining_seconds = (flash.remaining_seconds - dt).max(0.0);
            clear = flash.remaining_seconds <= f32::EPSILON;
        }
        if clear {
            let _ = world.remove::<HitFlashState>(entity);
        }
    }
}

fn tick_local_player_cooldowns(world: &mut World, dt: f32) {
    let Some(entity) = resolve_local_player_entity(world) else {
        return;
    };

    if let Some(mut weapon) = world.get_mut::<WeaponState>(entity) {
        weapon.cooldown_remaining = (weapon.cooldown_remaining - dt).max(0.0);
    }
    if let Some(mut dash) = world.get_mut::<DashState>(entity) {
        dash.cooldown_remaining = (dash.cooldown_remaining - dt).max(0.0);
        dash.invulnerability_remaining = (dash.invulnerability_remaining - dt).max(0.0);
    }
}

fn step_local_controlled_player(world: &mut World, dt: f32) -> Result<()> {
    let Some(entity) = resolve_local_player_entity(world) else {
        return Ok(());
    };
    let control = world.resource::<CavernControlState>()?.to_owned();
    move_player_with_control(world, entity, control, dt)?;
    fire_player_weapon_with_control(world, entity, control)
}

fn step_server_controlled_players(world: &mut World, dt: f32) -> Result<()> {
    let controls = world
        .resource::<CavernServerControlMap>()
        .cloned()
        .unwrap_or_default();
    let mut applied_input_ticks = world
        .resource::<CavernServerAppliedInputTickMap>()
        .cloned()
        .unwrap_or_default();
    let players = world
        .query::<(Entity, &PlayerId)>()
        .iter()
        .filter_map(|(entity, player_id)| {
            world
                .get::<PlayerActive>(entity)
                .is_some()
                .then_some((entity, player_id.0))
        })
        .collect::<Vec<_>>();
    let mut active_player_ids = BTreeSet::new();
    for (entity, player_id) in players {
        active_player_ids.insert(player_id);
        let mut control = if world.get::<PlayerCompanion>(entity).is_some() {
            controls
                .by_player_id
                .get(&player_id)
                .copied()
                .unwrap_or_else(|| build_companion_control(world, entity))
        } else {
            controls
                .by_player_id
                .get(&player_id)
                .copied()
                .unwrap_or_default()
        };
        if control.source_tick == SimulationTick::default() {
            control.source_tick = world
                .resource::<SimulationTick>()
                .copied()
                .unwrap_or_default();
        }
        move_player_with_control(world, entity, control, dt)?;
        fire_player_weapon_with_control(world, entity, control)?;
        applied_input_ticks
            .by_player_id
            .insert(player_id, control.source_tick);
    }
    applied_input_ticks
        .by_player_id
        .retain(|player_id, _| active_player_ids.contains(player_id));
    world.insert_resource(applied_input_ticks);
    Ok(())
}

fn build_companion_control(world: &World, entity: Entity) -> CavernControlState {
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let Some(transform) = world.get::<Transform2>(entity).copied() else {
        return CavernControlState {
            source_tick: tick,
            ..CavernControlState::default()
        };
    };
    let Some(health) = world.get::<Health>(entity).copied() else {
        return CavernControlState {
            source_tick: tick,
            ..CavernControlState::default()
        };
    };
    if health.current <= 0.0 {
        return CavernControlState {
            source_tick: tick,
            ..CavernControlState::default()
        };
    }

    let companion_role = world
        .get::<PlayerCompanion>(entity)
        .copied()
        .map(|companion| companion.behavior_role());
    let nearest_enemy = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter_map(|(candidate, enemy_transform)| {
            let kind = world.get::<EnemyKind>(candidate)?;
            let enemy_health = world.get::<Health>(candidate).copied()?;
            if enemy_health.current <= 0.0 {
                return None;
            }
            let dx = enemy_transform.x - transform.x;
            let dy = enemy_transform.y - transform.y;
            let distance_sq = dx * dx + dy * dy;
            let priority = if distance_sq <= 14.0_f32.powi(2) {
                match (companion_role, *kind) {
                    (_, EnemyKind::NestGuardian) => 0_u8,
                    (_, EnemyKind::Spitter) => 1,
                    (
                        Some(crate::CompanionBehaviorRole::Skirmisher),
                        EnemyKind::Bruiser,
                    ) => 2,
                    (
                        Some(crate::CompanionBehaviorRole::SupportShooter),
                        EnemyKind::Bruiser,
                    ) => 3,
                    (_, EnemyKind::Bruiser) => 2,
                    (_, EnemyKind::Swarmer) => 4,
                }
            } else {
                10
            };
            Some((
                priority,
                distance_sq,
                [enemy_transform.x, enemy_transform.y],
            ))
        })
        .min_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));

    let Some((_, distance_sq, enemy_pos)) = nearest_enemy else {
        return CavernControlState {
            aim_world: [transform.x + 1.0, transform.y],
            source_tick: tick,
            ..CavernControlState::default()
        };
    };

    let dx = enemy_pos[0] - transform.x;
    let dy = enemy_pos[1] - transform.y;
    let (move_x, move_y) = normalized_vector(dx, dy);
    let distance = distance_sq.sqrt();
    let desired_range = match companion_role {
        Some(crate::CompanionBehaviorRole::Skirmisher) => 4.0,
        Some(crate::CompanionBehaviorRole::SupportShooter) => 6.5,
        None => 5.0,
    };
    let movement = if distance > desired_range {
        [move_x, move_y]
    } else if distance < 2.0 {
        [-move_x, -move_y]
    } else {
        [0.0, 0.0]
    };

    CavernControlState {
        movement,
        aim_world: enemy_pos,
        fire_pressed: distance <= 12.0,
        dash_pressed: false,
        interact_pressed: false,
        source_tick: tick,
    }
}
