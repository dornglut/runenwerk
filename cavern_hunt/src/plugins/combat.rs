use crate::domain::{
    AimTarget2, CavernAimState, CavernControlState, CavernLayout, CavernRunPhase, CavernRunState,
    CavernServerControlMap, ColliderRadius, DashState, Faction, Health, LocalPlayerRef, PlayerId,
    Projectile, Transform2, Velocity2, WeaponState,
};
use crate::plugins::render_sdf;
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, Entity, FixedUpdate, InputState, Plugin, PreUpdate,
    SimulationProfileConfig, SimulationTick, SystemConfigExt, Time, WindowState, World, WorldMut,
};

pub struct CavernHuntCombatPlugin;

impl Plugin for CavernHuntCombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_local_aim_system.in_set(CoreSet::Input));
        app.add_systems(
            FixedUpdate,
            fixed_step_combat_system.in_set(CoreSet::Simulation),
        );
    }
}

fn update_local_aim_system(mut world: WorldMut) -> Result<()> {
    update_local_aim(&mut world)
}

pub(crate) fn update_local_aim(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }

    let layout = world.resource::<CavernLayout>()?.clone();
    let camera = world
        .resource::<crate::domain::CavernCameraState>()?
        .clone();
    let window_size = {
        let window = world.resource::<WindowState>()?;
        window.clone()
    };
    let cursor = {
        let input = world.resource::<InputState>()?;
        input.mouse_position
    };
    let local_entity = world.resource::<LocalPlayerRef>()?.entity;

    let aim_world = render_sdf::project_mouse_to_world(&camera, &window_size, &layout, cursor);
    world.insert_resource(CavernAimState {
        world_point: aim_world,
    });
    let (movement, fire_pressed, dash_pressed) = {
        let input = world.resource::<InputState>()?;
        let movement = normalized_vector(
            (input.world_move_right as i32 - input.world_move_left as i32) as f32,
            (input.world_move_up as i32 - input.world_move_down as i32) as f32,
        );
        (
            [movement.0, movement.1],
            input.left_mouse_down(),
            input.right_mouse_down(),
        )
    };
    let next_tick = world
        .resource::<SimulationTick>()
        .copied()
        .map(|tick| SimulationTick(tick.0.saturating_add(1)))
        .unwrap_or_default();
    if let Ok(mut control) = world.resource_mut::<CavernControlState>() {
        control.movement = movement;
        control.aim_world = aim_world;
        control.fire_pressed = fire_pressed;
        control.dash_pressed = dash_pressed;
        control.source_tick = next_tick;
    }

    if let Some(entity) = local_entity {
        if let Some(mut aim) = world.get_mut::<AimTarget2>(entity) {
            aim.x = aim_world[0];
            aim.y = aim_world[1];
        }
    }

    Ok(())
}

