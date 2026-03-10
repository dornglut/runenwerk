use anyhow::Result;
#[cfg(test)]
use engine::prelude::SimulationTick;
use engine::prelude::{NetworkInboundQueue, World};
use engine_net::ServerMessage;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

mod entities;
mod player;
mod snapshot;

use super::chunking::{CavernRunEventChunk, ClientRunEventChunkState};
use super::protocol::CavernRunEventCode;
use super::state::{CavernNetRuntimeState, ClientReplicationState};
#[cfg(test)]
use crate::CavernPlayerPatchOp;
use crate::{
    CavernKeyframeEvent, CavernPatchEvent, CavernRunStatePatch, ReplicationRuntimeMetrics,
};


pub(super) fn client_apply_replication_events(world: &mut World) -> Result<()> {
    let events = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .map(|queue| queue.server_messages().to_vec())
        .unwrap_or_default();
    if events.is_empty() {
        return Ok(());
    }

    let mut metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    metrics.bytes_received_last_frame = 0;
    metrics.keyframes_received_last_frame = 0;
    metrics.patches_received_last_frame = 0;
    metrics.patches_applied_last_frame = 0;
    metrics.patches_skipped_base_mismatch_last_frame = 0;
    metrics.patches_stale_ignored_last_frame = 0;
    metrics.patch_apply_micros_last = 0;
    metrics.patch_player_ops_last_tick = 0;
    metrics.patch_enemy_ops_last_tick = 0;
    metrics.patch_projectile_ops_last_tick = 0;
    metrics.patch_pickup_ops_last_tick = 0;
    metrics.patch_extraction_ops_last_tick = 0;

    let mut geometry_events = Vec::new();
    let mut keyframes_by_stream_cursor = BTreeMap::<u64, CavernKeyframeEvent>::new();
    let mut patches_by_stream_cursor = BTreeMap::<u64, CavernPatchEvent>::new();

    for message in events {
        match message {
            ServerMessage::RunEvent(run_event) => {
                if matches!(
                    CavernRunEventCode::parse(run_event.code.as_str()),
                    Some(CavernRunEventCode::Chunk)
                ) {
                    metrics.bytes_received_last_frame = metrics
                        .bytes_received_last_frame
                        .saturating_add(run_event.payload.len() as u64);
                    let chunk: CavernRunEventChunk = postcard::from_bytes(&run_event.payload)?;
                    if world.resource::<ClientRunEventChunkState>().is_err() {
                        world.insert_resource(ClientRunEventChunkState::default());
                    }
                    let reassembled = {
                        let mut state = world.resource_mut::<ClientRunEventChunkState>()?;
                        state.consume_chunk(chunk)
                    };
                    if let Some(run_event) = reassembled {
                        route_run_event(
                            run_event,
                            false,
                            &mut metrics,
                            &mut geometry_events,
                            &mut keyframes_by_stream_cursor,
                            &mut patches_by_stream_cursor,
                        )?;
                    }
                } else {
                    route_run_event(
                        run_event,
                        true,
                        &mut metrics,
                        &mut geometry_events,
                        &mut keyframes_by_stream_cursor,
                        &mut patches_by_stream_cursor,
                    )?;
                }
            }
            _ => {}
        }
    }

    for event in geometry_events {
        snapshot::apply_authoritative_geometry_edits(world, event)?;
    }

    let highest_patch_stream_cursor = patches_by_stream_cursor.keys().next_back().copied();

    let mut ordered_stream_cursors = BTreeSet::new();
    ordered_stream_cursors.extend(keyframes_by_stream_cursor.keys().copied());
    ordered_stream_cursors.extend(patches_by_stream_cursor.keys().copied());

    let mut replication_state = world.resource_mut::<ClientReplicationState>()?;
    for stream_cursor in ordered_stream_cursors {
        if let Some(keyframe) = keyframes_by_stream_cursor.remove(&stream_cursor) {
            let keyframe_is_newer =
                keyframe.cursor.stream_cursor > replication_state.last_cursor.stream_cursor;
            let can_accept_without_restore = replication_state.has_keyframe
                && keyframe.cursor.base_cursor == replication_state.last_cursor.stream_cursor;
            if keyframe_is_newer && can_accept_without_restore {
                let cursor = keyframe.cursor;
                let snapshot = keyframe.snapshot;
                drop(replication_state);
                if let Ok(mut net_state) = world.resource_mut::<CavernNetRuntimeState>() {
                    net_state.last_received_cursor = cursor.stream_cursor;
                    net_state.last_received_snapshot = Some(snapshot);
                }
                let mut state = world.resource_mut::<ClientReplicationState>()?;
                state.last_cursor = cursor;
                state.has_keyframe = true;
                replication_state = state;
                metrics.keyframes_applied = metrics.keyframes_applied.saturating_add(1);
            } else if keyframe_is_newer {
                drop(replication_state);
                snapshot::apply_authoritative_cavern_snapshot(
                    world,
                    keyframe.cursor.server_tick,
                    keyframe.cursor.stream_cursor,
                    keyframe.snapshot,
                )?;
                let mut state = world.resource_mut::<ClientReplicationState>()?;
                state.last_cursor = keyframe.cursor;
                state.has_keyframe = true;
                replication_state = state;
                metrics.keyframes_applied = metrics.keyframes_applied.saturating_add(1);
            }
        }

        if let Some(patch) = patches_by_stream_cursor.remove(&stream_cursor) {
            if patch.cursor.stream_cursor <= replication_state.last_cursor.stream_cursor {
                metrics.patches_stale_ignored_last_frame =
                    metrics.patches_stale_ignored_last_frame.saturating_add(1);
                continue;
            }

            let can_apply = replication_state.has_keyframe
                && replication_state.last_cursor.stream_cursor == patch.cursor.base_cursor;
            if can_apply {
                let start = Instant::now();
                let cursor = patch.cursor;
                let apply_local_owned_correction = highest_patch_stream_cursor
                    .map(|highest| cursor.stream_cursor == highest)
                    .unwrap_or(true);
                let player_ops_len = patch.player_ops.len() as u64;
                let enemy_ops_len = patch.enemy_ops.len() as u64;
                let projectile_ops_len = patch.projectile_ops.len() as u64;
                let pickup_ops_len = patch.pickup_ops.len() as u64;
                let extraction_ops_len = patch.extraction_ops.len() as u64;
                drop(replication_state);
                apply_patch_event(world, patch, apply_local_owned_correction)?;
                let elapsed = start.elapsed();
                let micros = elapsed.as_micros().min(u64::MAX as u128) as u64;
                let mut state = world.resource_mut::<ClientReplicationState>()?;
                state.last_cursor = cursor;
                state.has_keyframe = true;
                replication_state = state;

                metrics.patch_player_ops_last_tick = metrics
                    .patch_player_ops_last_tick
                    .saturating_add(player_ops_len);
                metrics.patch_enemy_ops_last_tick = metrics
                    .patch_enemy_ops_last_tick
                    .saturating_add(enemy_ops_len);
                metrics.patch_projectile_ops_last_tick = metrics
                    .patch_projectile_ops_last_tick
                    .saturating_add(projectile_ops_len);
                metrics.patch_pickup_ops_last_tick = metrics
                    .patch_pickup_ops_last_tick
                    .saturating_add(pickup_ops_len);
                metrics.patch_extraction_ops_last_tick = metrics
                    .patch_extraction_ops_last_tick
                    .saturating_add(extraction_ops_len);
                metrics.patch_apply_micros_last =
                    metrics.patch_apply_micros_last.saturating_add(micros);
                metrics.patch_apply_micros_total =
                    metrics.patch_apply_micros_total.saturating_add(micros);
                metrics.patches_applied = metrics.patches_applied.saturating_add(1);
                metrics.patches_applied_last_frame =
                    metrics.patches_applied_last_frame.saturating_add(1);
            } else {
                metrics.patches_skipped_base_mismatch_last_frame = metrics
                    .patches_skipped_base_mismatch_last_frame
                    .saturating_add(1);
            }
        }
    }

    metrics.bytes_received_total = metrics
        .bytes_received_total
        .saturating_add(metrics.bytes_received_last_frame);
    let (full_world_restores, local_correction_distance_last, local_correction_hard_snaps_total) =
        world
            .resource::<ReplicationRuntimeMetrics>()
            .ok()
            .map(|state| {
                (
                    state.full_world_restores,
                    state.local_correction_distance_last,
                    state.local_correction_hard_snaps_total,
                )
            })
            .unwrap_or((
                metrics.full_world_restores,
                metrics.local_correction_distance_last,
                metrics.local_correction_hard_snaps_total,
            ));
    metrics.full_world_restores = full_world_restores;
    metrics.local_correction_distance_last = local_correction_distance_last;
    metrics.local_correction_hard_snaps_total = local_correction_hard_snaps_total;
    world.insert_resource(metrics);
    Ok(())
}

