use anyhow::Result;
use engine::prelude::World;

use crate::{
    CavernEnemyPatchOpV2, CavernExtractionPatchOpV2, CavernPickupPatchOpV2,
    CavernProjectilePatchOpV2, ClientReplicationMap, NetworkEntityId, Transform2, Velocity2,
};

pub(super) fn apply_enemy_patch_ops_v2(
    world: &mut World,
    ops: Vec<CavernEnemyPatchOpV2>,
) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    let mut client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    for op in ops {
        match op {
            CavernEnemyPatchOpV2::Despawn { entity_id } => {
                let entity = client_map
                    .by_network_entity_id
                    .remove(&entity_id)
                    .or_else(|| find_enemy_entity_by_replication_id(world, entity_id));
                if let Some(entity) = entity {
                    let _ = world.despawn(entity);
                }
            }
            CavernEnemyPatchOpV2::Spawn {
                entity_id, state, ..
            }
            | CavernEnemyPatchOpV2::Patch {
                entity_id, state, ..
            } => {
                let mut entity = client_map.by_network_entity_id.get(&entity_id).copied();
                if entity.is_none() {
                    entity = find_enemy_entity_by_replication_id(world, entity_id);
                }
                if entity.is_none() {
                    entity = Some(spawn_enemy_entity_from_snapshot(world, &state));
                }
                let entity = entity.expect("spawned enemy should exist");
                client_map.by_network_entity_id.insert(entity_id, entity);
                apply_enemy_snapshot(world, entity, &state);
            }
        }
    }
    world.insert_resource(client_map);
    Ok(())
}

pub(super) fn apply_projectile_patch_ops_v2(
    world: &mut World,
    ops: Vec<CavernProjectilePatchOpV2>,
) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    let mut client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    for op in ops {
        match op {
            CavernProjectilePatchOpV2::Despawn { entity_id } => {
                let entity = client_map
                    .by_network_entity_id
                    .remove(&entity_id)
                    .or_else(|| find_projectile_entity_by_replication_id(world, entity_id));
                if let Some(entity) = entity {
                    let _ = world.despawn(entity);
                }
            }
            CavernProjectilePatchOpV2::Spawn {
                entity_id, state, ..
            }
            | CavernProjectilePatchOpV2::Patch {
                entity_id, state, ..
            } => {
                let mut entity = client_map.by_network_entity_id.get(&entity_id).copied();
                if entity.is_none() {
                    entity = find_projectile_entity_by_replication_id(world, entity_id);
                }
                if entity.is_none() {
                    entity = Some(spawn_projectile_entity_from_snapshot(world, &state));
                }
                let entity = entity.expect("spawned projectile should exist");
                client_map.by_network_entity_id.insert(entity_id, entity);
                apply_projectile_snapshot(world, entity, &state);
            }
        }
    }
    world.insert_resource(client_map);
    Ok(())
}

pub(super) fn apply_pickup_patch_ops_v2(
    world: &mut World,
    ops: Vec<CavernPickupPatchOpV2>,
) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    let mut client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    for op in ops {
        match op {
            CavernPickupPatchOpV2::Despawn { entity_id } => {
                let entity = client_map
                    .by_network_entity_id
                    .remove(&entity_id)
                    .or_else(|| find_pickup_entity_by_replication_id(world, entity_id));
                if let Some(entity) = entity {
                    let _ = world.despawn(entity);
                }
            }
            CavernPickupPatchOpV2::Spawn {
                entity_id, state, ..
            }
            | CavernPickupPatchOpV2::Patch {
                entity_id, state, ..
            } => {
                let mut entity = client_map.by_network_entity_id.get(&entity_id).copied();
                if entity.is_none() {
                    entity = find_pickup_entity_by_replication_id(world, entity_id);
                }
                if entity.is_none() {
                    entity = Some(spawn_pickup_entity_from_snapshot(world, &state));
                }
                let entity = entity.expect("spawned pickup should exist");
                client_map.by_network_entity_id.insert(entity_id, entity);
                apply_pickup_snapshot(world, entity, &state);
            }
        }
    }
    world.insert_resource(client_map);
    Ok(())
}

pub(super) fn apply_extraction_patch_ops_v2(
    world: &mut World,
    ops: Vec<CavernExtractionPatchOpV2>,
) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    let mut client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    for op in ops {
        match op {
            CavernExtractionPatchOpV2::Despawn { entity_id } => {
                let entity = client_map
                    .by_network_entity_id
                    .remove(&entity_id)
                    .or_else(|| find_extraction_entity_by_replication_id(world, entity_id));
                if let Some(entity) = entity {
                    let _ = world.despawn(entity);
                }
            }
            CavernExtractionPatchOpV2::Spawn {
                entity_id, state, ..
            }
            | CavernExtractionPatchOpV2::Patch {
                entity_id, state, ..
            } => {
                let mut entity = client_map.by_network_entity_id.get(&entity_id).copied();
                if entity.is_none() {
                    entity = find_extraction_entity_by_replication_id(world, entity_id);
                }
                if entity.is_none() {
                    entity = Some(spawn_extraction_entity_from_snapshot(world, &state));
                }
                let entity = entity.expect("spawned extraction should exist");
                client_map.by_network_entity_id.insert(entity_id, entity);
                apply_extraction_snapshot(world, entity, &state);
            }
        }
    }
    world.insert_resource(client_map);
    Ok(())
}

