use crate::domain::{
    AdaptiveSmoothingState, CavernCollisionField, CavernControlState, CavernEnemyPatchOpV2,
    CavernExtractionPatchOpV2, CavernGeometryGraph, CavernGeometryRuntimeState,
    CavernKeyframeEventV2, CavernPatchEventV2, CavernPatchPriorityV2, CavernPickupPatchOpV2,
    CavernPlayerOwnershipState, CavernPlayerPatchOpV2, CavernPredictedFrame, CavernPredictionState,
    CavernProjectilePatchOpV2, CavernRunDeltaV1, CavernRunSnapshotV1, CavernRunStatePatchV2,
    CavernServerControlMap, ClientReplicationMap, CorrectionStats, GeometryEditEvent,
    GeometryPrimitiveId, InterpolationConfig, LocalPlayerRef, NetDiagnosticsConfigAssetV1,
    NetSyncModeConfig, NetworkEntityId, ReplicationBudgetConfig, ReplicationCadenceConfig,
    ReplicationCursor, ReplicationKeyframeConfig, ReplicationLoadShedConfig,
    ReplicationRuntimeMetrics, ServerReplicationMap, Transform2, Velocity2, apply_cavern_run_delta,
    build_cavern_run_delta, capture_cavern_run_snapshot, restore_cavern_run_snapshot,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, FixedTimeConfig, FixedUpdate, NetworkClientOutbox,
    NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus, Plugin, PreUpdate,
    RoundTripMetrics, SimulationProfileConfig, SimulationTick, SystemConfigExt, Time, Update,
    World, WorldMut,
};
use engine_net::{
    AbilityCommand, AimCommand, ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame,
    InteractCommand, MoveCommand, RunEvent, ServerMessage, ServerSessionState,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const RUN_EVENT_SNAPSHOT: &str = "cavern_hunt.snapshot.v1";
const RUN_EVENT_DELTA: &str = "cavern_hunt.delta.v1";
const RUN_EVENT_GEOMETRY_EDITS: &str = "cavern_hunt.geometry.edits.v1";
const RUN_EVENT_KEYFRAME_V2: &str = "cavern_hunt.keyframe.v2";
const RUN_EVENT_PATCH_V2: &str = "cavern_hunt.patch.v2";
// V1 cadence (fallback path)
const REPLICATION_DELTA_INTERVAL_TICKS: u64 = 3;
const REPLICATION_FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 60;
#[cfg(test)]
const ENV_NET_TUNING_PRESET: &str = "CAVERN_NET_TUNING_PRESET";

#[derive(Debug, Clone, Default, PartialEq)]
struct CavernNetSyncState {
    active_connection_id: Option<u64>,
    initial_snapshot_sent: bool,
    last_cursor: u64,
    last_sent_snapshot: Option<CavernRunSnapshotV1>,
    last_sent_geometry_edit_count: usize,
    last_received_cursor: u64,
    last_received_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
enum CavernNetSyncMode {
    #[default]
    V1,
    V2,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ServerReplicationStateByConnection {
    cursors_by_connection: BTreeMap<u64, ReplicationCursor>,
    latest_cursor: ReplicationCursor,
    last_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ClientReplicationStateV2 {
    last_cursor: ReplicationCursor,
    has_keyframe: bool,
    remote_targets_by_player_id: BTreeMap<u32, RemotePlayerTarget>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct RemotePlayerTarget {
    pos: [f32; 2],
    velocity: [f32; 2],
    yaw: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct PatchBuildStats {
    dropped_enemy_ops: u64,
    dropped_projectile_ops: u64,
    dropped_pickup_ops: u64,
    dropped_extraction_ops: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct NetSyncDiagnosticsLogState {
    last_logged_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernSnapshotEventV1 {
    tick: SimulationTick,
    cursor: u64,
    snapshot: CavernRunSnapshotV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernDeltaEventV1 {
    tick: SimulationTick,
    base_cursor: u64,
    cursor: u64,
    delta: CavernRunDeltaV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernGeometryEditsEventV1 {
    tick: SimulationTick,
    from_index: usize,
    to_index: usize,
    extraction_seal_primitive: Option<GeometryPrimitiveId>,
    edits: Vec<GeometryEditEvent>,
}

pub struct CavernHuntNetSyncPlugin;

impl Plugin for CavernHuntNetSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernNetSyncState>();
        app.init_resource::<ServerReplicationStateByConnection>();
        app.init_resource::<ClientReplicationStateV2>();
        app.init_resource::<NetSyncDiagnosticsLogState>();
        app.add_systems(
            PreUpdate,
            (
                client_send_control_input_system
                    .after(CoreSet::NetReceive)
                    .after(CoreSet::Input),
                server_capture_control_input_system.after(CoreSet::NetReceive),
                client_apply_replication_events_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(
            Update,
            (client_smoothing_system, net_sync_diagnostics_log_system),
        );
        app.add_systems(FixedUpdate, server_emit_replication_system);
    }
}

fn client_send_control_input_system(mut world: WorldMut) -> Result<()> {
    client_send_control_input(&mut world)
}

fn client_send_control_input(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }
    let phase_active = world
        .resource::<NetworkSessionStatus>()
        .map(|status| status.connected)
        .unwrap_or(false);
    if !phase_active {
        return Ok(());
    }
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .map(|tick| SimulationTick(tick.0.saturating_add(1)))
        .unwrap_or_default();
    let mut control = match world.resource::<CavernControlState>() {
        Ok(control) => *control,
        Err(_) => return Ok(()),
    };
    control.source_tick = tick;
    if let Ok(mut prediction) = world.resource_mut::<CavernPredictionState>() {
        if let Some(existing) = prediction
            .pending_frames
            .iter_mut()
            .find(|frame| frame.tick == tick)
        {
            existing.control = control;
        } else {
            prediction
                .pending_frames
                .push(CavernPredictedFrame { tick, control });
        }
        prediction.pending_frames.sort_by_key(|frame| frame.tick.0);
    }
    if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
        let mut commands = vec![
            ClientCommandEnvelope::Move(MoveCommand {
                x: control.movement[0],
                y: control.movement[1],
            }),
            ClientCommandEnvelope::Aim(AimCommand {
                x: control.aim_world[0],
                y: control.aim_world[1],
            }),
        ];
        if control.dash_pressed {
            commands.push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 0 }));
        }
        if control.fire_pressed {
            commands.push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 1 }));
        }
        if control.interact_pressed {
            commands.push(ClientCommandEnvelope::Interact(InteractCommand {
                target: None,
            }));
        }
        outbox.push(ClientMessage::InputFrame(InputFrame { tick, commands }));
    }
    Ok(())
}

fn server_capture_control_input_system(mut world: WorldMut) -> Result<()> {
    server_capture_control_input(&mut world)
}

fn server_capture_control_input(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }
    let input_frames = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .map(|queue| {
            queue
                .client_messages()
                .iter()
                .filter_map(|incoming| match &incoming.message {
                    ClientMessage::InputFrame(frame) => {
                        Some((incoming.connection_id, frame.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if input_frames.is_empty() {
        return Ok(());
    }

    let max_players = world
        .resource::<crate::domain::CavernRunConfig>()
        .map(|config| config.max_players.max(1))
        .unwrap_or(1);

    let mut ownership = world
        .resource::<CavernPlayerOwnershipState>()
        .cloned()
        .unwrap_or_default();
    if let Ok(session_state) = world.resource::<ServerSessionState>() {
        if !session_state.active_connections.is_empty() {
            ownership.retain_active_connections(
                session_state
                    .active_connections
                    .iter()
                    .map(|connection_id| connection_id.0),
            );
        }
    }
    let mut controls = world
        .resource::<CavernServerControlMap>()
        .cloned()
        .unwrap_or_default();
    let current_source_tick = world
        .resource::<CavernControlState>()
        .map(|control| control.source_tick)
        .unwrap_or_default();
    let mut latest_global_control = world
        .resource::<CavernControlState>()
        .copied()
        .unwrap_or_default();

    for (connection_id, frame) in input_frames {
        let control_state = control_state_from_frame(frame);
        if control_state.source_tick.0 >= latest_global_control.source_tick.0 {
            latest_global_control = control_state;
        }
        if control_state.source_tick.0 < current_source_tick.0 {
            continue;
        }
        if let Some(player_id) = resolve_owned_player_id(max_players, &mut ownership, connection_id)
        {
            let should_replace = controls
                .by_player_id
                .get(&player_id)
                .map(|existing| existing.source_tick.0 <= control_state.source_tick.0)
                .unwrap_or(true);
            if should_replace {
                controls.by_player_id.insert(player_id, control_state);
            }
        }
    }

    world.insert_resource(ownership);
    world.insert_resource(controls);
    if latest_global_control.source_tick.0 >= current_source_tick.0 {
        world.insert_resource(latest_global_control);
    }
    Ok(())
}

fn control_state_from_frame(frame: InputFrame) -> CavernControlState {
    let mut movement = [0.0, 0.0];
    let mut aim_world = [0.0, 0.0];
    let mut fire_pressed = false;
    let mut dash_pressed = false;
    let mut interact_pressed = false;
    for command in frame.commands {
        match command {
            ClientCommandEnvelope::Move(command) => movement = [command.x, command.y],
            ClientCommandEnvelope::Aim(command) => aim_world = [command.x, command.y],
            ClientCommandEnvelope::Ability(command) => match command.slot {
                0 => dash_pressed = true,
                1 => fire_pressed = true,
                _ => {}
            },
            ClientCommandEnvelope::Interact(_) => interact_pressed = true,
        }
    }

    CavernControlState {
        movement,
        aim_world,
        fire_pressed,
        dash_pressed,
        interact_pressed,
        source_tick: frame.tick,
    }
}

fn resolve_owned_player_id(
    max_players: u8,
    ownership: &mut CavernPlayerOwnershipState,
    connection_id: Option<ConnectionId>,
) -> Option<u32> {
    if max_players == 0 {
        ownership.by_connection_id.clear();
        return None;
    }
    let valid_player_ids = (1..=u32::from(max_players)).collect::<std::collections::BTreeSet<_>>();

    ownership
        .by_connection_id
        .retain(|_, player_id| valid_player_ids.contains(player_id));

    let Some(connection_id) = connection_id else {
        return Some(1);
    };
    if let Some(existing) = ownership.by_connection_id.get(&connection_id.0).copied() {
        return Some(existing);
    }

    let assigned = ownership
        .by_connection_id
        .values()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let player_id = (1..=u32::from(max_players))
        .find(|player_id| !assigned.contains(player_id))
        .or(Some(1))?;
    ownership
        .by_connection_id
        .insert(connection_id.0, player_id);
    Some(player_id)
}

fn server_emit_replication_system(mut world: WorldMut) -> Result<()> {
    server_emit_replication(&mut world)
}

fn server_emit_replication(world: &mut World) -> Result<()> {
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
    if matches!(current_net_sync_mode(world), CavernNetSyncMode::V2) {
        return server_emit_replication_v2(world);
    }

    let connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|id| id.0));
    if connection_id.is_none() {
        if let Ok(mut state) = world.resource_mut::<CavernNetSyncState>() {
            *state = CavernNetSyncState::default();
        }
        return Ok(());
    }

    let snapshot = strip_network_only_geometry(capture_cavern_run_snapshot(&world)?);
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let runtime_geometry = world
        .resource::<CavernGeometryRuntimeState>()
        .cloned()
        .unwrap_or_default();
    {
        let mut state = world.resource_mut::<CavernNetSyncState>()?;
        if state.active_connection_id != connection_id {
            state.active_connection_id = connection_id;
            state.initial_snapshot_sent = false;
            state.last_cursor = 0;
            state.last_sent_snapshot = None;
            state.last_sent_geometry_edit_count = 0;
        }
    }

    let geometry_event_payload = {
        let mut state = world.resource_mut::<CavernNetSyncState>()?;
        if state.initial_snapshot_sent
            && runtime_geometry.edit_events.len() > state.last_sent_geometry_edit_count
        {
            let from_index = state.last_sent_geometry_edit_count;
            let to_index = runtime_geometry.edit_events.len();
            let edits = runtime_geometry.edit_events[from_index..to_index].to_vec();
            state.last_sent_geometry_edit_count = to_index;
            Some(postcard::to_allocvec(&CavernGeometryEditsEventV1 {
                tick,
                from_index,
                to_index,
                extraction_seal_primitive: runtime_geometry.extraction_seal_primitive,
                edits,
            })?)
        } else {
            None
        }
    };

    if let Some(payload) = geometry_event_payload
        && let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>()
    {
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: RUN_EVENT_GEOMETRY_EDITS.to_string(),
            payload,
        }));
    }

    let should_emit_replication = {
        let state = world.resource::<CavernNetSyncState>()?;
        !state.initial_snapshot_sent || tick.0 % REPLICATION_DELTA_INTERVAL_TICKS == 0
    };
    if !should_emit_replication {
        return Ok(());
    }

    let (event_code, payload) = {
        let mut state = world.resource_mut::<CavernNetSyncState>()?;
        let emit_full_snapshot =
            !state.initial_snapshot_sent || tick.0 % REPLICATION_FULL_SNAPSHOT_INTERVAL_TICKS == 0;
        state.last_cursor = state.last_cursor.saturating_add(1);
        let cursor = state.last_cursor;
        if emit_full_snapshot {
            state.initial_snapshot_sent = true;
            state.last_sent_geometry_edit_count = runtime_geometry.edit_events.len();
            state.last_sent_snapshot = Some(snapshot.clone());
            (
                RUN_EVENT_SNAPSHOT.to_string(),
                postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick,
                    cursor,
                    snapshot,
                })?,
            )
        } else {
            let base_cursor = cursor.saturating_sub(1);
            let base_snapshot = state
                .last_sent_snapshot
                .clone()
                .unwrap_or_else(|| snapshot.clone());
            let delta = build_cavern_run_delta(&base_snapshot, &snapshot);
            state.last_sent_snapshot = Some(snapshot);
            (
                RUN_EVENT_DELTA.to_string(),
                postcard::to_allocvec(&CavernDeltaEventV1 {
                    tick,
                    base_cursor,
                    cursor,
                    delta,
                })?,
            )
        }
    };

    if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: event_code,
            payload,
        }));
    }
    Ok(())
}

fn strip_network_only_geometry(mut snapshot: CavernRunSnapshotV1) -> CavernRunSnapshotV1 {
    snapshot.topology = None;
    snapshot.geometry = None;
    snapshot
}