fn route_run_event(
    run_event: engine_net::RunEvent,
    count_payload_bytes: bool,
    metrics: &mut ReplicationRuntimeMetrics,
    geometry_events: &mut Vec<snapshot::CavernGeometryEditsEvent>,
    keyframes_by_stream_cursor: &mut BTreeMap<u64, CavernKeyframeEvent>,
    patches_by_stream_cursor: &mut BTreeMap<u64, CavernPatchEvent>,
) -> Result<()> {
    if count_payload_bytes {
        metrics.bytes_received_last_frame = metrics
            .bytes_received_last_frame
            .saturating_add(run_event.payload.len() as u64);
    }

    match CavernRunEventCode::parse(run_event.code.as_str()) {
        Some(CavernRunEventCode::GeometryEdits) => {
            let event: snapshot::CavernGeometryEditsEvent =
                postcard::from_bytes(&run_event.payload)?;
            geometry_events.push(event);
        }
        Some(CavernRunEventCode::Keyframe) => {
            metrics.keyframes_received_last_frame =
                metrics.keyframes_received_last_frame.saturating_add(1);
            let event: CavernKeyframeEvent = postcard::from_bytes(&run_event.payload)?;
            keyframes_by_stream_cursor.insert(event.cursor.stream_cursor, event);
        }
        Some(CavernRunEventCode::Patch) => {
            metrics.patches_received_last_frame = metrics
                .patches_received_last_frame
                .saturating_add(1);
            let event: CavernPatchEvent = postcard::from_bytes(&run_event.payload)?;
            patches_by_stream_cursor.insert(event.cursor.stream_cursor, event);
        }
        Some(CavernRunEventCode::Chunk) | None => {}
    }
    Ok(())
}

