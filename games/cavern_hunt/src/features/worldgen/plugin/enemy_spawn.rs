use super::spawn_bundles::EnemySpawnBundle;
use super::*;

pub(super) fn spawn_enemy(world: &mut World, room: &crate::CavernRoom, kind: EnemyKind) -> Entity {
    let difficulty = world
        .resource::<SessionSpawnPolicy>()
        .map(|policy| policy.difficulty)
        .unwrap_or_default();
    let (mut health, radius, yaw) = match kind {
        EnemyKind::Swarmer => (Health::new(3.5), 0.42, 0.0),
        EnemyKind::Bruiser => (Health::new(8.0), 0.78, 0.5),
        EnemyKind::Spitter => (Health::new(5.5), 0.58, 1.0),
        EnemyKind::NestGuardian => (Health::new(18.0), 0.92, 0.8),
    };
    health.max *= difficulty.enemy_health_scale;
    health.current = health.max;
    if kind == EnemyKind::NestGuardian {
        health.max += difficulty.elite_health_bonus;
        health.current = health.max;
    }
    let entity = world.spawn(EnemySpawnBundle {
        enemy: Enemy,
        enemy_kind: kind,
        transform: Transform2::new(room.spawn_anchor[0], room.spawn_anchor[1], yaw),
        velocity: Velocity2::default(),
        health,
        faction: Faction::CavernBeasts,
        collider_radius: ColliderRadius(radius),
        aggro_state: match kind {
            EnemyKind::Swarmer => AggroState { radius: 10.5 },
            EnemyKind::Bruiser => AggroState { radius: 11.5 },
            EnemyKind::Spitter => AggroState { radius: 14.0 },
            EnemyKind::NestGuardian => AggroState { radius: 16.0 },
        },
        spawn_room: SpawnRoom(room.id),
        room_anchor: RoomAnchor { room_id: room.id },
    });

    match kind {
        EnemyKind::Swarmer => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 0.85,
                        damage: 0.9 * difficulty.enemy_damage_scale,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 0.7,
                        projectile_speed: 0.0,
                        damage: 0.9 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::Bruiser => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 1.1,
                        damage: 1.5 * difficulty.enemy_damage_scale,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.2,
                        projectile_speed: 0.0,
                        damage: 1.5 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::Spitter => {
            let _ = world.insert(
                entity,
                (
                    ProjectileAttack {
                        cooldown_seconds: 1.3,
                        projectile_speed: 8.5,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.3,
                        projectile_speed: 8.5,
                        damage: 1.1 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::NestGuardian => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 1.3,
                        damage: 2.2 * difficulty.enemy_damage_scale,
                    },
                    ProjectileAttack {
                        cooldown_seconds: 1.0,
                        projectile_speed: 9.0,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.0,
                        projectile_speed: 9.0,
                        damage: 2.0 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
    }

    entity
}