fn client_apply_replication_events_system(mut world: WorldMut) -> Result<()> {
    client_apply_replication_events(&mut world)
}

fn client_apply_replication_events(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }
    if matches!(current_net_sync_mode(world), CavernNetSyncMode::V2) {
        return client_apply_replication_events_v2(world);
    }

    let events = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .map(|queue| queue.server_messages().to_vec())
        .unwrap_or_default();
    if events.is_empty() {
        return Ok(());
    }

    let mut geometry_events = Vec::new();
    let mut latest_snapshot: Option<CavernSnapshotEventV1> = None;
    let mut latest_delta: Option<CavernDeltaEventV1> = None;

    for message in events {
        match message {
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_GEOMETRY_EDITS => {
                let event: CavernGeometryEditsEventV1 = postcard::from_bytes(&run_event.payload)?;
                geometry_events.push(event);
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_SNAPSHOT => {
                let event: CavernSnapshotEventV1 = postcard::from_bytes(&run_event.payload)?;
                if latest_snapshot
                    .as_ref()
                    .map(|latest| event.cursor > latest.cursor)
                    .unwrap_or(true)
                {
                    latest_snapshot = Some(event);
                }
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_DELTA => {
                let event: CavernDeltaEventV1 = postcard::from_bytes(&run_event.payload)?;
                if latest_delta
                    .as_ref()
                    .map(|latest| event.cursor > latest.cursor)
                    .unwrap_or(true)
                {
                    latest_delta = Some(event);
                }
            }
            _ => {}
        }
    }

    for event in geometry_events {
        apply_authoritative_geometry_edits(world, event)?;
    }

    let latest_snapshot_cursor = latest_snapshot
        .as_ref()
        .map(|event| event.cursor)
        .unwrap_or(0);
    let latest_delta_cursor = latest_delta.as_ref().map(|event| event.cursor).unwrap_or(0);
    if latest_delta_cursor > latest_snapshot_cursor {
        if let Some(event) = latest_delta {
            let rebuilt = {
                let state = world.resource::<CavernNetSyncState>()?;
                let Some(base) = state.last_received_snapshot.as_ref() else {
                    return Ok(());
                };
                if state.last_received_cursor != event.base_cursor {
                    if let Some(snapshot) = latest_snapshot {
                        apply_authoritative_cavern_snapshot(
                            world,
                            snapshot.tick,
                            snapshot.cursor,
                            snapshot.snapshot,
                        )?;
                    }
                    return Ok(());
                }
                apply_cavern_run_delta(base, &event.delta)
            };
            apply_authoritative_cavern_snapshot(world, event.tick, event.cursor, rebuilt)?;
            return Ok(());
        }
    }

    if let Some(event) = latest_snapshot {
        apply_authoritative_cavern_snapshot(world, event.tick, event.cursor, event.snapshot)?;
    }

    Ok(())
}

fn current_net_sync_mode(world: &World) -> CavernNetSyncMode {
    match world
        .resource::<NetSyncModeConfig>()
        .copied()
        .unwrap_or(NetSyncModeConfig::V1)
    {
        NetSyncModeConfig::V1 => CavernNetSyncMode::V1,
        NetSyncModeConfig::V2 => CavernNetSyncMode::V2,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn configure_replication_tuning_from_env_system(mut world: WorldMut) -> Result<()> {
    configure_replication_tuning_from_env(&mut world)
}

#[cfg(test)]
#[allow(dead_code)]
fn configure_replication_tuning_from_env(world: &mut World) -> Result<()> {
    let mut budget = world
        .resource::<ReplicationBudgetConfig>()
        .copied()
        .unwrap_or_default();
    let mut cadence = world
        .resource::<ReplicationCadenceConfig>()
        .copied()
        .unwrap_or_default();
    let mut interpolation = world
        .resource::<InterpolationConfig>()
        .copied()
        .unwrap_or_default();
    let mut diagnostics = Vec::new();
    let preset = std::env::var(ENV_NET_TUNING_PRESET).ok();

    apply_replication_tuning_preset(
        &mut budget,
        &mut cadence,
        preset.as_deref(),
        &mut diagnostics,
    );
    apply_replication_tuning_overrides_from_reader(
        &mut budget,
        &mut cadence,
        |key| std::env::var(key).ok(),
        &mut diagnostics,
    );
    apply_interpolation_overrides_from_reader(
        &mut interpolation,
        |key| std::env::var(key).ok(),
        &mut diagnostics,
    );

    world.insert_resource(budget);
    world.insert_resource(cadence);
    world.insert_resource(interpolation);

    for diagnostic in diagnostics {
        tracing::warn!(diagnostic = %diagnostic, "cavern net tuning diagnostic");
    }
    tracing::info!(
        preset = preset.unwrap_or_else(|| "default".to_string()),
        ?budget,
        ?cadence,
        ?interpolation,
        "cavern net replication tuning ready"
    );
    Ok(())
}

#[cfg(test)]
fn apply_replication_tuning_preset(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    preset: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    let Some(raw) = preset else {
        return;
    };
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "two_local" | "2local" | "balanced" => {
            budget.enemy_ops_per_patch_level0 = 160;
            budget.enemy_ops_per_patch_level1 = 96;
            budget.enemy_ops_per_patch_level2 = 48;
            budget.projectile_ops_per_patch_level0 = 320;
            budget.projectile_ops_per_patch_level1 = 176;
            budget.projectile_ops_per_patch_level2 = 88;
            budget.pickup_ops_per_patch_level0 = 64;
            budget.pickup_ops_per_patch_level1 = 40;
            budget.pickup_ops_per_patch_level2 = 20;
            budget.extraction_ops_per_patch_level0 = 16;
            budget.extraction_ops_per_patch_level1 = 10;
            budget.extraction_ops_per_patch_level2 = 6;

            cadence.patch_emit_interval_level0 = 1;
            cadence.patch_emit_interval_level1 = 1;
            cadence.patch_emit_interval_level2 = 2;
            cadence.enemy_patch_interval_level0 = 1;
            cadence.enemy_patch_interval_level1 = 2;
            cadence.enemy_patch_interval_level2 = 3;
            cadence.projectile_patch_interval_level0 = 1;
            cadence.projectile_patch_interval_level1 = 2;
            cadence.projectile_patch_interval_level2 = 2;
            cadence.pickup_patch_interval_level0 = 4;
            cadence.pickup_patch_interval_level1 = 5;
            cadence.pickup_patch_interval_level2 = 8;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 1;
            cadence.extraction_patch_interval_level2 = 1;
        }
        "four_local" | "4local" | "conservative" => {
            budget.enemy_ops_per_patch_level0 = 96;
            budget.enemy_ops_per_patch_level1 = 56;
            budget.enemy_ops_per_patch_level2 = 28;
            budget.projectile_ops_per_patch_level0 = 160;
            budget.projectile_ops_per_patch_level1 = 96;
            budget.projectile_ops_per_patch_level2 = 48;
            budget.pickup_ops_per_patch_level0 = 32;
            budget.pickup_ops_per_patch_level1 = 16;
            budget.pickup_ops_per_patch_level2 = 8;
            budget.extraction_ops_per_patch_level0 = 12;
            budget.extraction_ops_per_patch_level1 = 8;
            budget.extraction_ops_per_patch_level2 = 4;

            cadence.patch_emit_interval_level0 = 2;
            cadence.patch_emit_interval_level1 = 2;
            cadence.patch_emit_interval_level2 = 3;
            cadence.enemy_patch_interval_level0 = 2;
            cadence.enemy_patch_interval_level1 = 3;
            cadence.enemy_patch_interval_level2 = 4;
            cadence.projectile_patch_interval_level0 = 2;
            cadence.projectile_patch_interval_level1 = 3;
            cadence.projectile_patch_interval_level2 = 4;
            cadence.pickup_patch_interval_level0 = 6;
            cadence.pickup_patch_interval_level1 = 8;
            cadence.pickup_patch_interval_level2 = 12;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 2;
            cadence.extraction_patch_interval_level2 = 2;
        }
        "aggressive" | "lan" => {
            budget.enemy_ops_per_patch_level0 = 256;
            budget.enemy_ops_per_patch_level1 = 160;
            budget.enemy_ops_per_patch_level2 = 80;
            budget.projectile_ops_per_patch_level0 = 512;
            budget.projectile_ops_per_patch_level1 = 320;
            budget.projectile_ops_per_patch_level2 = 160;
            budget.pickup_ops_per_patch_level0 = 128;
            budget.pickup_ops_per_patch_level1 = 80;
            budget.pickup_ops_per_patch_level2 = 40;
            budget.extraction_ops_per_patch_level0 = 24;
            budget.extraction_ops_per_patch_level1 = 16;
            budget.extraction_ops_per_patch_level2 = 8;

            cadence.patch_emit_interval_level0 = 1;
            cadence.patch_emit_interval_level1 = 1;
            cadence.patch_emit_interval_level2 = 2;
            cadence.enemy_patch_interval_level0 = 1;
            cadence.enemy_patch_interval_level1 = 1;
            cadence.enemy_patch_interval_level2 = 2;
            cadence.projectile_patch_interval_level0 = 1;
            cadence.projectile_patch_interval_level1 = 1;
            cadence.projectile_patch_interval_level2 = 2;
            cadence.pickup_patch_interval_level0 = 2;
            cadence.pickup_patch_interval_level1 = 3;
            cadence.pickup_patch_interval_level2 = 5;
            cadence.extraction_patch_interval_level0 = 1;
            cadence.extraction_patch_interval_level1 = 1;
            cadence.extraction_patch_interval_level2 = 1;
        }
        _ => diagnostics.push(format!(
            "unknown {} preset '{}' (supported: two_local, four_local, aggressive)",
            ENV_NET_TUNING_PRESET, raw
        )),
    }
}

#[cfg(test)]
fn apply_replication_tuning_overrides_from_reader<F>(
    budget: &mut ReplicationBudgetConfig,
    cadence: &mut ReplicationCadenceConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    macro_rules! override_usize {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_usize(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    macro_rules! override_u64 {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_u64(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    override_usize!(
        budget.enemy_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_ENEMY_L0",
        0,
        4096
    );
    override_usize!(
        budget.enemy_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_ENEMY_L1",
        0,
        4096
    );
    override_usize!(
        budget.enemy_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_ENEMY_L2",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_PROJECTILE_L0",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_PROJECTILE_L1",
        0,
        4096
    );
    override_usize!(
        budget.projectile_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_PROJECTILE_L2",
        0,
        4096
    );
    override_usize!(
        budget.pickup_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_PICKUP_L0",
        0,
        2048
    );
    override_usize!(
        budget.pickup_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_PICKUP_L1",
        0,
        2048
    );
    override_usize!(
        budget.pickup_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_PICKUP_L2",
        0,
        2048
    );
    override_usize!(
        budget.extraction_ops_per_patch_level0,
        "CAVERN_NET_BUDGET_EXTRACTION_L0",
        0,
        512
    );
    override_usize!(
        budget.extraction_ops_per_patch_level1,
        "CAVERN_NET_BUDGET_EXTRACTION_L1",
        0,
        512
    );
    override_usize!(
        budget.extraction_ops_per_patch_level2,
        "CAVERN_NET_BUDGET_EXTRACTION_L2",
        0,
        512
    );

    override_u64!(
        cadence.enemy_patch_interval_level0,
        "CAVERN_NET_CADENCE_ENEMY_L0",
        0,
        120
    );
    override_u64!(
        cadence.enemy_patch_interval_level1,
        "CAVERN_NET_CADENCE_ENEMY_L1",
        0,
        120
    );
    override_u64!(
        cadence.enemy_patch_interval_level2,
        "CAVERN_NET_CADENCE_ENEMY_L2",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level0,
        "CAVERN_NET_CADENCE_PROJECTILE_L0",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level1,
        "CAVERN_NET_CADENCE_PROJECTILE_L1",
        0,
        120
    );
    override_u64!(
        cadence.projectile_patch_interval_level2,
        "CAVERN_NET_CADENCE_PROJECTILE_L2",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level0,
        "CAVERN_NET_CADENCE_PICKUP_L0",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level1,
        "CAVERN_NET_CADENCE_PICKUP_L1",
        0,
        120
    );
    override_u64!(
        cadence.pickup_patch_interval_level2,
        "CAVERN_NET_CADENCE_PICKUP_L2",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level0,
        "CAVERN_NET_CADENCE_EXTRACTION_L0",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level1,
        "CAVERN_NET_CADENCE_EXTRACTION_L1",
        0,
        120
    );
    override_u64!(
        cadence.extraction_patch_interval_level2,
        "CAVERN_NET_CADENCE_EXTRACTION_L2",
        0,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level0,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L0",
        1,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level1,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L1",
        1,
        120
    );
    override_u64!(
        cadence.patch_emit_interval_level2,
        "CAVERN_NET_CADENCE_PATCH_EMIT_L2",
        1,
        120
    );
}

#[cfg(test)]
#[allow(dead_code)]
fn apply_interpolation_overrides_from_reader<F>(
    interpolation: &mut InterpolationConfig,
    read_var: F,
    diagnostics: &mut Vec<String>,
) where
    F: Fn(&str) -> Option<String>,
{
    macro_rules! override_f32 {
        ($field:expr, $name:literal, $min:expr, $max:expr) => {
            if let Some(parsed) = parse_env_f32(read_var($name), $name, $min, $max, diagnostics) {
                $field = parsed;
            }
        };
    }

    override_f32!(
        interpolation.min_delay_ms,
        "CAVERN_NET_INTERP_MIN_DELAY_MS",
        5.0,
        500.0
    );
    override_f32!(
        interpolation.max_delay_ms,
        "CAVERN_NET_INTERP_MAX_DELAY_MS",
        5.0,
        800.0
    );
    override_f32!(
        interpolation.small_error_distance,
        "CAVERN_NET_INTERP_SMALL_ERROR",
        0.01,
        5.0
    );
    override_f32!(
        interpolation.medium_error_distance,
        "CAVERN_NET_INTERP_MEDIUM_ERROR",
        0.01,
        8.0
    );
    override_f32!(
        interpolation.large_error_distance,
        "CAVERN_NET_INTERP_LARGE_ERROR",
        0.01,
        16.0
    );
    override_f32!(
        interpolation.hard_snap_distance,
        "CAVERN_NET_INTERP_HARD_SNAP",
        0.01,
        64.0
    );
    if interpolation.medium_error_distance < interpolation.small_error_distance {
        diagnostics.push(format!(
            "CAVERN_NET_INTERP_MEDIUM_ERROR < CAVERN_NET_INTERP_SMALL_ERROR; adjusting medium to small"
        ));
        interpolation.medium_error_distance = interpolation.small_error_distance;
    }
    if interpolation.large_error_distance < interpolation.medium_error_distance {
        diagnostics.push(format!(
            "CAVERN_NET_INTERP_LARGE_ERROR < CAVERN_NET_INTERP_MEDIUM_ERROR; adjusting large to medium"
        ));
        interpolation.large_error_distance = interpolation.medium_error_distance;
    }
    if interpolation.hard_snap_distance < interpolation.large_error_distance {
        diagnostics.push(format!(
            "CAVERN_NET_INTERP_HARD_SNAP < CAVERN_NET_INTERP_LARGE_ERROR; adjusting hard snap to large"
        ));
        interpolation.hard_snap_distance = interpolation.large_error_distance;
    }
}

#[cfg(test)]
fn parse_env_usize(
    value: Option<String>,
    key: &str,
    min: usize,
    max: usize,
    diagnostics: &mut Vec<String>,
) -> Option<usize> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<usize>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{}..={}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid integer, ignoring",
                key, trimmed
            ));
            None
        }
    }
}

#[cfg(test)]
fn parse_env_u64(
    value: Option<String>,
    key: &str,
    min: u64,
    max: u64,
    diagnostics: &mut Vec<String>,
) -> Option<u64> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<u64>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{}..={}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid integer, ignoring",
                key, trimmed
            ));
            None
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn parse_env_f32(
    value: Option<String>,
    key: &str,
    min: f32,
    max: f32,
    diagnostics: &mut Vec<String>,
) -> Option<f32> {
    let value = value?;
    let trimmed = value.trim();
    match trimmed.parse::<f32>() {
        Ok(parsed) => {
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{}={} is out of range [{:.3}..={:.3}], clamping",
                    key, parsed, min, max
                ));
            }
            Some(parsed.clamp(min, max))
        }
        Err(_) => {
            diagnostics.push(format!(
                "{}='{}' is not a valid float, ignoring",
                key, trimmed
            ));
            None
        }
    }
}

