use anyhow::Result;
use engine::prelude::{NetworkSessionStatus, SimulationTick, World};

use super::super::legacy::{ClientReplicationStateV2, RemotePlayerTarget};
use crate::{
    AdaptiveSmoothingState, CavernPlayerPatchOpV2, ClientReplicationMap, CorrectionStats,
    InterpolationConfig, LocalPlayerRef, ReplicationRuntimeMetrics,
};

mod correction;
mod snapshot_state;

use correction::apply_local_player_snapshot_correction;
use snapshot_state::{apply_non_transform_player_snapshot, spawn_player_entity_from_snapshot};

pub(super) fn apply_player_patch_ops_v2(
    world: &mut World,
    ops: Vec<CavernPlayerPatchOpV2>,
    cursor_authoritative_tick: Option<SimulationTick>,
    apply_local_owned_correction: bool,
) -> Result<()> {
    let local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id);
    let local_connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|connection| connection.0));
    let interpolation = world
        .resource::<InterpolationConfig>()
        .copied()
        .unwrap_or_default();
    let smoothing = world
        .resource::<AdaptiveSmoothingState>()
        .copied()
        .unwrap_or_default();
    let jitter_factor = (smoothing.jitter_ms / 60.0).clamp(0.0, 1.5);
    let dynamic_hard_snap_distance = (interpolation.hard_snap_distance
        * (1.0 + jitter_factor * 0.35))
        .max(interpolation.large_error_distance);

    let mut client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    let mut state_v2 = world
        .resource::<ClientReplicationStateV2>()
        .cloned()
        .unwrap_or_default();
    let mut correction_stats = world
        .resource::<CorrectionStats>()
        .copied()
        .unwrap_or_default();
    correction_stats.last_distance = 0.0;
    let mut local_binding_from_owner: Option<(u32, engine::prelude::Entity)> = None;
    let mut local_binding_cleared = false;

    for op in ops {
        match op {
            CavernPlayerPatchOpV2::Despawn {
                entity_id,
                player_id,
            } => {
                if Some(player_id) == local_player_id {
                    local_binding_cleared = true;
                }
                let entity = client_map
                    .by_network_entity_id
                    .remove(&entity_id)
                    .or_else(|| find_player_entity_by_player_id(world, player_id));
                if let Some(entity) = entity {
                    let _ = world.despawn(entity);
                }
                client_map.by_player_id.remove(&player_id);
                state_v2.remote_targets_by_player_id.remove(&player_id);
            }
            CavernPlayerPatchOpV2::Spawn {
                entity_id, state, ..
            }
            | CavernPlayerPatchOpV2::Patch {
                entity_id, state, ..
            } => {
                let mut entity = client_map.by_network_entity_id.get(&entity_id).copied();
                if entity.is_none() {
                    entity = find_player_entity_by_player_id(world, state.player_id);
                }
                if entity.is_none() {
                    entity = Some(spawn_player_entity_from_snapshot(world, &state));
                }
                let entity = entity.expect("spawned entity should exist");
                client_map.by_network_entity_id.insert(entity_id, entity);
                client_map.by_player_id.insert(state.player_id, entity_id);
                let is_owned_by_this_client = local_connection_id
                    .zip(state.owner_connection_id)
                    .map(|(local, owner)| local == owner)
                    .unwrap_or(false);
                if is_owned_by_this_client {
                    local_binding_from_owner = Some((state.player_id, entity));
                }

                if Some(state.player_id) == local_player_id || is_owned_by_this_client {
                    let player_authoritative_tick = usable_authoritative_tick(
                        state.authoritative_input_tick,
                        cursor_authoritative_tick,
                    );
                    if apply_local_owned_correction {
                        apply_local_player_snapshot_correction(
                            world,
                            entity,
                            &state,
                            interpolation,
                            dynamic_hard_snap_distance,
                            player_authoritative_tick,
                            &mut correction_stats,
                        )?;
                    } else {
                        apply_non_transform_player_snapshot(world, entity, &state);
                    }
                } else {
                    apply_non_transform_player_snapshot(world, entity, &state);
                    state_v2
                        .remote_targets_by_player_id
                        .insert(state.player_id, remote_target_from_snapshot(&state));
                }
            }
        }
    }

    if let Some((player_id, entity)) = local_binding_from_owner {
        if let Ok(mut local) = world.resource_mut::<LocalPlayerRef>() {
            local.player_id = Some(player_id);
            local.entity = Some(entity);
        }
    } else if local_binding_cleared
        && local_player_id.is_some()
        && let Ok(mut local) = world.resource_mut::<LocalPlayerRef>()
        && local.player_id == local_player_id
    {
        local.player_id = None;
        local.entity = None;
    }

    if let Ok(mut metrics) = world.resource_mut::<ReplicationRuntimeMetrics>() {
        metrics.local_correction_distance_last = correction_stats.last_distance;
        metrics.local_correction_hard_snaps_total = correction_stats.hard_snaps;
    }
    world.insert_resource(client_map);
    world.insert_resource(state_v2);
    world.insert_resource(correction_stats);
    Ok(())
}

pub(super) fn find_player_entity_by_player_id(
    world: &World,
    player_id: u32,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::PlayerId)>()
        .iter()
        .find_map(|(entity, id)| (id.0 == player_id).then_some(entity))
}

pub(super) fn usable_authoritative_tick(
    snapshot_authoritative_input_tick: Option<SimulationTick>,
    cursor_authoritative_tick: Option<SimulationTick>,
) -> Option<SimulationTick> {
    snapshot_authoritative_input_tick
        .filter(|tick| tick.0 > 0)
        .or(cursor_authoritative_tick)
}

pub(super) fn remote_target_from_snapshot(
    snapshot: &crate::CavernPlayerSnapshotV1,
) -> RemotePlayerTarget {
    RemotePlayerTarget {
        pos: [snapshot.x, snapshot.y],
        velocity: [snapshot.velocity[0], snapshot.velocity[1]],
        yaw: snapshot.yaw,
    }
}

pub(super) fn replay_pending_prediction_frames(
    world: &mut World,
    authoritative_tick: SimulationTick,
    local_simulated_tick: SimulationTick,
) -> Result<usize> {
    correction::replay_pending_prediction_frames(world, authoritative_tick, local_simulated_tick)
}
