use std::collections::BTreeSet;

use anyhow::Result;
use engine::prelude::{
    AuthorityRole, NetworkServerOutbox, SimulationProfileConfig, SimulationTick, World,
};
use engine_net::{RunEvent, ServerMessage, ServerSessionState};
use serde::{Deserialize, Serialize};

use super::legacy::ServerReplicationStateByConnection;
use crate::domain::{
    CavernGeometryRuntimeState, CavernKeyframeEventV2, CavernRunSnapshotV1, GeometryEditEvent,
    GeometryPrimitiveId, ReplicationBudgetConfig, ReplicationCadenceConfig, ReplicationCursor,
    ReplicationKeyframeConfig, ReplicationLoadShedConfig, ReplicationRuntimeMetrics,
    ServerReplicationMap, capture_cavern_run_snapshot,
};

mod patch_builder;
use patch_builder::{PatchBuildStats, build_patch_event_v2};

const RUN_EVENT_GEOMETRY_EDITS: &str = "cavern_hunt.geometry.edits.v1";
const RUN_EVENT_KEYFRAME_V2: &str = "cavern_hunt.keyframe.v2";
const RUN_EVENT_PATCH_V2: &str = "cavern_hunt.patch.v2";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernGeometryEditsEventV1 {
    tick: SimulationTick,
    from_index: usize,
    to_index: usize,
    extraction_seal_primitive: Option<GeometryPrimitiveId>,
    edits: Vec<GeometryEditEvent>,
}

pub(super) fn server_emit_replication(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }
    if world.resource::<NetworkServerOutbox>().is_err() {
        return Ok(());
    }
    // V2-only runtime protocol.
    server_emit_replication_v2(world)
}

pub(super) fn strip_network_only_geometry(
    mut snapshot: CavernRunSnapshotV1,
) -> CavernRunSnapshotV1 {
    snapshot.topology = None;
    snapshot.geometry = None;
    snapshot
}