fn server_emit_replication_v2(world: &mut World) -> Result<()> {
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
    let runtime_geometry = world
        .resource::<CavernGeometryRuntimeState>()
        .cloned()
        .unwrap_or_default();

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
        let mut legacy_state = world.resource_mut::<CavernNetSyncState>()?;
        if legacy_state.initial_snapshot_sent
            && runtime_geometry.edit_events.len() > legacy_state.last_sent_geometry_edit_count
        {
            let from_index = legacy_state.last_sent_geometry_edit_count;
            let to_index = runtime_geometry.edit_events.len();
            let edits = runtime_geometry.edit_events[from_index..to_index].to_vec();
            legacy_state.last_sent_geometry_edit_count = to_index;
            Some(postcard::to_allocvec(&CavernGeometryEditsEventV1 {
                tick,
                from_index,
                to_index,
                extraction_seal_primitive: runtime_geometry.extraction_seal_primitive,
                edits,
            })?)
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

    if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: event_code.to_string(),
            payload,
        }));
    }

    world.insert_resource(replication_map);

    metrics.bytes_sent_last_tick = metrics
        .bytes_sent_last_tick
        .saturating_add(payload_len as u64);
    if event_code == RUN_EVENT_KEYFRAME_V2 {
        metrics.bytes_sent_keyframe_last_tick = metrics
            .bytes_sent_keyframe_last_tick
            .saturating_add(payload_len as u64);
    } else if event_code == RUN_EVENT_PATCH_V2 {
        metrics.bytes_sent_patch_last_tick = metrics
            .bytes_sent_patch_last_tick
            .saturating_add(payload_len as u64);
    }
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
    metrics.bytes_sent_total = metrics
        .bytes_sent_total
        .saturating_add(metrics.bytes_sent_last_tick);
    world.insert_resource(metrics);
    Ok(())
}

fn build_patch_event_v2(
    replication_map: &mut ServerReplicationMap,
    cursor: ReplicationCursor,
    previous: Option<&CavernRunSnapshotV1>,
    current: &CavernRunSnapshotV1,
    load_shed_level: u8,
    budget_config: &ReplicationBudgetConfig,
    cadence_config: &ReplicationCadenceConfig,
) -> (CavernPatchEventV2, PatchBuildStats) {
    let run_state = previous.and_then(|base| {
        let patch = CavernRunStatePatchV2 {
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
            objective: (base.objective != current.objective).then_some(current.objective.clone()),
            extraction: (base.extraction != current.extraction)
                .then_some(current.extraction.clone()),
        };
        let has_changes = patch.phase.is_some()
            || patch.elite_defeated.is_some()
            || patch.extraction_active.is_some()
            || patch.extraction_started_at_tick.is_some()
            || patch.party_alive_count.is_some()
            || patch.enemy_kills.is_some()
            || patch.objective.is_some()
            || patch.extraction.is_some();
        has_changes.then_some(patch)
    });

    let mut player_ops = Vec::new();
    let mut previous_by_player_id = previous
        .map(|snapshot| {
            snapshot
                .players
                .iter()
                .map(|player| (player.player_id, player))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    for player in &current.players {
        let network_entity_id = replication_map
            .by_player_id
            .entry(player.player_id)
            .or_insert_with(|| NetworkEntityId(0x1000_0000 + player.player_id as u64))
            .to_owned();
        match previous_by_player_id.remove(&player.player_id) {
            None => player_ops.push(CavernPlayerPatchOpV2::Spawn {
                entity_id: network_entity_id,
                priority: CavernPatchPriorityV2::Critical,
                state: player.clone(),
            }),
            Some(previous_state) if previous_state != player => {
                player_ops.push(CavernPlayerPatchOpV2::Patch {
                    entity_id: network_entity_id,
                    priority: CavernPatchPriorityV2::High,
                    state: player.clone(),
                })
            }
            _ => {}
        }
    }

    for (player_id, _) in previous_by_player_id {
        let entity_id = replication_map
            .by_player_id
            .remove(&player_id)
            .unwrap_or(NetworkEntityId(0x1000_0000 + player_id as u64));
        player_ops.push(CavernPlayerPatchOpV2::Despawn {
            entity_id,
            player_id,
        });
    }

    let emit_enemies = should_emit_patch_channel(
        cursor.stream_cursor,
        enemy_patch_interval(cadence_config, load_shed_level),
    );
    let emit_projectiles = should_emit_patch_channel(
        cursor.stream_cursor,
        projectile_patch_interval(cadence_config, load_shed_level),
    );
    let emit_pickups = should_emit_patch_channel(
        cursor.stream_cursor,
        pickup_patch_interval(cadence_config, load_shed_level),
    );
    let emit_extraction = should_emit_patch_channel(
        cursor.stream_cursor,
        extraction_patch_interval(cadence_config, load_shed_level),
    );
    let mut enemy_ops = Vec::new();
    if emit_enemies {
        let mut previous_by_entity = previous
            .map(|snapshot| {
                snapshot
                    .enemies
                    .iter()
                    .map(|enemy| (enemy.network_entity_id, enemy))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for enemy in &current.enemies {
            let entity_id = enemy.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => enemy_ops.push(CavernEnemyPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::High,
                    state: enemy.clone(),
                }),
                Some(previous_state) if previous_state != enemy => {
                    enemy_ops.push(CavernEnemyPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::High,
                        state: enemy.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            enemy_ops.push(CavernEnemyPatchOpV2::Despawn { entity_id });
        }
    }

    let mut projectile_ops = Vec::new();
    if emit_projectiles {
        let mut previous_by_entity = previous
            .map(|snapshot| {
                snapshot
                    .projectiles
                    .iter()
                    .map(|projectile| (projectile.network_entity_id, projectile))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for projectile in &current.projectiles {
            let entity_id = projectile.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => projectile_ops.push(CavernProjectilePatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Medium,
                    state: projectile.clone(),
                }),
                Some(previous_state) if previous_state != projectile => {
                    projectile_ops.push(CavernProjectilePatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Medium,
                        state: projectile.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            projectile_ops.push(CavernProjectilePatchOpV2::Despawn { entity_id });
        }
    }

    let mut pickup_ops = Vec::new();
    if emit_pickups {
        let mut previous_by_entity = previous
            .map(|snapshot| {
                snapshot
                    .pickups
                    .iter()
                    .map(|pickup| (pickup.network_entity_id, pickup))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for pickup in &current.pickups {
            let entity_id = pickup.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => pickup_ops.push(CavernPickupPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Low,
                    state: pickup.clone(),
                }),
                Some(previous_state) if previous_state != pickup => {
                    pickup_ops.push(CavernPickupPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Low,
                        state: pickup.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            pickup_ops.push(CavernPickupPatchOpV2::Despawn { entity_id });
        }
    }

    let mut extraction_ops = Vec::new();
    if emit_extraction {
        let mut previous_by_entity = previous
            .map(|snapshot| {
                snapshot
                    .extraction_zones
                    .iter()
                    .map(|zone| (zone.network_entity_id, zone))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        for zone in &current.extraction_zones {
            let entity_id = zone.network_entity_id;
            match previous_by_entity.remove(&entity_id) {
                None => extraction_ops.push(CavernExtractionPatchOpV2::Spawn {
                    entity_id,
                    priority: CavernPatchPriorityV2::Critical,
                    state: zone.clone(),
                }),
                Some(previous_state) if previous_state != zone => {
                    extraction_ops.push(CavernExtractionPatchOpV2::Patch {
                        entity_id,
                        priority: CavernPatchPriorityV2::Critical,
                        state: zone.clone(),
                    })
                }
                _ => {}
            }
        }
        for (entity_id, _) in previous_by_entity {
            extraction_ops.push(CavernExtractionPatchOpV2::Despawn { entity_id });
        }
    }

    let (enemy_ops, dropped_enemy_ops) =
        cap_enemy_ops(enemy_ops, enemy_ops_budget(budget_config, load_shed_level));
    let (projectile_ops, dropped_projectile_ops) = cap_projectile_ops(
        projectile_ops,
        projectile_ops_budget(budget_config, load_shed_level),
    );
    let (pickup_ops, dropped_pickup_ops) = cap_pickup_ops(
        pickup_ops,
        pickup_ops_budget(budget_config, load_shed_level),
    );
    let (extraction_ops, dropped_extraction_ops) = cap_extraction_ops(
        extraction_ops,
        extraction_ops_budget(budget_config, load_shed_level),
    );

    (
        CavernPatchEventV2 {
            cursor,
            run_state,
            player_ops,
            enemy_ops,
            projectile_ops,
            pickup_ops,
            extraction_ops,
        },
        PatchBuildStats {
            dropped_enemy_ops,
            dropped_projectile_ops,
            dropped_pickup_ops,
            dropped_extraction_ops,
        },
    )
}

fn compute_load_shed_level_v2(
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

fn should_emit_patch_channel(stream_cursor: u64, interval_ticks: u64) -> bool {
    match interval_ticks {
        0 => false,
        1 => true,
        interval => stream_cursor % interval == 0,
    }
}

fn enemy_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.enemy_patch_interval_level0,
        1 => config.enemy_patch_interval_level1,
        _ => config.enemy_patch_interval_level2,
    }
}

fn projectile_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.projectile_patch_interval_level0,
        1 => config.projectile_patch_interval_level1,
        _ => config.projectile_patch_interval_level2,
    }
}

fn pickup_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.pickup_patch_interval_level0,
        1 => config.pickup_patch_interval_level1,
        _ => config.pickup_patch_interval_level2,
    }
}

fn extraction_patch_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.extraction_patch_interval_level0,
        1 => config.extraction_patch_interval_level1,
        _ => config.extraction_patch_interval_level2,
    }
}

fn patch_emit_interval(config: &ReplicationCadenceConfig, load_shed_level: u8) -> u64 {
    match load_shed_level {
        0 => config.patch_emit_interval_level0,
        1 => config.patch_emit_interval_level1,
        _ => config.patch_emit_interval_level2,
    }
}

fn enemy_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.enemy_ops_per_patch_level0,
        1 => config.enemy_ops_per_patch_level1,
        _ => config.enemy_ops_per_patch_level2,
    }
}

fn projectile_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.projectile_ops_per_patch_level0,
        1 => config.projectile_ops_per_patch_level1,
        _ => config.projectile_ops_per_patch_level2,
    }
}

fn pickup_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.pickup_ops_per_patch_level0,
        1 => config.pickup_ops_per_patch_level1,
        _ => config.pickup_ops_per_patch_level2,
    }
}

fn extraction_ops_budget(config: &ReplicationBudgetConfig, load_shed_level: u8) -> usize {
    match load_shed_level {
        0 => config.extraction_ops_per_patch_level0,
        1 => config.extraction_ops_per_patch_level1,
        _ => config.extraction_ops_per_patch_level2,
    }
}

