use anyhow::Result;
use engine::prelude::{SimulationTick, World};
use serde::Deserialize;
use std::collections::BTreeMap;

use super::super::legacy::{
    CavernNetSyncMode, CavernNetSyncState, ClientReplicationStateV2, current_net_sync_mode,
};
use super::player::{
    find_player_entity_by_player_id, remote_target_from_snapshot, replay_pending_prediction_frames,
};
use crate::domain::{
    CavernCollisionField, CavernGeometryGraph, CavernGeometryRuntimeState, CavernRunSnapshotV1,
    ClientReplicationMap, CorrectionStats, GeometryEditEvent, GeometryPrimitiveId, LocalPlayerRef,
    NetworkEntityId, ReplicationRuntimeMetrics, Transform2, Velocity2, restore_cavern_run_snapshot,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(super) struct CavernGeometryEditsEventV1 {
    tick: SimulationTick,
    from_index: usize,
    to_index: usize,
    extraction_seal_primitive: Option<GeometryPrimitiveId>,
    edits: Vec<GeometryEditEvent>,
}

pub(super) fn apply_authoritative_geometry_edits(
    world: &mut World,
    event: CavernGeometryEditsEventV1,
) -> Result<()> {
    let mut runtime = world
        .resource::<CavernGeometryRuntimeState>()
        .cloned()
        .unwrap_or_default();
    if runtime.edit_events.len() != event.from_index {
        return Ok(());
    }

    let mut graph = world
        .resource::<CavernGeometryGraph>()
        .cloned()
        .unwrap_or_default();
    let mut invalidated_bounds = Vec::new();
    for edit_event in &event.edits {
        if let Some(bounds) = graph.apply_edit(&edit_event.edit) {
            invalidated_bounds.push(bounds);
        }
    }
    world.insert_resource(graph.clone());

    if !invalidated_bounds.is_empty()
        && let Ok(mut field) = world.resource_mut::<CavernCollisionField>()
    {
        for bounds in invalidated_bounds {
            field.invalidate_bounds(bounds);
        }
        field.sync_revision(&graph);
    }

    runtime.edit_events.extend(event.edits);
    runtime.extraction_seal_primitive = event.extraction_seal_primitive;
    world.insert_resource(runtime);
    Ok(())
}

pub(super) fn apply_authoritative_cavern_snapshot(
    world: &mut World,
    authoritative_tick: SimulationTick,
    cursor: u64,
    snapshot: CavernRunSnapshotV1,
) -> Result<()> {
    let preserve_local_pose = matches!(current_net_sync_mode(world), CavernNetSyncMode::V2)
        && world
            .resource::<ClientReplicationStateV2>()
            .map(|state| state.has_keyframe)
            .unwrap_or(false);
    let pre_restore_player_poses = if preserve_local_pose {
        world
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .filter_map(|(entity, player_id)| {
                world.get::<Transform2>(entity).copied().map(|transform| {
                    let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
                    (player_id.0, (transform, velocity))
                })
            })
            .collect::<BTreeMap<_, _>>()
    } else {
        BTreeMap::new()
    };
    let local_pre_restore = if preserve_local_pose {
        world
            .resource::<LocalPlayerRef>()
            .ok()
            .and_then(|local| local.player_id)
            .and_then(|player_id| {
                pre_restore_player_poses
                    .get(&player_id)
                    .copied()
                    .map(|(transform, velocity)| (player_id, transform, velocity))
            })
    } else {
        None
    };

    if let Ok(mut metrics) = world.resource_mut::<ReplicationRuntimeMetrics>() {
        metrics.full_world_restores = metrics.full_world_restores.saturating_add(1);
    }
    let local_simulated_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    restore_cavern_run_snapshot(world, &snapshot)?;
    if let Ok(mut state) = world.resource_mut::<CavernNetSyncState>() {
        state.last_received_cursor = cursor;
        state.last_received_snapshot = Some(snapshot.clone());
    }
    if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
        *tick = authoritative_tick;
    }

    let replay_authoritative_tick = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id)
        .and_then(|local_player_id| {
            snapshot
                .players
                .iter()
                .find(|player| player.player_id == local_player_id)
                .and_then(|player| player.authoritative_input_tick)
        })
        .and_then(|tick| (tick.0 > 0).then_some(tick))
        .unwrap_or(authoritative_tick);

    let replayed_frames =
        replay_pending_prediction_frames(world, replay_authoritative_tick, local_simulated_tick)?;

    if preserve_local_pose
        && replayed_frames == 0
        && let Some((local_player_id, pre_transform, pre_velocity)) = local_pre_restore
        && let Some(local_entity) = find_player_entity_by_player_id(world, local_player_id)
        && let Some(local_snapshot) = snapshot
            .players
            .iter()
            .find(|player| player.player_id == local_player_id)
    {
        let _ = world.insert(local_entity, pre_transform);
        let _ = world.insert(local_entity, pre_velocity);

        let dx = local_snapshot.x - pre_transform.x;
        let dy = local_snapshot.y - pre_transform.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let mut correction_stats = world
            .resource::<CorrectionStats>()
            .copied()
            .unwrap_or_default();
        correction_stats.last_distance = distance;
        correction_stats.total_distance += distance;
        if correction_stats.ema_distance <= f32::EPSILON {
            correction_stats.ema_distance = distance;
        } else {
            correction_stats.ema_distance += (distance - correction_stats.ema_distance) * 0.1;
        }
        let hard_snaps = correction_stats.hard_snaps;
        world.insert_resource(correction_stats);

        tracing::debug!(
            distance,
            cursor,
            authoritative_tick = authoritative_tick.0,
            replay_authoritative_tick = replay_authoritative_tick.0,
            "preserved local player pose across v2 keyframe restore"
        );

        if let Ok(mut metrics) = world.resource_mut::<ReplicationRuntimeMetrics>() {
            metrics.local_correction_distance_last = distance;
            metrics.local_correction_hard_snaps_total = hard_snaps;
        }
    }

    if preserve_local_pose {
        let local_player_id = world
            .resource::<LocalPlayerRef>()
            .ok()
            .and_then(|local| local.player_id);

        let mut remote_targets = BTreeMap::new();
        for remote_snapshot in snapshot
            .players
            .iter()
            .filter(|player| Some(player.player_id) != local_player_id)
        {
            if let Some((pre_transform, pre_velocity)) =
                pre_restore_player_poses.get(&remote_snapshot.player_id)
                && let Some(entity) =
                    find_player_entity_by_player_id(world, remote_snapshot.player_id)
            {
                let _ = world.insert(entity, *pre_transform);
                let _ = world.insert(entity, *pre_velocity);
            }
            remote_targets.insert(
                remote_snapshot.player_id,
                remote_target_from_snapshot(remote_snapshot),
            );
        }

        if let Ok(mut state_v2) = world.resource_mut::<ClientReplicationStateV2>() {
            state_v2
                .remote_targets_by_player_id
                .retain(|player_id, _| remote_targets.contains_key(player_id));
            for (player_id, target) in remote_targets {
                state_v2
                    .remote_targets_by_player_id
                    .insert(player_id, target);
            }
        }

        let mut client_map = world
            .resource::<ClientReplicationMap>()
            .cloned()
            .unwrap_or_default();
        let old_player_entity_ids = client_map
            .by_player_id
            .values()
            .copied()
            .collect::<Vec<_>>();
        for entity_id in old_player_entity_ids {
            client_map.by_network_entity_id.remove(&entity_id);
        }
        client_map.by_player_id.clear();
        for player in &snapshot.players {
            if let Some(entity) = find_player_entity_by_player_id(world, player.player_id) {
                let entity_id = NetworkEntityId(0x1000_0000 + player.player_id as u64);
                client_map.by_player_id.insert(player.player_id, entity_id);
                client_map.by_network_entity_id.insert(entity_id, entity);
            }
        }
        world.insert_resource(client_map);
    }

    Ok(())
}
