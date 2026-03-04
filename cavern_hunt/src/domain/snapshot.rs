use crate::domain::components::{
    AggroState, AimTarget2, Chest, ColliderRadius, DashState, EliteObjective, Enemy, EnemyKind,
    Extracting, ExtractionZone, Faction, Health, InventoryRunState, LootDrop, Pickup, Player,
    PlayerId, Projectile, ProjectileAttack, RoomAnchor, SpawnRoom, Transform2, Velocity2,
    WeaponState,
};
use crate::domain::loot::{PickupKind, RelicKind, WeaponModKind};
use crate::domain::resources::{CavernRunPhase, CavernRunState, CavernSeed, LocalPlayerRef};
use crate::domain::worldgen::CavernLayout;
use crate::domain::worldgen::RoomId;
use anyhow::Result;
use engine::prelude::{Bundle, Entity, SimulationTick, World};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernInventorySnapshotV1 {
    pub scrap: u32,
    pub weapon_mods: Vec<WeaponModKind>,
    pub relics: Vec<RelicKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernPlayerSnapshotV1 {
    pub player_id: u32,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub health_current: f32,
    pub health_max: f32,
    pub collider_radius: f32,
    pub aim: [f32; 2],
    pub dash: DashState,
    pub weapon: WeaponState,
    pub inventory: CavernInventorySnapshotV1,
    pub room_anchor: Option<RoomId>,
    pub extracting: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernEnemySnapshotV1 {
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub health_current: f32,
    pub health_max: f32,
    pub collider_radius: f32,
    pub aggro: Option<AggroState>,
    pub projectile_attack: Option<ProjectileAttack>,
    pub melee_attack: Option<crate::domain::MeleeAttack>,
    pub weapon: Option<WeaponState>,
    pub spawn_room: Option<RoomId>,
    pub room_anchor: Option<RoomId>,
    pub elite_objective: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernProjectileSnapshotV1 {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub damage: f32,
    pub lifetime_seconds: f32,
    pub collider_radius: f32,
    pub faction: Faction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernPickupSnapshotV1 {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub collider_radius: f32,
    pub pickup: PickupKind,
    pub loot_drop: bool,
    pub chest: bool,
    pub room_anchor: Option<RoomId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernExtractionSnapshotV1 {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub collider_radius: f32,
    pub room_anchor: Option<RoomId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLayoutSnapshotV1 {
    pub seed: CavernSeed,
    pub layout: CavernLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunSnapshotV1 {
    pub run_id: u64,
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub extraction_started_at_tick: Option<SimulationTick>,
    pub party_alive_count: u8,
    pub enemy_kills: u32,
    pub layout: CavernLayoutSnapshotV1,
    pub players: Vec<CavernPlayerSnapshotV1>,
    pub enemies: Vec<CavernEnemySnapshotV1>,
    pub projectiles: Vec<CavernProjectileSnapshotV1>,
    pub pickups: Vec<CavernPickupSnapshotV1>,
    pub extraction_zones: Vec<CavernExtractionSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunDeltaV1 {
    pub run_id: Option<u64>,
    pub seed: CavernSeed,
    pub phase: Option<CavernRunPhase>,
    pub elite_defeated: Option<bool>,
    pub extraction_active: Option<bool>,
    pub extraction_started_at_tick: Option<Option<SimulationTick>>,
    pub party_alive_count: Option<u8>,
    pub enemy_kills: Option<u32>,
    pub layout: Option<CavernLayoutSnapshotV1>,
    pub players: Option<Vec<CavernPlayerSnapshotV1>>,
    pub enemies: Option<Vec<CavernEnemySnapshotV1>>,
    pub projectiles: Option<Vec<CavernProjectileSnapshotV1>>,
    pub pickups: Option<Vec<CavernPickupSnapshotV1>>,
    pub extraction_zones: Option<Vec<CavernExtractionSnapshotV1>>,
}

#[derive(Bundle)]
struct PlayerSnapshotBundle {
    player: Player,
    player_id: PlayerId,
    transform: Transform2,
    velocity: Velocity2,
    health: Health,
    faction: Faction,
    collider_radius: ColliderRadius,
    aim_target: AimTarget2,
    dash_state: DashState,
    weapon_state: WeaponState,
    inventory: InventoryRunState,
}

#[derive(Bundle)]
struct EnemySnapshotBundle {
    enemy: Enemy,
    enemy_kind: EnemyKind,
    transform: Transform2,
    velocity: Velocity2,
    health: Health,
    faction: Faction,
    collider_radius: ColliderRadius,
}

#[derive(Bundle)]
struct ProjectileSnapshotBundle {
    projectile: Projectile,
    transform: Transform2,
    velocity: Velocity2,
    collider_radius: ColliderRadius,
    faction: Faction,
}

#[derive(Bundle)]
struct PickupSnapshotBundle {
    pickup: Pickup,
    transform: Transform2,
    collider_radius: ColliderRadius,
}

#[derive(Bundle)]
struct ExtractionSnapshotBundle {
    extraction_zone: ExtractionZone,
    transform: Transform2,
    collider_radius: ColliderRadius,
}

pub fn capture_cavern_run_snapshot(world: &World) -> Result<CavernRunSnapshotV1> {
    let layout = world.resource::<CavernLayout>()?.clone();
    let run_state = world.resource::<CavernRunState>()?.clone();
    let mut players = Vec::new();
    let mut enemies = Vec::new();
    let mut projectiles = Vec::new();
    let mut pickups = Vec::new();
    let mut extraction_zones = Vec::new();

    for (entity, transform) in world.query::<(Entity, &Transform2)>().iter() {
        if world.get::<Player>(entity).is_some() {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.55))
                .0;
            let aim = world
                .get::<AimTarget2>(entity)
                .copied()
                .unwrap_or(AimTarget2 {
                    x: transform.x,
                    y: transform.y,
                });
            let dash = world.get::<DashState>(entity).copied().unwrap_or_default();
            let weapon = world
                .get::<WeaponState>(entity)
                .copied()
                .unwrap_or_default();
            let inventory =
                world
                    .get::<InventoryRunState>(entity)
                    .cloned()
                    .unwrap_or(InventoryRunState {
                        scrap: 0,
                        weapon_mods: Vec::new(),
                        relics: Vec::new(),
                    });
            let player_id = world
                .get::<PlayerId>(entity)
                .copied()
                .unwrap_or(PlayerId(0))
                .0;
            players.push(CavernPlayerSnapshotV1 {
                player_id,
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                health_current: health.current,
                health_max: health.max,
                collider_radius,
                aim: [aim.x, aim.y],
                dash,
                weapon,
                inventory: CavernInventorySnapshotV1 {
                    scrap: inventory.scrap,
                    weapon_mods: inventory.weapon_mods,
                    relics: inventory.relics,
                },
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
                extracting: world.get::<Extracting>(entity).is_some(),
            });
            continue;
        }

        if let Some(kind) = world.get::<EnemyKind>(entity).copied() {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.6))
                .0;
            enemies.push(CavernEnemySnapshotV1 {
                kind,
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                health_current: health.current,
                health_max: health.max,
                collider_radius,
                aggro: world.get::<AggroState>(entity).copied(),
                projectile_attack: world.get::<ProjectileAttack>(entity).copied(),
                melee_attack: world.get::<crate::domain::MeleeAttack>(entity).copied(),
                weapon: world.get::<WeaponState>(entity).copied(),
                spawn_room: world.get::<SpawnRoom>(entity).map(|room| room.0),
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
                elite_objective: world.get::<EliteObjective>(entity).is_some(),
            });
            continue;
        }

        if let Some(projectile) = world.get::<Projectile>(entity).copied() {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.18))
                .0;
            let faction = world
                .get::<Faction>(entity)
                .copied()
                .unwrap_or(Faction::Neutral);
            projectiles.push(CavernProjectileSnapshotV1 {
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                damage: projectile.damage,
                lifetime_seconds: projectile.lifetime_seconds,
                collider_radius,
                faction,
            });
            continue;
        }

        if let Some(pickup) = world.get::<Pickup>(entity).copied() {
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.38))
                .0;
            pickups.push(CavernPickupSnapshotV1 {
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                collider_radius,
                pickup: pickup.kind,
                loot_drop: world.get::<LootDrop>(entity).is_some(),
                chest: world.get::<Chest>(entity).is_some(),
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
            });
            continue;
        }

        if world.get::<ExtractionZone>(entity).is_some() {
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(1.25))
                .0;
            extraction_zones.push(CavernExtractionSnapshotV1 {
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                collider_radius,
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
            });
        }
    }

    players.sort_by_key(|player| player.player_id);
    enemies.sort_by(|a, b| {
        enemy_kind_order(a.kind)
            .cmp(&enemy_kind_order(b.kind))
            .then_with(|| a.x.total_cmp(&b.x))
            .then_with(|| a.y.total_cmp(&b.y))
    });
    projectiles.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    pickups.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    extraction_zones.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));

    Ok(CavernRunSnapshotV1 {
        run_id: run_state.run_id,
        seed: run_state.seed,
        phase: run_state.phase,
        elite_defeated: run_state.elite_defeated,
        extraction_active: run_state.extraction_active,
        extraction_started_at_tick: run_state.extraction_started_at_tick,
        party_alive_count: run_state.party_alive_count,
        enemy_kills: run_state.enemy_kills,
        layout: CavernLayoutSnapshotV1 {
            seed: run_state.seed,
            layout,
        },
        players,
        enemies,
        projectiles,
        pickups,
        extraction_zones,
    })
}