fn cap_enemy_ops(
    mut ops: Vec<CavernEnemyPatchOpV2>,
    cap: usize,
) -> (Vec<CavernEnemyPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(enemy_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_projectile_ops(
    mut ops: Vec<CavernProjectilePatchOpV2>,
    cap: usize,
) -> (Vec<CavernProjectilePatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(projectile_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_pickup_ops(
    mut ops: Vec<CavernPickupPatchOpV2>,
    cap: usize,
) -> (Vec<CavernPickupPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(pickup_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn cap_extraction_ops(
    mut ops: Vec<CavernExtractionPatchOpV2>,
    cap: usize,
) -> (Vec<CavernExtractionPatchOpV2>, u64) {
    if cap == 0 {
        return (Vec::new(), ops.len() as u64);
    }
    if ops.len() <= cap {
        return (ops, 0);
    }
    ops.sort_by_key(extraction_op_sort_key);
    let dropped = ops.len().saturating_sub(cap) as u64;
    ops.truncate(cap);
    (ops, dropped)
}

fn enemy_op_sort_key(op: &CavernEnemyPatchOpV2) -> (u8, u64) {
    match op {
        CavernEnemyPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernEnemyPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernEnemyPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn projectile_op_sort_key(op: &CavernProjectilePatchOpV2) -> (u8, u64) {
    match op {
        CavernProjectilePatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernProjectilePatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernProjectilePatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn pickup_op_sort_key(op: &CavernPickupPatchOpV2) -> (u8, u64) {
    match op {
        CavernPickupPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernPickupPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernPickupPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn extraction_op_sort_key(op: &CavernExtractionPatchOpV2) -> (u8, u64) {
    match op {
        CavernExtractionPatchOpV2::Despawn { entity_id } => (0, entity_id.0),
        CavernExtractionPatchOpV2::Spawn { entity_id, .. } => (1, entity_id.0),
        CavernExtractionPatchOpV2::Patch { entity_id, .. } => (2, entity_id.0),
    }
}

fn client_apply_replication_events_v2(world: &mut World) -> Result<()> {
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
    let mut keyframes_by_stream_cursor = BTreeMap::<u64, CavernKeyframeEventV2>::new();
    let mut patches_by_stream_cursor = BTreeMap::<u64, CavernPatchEventV2>::new();

    for message in events {
        match message {
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_GEOMETRY_EDITS => {
                metrics.bytes_received_last_frame = metrics
                    .bytes_received_last_frame
                    .saturating_add(run_event.payload.len() as u64);
                let event: CavernGeometryEditsEventV1 = postcard::from_bytes(&run_event.payload)?;
                geometry_events.push(event);
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_KEYFRAME_V2 => {
                metrics.bytes_received_last_frame = metrics
                    .bytes_received_last_frame
                    .saturating_add(run_event.payload.len() as u64);
                metrics.keyframes_received_last_frame =
                    metrics.keyframes_received_last_frame.saturating_add(1);
                let event: CavernKeyframeEventV2 = postcard::from_bytes(&run_event.payload)?;
                keyframes_by_stream_cursor.insert(event.cursor.stream_cursor, event);
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_PATCH_V2 => {
                metrics.bytes_received_last_frame = metrics
                    .bytes_received_last_frame
                    .saturating_add(run_event.payload.len() as u64);
                metrics.patches_received_last_frame =
                    metrics.patches_received_last_frame.saturating_add(1);
                let event: CavernPatchEventV2 = postcard::from_bytes(&run_event.payload)?;
                patches_by_stream_cursor.insert(event.cursor.stream_cursor, event);
            }
            // fallback compatibility while V2 flag is active
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_SNAPSHOT => {
                let event: CavernSnapshotEventV1 = postcard::from_bytes(&run_event.payload)?;
                metrics.keyframes_received_last_frame =
                    metrics.keyframes_received_last_frame.saturating_add(1);
                let keyframe = CavernKeyframeEventV2 {
                    cursor: ReplicationCursor {
                        server_tick: event.tick,
                        stream_cursor: event.cursor,
                        base_cursor: event.cursor.saturating_sub(1),
                    },
                    snapshot: event.snapshot,
                };
                keyframes_by_stream_cursor.insert(keyframe.cursor.stream_cursor, keyframe);
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_DELTA => {
                let event: CavernDeltaEventV1 = postcard::from_bytes(&run_event.payload)?;
                let base_snapshot = world
                    .resource::<CavernNetSyncState>()
                    .ok()
                    .and_then(|state| state.last_received_snapshot.clone());
                if let Some(base_snapshot) = base_snapshot {
                    let rebuilt = apply_cavern_run_delta(&base_snapshot, &event.delta);
                    metrics.patches_received_last_frame =
                        metrics.patches_received_last_frame.saturating_add(1);
                    let patch = CavernPatchEventV2 {
                        cursor: ReplicationCursor {
                            server_tick: event.tick,
                            stream_cursor: event.cursor,
                            base_cursor: event.base_cursor,
                        },
                        run_state: None,
                        player_ops: rebuilt
                            .players
                            .iter()
                            .map(|player| CavernPlayerPatchOpV2::Patch {
                                entity_id: NetworkEntityId(0x1000_0000 + player.player_id as u64),
                                priority: CavernPatchPriorityV2::High,
                                state: player.clone(),
                            })
                            .collect(),
                        enemy_ops: rebuilt
                            .enemies
                            .iter()
                            .map(|enemy| CavernEnemyPatchOpV2::Patch {
                                entity_id: enemy.network_entity_id,
                                priority: CavernPatchPriorityV2::High,
                                state: enemy.clone(),
                            })
                            .collect(),
                        projectile_ops: rebuilt
                            .projectiles
                            .iter()
                            .map(|projectile| CavernProjectilePatchOpV2::Patch {
                                entity_id: projectile.network_entity_id,
                                priority: CavernPatchPriorityV2::Medium,
                                state: projectile.clone(),
                            })
                            .collect(),
                        pickup_ops: rebuilt
                            .pickups
                            .iter()
                            .map(|pickup| CavernPickupPatchOpV2::Patch {
                                entity_id: pickup.network_entity_id,
                                priority: CavernPatchPriorityV2::Low,
                                state: pickup.clone(),
                            })
                            .collect(),
                        extraction_ops: rebuilt
                            .extraction_zones
                            .iter()
                            .map(|zone| CavernExtractionPatchOpV2::Patch {
                                entity_id: zone.network_entity_id,
                                priority: CavernPatchPriorityV2::Critical,
                                state: zone.clone(),
                            })
                            .collect(),
                    };
                    patches_by_stream_cursor.insert(patch.cursor.stream_cursor, patch);
                }
            }
            _ => {}
        }
    }

    for event in geometry_events {
        apply_authoritative_geometry_edits(world, event)?;
    }

    let highest_patch_stream_cursor = patches_by_stream_cursor.keys().next_back().copied();

    let mut ordered_stream_cursors = BTreeSet::new();
    ordered_stream_cursors.extend(keyframes_by_stream_cursor.keys().copied());
    ordered_stream_cursors.extend(patches_by_stream_cursor.keys().copied());

    let mut replication_state = world.resource_mut::<ClientReplicationStateV2>()?;
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
                if let Ok(mut net_state) = world.resource_mut::<CavernNetSyncState>() {
                    net_state.last_received_cursor = cursor.stream_cursor;
                    net_state.last_received_snapshot = Some(snapshot);
                }
                let mut state = world.resource_mut::<ClientReplicationStateV2>()?;
                state.last_cursor = cursor;
                state.has_keyframe = true;
                replication_state = state;
                metrics.keyframes_applied = metrics.keyframes_applied.saturating_add(1);
            } else if keyframe_is_newer {
                drop(replication_state);
                apply_authoritative_cavern_snapshot(
                    world,
                    keyframe.cursor.server_tick,
                    keyframe.cursor.stream_cursor,
                    keyframe.snapshot,
                )?;
                let mut state = world.resource_mut::<ClientReplicationStateV2>()?;
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
                apply_patch_event_v2(world, patch, apply_local_owned_correction)?;
                let elapsed = start.elapsed();
                let micros = elapsed.as_micros().min(u64::MAX as u128) as u64;
                let mut state = world.resource_mut::<ClientReplicationStateV2>()?;
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

fn apply_patch_event_v2(
    world: &mut World,
    patch: CavernPatchEventV2,
    apply_local_owned_correction: bool,
) -> Result<()> {
    let authoritative_tick = patch.cursor.server_tick;
    if let Some(run_state) = patch.run_state {
        apply_run_state_patch_v2(world, run_state);
    }
    apply_player_patch_ops_v2(
        world,
        patch.player_ops,
        Some(authoritative_tick),
        apply_local_owned_correction,
    )?;
    apply_enemy_patch_ops_v2(world, patch.enemy_ops)?;
    apply_projectile_patch_ops_v2(world, patch.projectile_ops)?;
    apply_pickup_patch_ops_v2(world, patch.pickup_ops)?;
    apply_extraction_patch_ops_v2(world, patch.extraction_ops)?;
    Ok(())
}

fn apply_run_state_patch_v2(world: &mut World, patch: CavernRunStatePatchV2) {
    if let Ok(mut run) = world.resource_mut::<crate::domain::CavernRunState>() {
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

fn apply_player_patch_ops_v2(
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

fn find_player_entity_by_player_id(
    world: &World,
    player_id: u32,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
        .iter()
        .find_map(|(entity, id)| (id.0 == player_id).then_some(entity))
}

fn spawn_player_entity_from_snapshot(
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

fn apply_non_transform_player_snapshot(
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

fn apply_local_player_snapshot_correction(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernPlayerSnapshotV1,
    interpolation: InterpolationConfig,
    dynamic_hard_snap_distance: f32,
    authoritative_tick: Option<SimulationTick>,
    correction_stats: &mut CorrectionStats,
) -> Result<()> {
    let current = world
        .get::<Transform2>(entity)
        .copied()
        .unwrap_or_else(|| Transform2::new(snapshot.x, snapshot.y, snapshot.yaw));
    let current_velocity = world
        .get::<Velocity2>(entity)
        .copied()
        .unwrap_or(Velocity2 {
            x: snapshot.velocity[0],
            y: snapshot.velocity[1],
        });
    let target = [snapshot.x, snapshot.y];
    let dx = target[0] - current.x;
    let dy = target[1] - current.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let local_simulated_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let (has_pending_for_replay, authoritative_tick_is_newer) = authoritative_tick
        .map(|tick| {
            world
                .resource::<CavernPredictionState>()
                .map(|prediction| {
                    let has_pending_for_replay = prediction.pending_frames.iter().any(|frame| {
                        frame.tick.0 > tick.0 && frame.tick.0 <= local_simulated_tick.0
                    });
                    let authoritative_tick_is_newer = tick.0 > prediction.last_authoritative_tick.0;
                    (has_pending_for_replay, authoritative_tick_is_newer)
                })
                .unwrap_or((false, true))
        })
        .unwrap_or((false, true));
    if !authoritative_tick_is_newer {
        correction_stats.last_distance = 0.0;
        apply_non_transform_player_snapshot(world, entity, snapshot);
        return Ok(());
    }

    let should_replay_reconciliation =
        has_pending_for_replay && distance > interpolation.small_error_distance;

    if should_replay_reconciliation {
        if distance <= interpolation.small_error_distance {
            correction_stats.small_corrections =
                correction_stats.small_corrections.saturating_add(1);
        } else if distance <= interpolation.medium_error_distance {
            correction_stats.medium_corrections =
                correction_stats.medium_corrections.saturating_add(1);
        } else {
            correction_stats.large_corrections =
                correction_stats.large_corrections.saturating_add(1);
        }
        correction_stats.last_distance = distance;
        correction_stats.total_distance += distance;
        if correction_stats.ema_distance <= f32::EPSILON {
            correction_stats.ema_distance = distance;
        } else {
            correction_stats.ema_distance += (distance - correction_stats.ema_distance) * 0.1;
        }

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
        apply_non_transform_player_snapshot(world, entity, snapshot);

        if let Some(tick) = authoritative_tick {
            let replayed_frames =
                replay_pending_prediction_frames(world, tick, local_simulated_tick)?;
            if replayed_frames > 0 {
                tracing::trace!(
                    replayed_frames,
                    authoritative_tick = tick.0,
                    local_simulated_tick = local_simulated_tick.0,
                    "replayed local prediction after v2 patch"
                );
            }
        }
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = local_simulated_tick;
        }
        mark_prediction_authoritative_tick(world, authoritative_tick);
        return Ok(());
    }

    if !has_pending_for_replay && distance <= interpolation.medium_error_distance {
        correction_stats.last_distance = 0.0;
        mark_prediction_authoritative_tick(world, authoritative_tick);
        apply_non_transform_player_snapshot(world, entity, snapshot);
        return Ok(());
    }

    let (correction_gain, velocity_blend, hard_snap) = if distance
        <= interpolation.small_error_distance
    {
        correction_stats.small_corrections = correction_stats.small_corrections.saturating_add(1);
        (0.0, 0.1, false)
    } else if distance <= interpolation.medium_error_distance {
        correction_stats.medium_corrections = correction_stats.medium_corrections.saturating_add(1);
        (0.12, 0.18, false)
    } else if distance <= interpolation.large_error_distance {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.2, 0.26, false)
    } else if distance <= dynamic_hard_snap_distance {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.28, 0.34, false)
    } else if distance <= dynamic_hard_snap_distance * 2.25 {
        correction_stats.large_corrections = correction_stats.large_corrections.saturating_add(1);
        (0.4, 0.48, false)
    } else {
        correction_stats.hard_snaps = correction_stats.hard_snaps.saturating_add(1);
        (1.0, 1.0, true)
    };

    let corrected = if hard_snap {
        [snapshot.x, snapshot.y, snapshot.yaw]
    } else {
        let fixed_dt = world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds.max(1.0 / 120.0))
            .unwrap_or(1.0 / 60.0);
        let mut step_x = dx * correction_gain;
        let mut step_y = dy * correction_gain;
        let desired_step = (step_x * step_x + step_y * step_y).sqrt();
        let current_speed = (current_velocity.x * current_velocity.x
            + current_velocity.y * current_velocity.y)
            .sqrt();
        let base_step_budget =
            current_speed * fixed_dt * 0.9 + interpolation.small_error_distance * 0.25;
        let error_bonus = (distance - interpolation.medium_error_distance).max(0.0) * 0.12;
        let max_step = (base_step_budget + error_bonus).clamp(
            interpolation.small_error_distance * 0.08,
            interpolation.large_error_distance * 0.4,
        );
        if desired_step > max_step && desired_step > f32::EPSILON {
            let scale = max_step / desired_step;
            step_x *= scale;
            step_y *= scale;
        }
        let yaw_gain = (correction_gain * 0.85).clamp(0.0, 0.55);
        [
            current.x + step_x,
            current.y + step_y,
            current.yaw + angle_delta(current.yaw, snapshot.yaw) * yaw_gain,
        ]
    };
    correction_stats.last_distance = distance;
    correction_stats.total_distance += distance;
    if correction_stats.ema_distance <= f32::EPSILON {
        correction_stats.ema_distance = distance;
    } else {
        correction_stats.ema_distance += (distance - correction_stats.ema_distance) * 0.1;
    }
    if hard_snap {
        tracing::debug!(
            distance,
            dynamic_hard_snap_distance,
            "local player hard snap correction applied"
        );
    }

    let _ = world.insert(
        entity,
        Transform2::new(corrected[0], corrected[1], corrected[2]),
    );
    let _ = world.insert(
        entity,
        Velocity2 {
            x: current_velocity.x + (snapshot.velocity[0] - current_velocity.x) * velocity_blend,
            y: current_velocity.y + (snapshot.velocity[1] - current_velocity.y) * velocity_blend,
        },
    );
    mark_prediction_authoritative_tick(world, authoritative_tick);
    apply_non_transform_player_snapshot(world, entity, snapshot);
    Ok(())
}

fn mark_prediction_authoritative_tick(
    world: &mut World,
    authoritative_tick: Option<SimulationTick>,
) {
    if let Some(tick) = authoritative_tick
        && let Ok(mut prediction) = world.resource_mut::<CavernPredictionState>()
        && tick.0 > prediction.last_authoritative_tick.0
    {
        prediction.last_authoritative_tick = tick;
    }
}

fn usable_authoritative_tick(
    snapshot_authoritative_input_tick: Option<SimulationTick>,
    cursor_authoritative_tick: Option<SimulationTick>,
) -> Option<SimulationTick> {
    snapshot_authoritative_input_tick
        .filter(|tick| tick.0 > 0)
        .or(cursor_authoritative_tick)
}

fn remote_target_from_snapshot(
    snapshot: &crate::domain::CavernPlayerSnapshotV1,
) -> RemotePlayerTarget {
    RemotePlayerTarget {
        pos: [snapshot.x, snapshot.y],
        velocity: [snapshot.velocity[0], snapshot.velocity[1]],
        yaw: snapshot.yaw,
    }
}

fn angle_delta(current: f32, target: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn apply_enemy_patch_ops_v2(world: &mut World, ops: Vec<CavernEnemyPatchOpV2>) -> Result<()> {
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

fn apply_projectile_patch_ops_v2(
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

fn find_enemy_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::domain::EnemyReplicationId)>()
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
            &crate::domain::ProjectileReplicationId,
        )>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn spawn_enemy_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::domain::CavernEnemySnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn((crate::domain::Enemy, snapshot.kind));
    let _ = world.insert(
        entity,
        crate::domain::EnemyReplicationId(snapshot.network_entity_id),
    );
    apply_enemy_snapshot(world, entity, snapshot);
    entity
}

fn apply_enemy_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernEnemySnapshotV1,
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
        crate::domain::Health {
            current: snapshot.health_current,
            max: snapshot.health_max,
        },
    );
    let _ = world.insert(entity, crate::domain::Faction::CavernBeasts);
    let _ = world.insert(
        entity,
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::domain::EnemyReplicationId(snapshot.network_entity_id),
    );
    if let Some(aggro) = snapshot.aggro {
        let _ = world.insert(entity, aggro);
    } else {
        let _ = world.remove::<crate::domain::AggroState>(entity);
    }
    if let Some(projectile_attack) = snapshot.projectile_attack {
        let _ = world.insert(entity, projectile_attack);
    } else {
        let _ = world.remove::<crate::domain::ProjectileAttack>(entity);
    }
    if let Some(melee_attack) = snapshot.melee_attack {
        let _ = world.insert(entity, melee_attack);
    } else {
        let _ = world.remove::<crate::domain::MeleeAttack>(entity);
    }
    if let Some(weapon) = snapshot.weapon {
        let _ = world.insert(entity, weapon);
    } else {
        let _ = world.remove::<crate::domain::WeaponState>(entity);
    }
    if let Some(spawn_room) = snapshot.spawn_room {
        let _ = world.insert(entity, crate::domain::SpawnRoom(spawn_room));
    } else {
        let _ = world.remove::<crate::domain::SpawnRoom>(entity);
    }
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::domain::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::domain::RoomAnchor>(entity);
    }
    if snapshot.elite_objective {
        let _ = world.insert(entity, crate::domain::EliteObjective);
    } else {
        let _ = world.remove::<crate::domain::EliteObjective>(entity);
    }
}

fn spawn_projectile_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::domain::CavernProjectileSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::domain::Projectile {
        damage: snapshot.damage,
        lifetime_seconds: snapshot.lifetime_seconds,
    });
    let _ = world.insert(
        entity,
        crate::domain::ProjectileReplicationId(snapshot.network_entity_id),
    );
    apply_projectile_snapshot(world, entity, snapshot);
    entity
}

fn apply_projectile_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernProjectileSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::domain::Projectile {
            damage: snapshot.damage,
            lifetime_seconds: snapshot.lifetime_seconds,
        },
    );
    let _ = world.insert(
        entity,
        crate::domain::ProjectileVisualState {
            source_team: if snapshot.faction == crate::domain::Faction::Hunters {
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
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(entity, snapshot.faction);
    let _ = world.insert(
        entity,
        crate::domain::ProjectileReplicationId(snapshot.network_entity_id),
    );
}

fn apply_pickup_patch_ops_v2(world: &mut World, ops: Vec<CavernPickupPatchOpV2>) -> Result<()> {
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

fn apply_extraction_patch_ops_v2(
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

fn find_pickup_entity_by_replication_id(
    world: &World,
    entity_id: NetworkEntityId,
) -> Option<engine::prelude::Entity> {
    world
        .query::<(engine::prelude::Entity, &crate::domain::PickupReplicationId)>()
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
            &crate::domain::ExtractionReplicationId,
        )>()
        .iter()
        .find_map(|(entity, replication_id)| (replication_id.0 == entity_id).then_some(entity))
}

fn spawn_pickup_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::domain::CavernPickupSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::domain::Pickup {
        kind: snapshot.pickup,
    });
    let _ = world.insert(
        entity,
        crate::domain::PickupReplicationId(snapshot.network_entity_id),
    );
    apply_pickup_snapshot(world, entity, snapshot);
    entity
}

fn apply_pickup_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernPickupSnapshotV1,
) {
    let _ = world.insert(
        entity,
        crate::domain::Pickup {
            kind: snapshot.pickup,
        },
    );
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::domain::PickupReplicationId(snapshot.network_entity_id),
    );
    if snapshot.loot_drop {
        let _ = world.insert(entity, crate::domain::LootDrop);
    } else {
        let _ = world.remove::<crate::domain::LootDrop>(entity);
    }
    if snapshot.chest {
        let _ = world.insert(entity, crate::domain::Chest);
    } else {
        let _ = world.remove::<crate::domain::Chest>(entity);
    }
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::domain::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::domain::RoomAnchor>(entity);
    }
}

fn spawn_extraction_entity_from_snapshot(
    world: &mut World,
    snapshot: &crate::domain::CavernExtractionSnapshotV1,
) -> engine::prelude::Entity {
    let entity = world.spawn(crate::domain::ExtractionZone);
    let _ = world.insert(
        entity,
        crate::domain::ExtractionReplicationId(snapshot.network_entity_id),
    );
    apply_extraction_snapshot(world, entity, snapshot);
    entity
}

fn apply_extraction_snapshot(
    world: &mut World,
    entity: engine::prelude::Entity,
    snapshot: &crate::domain::CavernExtractionSnapshotV1,
) {
    let _ = world.insert(entity, crate::domain::ExtractionZone);
    let _ = world.insert(
        entity,
        Transform2::new(snapshot.x, snapshot.y, snapshot.yaw),
    );
    let _ = world.insert(
        entity,
        crate::domain::ColliderRadius(snapshot.collider_radius),
    );
    let _ = world.insert(
        entity,
        crate::domain::ExtractionReplicationId(snapshot.network_entity_id),
    );
    if let Some(room_anchor) = snapshot.room_anchor {
        let _ = world.insert(
            entity,
            crate::domain::RoomAnchor {
                room_id: room_anchor,
            },
        );
    } else {
        let _ = world.remove::<crate::domain::RoomAnchor>(entity);
    }
}

fn client_smoothing_system(mut world: WorldMut) -> Result<()> {
    if !matches!(current_net_sync_mode(&world), CavernNetSyncMode::V2) {
        return Ok(());
    }
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
        return Ok(());
    }

    let dt = world
        .resource::<Time>()
        .map(|time| time.delta_seconds.max(0.0))
        .unwrap_or(0.0);
    if dt <= f32::EPSILON {
        return Ok(());
    }
    let interpolation = world
        .resource::<InterpolationConfig>()
        .copied()
        .unwrap_or_default();
    let replication_metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let dropped_ops_last_tick = replication_metrics
        .dropped_enemy_ops_last_tick
        .saturating_add(replication_metrics.dropped_projectile_ops_last_tick)
        .saturating_add(replication_metrics.dropped_pickup_ops_last_tick)
        .saturating_add(replication_metrics.dropped_extraction_ops_last_tick);
    let load_shed_level = replication_metrics.load_shed_level_last_tick as f32;

    let rtt_ms = world
        .resource::<RoundTripMetrics>()
        .ok()
        .and_then(|metrics| metrics.last_rtt_millis.map(|value| value as f32))
        .unwrap_or(0.0);
    if let Ok(mut smoothing) = world.resource_mut::<AdaptiveSmoothingState>() {
        let measured_rtt_ms = if rtt_ms > 0.0 {
            rtt_ms
        } else {
            smoothing.last_rtt_ms
        };
        if measured_rtt_ms > 0.0 {
            let delta = (measured_rtt_ms - smoothing.last_rtt_ms).abs();
            smoothing.jitter_ms = smoothing.jitter_ms * 0.9 + delta * 0.1;
            smoothing.last_rtt_ms = measured_rtt_ms;
            smoothing.samples = smoothing.samples.saturating_add(1);
        }
        let shed_penalty_ms = load_shed_level * 10.0;
        let drop_penalty_ms = (dropped_ops_last_tick as f32).sqrt().min(20.0);
        let target = (measured_rtt_ms * 0.5
            + smoothing.jitter_ms * 1.5
            + 35.0
            + shed_penalty_ms
            + drop_penalty_ms)
            .clamp(interpolation.min_delay_ms, interpolation.max_delay_ms);
        smoothing.target_delay_ms = target;
        let blend = (dt * 6.0).clamp(0.0, 1.0);
        smoothing.effective_delay_ms += (target - smoothing.effective_delay_ms) * blend;
    }

    let (delay_seconds, extrapolation_seconds) = world
        .resource::<AdaptiveSmoothingState>()
        .map(|state| {
            let delay_seconds = (state.effective_delay_ms / 1000.0).max(0.01);
            let extrapolation_seconds =
                ((state.effective_delay_ms / 1000.0) + load_shed_level * 0.012).clamp(0.0, 0.22);
            (delay_seconds, extrapolation_seconds)
        })
        .unwrap_or((0.08, 0.08));
    let base_alpha = (dt / delay_seconds).clamp(0.0, 1.0);

    let local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id);
    let client_map = world
        .resource::<ClientReplicationMap>()
        .cloned()
        .unwrap_or_default();
    let state = world
        .resource_mut::<ClientReplicationStateV2>()
        .map(|state| state.clone())
        .unwrap_or_default();

    let mut smoothing_samples = 0_u64;
    let mut smoothing_error_sum = 0.0_f32;
    let mut smoothing_error_max = 0.0_f32;
    let mut smoothing_alpha_sum = 0.0_f32;

    for (player_id, target) in &state.remote_targets_by_player_id {
        if Some(*player_id) == local_player_id {
            continue;
        }
        let Some(network_entity_id) = client_map.by_player_id.get(player_id).copied() else {
            continue;
        };
        let Some(entity) = client_map
            .by_network_entity_id
            .get(&network_entity_id)
            .copied()
        else {
            continue;
        };
        let velocity_alpha = {
            let Some(mut transform) = world.get_mut::<Transform2>(entity) else {
                continue;
            };
            let predicted_x = target.pos[0] + target.velocity[0] * extrapolation_seconds;
            let predicted_y = target.pos[1] + target.velocity[1] * extrapolation_seconds;
            let error_dx = predicted_x - transform.x;
            let error_dy = predicted_y - transform.y;
            let error_distance = (error_dx * error_dx + error_dy * error_dy).sqrt();
            let catch_up = if error_distance <= interpolation.small_error_distance {
                0.8
            } else if error_distance <= interpolation.medium_error_distance {
                1.0
            } else if error_distance <= interpolation.large_error_distance {
                1.35
            } else {
                1.8
            };
            let entity_alpha = (base_alpha * catch_up).clamp(0.0, 1.0);

            let mut step_x = error_dx * entity_alpha;
            let mut step_y = error_dy * entity_alpha;
            let desired_step = (step_x * step_x + step_y * step_y).sqrt();
            let target_speed = (target.velocity[0] * target.velocity[0]
                + target.velocity[1] * target.velocity[1])
                .sqrt();
            let base_step_budget =
                target_speed * dt * 1.4 + interpolation.small_error_distance * 0.4;
            let error_bonus = (error_distance - interpolation.medium_error_distance).max(0.0) * 0.2;
            let max_step = (base_step_budget + error_bonus).clamp(
                interpolation.small_error_distance * 0.35,
                interpolation.large_error_distance * 0.85,
            );
            if desired_step > max_step && desired_step > f32::EPSILON {
                let scale = max_step / desired_step;
                step_x *= scale;
                step_y *= scale;
            }

            transform.x += step_x;
            transform.y += step_y;
            transform.yaw += angle_delta(transform.yaw, target.yaw) * entity_alpha;

            smoothing_samples = smoothing_samples.saturating_add(1);
            smoothing_error_sum += error_distance;
            smoothing_error_max = smoothing_error_max.max(error_distance);
            smoothing_alpha_sum += entity_alpha;

            (entity_alpha * 1.15).clamp(0.0, 1.0)
        };
        if let Some(mut velocity) = world.get_mut::<Velocity2>(entity) {
            velocity.x += (target.velocity[0] - velocity.x) * velocity_alpha;
            velocity.y += (target.velocity[1] - velocity.y) * velocity_alpha;
        }
    }
    if let Ok(mut metrics) = world.resource_mut::<ReplicationRuntimeMetrics>() {
        metrics.smoothing_samples_last_frame = smoothing_samples;
        metrics.smoothing_error_mean_last_frame = if smoothing_samples > 0 {
            smoothing_error_sum / smoothing_samples as f32
        } else {
            0.0
        };
        metrics.smoothing_error_max_last_frame = smoothing_error_max;
        metrics.smoothing_alpha_mean_last_frame = if smoothing_samples > 0 {
            smoothing_alpha_sum / smoothing_samples as f32
        } else {
            0.0
        };
        metrics.smoothing_extrapolation_ms_last_frame = extrapolation_seconds * 1000.0;
    }
    Ok(())
}

fn net_sync_diagnostics_log_system(mut world: WorldMut) -> Result<()> {
    let diagnostics = world
        .resource::<NetDiagnosticsConfigAssetV1>()
        .copied()
        .unwrap_or_default();
    if !diagnostics.enable_periodic_log {
        return Ok(());
    }
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default()
        .0;
    let interval_ticks = diagnostics.log_interval_ticks.max(1);
    if interval_ticks == 0 {
        return Ok(());
    }
    {
        let mut state = world.resource_mut::<NetSyncDiagnosticsLogState>()?;
        if tick == 0 || tick < state.last_logged_tick.saturating_add(interval_ticks) {
            return Ok(());
        }
        state.last_logged_tick = tick;
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let connected = world
        .resource::<NetworkSessionStatus>()
        .map(|status| status.connected)
        .unwrap_or(false);
    let connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|id| id.0));
    let local_player_id = world
        .resource::<LocalPlayerRef>()
        .ok()
        .and_then(|local| local.player_id);
    let metrics = world
        .resource::<ReplicationRuntimeMetrics>()
        .copied()
        .unwrap_or_default();
    let smoothing = world
        .resource::<AdaptiveSmoothingState>()
        .copied()
        .unwrap_or_default();
    let correction = world
        .resource::<CorrectionStats>()
        .copied()
        .unwrap_or_default();
    let replication_state = world
        .resource::<ClientReplicationStateV2>()
        .cloned()
        .unwrap_or_default();
    let rtt_ms = world
        .resource::<RoundTripMetrics>()
        .ok()
        .and_then(|metrics| metrics.last_rtt_millis.map(|millis| millis as f32))
        .unwrap_or(smoothing.last_rtt_ms);

    tracing::info!(
        tick,
        ?authority,
        connected,
        connection_id,
        local_player_id,
        tx_bytes = metrics.bytes_sent_last_tick,
        rx_bytes = metrics.bytes_received_last_frame,
        rx_keyframes = metrics.keyframes_received_last_frame,
        rx_patches = metrics.patches_received_last_frame,
        load_shed_level = metrics.load_shed_level_last_tick,
        patch_us = metrics.patch_apply_micros_last,
        keyframes = metrics.keyframes_applied,
        patches = metrics.patches_applied,
        patches_frame = metrics.patches_applied_last_frame,
        patch_skip_mismatch = metrics.patches_skipped_base_mismatch_last_frame,
        patch_skip_stale = metrics.patches_stale_ignored_last_frame,
        restores = metrics.full_world_restores,
        patch_players = metrics.patch_player_ops_last_tick,
        patch_enemies = metrics.patch_enemy_ops_last_tick,
        patch_projectiles = metrics.patch_projectile_ops_last_tick,
        patch_pickups = metrics.patch_pickup_ops_last_tick,
        patch_extraction = metrics.patch_extraction_ops_last_tick,
        repl_cursor = replication_state.last_cursor.stream_cursor,
        repl_has_keyframe = replication_state.has_keyframe,
        drop_enemy = metrics.dropped_enemy_ops_last_tick,
        drop_projectile = metrics.dropped_projectile_ops_last_tick,
        drop_pickup = metrics.dropped_pickup_ops_last_tick,
        drop_extraction = metrics.dropped_extraction_ops_last_tick,
        rtt_ms,
        jitter_ms = smoothing.jitter_ms,
        smooth_delay_ms = smoothing.effective_delay_ms,
        smooth_samples = metrics.smoothing_samples_last_frame,
        smooth_err_mean = metrics.smoothing_error_mean_last_frame,
        smooth_err_max = metrics.smoothing_error_max_last_frame,
        smooth_alpha_mean = metrics.smoothing_alpha_mean_last_frame,
        smooth_extrapolation_ms = metrics.smoothing_extrapolation_ms_last_frame,
        correction_last = metrics.local_correction_distance_last,
        correction_ema = correction.ema_distance,
        correction_hard_snaps = metrics.local_correction_hard_snaps_total,
        "cavern net sync diagnostics"
    );

    Ok(())
}
fn apply_authoritative_geometry_edits(
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

fn apply_authoritative_cavern_snapshot(
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

fn replay_pending_prediction_frames(
    world: &mut World,
    authoritative_tick: SimulationTick,
    local_simulated_tick: SimulationTick,
) -> Result<usize> {
    let fixed_dt = world
        .resource::<FixedTimeConfig>()
        .map(|config| config.step_seconds.max(1.0 / 120.0))
        .unwrap_or(1.0 / 60.0);
    let frames_to_replay = {
        let mut prediction = world.resource_mut::<CavernPredictionState>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > authoritative_tick.0);
        let replay = prediction
            .pending_frames
            .iter()
            .copied()
            .filter(|frame| frame.tick.0 <= local_simulated_tick.0)
            .collect::<Vec<_>>();
        if !replay.is_empty() {
            prediction.corrections_applied = prediction.corrections_applied.saturating_add(1);
        }
        if authoritative_tick.0 > prediction.last_authoritative_tick.0 {
            prediction.last_authoritative_tick = authoritative_tick;
        }
        replay
    };

    let replayed_count = frames_to_replay.len();
    for frame in frames_to_replay {
        crate::plugins::combat::replay_predicted_local_frame(world, frame.control, fixed_dt)?;
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = frame.tick;
        }
    }

    Ok(replayed_count)
}

#[cfg(test)]
mod tests {
    use super::{
        CavernDeltaEventV1, CavernGeometryEditsEventV1, CavernNetSyncState, CavernPatchEventV2,
        CavernSnapshotEventV1, ClientReplicationStateV2, REPLICATION_DELTA_INTERVAL_TICKS,
        RUN_EVENT_DELTA, RUN_EVENT_GEOMETRY_EDITS, RUN_EVENT_KEYFRAME_V2, RUN_EVENT_PATCH_V2,
        RUN_EVENT_SNAPSHOT, ServerReplicationStateByConnection,
        apply_replication_tuning_overrides_from_reader, apply_replication_tuning_preset,
        client_apply_replication_events, client_apply_replication_events_v2,
        compute_load_shed_level_v2, server_capture_control_input, server_emit_replication,
        server_emit_replication_v2, should_emit_patch_channel,
    };
    use crate::domain::{
        AdaptiveSmoothingState, CavernAimState, CavernCameraState, CavernControlState,
        CavernMetaProfile, CavernPlayerOwnershipState, CavernPredictionState, CavernRunConfig,
        CavernRunState, CavernServerControlMap, ClientReplicationMap, CorrectionStats,
        GeometryEdit, GeometryEditKind, GeometryOp, GeometryPrimitiveShape3, InterpolationConfig,
        LocalPlayerRef, LootTableRegistry, NetworkEntityId, ReplicationBudgetConfig,
        ReplicationCadenceConfig, ReplicationLoadShedConfig, ReplicationRuntimeMetrics,
        ServerReplicationMap, SpawnDirector, capture_cavern_run_snapshot,
        restore_cavern_run_snapshot,
    };
    use crate::plugins::{combat, game, worldgen};
    use engine::prelude::{
        FixedTimeConfig, NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus,
        SimulationProfile, SimulationProfileConfig, SimulationTick, World,
    };
    use engine_net::{
        ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame, MoveCommand, RunEvent,
        ServerMessage,
    };

    fn server_world() -> World {
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
        world.insert_resource(CavernControlState::default());
        world.insert_resource(CavernPredictionState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState::default());
        world.insert_resource(NetworkServerOutbox::default());
        world.insert_resource(NetworkSessionStatus {
            phase: engine_net::SessionPhase::Active,
            connection_id: Some(ConnectionId(7)),
            connected: true,
            ..Default::default()
        });
        world.insert_resource(CavernNetSyncState::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Server,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        world.insert_resource(SimulationTick(1));
        worldgen::initialize_run_world(&mut world, false).unwrap();
        world
    }

    #[test]
    fn v2_load_shed_level_accounts_for_connections_and_drops() {
        let cfg = ReplicationLoadShedConfig::default();
        assert_eq!(compute_load_shed_level_v2(10_000, 0, 2, &cfg), 0);
        assert_eq!(compute_load_shed_level_v2(10_000, 0, 3, &cfg), 1);
        assert_eq!(compute_load_shed_level_v2(55_000, 0, 4, &cfg), 2);
        assert_eq!(compute_load_shed_level_v2(30_000, 2, 2, &cfg), 1);
        assert_eq!(compute_load_shed_level_v2(30_000, 30, 2, &cfg), 2);
    }

    #[test]
    fn v2_patch_channel_interval_gating_is_deterministic() {
        assert!(should_emit_patch_channel(7, 1));
        assert!(!should_emit_patch_channel(7, 0));
        assert!(!should_emit_patch_channel(7, 2));
        assert!(should_emit_patch_channel(8, 2));
        assert!(should_emit_patch_channel(12, 3));
    }

    #[test]
    fn env_tuning_overrides_apply_and_clamp() {
        let mut budget = ReplicationBudgetConfig::default();
        let mut cadence = ReplicationCadenceConfig::default();
        let mut diagnostics = Vec::new();
        let vars = std::collections::BTreeMap::from([
            ("CAVERN_NET_BUDGET_ENEMY_L0", "220".to_string()),
            ("CAVERN_NET_BUDGET_PROJECTILE_L2", "99999".to_string()),
            ("CAVERN_NET_CADENCE_ENEMY_L1", "0".to_string()),
            ("CAVERN_NET_CADENCE_PICKUP_L2", "x".to_string()),
        ]);
        apply_replication_tuning_overrides_from_reader(
            &mut budget,
            &mut cadence,
            |key| vars.get(key).cloned(),
            &mut diagnostics,
        );
        assert_eq!(budget.enemy_ops_per_patch_level0, 220);
        assert_eq!(budget.projectile_ops_per_patch_level2, 4096);
        assert_eq!(cadence.enemy_patch_interval_level1, 0);
        assert!(diagnostics.iter().any(|d| d.contains("PROJECTILE_L2")));
        assert!(diagnostics.iter().any(|d| d.contains("PICKUP_L2")));
    }

    #[test]
    fn preset_tuning_applies_four_local_profile() {
        let mut budget = ReplicationBudgetConfig::default();
        let mut cadence = ReplicationCadenceConfig::default();
        let mut diagnostics = Vec::new();
        apply_replication_tuning_preset(
            &mut budget,
            &mut cadence,
            Some("four_local"),
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());
        assert_eq!(budget.enemy_ops_per_patch_level0, 96);
        assert_eq!(budget.projectile_ops_per_patch_level1, 96);
        assert_eq!(cadence.enemy_patch_interval_level0, 2);
        assert_eq!(cadence.pickup_patch_interval_level2, 12);
    }

    #[test]
    fn server_maps_input_frames_to_stable_player_ids_by_connection() {
        let mut world = server_world();
        world.insert_resource(NetworkInboundQueue::default());

        {
            let mut inbound = world.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.push_client(
                Some(ConnectionId(11)),
                ClientMessage::InputFrame(InputFrame {
                    tick: SimulationTick(3),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 })],
                }),
            );
            inbound.push_client(
                Some(ConnectionId(22)),
                ClientMessage::InputFrame(InputFrame {
                    tick: SimulationTick(4),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.0, y: 1.0 })],
                }),
            );
        }

        server_capture_control_input(&mut world).unwrap();
        game::sync_active_player_slots(&mut world).unwrap();

        let ownership = world.resource::<CavernPlayerOwnershipState>().unwrap();
        assert_eq!(ownership.by_connection_id.get(&11), Some(&1));
        assert_eq!(ownership.by_connection_id.get(&22), Some(&2));

        let controls = world.resource::<CavernServerControlMap>().unwrap();
        assert_eq!(
            controls.by_player_id.get(&1).map(|state| state.movement),
            Some([1.0, 0.0])
        );
        assert_eq!(
            controls.by_player_id.get(&2).map(|state| state.movement),
            Some([0.0, 1.0])
        );
    }

    #[test]
    fn v2_patch_rebinds_local_player_from_owner_connection_id() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(22, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut player_state = snapshot.players.first().cloned().unwrap();
        player_state.owner_connection_id = Some(22);

        let mut client = World::new();
        client.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(22)),
            connected: true,
            ..Default::default()
        });
        client.insert_resource(LocalPlayerRef {
            player_id: Some(99),
            entity: None,
        });
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());

        super::apply_player_patch_ops_v2(
            &mut client,
            vec![super::CavernPlayerPatchOpV2::Spawn {
                entity_id: NetworkEntityId(0x1000_0001),
                priority: super::CavernPatchPriorityV2::Critical,
                state: player_state,
            }],
            None,
            true,
        )
        .unwrap();

        let local = client.resource::<LocalPlayerRef>().unwrap().clone();
        assert_eq!(local.player_id, Some(1));
        let local_entity = local.entity.expect("local player entity should be set");
        assert_eq!(
            client
                .get::<crate::domain::PlayerId>(local_entity)
                .unwrap()
                .0,
            1
        );
    }

    #[test]
    fn v2_local_patch_ignores_duplicate_authoritative_tick_for_transform() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(22, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut state_a = snapshot.players.first().cloned().unwrap();
        state_a.owner_connection_id = Some(22);
        let mut state_b = state_a.clone();
        state_b.x += 2.0;
        state_b.y += 1.5;

        let mut client = World::new();
        client.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(22)),
            connected: true,
            ..Default::default()
        });
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(CavernPredictionState {
            pending_frames: Vec::new(),
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick::default(),
        });
        client.insert_resource(SimulationTick(90));

        let entity_id = NetworkEntityId(0x1000_0001);
        super::apply_player_patch_ops_v2(
            &mut client,
            vec![super::CavernPlayerPatchOpV2::Spawn {
                entity_id,
                priority: super::CavernPatchPriorityV2::Critical,
                state: state_a.clone(),
            }],
            Some(SimulationTick(4)),
            true,
        )
        .unwrap();
        let local_entity = client.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        let transform_before = client
            .get::<crate::domain::Transform2>(local_entity)
            .unwrap()
            .x;

        super::apply_player_patch_ops_v2(
            &mut client,
            vec![super::CavernPlayerPatchOpV2::Patch {
                entity_id,
                priority: super::CavernPatchPriorityV2::High,
                state: state_b,
            }],
            Some(SimulationTick(4)),
            true,
        )
        .unwrap();

        let transform_after = client
            .get::<crate::domain::Transform2>(local_entity)
            .unwrap();
        assert!(
            (transform_after.x - transform_before).abs() < 0.0001,
            "duplicate authoritative tick should not re-correct x: before={} after={}",
            transform_before,
            transform_after.x
        );
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .local_correction_distance_last,
            0.0
        );
    }

    #[test]
    fn v2_local_patch_prefers_player_authoritative_input_tick_over_cursor_tick() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(22, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut state_a = snapshot.players.first().cloned().unwrap();
        state_a.owner_connection_id = Some(22);
        state_a.authoritative_input_tick = Some(SimulationTick(10));
        let mut state_b = state_a.clone();
        state_b.x += 2.0;
        state_b.y += 1.5;
        state_b.authoritative_input_tick = Some(SimulationTick(10));

        let mut client = World::new();
        client.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(22)),
            connected: true,
            ..Default::default()
        });
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(CavernPredictionState {
            pending_frames: Vec::new(),
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick::default(),
        });
        client.insert_resource(SimulationTick(90));

        let entity_id = NetworkEntityId(0x1000_0001);
        super::apply_player_patch_ops_v2(
            &mut client,
            vec![super::CavernPlayerPatchOpV2::Spawn {
                entity_id,
                priority: super::CavernPatchPriorityV2::Critical,
                state: state_a,
            }],
            Some(SimulationTick(10)),
            true,
        )
        .unwrap();
        let local_entity = client.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        let transform_before = client
            .get::<crate::domain::Transform2>(local_entity)
            .unwrap()
            .x;

        super::apply_player_patch_ops_v2(
            &mut client,
            vec![super::CavernPlayerPatchOpV2::Patch {
                entity_id,
                priority: super::CavernPatchPriorityV2::High,
                state: state_b,
            }],
            Some(SimulationTick(120)),
            true,
        )
        .unwrap();

        let transform_after = client
            .get::<crate::domain::Transform2>(local_entity)
            .unwrap();
        assert!(
            (transform_after.x - transform_before).abs() < 0.0001,
            "player authoritative input tick should gate corrections even when cursor advances: before={} after={}",
            transform_before,
            transform_after.x
        );
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .local_correction_distance_last,
            0.0
        );
        assert_eq!(
            client
                .resource::<CavernPredictionState>()
                .unwrap()
                .last_authoritative_tick,
            SimulationTick(10)
        );
    }

    #[test]
    fn server_emits_snapshot_then_delta_events() {
        let mut world = server_world();
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut world).unwrap();
        server_emit_replication(&mut world).unwrap();
        let messages = world.resource_mut::<NetworkServerOutbox>().unwrap().drain();
        assert!(matches!(
            messages.first(),
            Some(ServerMessage::RunEvent(RunEvent { code, .. })) if code == RUN_EVENT_SNAPSHOT
        ));

        let local = world
            .query::<(engine::prelude::Entity, &crate::domain::Player)>()
            .iter()
            .map(|(entity, _)| entity)
            .next()
            .unwrap();
        if let Some(mut transform) = world.get_mut::<crate::domain::Transform2>(local) {
            transform.x += 1.0;
        }
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            let interval = REPLICATION_DELTA_INTERVAL_TICKS.max(1);
            tick.0 = ((tick.0 / interval) + 1) * interval;
        }
        server_emit_replication(&mut world).unwrap();
        let messages = world.resource_mut::<NetworkServerOutbox>().unwrap().drain();
        assert!(matches!(
            messages.first(),
            Some(ServerMessage::RunEvent(RunEvent { code, .. })) if code == RUN_EVENT_DELTA
        ));
    }

    #[test]
    fn v2_patch_flow_avoids_full_restore_after_initial_keyframe() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        server.insert_resource(engine_net::ServerSessionState {
            phase: engine_net::SessionPhase::Active,
            active_connection: Some(ConnectionId(7)),
            active_connections: [ConnectionId(7)].into_iter().collect(),
            ..Default::default()
        });
        server.insert_resource(ServerReplicationMap::default());
        server.insert_resource(ServerReplicationStateByConnection::default());
        server.insert_resource(ReplicationRuntimeMetrics::default());
        game::sync_active_player_slots(&mut server).unwrap();

        server_emit_replication_v2(&mut server).unwrap();
        let initial_messages = server
            .resource_mut::<NetworkServerOutbox>()
            .unwrap()
            .drain();
        assert!(initial_messages.iter().any(|message| {
            matches!(
                message,
                ServerMessage::RunEvent(RunEvent { code, .. }) if code == RUN_EVENT_KEYFRAME_V2
            )
        }));

        let player_entity = server
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        if let Some(mut transform) = server.get_mut::<crate::domain::Transform2>(player_entity) {
            transform.x += 0.75;
        }
        if let Ok(mut tick) = server.resource_mut::<SimulationTick>() {
            tick.0 = 2;
        }

        server_emit_replication_v2(&mut server).unwrap();
        let patch_messages = server
            .resource_mut::<NetworkServerOutbox>()
            .unwrap()
            .drain();
        assert!(patch_messages.iter().any(|message| {
            matches!(
                message,
                ServerMessage::RunEvent(RunEvent { code, .. }) if code == RUN_EVENT_PATCH_V2
            )
        }));

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick::default());

        {
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            for message in initial_messages {
                inbound.push_server(message);
            }
        }
        client_apply_replication_events_v2(&mut client).unwrap();
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .full_world_restores,
            1
        );
        let player_entity_after_keyframe = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();

        {
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            for message in patch_messages {
                inbound.push_server(message);
            }
        }
        client_apply_replication_events_v2(&mut client).unwrap();
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .full_world_restores,
            1
        );
        assert!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .patches_applied
                >= 1
        );
        let player_entity_after_patch = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        assert_eq!(player_entity_after_patch, player_entity_after_keyframe);
    }

    #[test]
    fn v2_applies_contiguous_patch_sequence_from_single_inbound_batch() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut player_step_1 = snapshot
            .players
            .iter()
            .find(|player| player.player_id == 1)
            .cloned()
            .unwrap();
        player_step_1.x += 0.75;
        let mut player_step_2 = player_step_1.clone();
        player_step_2.x += 0.75;

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(120));

        let keyframe_payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
            cursor: crate::domain::ReplicationCursor {
                server_tick: SimulationTick(60),
                stream_cursor: 1,
                base_cursor: 0,
            },
            snapshot,
        })
        .unwrap();
        let patch_1_payload = postcard::to_allocvec(&CavernPatchEventV2 {
            cursor: crate::domain::ReplicationCursor {
                server_tick: SimulationTick(61),
                stream_cursor: 2,
                base_cursor: 1,
            },
            run_state: None,
            player_ops: vec![crate::domain::CavernPlayerPatchOpV2::Patch {
                entity_id: NetworkEntityId(0x1000_0001),
                priority: crate::domain::CavernPatchPriorityV2::High,
                state: player_step_1,
            }],
            enemy_ops: Vec::new(),
            projectile_ops: Vec::new(),
            pickup_ops: Vec::new(),
            extraction_ops: Vec::new(),
        })
        .unwrap();
        let patch_2_payload = postcard::to_allocvec(&CavernPatchEventV2 {
            cursor: crate::domain::ReplicationCursor {
                server_tick: SimulationTick(62),
                stream_cursor: 3,
                base_cursor: 2,
            },
            run_state: None,
            player_ops: vec![crate::domain::CavernPlayerPatchOpV2::Patch {
                entity_id: NetworkEntityId(0x1000_0001),
                priority: crate::domain::CavernPatchPriorityV2::High,
                state: player_step_2.clone(),
            }],
            enemy_ops: Vec::new(),
            projectile_ops: Vec::new(),
            pickup_ops: Vec::new(),
            extraction_ops: Vec::new(),
        })
        .unwrap();

        {
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload: keyframe_payload,
            }));
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_PATCH_V2.to_string(),
                payload: patch_1_payload,
            }));
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_PATCH_V2.to_string(),
                payload: patch_2_payload,
            }));
        }

        client_apply_replication_events_v2(&mut client).unwrap();

        let state = client.resource::<ClientReplicationStateV2>().unwrap();
        assert!(state.has_keyframe);
        assert_eq!(state.last_cursor.stream_cursor, 3);

        let metrics = client.resource::<ReplicationRuntimeMetrics>().unwrap();
        assert_eq!(metrics.keyframes_applied, 1);
        assert_eq!(metrics.patches_applied, 2);
        assert_eq!(metrics.patches_applied_last_frame, 2);
        assert_eq!(metrics.patches_skipped_base_mismatch_last_frame, 0);
        assert_eq!(metrics.patches_stale_ignored_last_frame, 0);
    }

    #[test]
    fn v2_local_patch_replays_pending_prediction_frames() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(3));

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(1),
                    stream_cursor: 1,
                    base_cursor: 0,
                },
                snapshot: snapshot.clone(),
            })
            .unwrap();
            client
                .resource_mut::<NetworkInboundQueue>()
                .unwrap()
                .push_server(ServerMessage::RunEvent(RunEvent {
                    code: RUN_EVENT_KEYFRAME_V2.to_string(),
                    payload,
                }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let local_entity = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        if let Some(mut transform) = client.get_mut::<crate::domain::Transform2>(local_entity) {
            transform.x += 2.0;
        }

        client.insert_resource(CavernPredictionState {
            pending_frames: vec![crate::domain::CavernPredictedFrame {
                tick: SimulationTick(3),
                control: CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [100.0, 0.0],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: SimulationTick(3),
                },
            }],
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick(1),
        });
        client.insert_resource(SimulationTick(3));

        let local_state = snapshot
            .players
            .iter()
            .find(|player| player.player_id == 1)
            .cloned()
            .unwrap();
        {
            let payload = postcard::to_allocvec(&CavernPatchEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(2),
                    stream_cursor: 2,
                    base_cursor: 1,
                },
                run_state: None,
                player_ops: vec![crate::domain::CavernPlayerPatchOpV2::Patch {
                    entity_id: NetworkEntityId(0x1000_0001),
                    priority: crate::domain::CavernPatchPriorityV2::High,
                    state: local_state,
                }],
                enemy_ops: Vec::new(),
                projectile_ops: Vec::new(),
                pickup_ops: Vec::new(),
                extraction_ops: Vec::new(),
            })
            .unwrap();
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_PATCH_V2.to_string(),
                payload,
            }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let prediction = client.resource::<CavernPredictionState>().unwrap();
        assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
        assert_eq!(prediction.corrections_applied, 1);
        assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
    }

    #[test]
    fn v2_local_patch_replays_from_player_authoritative_input_tick() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(3));

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(1),
                    stream_cursor: 1,
                    base_cursor: 0,
                },
                snapshot: snapshot.clone(),
            })
            .unwrap();
            client
                .resource_mut::<NetworkInboundQueue>()
                .unwrap()
                .push_server(ServerMessage::RunEvent(RunEvent {
                    code: RUN_EVENT_KEYFRAME_V2.to_string(),
                    payload,
                }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let local_entity = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        if let Some(mut transform) = client.get_mut::<crate::domain::Transform2>(local_entity) {
            transform.x += 2.0;
        }

        client.insert_resource(CavernPredictionState {
            pending_frames: vec![crate::domain::CavernPredictedFrame {
                tick: SimulationTick(3),
                control: CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [100.0, 0.0],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: SimulationTick(3),
                },
            }],
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick(1),
        });
        client.insert_resource(SimulationTick(3));

        let mut local_state = snapshot
            .players
            .iter()
            .find(|player| player.player_id == 1)
            .cloned()
            .unwrap();
        local_state.authoritative_input_tick = Some(SimulationTick(2));
        {
            let payload = postcard::to_allocvec(&CavernPatchEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(120),
                    stream_cursor: 2,
                    base_cursor: 1,
                },
                run_state: None,
                player_ops: vec![crate::domain::CavernPlayerPatchOpV2::Patch {
                    entity_id: NetworkEntityId(0x1000_0001),
                    priority: crate::domain::CavernPatchPriorityV2::High,
                    state: local_state,
                }],
                enemy_ops: Vec::new(),
                projectile_ops: Vec::new(),
                pickup_ops: Vec::new(),
                extraction_ops: Vec::new(),
            })
            .unwrap();
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_PATCH_V2.to_string(),
                payload,
            }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let prediction = client.resource::<CavernPredictionState>().unwrap();
        assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
        assert_eq!(prediction.corrections_applied, 1);
        assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
    }

    #[test]
    fn v2_followup_keyframe_preserves_owned_pose_without_pending_prediction() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(120));

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(60),
                    stream_cursor: 1,
                    base_cursor: 0,
                },
                snapshot: snapshot.clone(),
            })
            .unwrap();
            client
                .resource_mut::<NetworkInboundQueue>()
                .unwrap()
                .push_server(ServerMessage::RunEvent(RunEvent {
                    code: RUN_EVENT_KEYFRAME_V2.to_string(),
                    payload,
                }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let local_entity = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        let predicted_transform = crate::domain::Transform2::new(9.5, -4.25, 0.45);
        let predicted_velocity = crate::domain::Velocity2 { x: 5.5, y: -2.0 };
        let _ = client.insert(local_entity, predicted_transform);
        let _ = client.insert(local_entity, predicted_velocity);
        assert!(matches!(
            super::current_net_sync_mode(&client),
            super::CavernNetSyncMode::V2
        ));
        assert!(
            client
                .resource::<ClientReplicationStateV2>()
                .unwrap()
                .has_keyframe
        );

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(120),
                    stream_cursor: 2,
                    base_cursor: 1,
                },
                snapshot,
            })
            .unwrap();
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload,
            }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let local_entity_after = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        let transform_after = client
            .get::<crate::domain::Transform2>(local_entity_after)
            .copied()
            .unwrap();
        let velocity_after = client
            .get::<crate::domain::Velocity2>(local_entity_after)
            .copied()
            .unwrap();
        assert!(
            (transform_after.x - predicted_transform.x).abs() < 0.001,
            "x after={} predicted={}",
            transform_after.x,
            predicted_transform.x
        );
        assert!(
            (transform_after.y - predicted_transform.y).abs() < 0.001,
            "y after={} predicted={}",
            transform_after.y,
            predicted_transform.y
        );
        assert!(
            (transform_after.yaw - predicted_transform.yaw).abs() < 0.001,
            "yaw after={} predicted={}",
            transform_after.yaw,
            predicted_transform.yaw
        );
        assert_eq!(velocity_after, predicted_velocity);
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .full_world_restores,
            1
        );
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .local_correction_hard_snaps_total,
            0
        );
    }

    #[test]
    fn v2_followup_keyframe_replays_from_player_authoritative_input_tick() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(3));

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(1),
                    stream_cursor: 1,
                    base_cursor: 0,
                },
                snapshot: snapshot.clone(),
            })
            .unwrap();
            client
                .resource_mut::<NetworkInboundQueue>()
                .unwrap()
                .push_server(ServerMessage::RunEvent(RunEvent {
                    code: RUN_EVENT_KEYFRAME_V2.to_string(),
                    payload,
                }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let local_entity = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 1).then_some(entity))
            .unwrap();
        if let Some(mut transform) = client.get_mut::<crate::domain::Transform2>(local_entity) {
            transform.x += 2.0;
        }

        client.insert_resource(CavernPredictionState {
            pending_frames: vec![crate::domain::CavernPredictedFrame {
                tick: SimulationTick(3),
                control: CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [100.0, 0.0],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: SimulationTick(3),
                },
            }],
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick(1),
        });
        client.insert_resource(SimulationTick(3));

        let mut followup_snapshot = snapshot;
        if let Some(local_player) = followup_snapshot
            .players
            .iter_mut()
            .find(|player| player.player_id == 1)
        {
            local_player.authoritative_input_tick = Some(SimulationTick(2));
        }
        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(120),
                    stream_cursor: 120,
                    base_cursor: 119,
                },
                snapshot: followup_snapshot,
            })
            .unwrap();
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload,
            }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let prediction = client.resource::<CavernPredictionState>().unwrap();
        assert_eq!(prediction.last_authoritative_tick, SimulationTick(2));
        assert_eq!(prediction.corrections_applied, 1);
        assert_eq!(client.resource::<SimulationTick>().unwrap().0, 3);
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .full_world_restores,
            2
        );
    }

    #[test]
    fn v2_followup_keyframe_preserves_remote_pose_for_smoothing() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1), (22, 2)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(11)),
            connected: true,
            ..Default::default()
        });
        client.insert_resource(LocalPlayerRef {
            player_id: Some(1),
            entity: None,
        });
        client.insert_resource(ClientReplicationStateV2::default());
        client.insert_resource(ClientReplicationMap::default());
        client.insert_resource(crate::domain::NetSyncModeConfig::V2);
        client.insert_resource(InterpolationConfig::default());
        client.insert_resource(AdaptiveSmoothingState::default());
        client.insert_resource(CorrectionStats::default());
        client.insert_resource(ReplicationRuntimeMetrics::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(120));

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(60),
                    stream_cursor: 1,
                    base_cursor: 0,
                },
                snapshot: snapshot.clone(),
            })
            .unwrap();
            client
                .resource_mut::<NetworkInboundQueue>()
                .unwrap()
                .push_server(ServerMessage::RunEvent(RunEvent {
                    code: RUN_EVENT_KEYFRAME_V2.to_string(),
                    payload,
                }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let remote_entity = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 2).then_some(entity))
            .unwrap();
        let remote_predicted = crate::domain::Transform2::new(-7.25, 4.0, -0.35);
        let _ = client.insert(remote_entity, remote_predicted);

        {
            let payload = postcard::to_allocvec(&crate::domain::CavernKeyframeEventV2 {
                cursor: crate::domain::ReplicationCursor {
                    server_tick: SimulationTick(120),
                    stream_cursor: 2,
                    base_cursor: 1,
                },
                snapshot,
            })
            .unwrap();
            let mut inbound = client.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.clear();
            inbound.push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_KEYFRAME_V2.to_string(),
                payload,
            }));
        }
        client_apply_replication_events_v2(&mut client).unwrap();

        let remote_entity_after = client
            .query::<(engine::prelude::Entity, &crate::domain::PlayerId)>()
            .iter()
            .find_map(|(entity, player_id)| (player_id.0 == 2).then_some(entity))
            .unwrap();
        let remote_after = client
            .get::<crate::domain::Transform2>(remote_entity_after)
            .copied()
            .unwrap();
        assert!((remote_after.x - remote_predicted.x).abs() < 0.001);
        assert!((remote_after.y - remote_predicted.y).abs() < 0.001);
        assert!((remote_after.yaw - remote_predicted.yaw).abs() < 0.001);
        assert_eq!(
            client
                .resource::<ReplicationRuntimeMetrics>()
                .unwrap()
                .full_world_restores,
            1
        );
    }

    #[test]
    fn v2_patch_budget_caps_projectile_ops_and_tracks_drops() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        server.insert_resource(engine_net::ServerSessionState {
            phase: engine_net::SessionPhase::Active,
            active_connection: Some(ConnectionId(7)),
            active_connections: [ConnectionId(7)].into_iter().collect(),
            ..Default::default()
        });
        server.insert_resource(ServerReplicationMap::default());
        server.insert_resource(ServerReplicationStateByConnection::default());
        server.insert_resource(ReplicationRuntimeMetrics::default());
        server.insert_resource(ReplicationBudgetConfig {
            enemy_ops_per_patch_level0: 64,
            enemy_ops_per_patch_level1: 64,
            enemy_ops_per_patch_level2: 64,
            projectile_ops_per_patch_level0: 2,
            projectile_ops_per_patch_level1: 2,
            projectile_ops_per_patch_level2: 2,
            pickup_ops_per_patch_level0: 64,
            pickup_ops_per_patch_level1: 64,
            pickup_ops_per_patch_level2: 64,
            extraction_ops_per_patch_level0: 64,
            extraction_ops_per_patch_level1: 64,
            extraction_ops_per_patch_level2: 64,
        });
        game::sync_active_player_slots(&mut server).unwrap();

        server_emit_replication_v2(&mut server).unwrap();
        server
            .resource_mut::<NetworkServerOutbox>()
            .unwrap()
            .drain();

        for i in 0..10 {
            combat::spawn_projectile(
                &mut server,
                [i as f32 * 0.1, 0.0],
                [1.0, 0.0],
                8.0,
                1.2,
                crate::domain::Faction::Hunters,
            );
        }
        if let Ok(mut tick) = server.resource_mut::<SimulationTick>() {
            tick.0 = 2;
        }

        server_emit_replication_v2(&mut server).unwrap();
        let messages = server
            .resource_mut::<NetworkServerOutbox>()
            .unwrap()
            .drain();
        let patch = messages
            .iter()
            .find_map(|message| match message {
                ServerMessage::RunEvent(RunEvent { code, payload })
                    if code == RUN_EVENT_PATCH_V2 =>
                {
                    postcard::from_bytes::<CavernPatchEventV2>(payload).ok()
                }
                _ => None,
            })
            .expect("expected v2 patch event");
        assert!(patch.projectile_ops.len() <= 2);
        let metrics = server.resource::<ReplicationRuntimeMetrics>().unwrap();
        assert!(metrics.dropped_projectile_ops_last_tick > 0);
        assert!(metrics.patch_projectile_ops_last_tick <= 2);
    }

    #[test]
    fn server_emits_geometry_edit_event_when_runtime_geometry_changes() {
        let mut world = server_world();
        server_emit_replication(&mut world).unwrap();
        world.resource_mut::<NetworkServerOutbox>().unwrap().drain();

        let _ = crate::plugins::worldgen::apply_runtime_geometry_edit(
            &mut world,
            &GeometryEdit {
                kind: GeometryEditKind::AddBlocker(GeometryPrimitiveShape3::Cylinder {
                    center: [0.0, crate::domain::CAVERN_GAMEPLAY_HEIGHT, 0.0],
                    radius: 1.2,
                    half_height: 1.4,
                }),
            },
        );

        server_emit_replication(&mut world).unwrap();
        let messages = world.resource_mut::<NetworkServerOutbox>().unwrap().drain();
        assert!(messages.iter().any(|message| {
            matches!(
                message,
                ServerMessage::RunEvent(RunEvent { code, .. }) if code == RUN_EVENT_GEOMETRY_EDITS
            )
        }));
    }

    #[test]
    fn client_applies_snapshot_and_delta_events() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let snapshot_event = RunEvent {
            code: RUN_EVENT_SNAPSHOT.to_string(),
            payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                tick: SimulationTick(2),
                cursor: 1,
                snapshot: snapshot.clone(),
            })
            .unwrap(),
        };

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick::default());
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(snapshot_event));
        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        assert_eq!(rebuilt, snapshot);

        let local = server
            .query::<(engine::prelude::Entity, &crate::domain::Player)>()
            .iter()
            .map(|(entity, _)| entity)
            .next()
            .unwrap();
        if let Some(mut transform) = server.get_mut::<crate::domain::Transform2>(local) {
            transform.x += 2.0;
        }
        combat::spawn_projectile(
            &mut server,
            [1.0, 1.0],
            [1.0, 0.0],
            6.0,
            1.5,
            crate::domain::Faction::Hunters,
        );
        let current = capture_cavern_run_snapshot(&server).unwrap();
        let delta = crate::domain::build_cavern_run_delta(&snapshot, &current);
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .clear();
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_DELTA.to_string(),
                payload: postcard::to_allocvec(&CavernDeltaEventV1 {
                    tick: SimulationTick(3),
                    base_cursor: 1,
                    cursor: 2,
                    delta,
                })
                .unwrap(),
            }));
        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        assert_eq!(rebuilt, current);
    }

    #[test]
    fn client_applies_geometry_edit_event_incrementally() {
        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick::default());
        client.insert_resource(CavernRunConfig::default());
        client.insert_resource(CavernRunState::default());
        client.insert_resource(crate::domain::CavernLayout::default());
        client.insert_resource(SpawnDirector::default());
        client.insert_resource(LootTableRegistry::default());
        client.insert_resource(CavernMetaProfile::default());
        client.insert_resource(CavernCameraState::default());
        client.insert_resource(CavernAimState::default());
        crate::plugins::worldgen::initialize_run_world(&mut client, true).unwrap();
        let baseline_runtime = client
            .resource::<crate::domain::CavernGeometryRuntimeState>()
            .unwrap()
            .clone();
        let baseline_edit_count = baseline_runtime.edit_events.len();
        let baseline_blocker_count = client
            .resource::<crate::domain::CavernGeometryGraph>()
            .unwrap()
            .primitives
            .iter()
            .filter(|primitive| primitive.op == GeometryOp::Blocker)
            .count();
        let next_revision = client
            .resource::<crate::domain::CavernGeometryGraph>()
            .unwrap()
            .revision
            .0
            .saturating_add(1);

        let edits = vec![crate::domain::GeometryEditEvent {
            revision: crate::domain::GeometryRevision(next_revision),
            edit: GeometryEdit {
                kind: GeometryEditKind::AddBlocker(GeometryPrimitiveShape3::Cylinder {
                    center: [0.0, crate::domain::CAVERN_GAMEPLAY_HEIGHT, 0.0],
                    radius: 1.2,
                    half_height: 1.4,
                }),
            },
        }];
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_GEOMETRY_EDITS.to_string(),
                payload: postcard::to_allocvec(&CavernGeometryEditsEventV1 {
                    tick: SimulationTick(5),
                    from_index: baseline_edit_count,
                    to_index: baseline_edit_count + edits.len(),
                    extraction_seal_primitive: None,
                    edits: edits.clone(),
                })
                .unwrap(),
            }));

        client_apply_replication_events(&mut client).unwrap();

        let runtime = client
            .resource::<crate::domain::CavernGeometryRuntimeState>()
            .unwrap();
        assert_eq!(runtime.edit_events.len(), baseline_edit_count + edits.len());
        assert_eq!(runtime.edit_events.last(), edits.last());
        let graph = client
            .resource::<crate::domain::CavernGeometryGraph>()
            .unwrap();
        let blocker_count = graph
            .primitives
            .iter()
            .filter(|primitive| primitive.op == GeometryOp::Blocker)
            .count();
        assert_eq!(blocker_count, baseline_blocker_count + 1);
    }

    #[test]
    fn client_replays_pending_predicted_frame_after_authoritative_snapshot() {
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState {
            pending_frames: vec![crate::domain::CavernPredictedFrame {
                tick: SimulationTick(2),
                control: CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [100.0, 0.0],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: SimulationTick(2),
                },
            }],
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick::default(),
        });
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(2));
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_SNAPSHOT.to_string(),
                payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick: SimulationTick(1),
                    cursor: 1,
                    snapshot: snapshot.clone(),
                })
                .unwrap(),
            }));

        let mut expected = World::new();
        expected.insert_resource(FixedTimeConfig::default());
        expected.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut expected, &snapshot).unwrap();
        combat::replay_predicted_local_frame(
            &mut expected,
            CavernControlState {
                movement: [1.0, 0.0],
                aim_world: [100.0, 0.0],
                fire_pressed: false,
                dash_pressed: false,
                interact_pressed: false,
                source_tick: SimulationTick(2),
            },
            FixedTimeConfig::default().step_seconds,
        )
        .unwrap();

        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        let expected_snapshot = capture_cavern_run_snapshot(&expected).unwrap();
        assert_eq!(rebuilt, expected_snapshot);
        let prediction = client.resource::<CavernPredictionState>().unwrap();
        assert_eq!(prediction.pending_frames.len(), 1);
        assert_eq!(prediction.last_authoritative_tick, SimulationTick(1));
        assert_eq!(prediction.corrections_applied, 1);
    }

    #[test]
    fn two_clients_restore_different_owned_players_from_same_server_run() {
        let mut server = server_world();
        {
            let mut ownership = server.resource_mut::<CavernPlayerOwnershipState>().unwrap();
            ownership.by_connection_id = [(11, 1), (22, 2)].into_iter().collect();
        }
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        assert_eq!(snapshot.players.len(), 2);

        let mut client_a = World::new();
        client_a.insert_resource(NetworkInboundQueue::default());
        client_a.insert_resource(CavernNetSyncState::default());
        client_a.insert_resource(CavernPredictionState::default());
        client_a.insert_resource(FixedTimeConfig::default());
        client_a.insert_resource(LocalPlayerRef::default());
        client_a.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(11)),
            connected: true,
            ..Default::default()
        });
        client_a.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client_a
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_SNAPSHOT.to_string(),
                payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick: SimulationTick(1),
                    cursor: 1,
                    snapshot: snapshot.clone(),
                })
                .unwrap(),
            }));
        client_apply_replication_events(&mut client_a).unwrap();
        assert_eq!(
            client_a.resource::<LocalPlayerRef>().unwrap().player_id,
            Some(1)
        );

        let mut client_b = World::new();
        client_b.insert_resource(NetworkInboundQueue::default());
        client_b.insert_resource(CavernNetSyncState::default());
        client_b.insert_resource(CavernPredictionState::default());
        client_b.insert_resource(FixedTimeConfig::default());
        client_b.insert_resource(LocalPlayerRef::default());
        client_b.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(22)),
            connected: true,
            ..Default::default()
        });
        client_b.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client_b
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_SNAPSHOT.to_string(),
                payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick: SimulationTick(1),
                    cursor: 1,
                    snapshot,
                })
                .unwrap(),
            }));
        client_apply_replication_events(&mut client_b).unwrap();
        assert_eq!(
            client_b.resource::<LocalPlayerRef>().unwrap().player_id,
            Some(2)
        );
    }
}