fn find_enemy_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::EnemyReplicationId)>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn find_projectile_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(
            engine::prelude::Entity,
            &crate::ProjectileReplicationId,
        )>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn spawn_enemy_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::CavernEnemySnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn((crate::Enemy, snapshot.kind));
    let _ = world.insert(
        entity,
        crate::EnemyReplicationId(snapshot.network_entity_id),
    );
    apply_enemy_snapshot(world, entity, snapshot);
    entity
}

fn apply_enemy_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::CavernEnemySnapshotV1,
) {
    let _ = world.insert(entity, snapshot.kind);
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        Velocity2 {
            x: snapshot.velocity[0],
            y: snapshot.velocity[1],
        },
    );
    let _ = world.insert(
        entity,
        crate::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(entity, crate::Faction::CavernBeasts);
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::EnemyReplicationId(snapshot.network_entity_id),
    );
    if let Some(aggro) = snapshot.aggro {
        let _ = world.insert(entity, aggro);
    } else {
        let _ = world.remove::<crate::AggroState>(entity);
    }
    if let Some(projectile_attack) = snapshot.projectile_attack {
        let _ = world.insert(entity, projectile_attack);
    } else {
        let _ = world.remove::<crate::ProjectileAttack>(entity);
    }
    if let Some(melee_attack) = snapshot.melee_attack {
        let _ = world.insert(entity, melee_attack);
    } else {
        let _ = world.remove::<crate::MeleeAttack>(entity);
    }
    if let Some(weapon) = snapshot.weapon {
        let _ = world.insert(entity, weapon);
    } else {
        let _ = world.remove::<crate::WeaponState>(entity);
    }
    if let Some(spawn_room) = snapshot.spawn_room {
        let _ = world.insert(entity, crate::SpawnRoom(spawn_room));
    } else {
        let _ = world.remove::<crate::SpawnRoom>(entity);
    }
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::RoomAnchor>(entity);
    }
    if snapshot.elite_objective {
        let _ = world.insert(entity, crate::EliteObjective);
    } else {
        let _ = world.remove::<crate::EliteObjective>(entity);
    }
}

fn spawn_projectile_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::CavernProjectileSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::Projectile {
        damage: snapshot.damage,
        lifetime_seconds: snapshot.lifetime_seconds,
    });
    let _ = world.insert(
        entity,
        crate::ProjectileReplicationId(snapshot.network_entity_id),
    );
    apply_projectile_snapshot(world, entity, snapshot);
    entity
}

fn apply_projectile_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::CavernProjectileSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::Projectile {
            damage: snapshot.damage,
            lifetime_seconds: snapshot.lifetime_seconds,
        },
    );
    let _ = world.insert(
        entity,
        crate::ProjectileVisualState {
            source_team: if snapshot.faction == crate::Faction::Hunters {
                0
            } else {
                1
            },
            life_elapsed_seconds: 0.0,
        },
    );
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        Velocity2 {
            x: snapshot.velocity[0],
            y: snapshot.velocity[1],
        },
    );
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(entity, snapshot.faction);
    let _ = world.insert(
        entity,
        crate::ProjectileReplicationId(snapshot.network_entity_id),
    );
}

fn find_pickup_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::PickupReplicationId)>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn find_extraction_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(
            engine::prelude::Entity,
            &crate::ExtractionReplicationId,
        )>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn spawn_pickup_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::CavernPickupSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::Pickup {
        kind: snapshot.pickup,
    });
    let _ = world.insert(
        entity,
        crate::PickupReplicationId(snapshot.network_entity_id),
    );
    apply_pickup_snapshot(world, entity, snapshot);
    entity
}

fn apply_pickup_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::CavernPickupSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::Pickup {
            kind: snapshot.pickup,
        },
    );
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::PickupReplicationId(snapshot.network_entity_id),
    );
    if snapshot.loot_drop {
        let _ = world.insert(entity, crate::LootDrop);
    } else {
        let _ = world.remove::<crate::LootDrop>(entity);
    }
    if snapshot.chest {
        let _ = world.insert(entity, crate::Chest);
    } else {
        let _ = world.remove::<crate::Chest>(entity);
    }
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::RoomAnchor>(entity);
    }
}

fn spawn_extraction_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::CavernExtractionSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::ExtractionZone);
    let _ = world.insert(
        entity,
        crate::ExtractionReplicationId(snapshot.network_entity_id),
    );
    apply_extraction_snapshot(world, entity, snapshot);
    entity
}

fn apply_extraction_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::CavernExtractionSnapshotV1,
) {
    let _ = world.insert(entity, crate::ExtractionZone);
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::ExtractionReplicationId(snapshot.network_entity_id),
    );
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::RoomAnchor>(entity);
    }
}