pub fn build_cavern_run_delta(
    base: &CavernRunSnapshotV1,
    current: &CavernRunSnapshotV1,
) -> CavernRunDeltaV1 {
    CavernRunDeltaV1 {
        run_id: (base.run_id != current.run_id).then_some(current.run_id),
        seed: current.seed,
        phase: (base.phase != current.phase).then_some(current.phase),
        elite_defeated: (base.elite_defeated != current.elite_defeated)
            .then_some(current.elite_defeated),
        extraction_active: (base.extraction_active != current.extraction_active)
            .then_some(current.extraction_active),
        extraction_started_at_tick: (base.extraction_started_at_tick
            != current.extraction_started_at_tick)
            .then_some(current.extraction_started_at_tick),
        party_alive_count: (base.party_alive_count != current.party_alive_count)
            .then_some(current.party_alive_count),
        enemy_kills: (base.enemy_kills != current.enemy_kills).then_some(current.enemy_kills),
        layout: (base.layout != current.layout).then_some(current.layout.clone()),
        players: (base.players != current.players).then_some(current.players.clone()),
        enemies: (base.enemies != current.enemies).then_some(current.enemies.clone()),
        projectiles: (base.projectiles != current.projectiles)
            .then_some(current.projectiles.clone()),
        pickups: (base.pickups != current.pickups).then_some(current.pickups.clone()),
        extraction_zones: (base.extraction_zones != current.extraction_zones)
            .then_some(current.extraction_zones.clone()),
    }
}

