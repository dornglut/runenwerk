use engine::prelude::World;

use crate::{Transform2, Velocity2};

pub(super) fn spawn_player_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::CavernPlayerSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::Player);
    let _ = world.insert(entity, crate::PlayerId(snapshot.player_id));
    let _ = world.insert(
        entity,
        crate::PlayerRosterIdentity {
            player_code: snapshot.player_code.clone(),
            roster_index: snapshot.roster_index,
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
        crate::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(entity, crate::Faction::Hunters);
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::AimTarget2 {
            x: snapshot.aim[0],
            y: snapshot.aim[1],
        },
    );
    let _ = world.insert(entity, snapshot.dash);
    let _ = world.insert(entity, snapshot.weapon);
    let _ = world.insert(
        entity,
        crate::InventoryRunState {
            scrap: snapshot.inventory.scrap,
            weapon_mods: snapshot.inventory.weapon_mods.clone(),
            relics: snapshot.inventory.relics.clone(),
        },
    );
    let _ = world.insert(
        entity,
        crate::PlayerSpawnState {
            profile: snapshot.spawn_profile,
        },
    );
    let _ = world.insert(entity, crate::PlayerActive);
    apply_non_transform_player_snapshot(world, entity, snapshot);
    entity
}

pub(super) fn apply_non_transform_player_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::CavernPlayerSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::PlayerRosterIdentity {
            player_code: snapshot.player_code.clone(),
            roster_index: snapshot.roster_index,
        },
    );
    let _ = world.insert(
        entity,
        crate::PlayerSpawnState {
            profile: snapshot.spawn_profile,
        },
    );
    let _ = world.insert(
        entity,
        crate::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(
        entity,
        crate::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(entity, crate::DashState { ..snapshot.dash });
    let _ = world.insert(entity, crate::WeaponState { ..snapshot.weapon });
    let _ = world.insert(
        entity,
        crate::InventoryRunState {
            scrap: snapshot.inventory.scrap,
            weapon_mods: snapshot.inventory.weapon_mods.clone(),
            relics: snapshot.inventory.relics.clone(),
        },
    );
    let _ = world.insert(
        entity,
        crate::AimTarget2 {
            x: snapshot.aim[0],
            y: snapshot.aim[1],
        },
    );
    let _ = world.insert(entity, crate::PlayerActive);

    if let Some(room_id) = snapshot.room_anchor {
        let _ = world.insert(entity, crate::RoomAnchor { room_id });
    } else {
        let _ = world.remove::<crate::RoomAnchor>(entity);
    }
    if snapshot.extracting {
        let _ = world.insert(entity, crate::Extracting);
    } else {
        let _ = world.remove::<crate::Extracting>(entity);
    }
    if snapshot.spectator {
        let _ = world.insert(entity, crate::PlayerSpectator);
    } else {
        let _ = world.remove::<crate::PlayerSpectator>(entity);
    }
    if snapshot.ai_controlled {
        let _ = world.insert(
            entity,
            crate::PlayerCompanion {
                fill_slot: snapshot.roster_index,
            },
        );
    } else {
        let _ = world.remove::<crate::PlayerCompanion>(entity);
    }
}
