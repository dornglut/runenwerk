use crate::domain::{
    AggroState, CavernCollisionField, CavernGeometryGraph, CavernRunPhase, CavernRunState,
    ColliderRadius, EnemyCombatTuning, EnemyKind, Faction, Health, MeleeAttack, ProjectileAttack,
    RoomAnchor, RoomEncounterRegistry, RoomEncounterState, Transform2, Velocity2, WeaponState,
    is_active_player_entity,
};
use crate::plugins::combat::{constrained_move, spawn_projectile};
use crate::plugins::timing::fixed_step_seconds;
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, Entity, FixedUpdate, Plugin, SimulationProfileConfig, World, WorldMut,
};

pub struct CavernHuntAiPlugin;

impl Plugin for CavernHuntAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, enemy_ai_system);
    }
}

fn enemy_ai_system(mut world: WorldMut) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }

    let phase = world.resource::<CavernRunState>()?.phase;
    if matches!(phase, CavernRunPhase::Success | CavernRunPhase::Failure) {
        return Ok(());
    }

    let dt = fixed_step_seconds(&world);
    if dt <= f32::EPSILON {
        return Ok(());
    }

    let graph = world.resource::<CavernGeometryGraph>()?.clone();
    let encounter_registry = world
        .resource::<RoomEncounterRegistry>()
        .cloned()
        .unwrap_or_default();
    let tuning = world
        .resource::<EnemyCombatTuning>()
        .copied()
        .unwrap_or_default();
    let living_players = collect_living_players(&world);
    if living_players.is_empty() {
        return Ok(());
    }

    let enemy_entities = world
        .query::<(Entity, &EnemyKind)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let mut projectile_spawns = Vec::new();
    let mut damage_events = Vec::new();

    for entity in enemy_entities {
        let Some(kind) = world.get::<EnemyKind>(entity).copied() else {
            continue;
        };
        let Some(health) = world.get::<Health>(entity).copied() else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        let Some(transform) = world.get::<Transform2>(entity).copied() else {
            continue;
        };
        let room_anchor = world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id);
        let room_active = room_anchor
            .and_then(|room_id| encounter_registry.by_room_id.get(&room_id))
            .map(|status| {
                matches!(
                    status.state,
                    RoomEncounterState::Active | RoomEncounterState::Cleared
                )
            })
            .unwrap_or(true);
        if !room_active {
            continue;
        }
        let radius = world
            .get::<ColliderRadius>(entity)
            .copied()
            .unwrap_or(ColliderRadius(0.6))
            .0;
        let aggro = world
            .get::<AggroState>(entity)
            .copied()
            .unwrap_or(AggroState { radius: 12.0 });
        let Some(target) =
            nearest_player(&living_players, [transform.x, transform.y], aggro.radius)
        else {
            continue;
        };

        let to_target = [
            target.position[0] - transform.x,
            target.position[1] - transform.y,
        ];
        let distance = (to_target[0] * to_target[0] + to_target[1] * to_target[1]).sqrt();
        let direction = if distance <= f32::EPSILON {
            [0.0, 0.0]
        } else {
            [to_target[0] / distance, to_target[1] / distance]
        };

        let speed = match kind {
            EnemyKind::Swarmer => tuning.swarmer_speed,
            EnemyKind::Bruiser => tuning.bruiser_speed,
            EnemyKind::Spitter => tuning.spitter_speed,
            EnemyKind::NestGuardian => tuning.elite_speed,
        };
        let stop_distance = if world.get::<ProjectileAttack>(entity).is_some()
            && world.get::<MeleeAttack>(entity).is_none()
        {
            4.0 + target.radius
        } else {
            0.9 + target.radius + radius
        };
        let next = if distance > stop_distance {
            let mut field = world.resource_mut::<CavernCollisionField>()?;
            constrained_move(
                &mut field,
                &graph,
                [transform.x, transform.y],
                [direction[0] * speed * dt, direction[1] * speed * dt],
                radius,
            )
        } else {
            [transform.x, transform.y]
        };
        if let Some(mut enemy_transform) = world.get_mut::<Transform2>(entity) {
            enemy_transform.x = next[0];
            enemy_transform.y = next[1];
            if direction[0].abs() > f32::EPSILON || direction[1].abs() > f32::EPSILON {
                enemy_transform.yaw = direction[1].atan2(direction[0]);
            }
        }
        if let Some(mut velocity) = world.get_mut::<Velocity2>(entity) {
            velocity.x = (next[0] - transform.x) / dt.max(0.0001);
            velocity.y = (next[1] - transform.y) / dt.max(0.0001);
        }

        if let Some(melee) = world.get::<MeleeAttack>(entity).copied() {
            let cooldown_ready = world
                .get::<WeaponState>(entity)
                .map(|weapon| weapon.cooldown_remaining <= f32::EPSILON)
                .unwrap_or(true);
            if cooldown_ready && distance <= melee.range + target.radius + radius {
                damage_events.push((target.entity, melee.damage));
                if let Some(mut weapon) = world.get_mut::<WeaponState>(entity) {
                    weapon.cooldown_remaining = weapon.fire_interval_seconds.max(0.4);
                }
                continue;
            }
        }

        if let Some(projectile) = world.get::<ProjectileAttack>(entity).copied() {
            let cooldown_ready = world
                .get::<WeaponState>(entity)
                .map(|weapon| weapon.cooldown_remaining <= f32::EPSILON)
                .unwrap_or(true);
            if cooldown_ready && distance <= aggro.radius {
                let damage = world
                    .get::<WeaponState>(entity)
                    .map(|weapon| weapon.damage)
                    .unwrap_or(1.0);
                projectile_spawns.push((
                    [
                        transform.x + direction[0] * (radius + 0.25),
                        transform.y + direction[1] * (radius + 0.25),
                    ],
                    direction,
                    projectile.projectile_speed,
                    damage,
                ));
                if let Some(mut weapon) = world.get_mut::<WeaponState>(entity) {
                    weapon.cooldown_remaining = projectile.cooldown_seconds;
                }
            }
        }
    }

    for (target, damage) in damage_events {
        if let Some(mut health) = world.get_mut::<Health>(target) {
            health.current = (health.current - damage).max(0.0);
        }
    }

    for (origin, direction, speed, damage) in projectile_spawns {
        spawn_projectile(
            &mut world,
            origin,
            direction,
            speed,
            damage,
            Faction::CavernBeasts,
        );
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct PlayerTarget {
    entity: Entity,
    position: [f32; 2],
    radius: f32,
}

fn collect_living_players(world: &World) -> Vec<PlayerTarget> {
    world
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
            Some(PlayerTarget {
                entity,
                position: [transform.x, transform.y],
                radius,
            })
        })
        .collect()
}

fn nearest_player(
    players: &[PlayerTarget],
    from: [f32; 2],
    max_distance: f32,
) -> Option<PlayerTarget> {
    let mut best = None;
    let mut best_distance = max_distance * max_distance;
    for player in players {
        let dx = player.position[0] - from[0];
        let dy = player.position[1] - from[1];
        let distance = dx * dx + dy * dy;
        if distance <= best_distance {
            best_distance = distance;
            best = Some(*player);
        }
    }
    best
}