fn apply_patch_event(
    world: &mut World,
    patch: CavernPatchEvent,
    apply_local_owned_correction: bool,
) -> Result<()> {
    let authoritative_tick = patch.cursor.server_tick;
    if let Some(run_state) = patch.run_state {
        apply_run_state_patch(world, run_state);
    }
    player::apply_player_patch_ops(
        world,
        patch.player_ops,
        Some(authoritative_tick),
        apply_local_owned_correction,
    )?;
    entities::apply_enemy_patch_ops(world, patch.enemy_ops)?;
    entities::apply_projectile_patch_ops(world, patch.projectile_ops)?;
    entities::apply_pickup_patch_ops(world, patch.pickup_ops)?;
    entities::apply_extraction_patch_ops(world, patch.extraction_ops)?;
    Ok(())
}

fn apply_run_state_patch(world: &mut World, patch: CavernRunStatePatch) {
    if let Ok(mut run) = world.resource_mut::<crate::CavernRunState>() {
        if let Some(phase) = patch.phase {
            run.phase = phase;
        }
        if let Some(elite_defeated) = patch.elite_defeated {
            run.elite_defeated = elite_defeated;
        }
        if let Some(extraction_active) = patch.extraction_active {
            run.extraction_active = extraction_active;
        }
        if let Some(extraction_started_at_tick) = patch.extraction_started_at_tick {
            run.extraction_started_at_tick = extraction_started_at_tick;
        }
        if let Some(party_alive_count) = patch.party_alive_count {
            run.party_alive_count = party_alive_count;
        }
        if let Some(enemy_kills) = patch.enemy_kills {
            run.enemy_kills = enemy_kills;
        }
    }
    if let Some(objective) = patch.objective {
        world.insert_resource(objective);
    }
    if let Some(extraction) = patch.extraction {
        world.insert_resource(extraction);
    }
}

#[cfg(test)]
pub(super) fn apply_player_patch_ops(
    world: &mut World,
    ops: Vec<CavernPlayerPatchOp>,
    cursor_authoritative_tick: Option<SimulationTick>,
    apply_local_owned_correction: bool,
) -> Result<()> {
    player::apply_player_patch_ops(
        world,
        ops,
        cursor_authoritative_tick,
        apply_local_owned_correction,
    )
}