pub fn apply_cavern_run_delta(
    base: &CavernRunSnapshotV1,
    delta: &CavernRunDeltaV1,
) -> CavernRunSnapshotV1 {
    CavernRunSnapshotV1 {
        run_id: delta.run_id.unwrap_or(base.run_id),
        seed: delta.seed,
        phase: delta.phase.unwrap_or(base.phase),
        elite_defeated: delta.elite_defeated.unwrap_or(base.elite_defeated),
        extraction_active: delta.extraction_active.unwrap_or(base.extraction_active),
        extraction_started_at_tick: delta
            .extraction_started_at_tick
            .unwrap_or(base.extraction_started_at_tick),
        party_alive_count: delta.party_alive_count.unwrap_or(base.party_alive_count),
        enemy_kills: delta.enemy_kills.unwrap_or(base.enemy_kills),
        layout: delta.layout.clone().unwrap_or_else(|| base.layout.clone()),
        players: delta
            .players
            .clone()
            .unwrap_or_else(|| base.players.clone()),
        enemies: delta
            .enemies
            .clone()
            .unwrap_or_else(|| base.enemies.clone()),
        projectiles: delta
            .projectiles
            .clone()
            .unwrap_or_else(|| base.projectiles.clone()),
        pickups: delta
            .pickups
            .clone()
            .unwrap_or_else(|| base.pickups.clone()),
        extraction_zones: delta
            .extraction_zones
            .clone()
            .unwrap_or_else(|| base.extraction_zones.clone()),
    }
}