pub(super) fn server_emit_replication_v2(world: &mut World) -> Result<()> {
    let mut metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let previous_sent_bytes = metrics.bytes_sent_last_tick;
    let previous_dropped_ops = metrics
        .dropped_enemy_ops_last_tick
        .saturating_add(metrics.dropped_projectile_ops_last_tick)
        .saturating_add(metrics.dropped_pickup_ops_last_tick)
        .saturating_add(metrics.dropped_extraction_ops_last_tick);
    metrics.bytes_sent_last_tick = 0;
    metrics.load_shed_level_last_tick = 0;
    metrics.bytes_sent_geometry_last_tick = 0;
    metrics.bytes_sent_keyframe_last_tick = 0;
    metrics.bytes_sent_patch_last_tick = 0;
    metrics.bytes_sent_player_ops_last_tick = 0;
    metrics.bytes_sent_enemy_ops_last_tick = 0;
    metrics.bytes_sent_projectile_ops_last_tick = 0;
    metrics.bytes_sent_pickup_ops_last_tick = 0;
    metrics.bytes_sent_extraction_ops_last_tick = 0;
    metrics.patch_player_ops_last_tick = 0;
    metrics.patch_enemy_ops_last_tick = 0;
    metrics.patch_projectile_ops_last_tick = 0;
    metrics.patch_pickup_ops_last_tick = 0;
    metrics.patch_extraction_ops_last_tick = 0;
    metrics.dropped_enemy_ops_last_tick = 0;
    metrics.dropped_projectile_ops_last_tick = 0;
    metrics.dropped_pickup_ops_last_tick = 0;
    metrics.dropped_extraction_ops_last_tick = 0;
    let budget_config = world
        .resource::<ReplicationBudgetConfig>()
        .copied()
        .unwrap_or_default();
    let cadence_config = world
        .resource::<ReplicationCadenceConfig>()
        .copied()
        .unwrap_or_default();
    let load_shed_config = world
        .resource::<ReplicationLoadShedConfig>()
        .copied()
        .unwrap_or_default();
    let keyframe_config = world
        .resource::<ReplicationKeyframeConfig>()
        .copied()
        .unwrap_or_default();

    let active_connections = world
        .resource::<ServerSessionState>()
        .ok()
        .map(|session| {
            session
                .active_connections
                .iter()
                .map(|connection| connection.0)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if active_connections.is_empty() {
        if let Ok(mut state) = world.resource_mut::<ServerReplicationStateByConnection>() {
            *state = ServerReplicationStateByConnection::default();
        }
        metrics.bytes_sent_last_tick = 0;
        world.insert_resource(metrics);
        return Ok(());
    }
    let connection_count = active_connections.len();

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let snapshot = strip_network_only_geometry(capture_cavern_run_snapshot(world)?);

    {
        let mut by_connection = world.resource_mut::<ServerReplicationStateByConnection>()?;
        by_connection
            .cursors_by_connection
            .retain(|connection_id, _| active_connections.contains(connection_id));
        for connection_id in &active_connections {
            by_connection
                .cursors_by_connection
                .entry(*connection_id)
                .or_default();
        }
    }

    let geometry_event_payload = {
        let runtime = world
            .resource::<CavernGeometryRuntimeState>()
            .cloned()
            .unwrap_or_default();
        let state = world.resource::<ServerReplicationStateByConnection>()?;
        let from_index = state
            .last_snapshot
            .as_ref()
            .map(|prev| prev.geometry_edits.len())
            .unwrap_or(0);
        let to_index = runtime.edit_events.len();
        if to_index > from_index {
            let extraction_seal_primitive = snapshot.extraction_seal_primitive;
            let event = CavernGeometryEditsEventV1 {
                tick,
                from_index,
                to_index,
                extraction_seal_primitive,
                edits: runtime.edit_events[from_index..to_index].to_vec(),
            };
            Some(postcard::to_allocvec(&event)?)
        } else {
            None
        }
    };

    if let Some(payload) = geometry_event_payload
        && let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>()
    {
        let geometry_bytes = payload.len() as u64;
        metrics.bytes_sent_last_tick = metrics.bytes_sent_last_tick.saturating_add(geometry_bytes);
        metrics.bytes_sent_geometry_last_tick = metrics
            .bytes_sent_geometry_last_tick
            .saturating_add(geometry_bytes);
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_GEOMETRY_EDITS.to_string(),
            payload,
        }));
    }

    let load_shed_level = compute_load_shed_level_v2(
        previous_sent_bytes,
        previous_dropped_ops,
        connection_count,
        &load_shed_config,
    );
    metrics.load_shed_level_last_tick = load_shed_level;
    let patch_emit_interval = patch_emit_interval(&cadence_config, load_shed_level).max(1);
    let keyframe_interval = keyframe_config.interval_ticks.max(1);

    let emit = {
        let state = world.resource::<ServerReplicationStateByConnection>()?;
        state.latest_cursor.stream_cursor == 0
            || tick.0 % patch_emit_interval == 0
            || tick.0 % keyframe_interval == 0
    };
    if !emit {
        metrics.bytes_sent_total = metrics
            .bytes_sent_total
            .saturating_add(metrics.bytes_sent_last_tick);
        world.insert_resource(metrics);
        return Ok(());
    }

    let mut replication_map = world
        .resource::<ServerReplicationMap>()
        .cloned()
        .unwrap_or_default();
    let (event_code, payload_len, payload, patch_stats) = {
        let mut state = world.resource_mut::<ServerReplicationStateByConnection>()?;
        let previous_snapshot = state.last_snapshot.clone();
        let base_cursor = state.latest_cursor.stream_cursor;
        let stream_cursor = base_cursor.saturating_add(1);
        let cursor = ReplicationCursor {
            server_tick: tick,
            stream_cursor,
            base_cursor,
        };
        let emit_keyframe = stream_cursor == 1 || tick.0 % keyframe_interval == 0;

        let (event_code, payload_len, payload, patch_stats) = if emit_keyframe {
            let payload = postcard::to_allocvec(&CavernKeyframeEventV2 {
                cursor,
                snapshot: snapshot.clone(),
            })?;
            (
                RUN_EVENT_KEYFRAME_V2,
                payload.len(),
                payload,
                PatchBuildStats::default(),
            )
        } else {
            let (patch, patch_stats) = build_patch_event_v2(
                &mut replication_map,
                cursor,
                previous_snapshot.as_ref(),
                &snapshot,
                load_shed_level,
                &budget_config,
                &cadence_config,
            );
            metrics.patch_player_ops_last_tick = patch.player_ops.len() as u64;
            metrics.patch_enemy_ops_last_tick = patch.enemy_ops.len() as u64;
            metrics.patch_projectile_ops_last_tick = patch.projectile_ops.len() as u64;
            metrics.patch_pickup_ops_last_tick = patch.pickup_ops.len() as u64;
            metrics.patch_extraction_ops_last_tick = patch.extraction_ops.len() as u64;
            metrics.bytes_sent_player_ops_last_tick =
                postcard::to_allocvec(&patch.player_ops)?.len() as u64;
            metrics.bytes_sent_enemy_ops_last_tick =
                postcard::to_allocvec(&patch.enemy_ops)?.len() as u64;
            metrics.bytes_sent_projectile_ops_last_tick =
                postcard::to_allocvec(&patch.projectile_ops)?.len() as u64;
            metrics.bytes_sent_pickup_ops_last_tick =
                postcard::to_allocvec(&patch.pickup_ops)?.len() as u64;
            metrics.bytes_sent_extraction_ops_last_tick =
                postcard::to_allocvec(&patch.extraction_ops)?.len() as u64;
            let payload = postcard::to_allocvec(&patch)?;
            (RUN_EVENT_PATCH_V2, payload.len(), payload, patch_stats)
        };

        state.latest_cursor = cursor;
        state.last_snapshot = Some(snapshot.clone());
        for connection_id in &active_connections {
            state.cursors_by_connection.insert(*connection_id, cursor);
        }

        (event_code, payload_len, payload, patch_stats)
    };

    let payload_len = payload_len as u64;
    metrics.bytes_sent_last_tick = metrics.bytes_sent_last_tick.saturating_add(payload_len);
    match event_code {
        RUN_EVENT_KEYFRAME_V2 => {
            metrics.bytes_sent_keyframe_last_tick = metrics
                .bytes_sent_keyframe_last_tick
                .saturating_add(payload_len);
        }
        RUN_EVENT_PATCH_V2 => {
            metrics.bytes_sent_patch_last_tick = metrics
                .bytes_sent_patch_last_tick
                .saturating_add(payload_len);
            metrics.dropped_enemy_ops_last_tick = patch_stats.dropped_enemy_ops;
            metrics.dropped_projectile_ops_last_tick = patch_stats.dropped_projectile_ops;
            metrics.dropped_pickup_ops_last_tick = patch_stats.dropped_pickup_ops;
            metrics.dropped_extraction_ops_last_tick = patch_stats.dropped_extraction_ops;
            metrics.dropped_enemy_ops_total = metrics
                .dropped_enemy_ops_total
                .saturating_add(patch_stats.dropped_enemy_ops);
            metrics.dropped_projectile_ops_total = metrics
                .dropped_projectile_ops_total
                .saturating_add(patch_stats.dropped_projectile_ops);
            metrics.dropped_pickup_ops_total = metrics
                .dropped_pickup_ops_total
                .saturating_add(patch_stats.dropped_pickup_ops);
            metrics.dropped_extraction_ops_total = metrics
                .dropped_extraction_ops_total
                .saturating_add(patch_stats.dropped_extraction_ops);
        }
        _ => {}
    }

    if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: event_code.to_string(),
            payload,
        }));
    }

    world.insert_resource(replication_map);
    metrics.bytes_sent_total = metrics
        .bytes_sent_total
        .saturating_add(metrics.bytes_sent_last_tick);
    world.insert_resource(metrics);
    Ok(())
}

