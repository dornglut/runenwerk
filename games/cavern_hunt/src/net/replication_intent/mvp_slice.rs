use super::components::{
    HealthReplicated, PlayerInputReplicated, PlayerReplicatedEntity, PlayerStateReplicated,
};
use crate::{
    CavernPlayerOwnershipState, CavernServerControlMap, Health, Player, PlayerId, Transform2,
    Velocity2,
};
use engine::prelude::{Entity, World};
use engine_net::protocol::{
    ComponentRemove, ComponentUpsert, EntityDespawn, EntitySpawn, Snapshot, SnapshotPayload,
};
use engine_net::replication::{NetEntityMap, SnapshotTimeline};
use engine_net::{ConnectionId, NetEntityId, SimulationTick};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, ecs::Resource)]
pub struct MvpClientEntityMap {
    by_net_entity: BTreeMap<NetEntityId, Entity>,
}

impl MvpClientEntityMap {
    pub fn resolve_entity(&self, net_entity_id: NetEntityId) -> Option<Entity> {
        self.by_net_entity.get(&net_entity_id).copied()
    }
}

pub fn build_mvp_full_snapshot(
    timeline: &mut SnapshotTimeline,
    tick: SimulationTick,
    world: &World,
    viewer: ConnectionId,
    ownership: &CavernPlayerOwnershipState,
    controls: &CavernServerControlMap,
    net_entity_map: &mut NetEntityMap,
) -> Result<Snapshot, postcard::Error> {
    let payload = build_mvp_snapshot_payload(world, viewer, ownership, controls, net_entity_map)?;
    timeline.build_full_snapshot(tick, payload)
}

pub fn build_mvp_snapshot_payload(
    world: &World,
    viewer: ConnectionId,
    ownership: &CavernPlayerOwnershipState,
    controls: &CavernServerControlMap,
    net_entity_map: &mut NetEntityMap,
) -> Result<SnapshotPayload, postcard::Error> {
    let owner_player_id = ownership.by_connection_id.get(&viewer.0).copied();
    let mut payload = SnapshotPayload::default();

    let player_query = world.query_state::<(Entity, &PlayerId), ()>();
    for (entity, player_id) in player_query.iter(world) {
        let Some(transform) = world.get::<Transform2>(entity).copied() else {
            continue;
        };
        let Some(velocity) = world.get::<Velocity2>(entity).copied() else {
            continue;
        };
        let Some(health) = world.get::<Health>(entity).copied() else {
            continue;
        };
        let net_entity_id = net_entity_map.get_or_assign(entity_key(entity));
        payload.spawns.push(EntitySpawn {
            net_entity_id,
            prefab: Some("Player".to_string()),
        });

        let authoritative_input_tick = controls
            .by_player_id
            .get(&player_id.0)
            .map(|control| control.source_tick)
            .unwrap_or_default();

        let player_state = PlayerStateReplicated::from_components(
            player_id.0,
            transform,
            velocity,
            authoritative_input_tick,
        );
        payload.upserts.push(ComponentUpsert {
            net_entity_id,
            component_name: player_state_component_name().to_string(),
            payload: postcard::to_allocvec(&player_state)?,
        });

        let health_state = HealthReplicated::from_health(player_id.0, health);
        payload.upserts.push(ComponentUpsert {
            net_entity_id,
            component_name: health_component_name().to_string(),
            payload: postcard::to_allocvec(&health_state)?,
        });

        if owner_player_id == Some(player_id.0)
            && let Some(control) = controls.by_player_id.get(&player_id.0).copied()
        {
            let input_state = PlayerInputReplicated::from_control(player_id.0, control);
            payload.upserts.push(ComponentUpsert {
                net_entity_id,
                component_name: player_input_component_name().to_string(),
                payload: postcard::to_allocvec(&input_state)?,
            });
        }
    }

    Ok(payload)
}

pub fn apply_mvp_snapshot_payload(
    world: &mut World,
    payload: &SnapshotPayload,
    entity_map: &mut MvpClientEntityMap,
) -> Result<(), String> {
    for spawn in &payload.spawns {
        ensure_entity(world, entity_map, spawn.net_entity_id);
    }

    for despawn in &payload.despawns {
        if let Some(entity) = entity_map.by_net_entity.remove(&despawn.net_entity_id) {
            let _ = world.despawn(entity);
        }
    }

    for upsert in &payload.upserts {
        let entity = ensure_entity(world, entity_map, upsert.net_entity_id);
        match upsert.component_name.as_str() {
            name if name == player_state_component_name() => {
                let value: PlayerStateReplicated = postcard::from_bytes(&upsert.payload)
                    .map_err(|error| format!("decode PlayerStateReplicated failed: {error}"))?;
                let (transform, velocity) = value.into_transform_velocity();
                let _ = world.insert(entity, PlayerId(value.player_id));
                let _ = world.insert(entity, transform);
                let _ = world.insert(entity, velocity);
            }
            name if name == health_component_name() => {
                let value: HealthReplicated = postcard::from_bytes(&upsert.payload)
                    .map_err(|error| format!("decode HealthReplicated failed: {error}"))?;
                let _ = world.insert(entity, value.into_health());
            }
            name if name == player_input_component_name() => {
                let value: PlayerInputReplicated = postcard::from_bytes(&upsert.payload)
                    .map_err(|error| format!("decode PlayerInputReplicated failed: {error}"))?;
                let _ = world.insert(entity, value);
            }
            _ => {}
        }
    }

    for remove in &payload.removes {
        if let Some(entity) = entity_map.resolve_entity(remove.net_entity_id) {
            remove_component(world, entity, remove);
        }
    }

    Ok(())
}

fn remove_component(world: &mut World, entity: Entity, remove: &ComponentRemove) {
    match remove.component_name.as_str() {
        name if name == player_state_component_name() => {
            let _ = world.remove::<Transform2>(entity);
            let _ = world.remove::<Velocity2>(entity);
        }
        name if name == health_component_name() => {
            let _ = world.remove::<Health>(entity);
        }
        name if name == player_input_component_name() => {
            let _ = world.remove::<PlayerInputReplicated>(entity);
        }
        _ => {}
    }
}

#[allow(dead_code)]
pub fn push_despawn(payload: &mut SnapshotPayload, net_entity_id: NetEntityId) {
    payload.despawns.push(EntityDespawn { net_entity_id });
}

fn ensure_entity(
    world: &mut World,
    map: &mut MvpClientEntityMap,
    net_entity_id: NetEntityId,
) -> Entity {
    if let Some(entity) = map.by_net_entity.get(&net_entity_id).copied() {
        return entity;
    }
    let entity = world.spawn(Player);
    let _ = world.insert(entity, PlayerReplicatedEntity);
    map.by_net_entity.insert(net_entity_id, entity);
    entity
}

fn entity_key(entity: Entity) -> u64 {
    ((entity.generation as u64) << 32) | entity.id as u64
}

pub fn player_state_component_name() -> &'static str {
    "PlayerStateReplicated"
}

pub fn player_input_component_name() -> &'static str {
    "PlayerInputReplicated"
}

pub fn health_component_name() -> &'static str {
    "HealthReplicated"
}