pub fn restore_cavern_run_snapshot(
    world: &mut World,
    snapshot: &CavernRunSnapshotV1,
) -> Result<()> {
    let previous_local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.entity)
        .and_then(|entity| world.get::<PlayerId>(entity).copied())
        .map(|player_id| player_id.0);

    clear_cavern_run_entities(world);
    world.insert_resource(snapshot.layout.layout.clone());
    world.insert_resource(CavernRunState {
        run_id: snapshot.run_id,
        seed: snapshot.seed,
        phase: snapshot.phase,
        elite_defeated: snapshot.elite_defeated,
        extraction_active: snapshot.extraction_active,
        extraction_started_at_tick: snapshot.extraction_started_at_tick,
        party_alive_count: snapshot.party_alive_count,
        enemy_kills: snapshot.enemy_kills,
    });

    let mut restored_local_entity = None;

    for player in &snapshot.players {
        let entity = world.spawn(PlayerSnapshotBundle {
            player: Player,
            player_id: PlayerId(player.player_id),
            transform: Transform2::new(player.x, player.y, player.yaw),
            velocity: Velocity2 {
                x: player.velocity[0],
                y: player.velocity[1],
            },
            health: Health {
                current: player.health_current,
                max: player.health_max,
            },
            faction: Faction::Hunters,
            collider_radius: ColliderRadius(player.collider_radius),
            aim_target: AimTarget2 {
                x: player.aim[0],
                y: player.aim[1],
            },
            dash_state: player.dash,
            weapon_state: player.weapon,
            inventory: InventoryRunState {
                scrap: player.inventory.scrap,
                weapon_mods: player.inventory.weapon_mods.clone(),
                relics: player.inventory.relics.clone(),
            },
        });
        if let Some(room_id) = player.room_anchor {
            let _ = world.insert(entity, RoomAnchor { room_id });
        }
        if player.extracting {
            let _ = world.insert(entity, Extracting);
        }
        if previous_local_player_id == Some(player.player_id) || restored_local_entity.is_none() {
            restored_local_entity = Some(entity);
        }
    }

    for enemy in &snapshot.enemies {
        let entity = world.spawn(EnemySnapshotBundle {
            enemy: Enemy,
            enemy_kind: enemy.kind,
            transform: Transform2::new(enemy.x, enemy.y, enemy.yaw),
            velocity: Velocity2 {
                x: enemy.velocity[0],
                y: enemy.velocity[1],
            },
            health: Health {
                current: enemy.health_current,
                max: enemy.health_max,
            },
            faction: Faction::CavernBeasts,
            collider_radius: ColliderRadius(enemy.collider_radius),
        });
        if let Some(aggro) = enemy.aggro {
            let _ = world.insert(entity, aggro);
        }
        if let Some(projectile_attack) = enemy.projectile_attack {
            let _ = world.insert(entity, projectile_attack);
        }
        if let Some(melee_attack) = enemy.melee_attack {
            let _ = world.insert(entity, melee_attack);
        }
        if let Some(weapon) = enemy.weapon {
            let _ = world.insert(entity, weapon);
        }
        if let Some(spawn_room) = enemy.spawn_room {
            let _ = world.insert(entity, SpawnRoom(spawn_room));
        }
        if let Some(room_anchor) = enemy.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
        if enemy.elite_objective {
            let _ = world.insert(entity, EliteObjective);
        }
    }

    for projectile in &snapshot.projectiles {
        world.spawn(ProjectileSnapshotBundle {
            projectile: Projectile {
                damage: projectile.damage,
                lifetime_seconds: projectile.lifetime_seconds,
            },
            transform: Transform2::new(projectile.x, projectile.y, projectile.yaw),
            velocity: Velocity2 {
                x: projectile.velocity[0],
                y: projectile.velocity[1],
            },
            collider_radius: ColliderRadius(projectile.collider_radius),
            faction: projectile.faction,
        });
    }

    for pickup in &snapshot.pickups {
        let entity = world.spawn(PickupSnapshotBundle {
            pickup: Pickup {
                kind: pickup.pickup,
            },
            transform: Transform2::new(pickup.x, pickup.y, pickup.yaw),
            collider_radius: ColliderRadius(pickup.collider_radius),
        });
        if pickup.loot_drop {
            let _ = world.insert(entity, LootDrop);
        }
        if pickup.chest {
            let _ = world.insert(entity, Chest);
        }
        if let Some(room_anchor) = pickup.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
    }

    for zone in &snapshot.extraction_zones {
        let entity = world.spawn(ExtractionSnapshotBundle {
            extraction_zone: ExtractionZone,
            transform: Transform2::new(zone.x, zone.y, zone.yaw),
            collider_radius: ColliderRadius(zone.collider_radius),
        });
        if let Some(room_anchor) = zone.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
    }

    if let Ok(mut local_player) = world.resource_mut::<LocalPlayerRef>() {
        local_player.entity = restored_local_entity;
    }

    Ok(())
}

