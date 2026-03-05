use engine::prelude::World;

use crate::domain::{Transform2, Velocity2};

pub(super) fn spawn_player_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::domain::CavernPlayerSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::domain::Player);
    let _ = world.insert(entity, crate::domain::PlayerId(snapshot.player_id));
    let _ = world.insert(
        entity,
        crate::domain::PlayerRosterIdentity {
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
        crate::domain::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(entity, crate::domain::Faction::Hunters);
    let _ = world.insert(
        entity,
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::domain::AimTarget2 {
            x: snapshot.aim[0],
            y: snapshot.aim[1],
        },
    );
    let _ = world.insert(entity, snapshot.dash);
    let _ = world.insert(entity, snapshot.weapon);
    let _ = world.insert(
        entity,
        crate::domain::InventoryRunState {
            scrap: snapshot.inventory.scrap,
            weapon_mods: snapshot.inventory.weapon_mods.clone(),
            relics: snapshot.inventory.relics.clone(),
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::PlayerSpawnState {
            profile: snapshot.spawn_profile,
        },
    );
    let _ = world.insert(entity, crate::domain::PlayerActive);
    apply_non_transform_player_snapshot(world, entity, snapshot);
    entity
}

pub(super) fn apply_non_transform_player_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernPlayerSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::domain::PlayerRosterIdentity {
            player_code: snapshot.player_code.clone(),
            roster_index: snapshot.roster_index,
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::PlayerSpawnState {
            profile: snapshot.spawn_profile,
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(entity, crate::domain::DashState { ..snapshot.dash });
    let _ = world.insert(entity, crate::domain::WeaponState { ..snapshot.weapon });
    let _ = world.insert(
        entity,
        crate::domain::InventoryRunState {
            scrap: snapshot.inventory.scrap,
            weapon_mods: snapshot.inventory.weapon_mods.clone(),
            relics: snapshot.inventory.relics.clone(),
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::AimTarget2 {
            x: snapshot.aim[0],
            y: snapshot.aim[1],
        },
    );
    let _ = world.insert(entity, crate::domain::PlayerActive);

    if let Some(room_id) = snapshot.room_anchor {
        let _ = world.insert(entity, crate::domain::RoomAnchor { room_id });
    } else {
        let _ = world.remove::<crate::domain::RoomAnchor>(entity);
    }
    if snapshot.extracting {
        let _ = world.insert(entity, crate::domain::Extracting);
    } else {
        let _ = world.remove::<crate::domain::Extracting>(entity);
    }
    if snapshot.spectator {
        let _ = world.insert(entity, crate::domain::PlayerSpectator);
    } else {
        let _ = world.remove::<crate::domain::PlayerSpectator>(entity);
    }
    if snapshot.ai_controlled {
        let _ = world.insert(
            entity,
            crate::domain::PlayerCompanion {
                fill_slot: snapshot.roster_index,
            },
        );
    } else {
        let _ = world.remove::<crate::domain::PlayerCompanion>(entity);
    }
}