fn fixed_step_combat_system(mut world: WorldMut) -> Result<()> {
    let dt = world.resource::<Time>()?.delta_seconds.max(0.0);
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

fn run_authoritative_combat_step(world: &mut World, dt: f32) -> Result<()> {
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
    let players = world
        .query::<(Entity, &PlayerId)>()
        .iter()
        .map(|(entity, player_id)| (entity, player_id.0))
        .collect::<Vec<_>>();
    for (entity, player_id) in players {
        let mut control = controls
            .by_player_id
            .get(&player_id)
            .copied()
            .unwrap_or_default();
        if control.source_tick == SimulationTick::default() {
            control.source_tick = world
                .resource::<SimulationTick>()
                .copied()
                .unwrap_or_default();
        }
        move_player_with_control(world, entity, control, dt)?;
        fire_player_weapon_with_control(world, entity, control)?;
    }
    Ok(())
}

fn move_player_with_control(
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

    let layout = world.resource::<CavernLayout>()?.clone();
    let Some(health) = world.get::<Health>(entity).copied() else {
        return Ok(());
    };
    if health.current <= 0.0 {
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

    let mut delta = [move_input[0] * 5.5 * dt, move_input[1] * 5.5 * dt];
    if dash_pressed && (move_input[0].abs() > f32::EPSILON || move_input[1].abs() > f32::EPSILON) {
        if let Some(mut dash) = world.get_mut::<DashState>(entity) {
            if dash.cooldown_remaining <= f32::EPSILON {
                delta = [
                    move_input[0] * dash.dash_distance,
                    move_input[1] * dash.dash_distance,
                ];
                dash.cooldown_remaining = dash.cooldown_seconds;
            }
        }
    }

    let next = constrained_move(&layout, [current.x, current.y], delta, radius);
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

fn fire_player_weapon_with_control(
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

    Ok(())
}

fn resolve_local_player_entity(world: &World) -> Option<Entity> {
    let local = world.resource::<LocalPlayerRef>().ok()?;
    if let Some(entity) = local.entity
        && world.get::<PlayerId>(entity).is_some()
    {
        return Some(entity);
    }
    if let Some(player_id) = local.player_id {
        return world
            .query::<(Entity, &PlayerId)>()
            .iter()
            .find_map(|(entity, id)| (id.0 == player_id).then_some(entity));
    }
    world
        .query::<(Entity, &PlayerId)>()
        .iter()
        .map(|(entity, _)| entity)
        .next()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ProjectileStepMode {
    Authoritative,
    PredictedLocal,
}

fn step_projectiles(world: &mut World, dt: f32, mode: ProjectileStepMode) -> Result<()> {
    let phase = world.resource::<CavernRunState>()?.phase;
    if matches!(phase, CavernRunPhase::Success | CavernRunPhase::Failure) {
        return Ok(());
    }

    let layout = world.resource::<CavernLayout>()?.clone();
    let projectile_entities = world
        .query::<(Entity, &Projectile)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let target_entities = world
        .query::<(Entity, &Health)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let mut despawns = Vec::new();

    for entity in projectile_entities {
        let Some(projectile) = world.get::<Projectile>(entity).copied() else {
            continue;
        };
        let Some(velocity) = world.get::<Velocity2>(entity).copied() else {
            despawns.push(entity);
            continue;
        };
        let radius = world
            .get::<ColliderRadius>(entity)
            .copied()
            .unwrap_or(ColliderRadius(0.18))
            .0;
        let faction = world
            .get::<Faction>(entity)
            .copied()
            .unwrap_or(Faction::Neutral);
        if matches!(mode, ProjectileStepMode::PredictedLocal) && faction != Faction::Hunters {
            continue;
        }
        let Some(mut transform) = world.get_mut::<Transform2>(entity) else {
            despawns.push(entity);
            continue;
        };

        transform.x += velocity.x * dt;
        transform.y += velocity.y * dt;
        transform.yaw = velocity.y.atan2(velocity.x);
        let current_pos = [transform.x, transform.y];
        drop(transform);

        if !layout.contains_point(current_pos, radius) {
            despawns.push(entity);
            continue;
        }

        if let Some(mut state) = world.get_mut::<Projectile>(entity) {
            state.lifetime_seconds -= dt;
            if state.lifetime_seconds <= 0.0 {
                despawns.push(entity);
                continue;
            }
        }

        let mut hit_target = None;
        for target in &target_entities {
            if *target == entity {
                continue;
            }
            let Some(target_health) = world.get::<Health>(*target).copied() else {
                continue;
            };
            if target_health.current <= 0.0 {
                continue;
            }
            let Some(target_faction) = world.get::<Faction>(*target).copied() else {
                continue;
            };
            if target_faction == faction || target_faction == Faction::Neutral {
                continue;
            }
            let Some(target_transform) = world.get::<Transform2>(*target).copied() else {
                continue;
            };
            let target_radius = world
                .get::<ColliderRadius>(*target)
                .copied()
                .unwrap_or(ColliderRadius(0.5))
                .0;
            if distance_squared(current_pos, [target_transform.x, target_transform.y])
                <= (radius + target_radius).powi(2)
            {
                hit_target = Some(*target);
                break;
            }
        }

        if let Some(target) = hit_target {
            if matches!(mode, ProjectileStepMode::Authoritative)
                && let Some(mut health) = world.get_mut::<Health>(target)
            {
                health.current = (health.current - projectile.damage).max(0.0);
            }
            despawns.push(entity);
        }
    }

    for entity in despawns {
        let _ = world.despawn(entity);
    }

    Ok(())
}

pub(crate) fn constrained_move(
    layout: &CavernLayout,
    current: [f32; 2],
    delta: [f32; 2],
    radius: f32,
) -> [f32; 2] {
    let candidate = [current[0] + delta[0], current[1] + delta[1]];
    if layout.contains_point(candidate, radius) {
        return candidate;
    }

    let x_only = [current[0] + delta[0], current[1]];
    if layout.contains_point(x_only, radius) {
        return x_only;
    }

    let y_only = [current[0], current[1] + delta[1]];
    if layout.contains_point(y_only, radius) {
        return y_only;
    }

    current
}

pub(crate) fn spawn_projectile(
    world: &mut World,
    origin: [f32; 2],
    direction: [f32; 2],
    speed: f32,
    damage: f32,
    faction: Faction,
) -> Entity {
    let velocity = [direction[0] * speed, direction[1] * speed];
    world.spawn((
        Projectile {
            damage,
            lifetime_seconds: 1.4,
        },
        Transform2::new(origin[0], origin[1], direction[1].atan2(direction[0])),
        Velocity2 {
            x: velocity[0],
            y: velocity[1],
        },
        ColliderRadius(0.18),
        faction,
    ))
}

fn normalized_vector(x: f32, y: f32) -> (f32, f32) {
    let length = (x * x + y * y).sqrt();
    if length <= f32::EPSILON {
        (0.0, 0.0)
    } else {
        (x / length, y / length)
    }
}

fn distance_squared(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::{constrained_move, update_local_aim};
    use crate::domain::{
        CavernAimState, CavernCameraState, CavernControlState, CavernLayout, CavernMetaProfile,
        CavernRunConfig, CavernRunState, CavernServerControlMap, LocalPlayerRef, LootTableRegistry,
        SpawnDirector,
    };
    use crate::plugins::worldgen;
    use engine::prelude::{
        AuthorityRole, InputState, SimulationProfile, SimulationProfileConfig, Time, WindowState,
        World,
    };

    #[test]
    fn constrained_move_stays_inside_layout() {
        let layout = CavernLayout::generate(
            crate::domain::CavernSeed::default(),
            &CavernRunConfig::default(),
        );
        let start = layout.room(layout.start_room).unwrap().center;
        let next = constrained_move(&layout, start, [100.0, 100.0], 0.5);
        assert!(layout.contains_point(next, 0.5));
    }

    #[test]
    fn local_aim_updates_from_mouse_projection() {
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
        world.insert_resource(InputState::default());
        world.insert_resource(WindowState::headless("test"));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_cursor_moved(640.0, 360.0);

        update_local_aim(&mut world).unwrap();
        let aim = world.resource::<CavernAimState>().unwrap();
        assert!(
            world
                .resource::<CavernLayout>()
                .unwrap()
                .contains_point(aim.world_point, 0.0)
        );
    }

    #[test]
    fn server_control_map_moves_targeted_player() {
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
        world.insert_resource(Time::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();
        let players = world
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .map(|(entity, player_id)| (entity, player_id.0))
            .collect::<Vec<_>>();
        let (_, target_id) = players[1];
        let target_entity = players[1].0;
        let before = world
            .get::<crate::domain::Transform2>(target_entity)
            .copied()
            .unwrap();
        world
            .resource_mut::<CavernServerControlMap>()
            .unwrap()
            .by_player_id
            .insert(
                target_id,
                CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [before.x + 10.0, before.y],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: engine::prelude::SimulationTick(1),
                },
            );

        super::run_authoritative_combat_step(&mut world, 1.0 / 60.0).unwrap();

        let after = world
            .get::<crate::domain::Transform2>(target_entity)
            .copied()
            .unwrap();
        assert!(after.x > before.x);
    }
}