fn clear_cavern_run_entities(world: &mut World) {
    let entities = world
        .query::<(Entity, &Transform2)>()
        .iter()
        .filter_map(|(entity, _)| {
            (world.get::<Player>(entity).is_some()
                || world.get::<Enemy>(entity).is_some()
                || world.get::<Projectile>(entity).is_some()
                || world.get::<Pickup>(entity).is_some()
                || world.get::<ExtractionZone>(entity).is_some())
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    for entity in entities {
        let _ = world.despawn(entity);
    }
}

fn enemy_kind_order(kind: EnemyKind) -> u8 {
    match kind {
        EnemyKind::Swarmer => 0,
        EnemyKind::Bruiser => 1,
        EnemyKind::Spitter => 2,
        EnemyKind::NestGuardian => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_cavern_run_delta, build_cavern_run_delta, capture_cavern_run_snapshot,
        restore_cavern_run_snapshot,
    };
    use crate::domain::{
        CavernAimState, CavernCameraState, CavernMetaProfile, CavernRunConfig, CavernRunState,
        Faction, Health, LocalPlayerRef, LootTableRegistry, SpawnDirector, Transform2,
    };
    use crate::plugins::{combat, worldgen};
    use engine::prelude::{InputState, SimulationRng, SimulationSeed, WindowState, World};

    fn seeded_world() -> World {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(crate::domain::CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(InputState::default());
        world.insert_resource(WindowState::headless("snapshot-test"));
        world.insert_resource(SimulationRng::from_seed(SimulationSeed(42)));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        world
    }

    #[test]
    fn cavern_run_delta_round_trips_current_snapshot() {
        let mut world = seeded_world();
        let base = capture_cavern_run_snapshot(&world).unwrap();

        let local = world.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        if let Some(mut transform) = world.get_mut::<Transform2>(local) {
            transform.x += 1.5;
            transform.y += 0.5;
        }
        if let Some(mut health) = world.get_mut::<Health>(local) {
            health.current -= 2.0;
        }
        combat::spawn_projectile(
            &mut world,
            [1.0, 1.0],
            [1.0, 0.0],
            6.0,
            2.0,
            Faction::Hunters,
        );

        let current = capture_cavern_run_snapshot(&world).unwrap();
        let delta = build_cavern_run_delta(&base, &current);
        let rebuilt = apply_cavern_run_delta(&base, &delta);
        assert_eq!(rebuilt, current);
    }

    #[test]
    fn restoring_snapshot_rebuilds_cavern_world_state() {
        let mut source = seeded_world();
        let local = source.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        if let Some(mut transform) = source.get_mut::<Transform2>(local) {
            transform.x += 2.0;
        }
        let snapshot = capture_cavern_run_snapshot(&source).unwrap();

        let mut restored = World::new();
        restored.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut restored, &snapshot).unwrap();
        let restored_snapshot = capture_cavern_run_snapshot(&restored).unwrap();
        assert_eq!(restored_snapshot, snapshot);
    }
}