pub(super) fn compute_load_shed_level_v2(
    previous_sent_bytes: u64,
    previous_dropped_ops: u64,
    connection_count: usize,
    config: &ReplicationLoadShedConfig,
) -> u8 {
    let mut level = if previous_sent_bytes > config.bytes_threshold_level2 {
        2
    } else if previous_sent_bytes > config.bytes_threshold_level1 {
        1
    } else {
        0
    };
    if previous_dropped_ops >= config.dropped_ops_threshold_level1
        && config.dropped_ops_threshold_level1 > 0
    {
        level = level.max(1);
    }
    if previous_dropped_ops >= config.dropped_ops_threshold_level2
        && config.dropped_ops_threshold_level2 > 0
    {
        level = 2;
    }
    if connection_count >= config.connections_force_level1_at_or_above.max(1) {
        level = level.max(1);
    }
    if connection_count > config.connections_force_level1_at_or_above
        && previous_sent_bytes > config.connections_force_level2_bytes_threshold
    {
        level = 2;
    }
    level
}

pub(super) fn should_emit_patch_channel(stream_cursor: u64, interval_ticks: u64) -> bool {
    match interval_ticks {
        0 => false,
        1 => true,
        interval => stream_cursor % interval == 0,
    }
}

fn patch_emit_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.patch_emit_interval_level0,
        1 => config.patch_emit_interval_level1,
        _ => config.patch_emit_interval_level2,
    }
}
