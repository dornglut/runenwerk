use super::*;

// Owner: Cavern Hunt Combat Plugin - Player Movement and Weapon Fire
pub(super) fn move_player_with_control(
    world: &mut World,
    entity: Entity,
    control: CavernControlState,
    dt: f32,
) -> Result<()> {
    let phase = world.resource::<CavernRunState>()?.phase;
    if matches!(phase, CavernRunPhase::Success | CavernRunPhase::Failure) {
        return Ok(());
    }

    let move_input = control.movement;
    let dash_pressed = control.dash_pressed;
    if let Some(mut aim) = world.get_mut::<AimTarget2>(entity) {
        aim.x = control.aim_world[0];
        aim.y = control.aim_world[1];
    }

    let graph = world.resource::<CavernGeometryGraph>()?.clone();
    let Some(health) = world.get::<Health>(entity).copied() else {
        return Ok(());
    };
    if health.current <= 0.0 {
        let _ = world.insert(entity, PlayerSpectator);
        if let Some(mut velocity) = world.get_mut::<Velocity2>(entity) {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return Ok(());
    }

    let current = world
        .get::<Transform2>(entity)
        .copied()
        .unwrap_or_else(|| Transform2::new(0.0, 0.0, 0.0));
    let radius = world
        .get::<ColliderRadius>(entity)
        .copied()
        .unwrap_or(ColliderRadius(0.45))
        .0;

    let move_speed = world
        .resource::<PlayerCombatTuning>()
        .map(|tuning| tuning.move_speed)
        .unwrap_or(5.5);
    let mut delta = [
        move_input[0] * move_speed * dt,
        move_input[1] * move_speed * dt,
    ];
    if dash_pressed && (move_input[0].abs() > f32::EPSILON || move_input[1].abs() > f32::EPSILON) {
        if let Some(mut dash) = world.get_mut::<DashState>(entity) {
            if dash.cooldown_remaining <= f32::EPSILON {
                delta = [
                    move_input[0] * dash.dash_distance,
                    move_input[1] * dash.dash_distance,
                ];
                dash.cooldown_remaining = dash.cooldown_seconds;
                dash.invulnerability_remaining = dash.invulnerability_seconds;
            }
        }
    }

    let next = {
        let mut field = world.resource_mut::<CavernCollisionField>()?;
        constrained_move(
            &mut field,
            &graph,
            [current.x, current.y],
            delta,
            movement_footprint_radius(radius),
        )
    };
    let aim = world.get::<AimTarget2>(entity).copied();
    if let Some(mut transform) = world.get_mut::<Transform2>(entity) {
        transform.x = next[0];
        transform.y = next[1];
        if let Some(aim) = aim {
            let facing = [aim.x - transform.x, aim.y - transform.y];
            if facing[0].abs() > f32::EPSILON || facing[1].abs() > f32::EPSILON {
                transform.yaw = facing[1].atan2(facing[0]);
            }
        }
    }
    if let Some(mut velocity) = world.get_mut::<Velocity2>(entity) {
        velocity.x = (next[0] - current.x) / dt.max(0.0001);
        velocity.y = (next[1] - current.y) / dt.max(0.0001);
    }

    Ok(())
}

pub(super) fn fire_player_weapon_with_control(
    world: &mut World,
    entity: Entity,
    control: CavernControlState,
) -> Result<()> {
    let phase = world.resource::<CavernRunState>()?.phase;
    if matches!(phase, CavernRunPhase::Success | CavernRunPhase::Failure) {
        return Ok(());
    }

    let should_fire = control.fire_pressed;
    if !should_fire {
        return Ok(());
    }

    let health = world
        .get::<Health>(entity)
        .copied()
        .unwrap_or_else(|| Health::new(1.0));
    if health.current <= 0.0 {
        let _ = world.insert(entity, PlayerSpectator);
        return Ok(());
    }

    let weapon = world
        .get::<WeaponState>(entity)
        .copied()
        .unwrap_or_default();
    if weapon.cooldown_remaining > f32::EPSILON {
        return Ok(());
    }

    let transform = world
        .get::<Transform2>(entity)
        .copied()
        .unwrap_or_else(|| Transform2::new(0.0, 0.0, 0.0));
    let aim = world
        .get::<AimTarget2>(entity)
        .copied()
        .unwrap_or(AimTarget2 {
            x: transform.x + 1.0,
            y: transform.y,
        });
    let direction = normalized_vector(aim.x - transform.x, aim.y - transform.y);
    let origin = [
        transform.x + direction.0 * 0.9,
        transform.y + direction.1 * 0.9,
    ];
    spawn_projectile(
        world,
        origin,
        [direction.0, direction.1],
        weapon.projectile_speed,
        weapon.damage,
        Faction::Hunters,
    );
    if let Some(mut weapon) = world.get_mut::<WeaponState>(entity) {
        weapon.cooldown_remaining = weapon.fire_interval_seconds;
    }
    let _ = world.insert(
        entity,
        DamageFeedbackState {
            last_damage_taken: world
                .get::<DamageFeedbackState>(entity)
                .map(|feedback| feedback.last_damage_taken)
                .unwrap_or(0.0),
            last_damage_dealt: weapon.damage,
        },
    );

    Ok(())
}

pub(super) fn resolve_local_player_entity(world: &World) -> Option<Entity> {
    let local = world.resource::<LocalPlayerRef>().ok()?;
    if let Some(entity) = local.entity
        && world.get::<PlayerId>(entity).is_some()
        && world.get::<PlayerActive>(entity).is_some()
    {
        return Some(entity);
    }
    if let Some(player_id) = local.player_id {
        return world
            .query::<(Entity, &PlayerId)>()
            .iter()
            .find_map(|(entity, id)| {
                (id.0 == player_id && world.get::<PlayerActive>(entity).is_some()).then_some(entity)
            });
    }
    world
        .query::<(Entity, &PlayerId)>()
        .iter()
        .find_map(|(entity, _)| {
            world
                .get::<PlayerActive>(entity)
                .is_some()
                .then_some(entity)
        })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum ProjectileStepMode {
    Authoritative,
    PredictedLocal,
}
