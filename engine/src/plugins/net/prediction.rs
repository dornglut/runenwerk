use super::*;
use crate::WorldMut;
use crate::plugins::world::ids::ChunkSyncCursor;
use crate::plugins::world::streaming::interest::WorldStreamingInterestResource;
use ecs::World;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::{AuthorityRole, SimulationProfileConfig, SimulationTick};

// engine/src/plugins/net/prediction.rs

const FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 30;
const MAX_SERVER_SNAPSHOT_HISTORY: usize = 256;
const MAX_CLIENT_SNAPSHOT_HISTORY: usize = 256;

pub fn replication_step_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    if !matches!(authority, AuthorityRole::Server | AuthorityRole::Peer) {
        let cursor = world
            .resource::<SnapshotCursor>()
            .map(|cursor| cursor.0)
            .unwrap_or(0);

        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.last_snapshot_cursor = cursor;
        }
        return Ok(());
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let cursor = {
        let mut cursor = world.resource_mut::<SnapshotCursor>()?;
        cursor.0 = cursor.0.saturating_add(1);
        *cursor
    };

    let active_connections = world
        .resource::<ServerSessionState>()
        .map(|session| {
            session
                .active_connections
                .iter()
                .copied()
                .collect::<Vec<ConnectionId>>()
        })
        .unwrap_or_default();

    let mut outbound = Vec::<OutboundServerMessage>::new();
    let mut world_streaming_updates = Vec::<(ConnectionId, ChunkSyncCursor, bool)>::new();
    if !active_connections.is_empty() {
        let mut snapshots_for_connections = Vec::<(ConnectionId, TDriver::Snapshot)>::new();
        for connection_id in &active_connections {
            let captured_snapshot =
                TDriver::capture_snapshot_for_connection(&world, *connection_id).map_err(|e| {
                    map_driver_error::<TDriver>(e, "capture snapshot for connection")
                })?;
            if let Some(snapshot) = captured_snapshot {
                snapshots_for_connections.push((*connection_id, snapshot));
            }
        }

        let mut state =
            world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()?;
        state.latest_tick = tick;
        state
            .checkpoints
            .retain(|connection_id, _| active_connections.contains(connection_id));
        state
            .snapshot_history_per_connection
            .retain(|connection_id, _| active_connections.contains(connection_id));
        state
            .latest_snapshot_per_connection
            .retain(|connection_id, _| active_connections.contains(connection_id));

        let mut first_snapshot_for_tick: Option<TDriver::Snapshot> = None;

        for (connection_id, snapshot) in snapshots_for_connections {
            if first_snapshot_for_tick.is_none() {
                first_snapshot_for_tick = Some(snapshot.clone());
            }

            state
                .latest_snapshot_per_connection
                .insert(connection_id, snapshot.clone());
            state
                .snapshot_history_per_connection
                .entry(connection_id)
                .or_default()
                .insert(cursor, snapshot.clone());
            prune_snapshot_history_for_connection(&mut state, connection_id);

            let (last_ack_cursor, needs_full_resync) = {
                let checkpoint = state.checkpoints.entry(connection_id).or_default();
                (checkpoint.last_ack_cursor, checkpoint.needs_full_resync)
            };

            let scheduled_full = cursor.0 % FULL_SNAPSHOT_INTERVAL_TICKS == 0;
            let mut send_full = needs_full_resync || scheduled_full || last_ack_cursor.0 == 0;

            let message = if send_full {
                let payload = TDriver::encode_snapshot(&snapshot)
                    .map_err(|e| map_driver_error::<TDriver>(e, "encode snapshot"))?;
                ServerMessage::Snapshot(Snapshot {
                    tick,
                    cursor,
                    last_applied: last_ack_cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            } else if let Some(base_snapshot) = state
                .snapshot_history_per_connection
                .get(&connection_id)
                .and_then(|history| history.get(&last_ack_cursor))
            {
                let delta = TDriver::build_delta(base_snapshot, &snapshot);
                let payload = TDriver::encode_delta(&delta)
                    .map_err(|e| map_driver_error::<TDriver>(e, "encode delta"))?;
                ServerMessage::DeltaSnapshot(DeltaSnapshot {
                    tick,
                    base: last_ack_cursor,
                    cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            } else {
                send_full = true;
                let payload = TDriver::encode_snapshot(&snapshot)
                    .map_err(|e| map_driver_error::<TDriver>(e, "encode fallback snapshot"))?;
                ServerMessage::Snapshot(Snapshot {
                    tick,
                    cursor,
                    last_applied: last_ack_cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            };

            {
                let checkpoint = state.checkpoints.entry(connection_id).or_default();
                checkpoint.last_sent_cursor = cursor;
                if send_full {
                    checkpoint.last_full_snapshot_cursor = cursor;
                    checkpoint.last_full_snapshot_tick = tick;
                    checkpoint.needs_full_resync = false;
                }
            }
            world_streaming_updates.push((connection_id, ChunkSyncCursor(cursor.0), send_full));

            outbound.push(OutboundServerMessage::ToConnection {
                connection_id,
                message,
            });
        }

        if let Some(snapshot) = first_snapshot_for_tick {
            state.latest_snapshot = Some(snapshot.clone());
            state.snapshot_history.insert(cursor, snapshot);
            prune_snapshot_history(&mut state);
        }
    }

    if !world_streaming_updates.is_empty()
        && let Ok(mut streaming_interest) = world.resource_mut::<WorldStreamingInterestResource>()
    {
        for (connection_id, cursor, sent_full_snapshot) in world_streaming_updates {
            streaming_interest.mark_snapshot_sent(connection_id, cursor, sent_full_snapshot);
        }
    }

    if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
        for message in &outbound {
            match message {
                OutboundServerMessage::ToConnection {
                    connection_id,
                    message,
                } => outbox.push_to(*connection_id, message.clone()),
                OutboundServerMessage::Broadcast(message) => outbox.push_broadcast(message.clone()),
            }
        }
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.last_snapshot_cursor = cursor.0;
        diagnostics.emitted_snapshots = diagnostics
            .emitted_snapshots
            .saturating_add(outbound.len() as u64);
    }

    Ok(())
}

pub fn prediction_step_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq,
{
    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    let commands = TDriver::take_local_input(&mut world)
        .map_err(|e| map_driver_error::<TDriver>(e, "take local input"))?;
    if commands.is_empty() {
        return Ok(());
    }

    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.commands_applied = diagnostics
            .commands_applied
            .saturating_add(commands.len() as u64);
    }

    if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
        let payload = TDriver::encode_input(&commands)
            .map_err(|e| map_driver_error::<TDriver>(e, "encode input"))?;

        if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
            outbox.push(ClientMessage::InputFrame(InputFrame { tick, payload }));
        }

        if let Ok(mut prediction) = world.resource_mut::<PredictionState<TDriver::Input>>() {
            prediction.pending_frames.push(PendingInputFrame {
                tick,
                commands: commands.clone(),
            });
        }
    }

    TDriver::apply_input(&mut world, &commands)
        .map_err(|e| map_driver_error::<TDriver>(e, "apply input"))?;

    Ok(())
}

pub fn update_connection_closed<TSnapshot>(
    world: &mut World,
    connection_id: Option<ConnectionId>,
    reason: Option<DisconnectReason>,
) where
    TSnapshot: Clone + PartialEq + 'static,
{
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    if matches!(authority, AuthorityRole::Server) {
        let mut active_connection = None;
        let mut has_active_connections = false;
        let mut active_connections = Vec::<ConnectionId>::new();

        if let Ok(mut session) = world.resource_mut::<ServerSessionState>() {
            match connection_id {
                Some(connection_id) => {
                    remove_server_connection(&mut session, connection_id, reason.clone());
                }
                None => {
                    session.active_connections.clear();
                    session.active_connection = None;
                    session.phase = SessionPhase::Closed;
                    session.last_disconnect = reason.clone();
                }
            }

            active_connection = session.active_connection;
            has_active_connections = !session.active_connections.is_empty();
            active_connections.extend(session.active_connections.iter().copied());
        }

        if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
            status.connected = has_active_connections;
            status.phase = if has_active_connections {
                SessionPhase::Active
            } else {
                SessionPhase::Closed
            };
            status.connection_id = active_connection;
            status.last_disconnect = reason.clone();
        }

        if let Ok(mut state) = world.resource_mut::<ServerSnapshotReplicationState<TSnapshot>>() {
            match connection_id {
                Some(connection_id) => {
                    state.checkpoints.remove(&connection_id);
                    state.snapshot_history_per_connection.remove(&connection_id);
                    state.latest_snapshot_per_connection.remove(&connection_id);
                }
                None => {
                    state.checkpoints.clear();
                    state.snapshot_history.clear();
                    state.snapshot_history_per_connection.clear();
                    state.latest_snapshot = None;
                    state.latest_snapshot_per_connection.clear();
                    state.latest_tick = SimulationTick::default();
                }
            }
            state
                .checkpoints
                .retain(|connection_id, _| active_connections.contains(connection_id));
            state
                .snapshot_history_per_connection
                .retain(|connection_id, _| active_connections.contains(connection_id));
            state
                .latest_snapshot_per_connection
                .retain(|connection_id, _| active_connections.contains(connection_id));
        }

        if let Ok(mut streaming_interest) = world.resource_mut::<WorldStreamingInterestResource>() {
            match connection_id {
                Some(connection_id) => {
                    streaming_interest.per_connection.remove(&connection_id);
                }
                None => {
                    streaming_interest.per_connection.clear();
                }
            }
            streaming_interest
                .per_connection
                .retain(|connection_id, _| active_connections.contains(connection_id));
        }
    } else {
        if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
            status.connected = false;
            status.phase = SessionPhase::Closed;
            status.last_disconnect = reason.clone();
        }
        if let Ok(mut state) = world.resource_mut::<ClientSnapshotReplicationState<TSnapshot>>() {
            reset_client_replication_state(&mut state);
        }
    }

    if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
        health.connected = false;
        health.close_events = health.close_events.saturating_add(1);
    }
}

pub fn apply_authoritative_snapshot<TDriver>(
    world: &mut World,
    tick: SimulationTick,
    cursor: SnapshotCursor,
    snapshot: Option<TDriver::Snapshot>,
    payload: &[u8],
) -> anyhow::Result<bool>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let snapshot = match snapshot {
        Some(snapshot) => snapshot,
        None => TDriver::decode_snapshot(payload)
            .map_err(|e| map_driver_error::<TDriver>(e, "decode snapshot"))?,
    };

    let corrected = TDriver::apply_snapshot(world, tick, snapshot.clone())
        .map_err(|e| map_driver_error::<TDriver>(e, "apply snapshot"))?;

    if let Ok(mut tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }

    if let Ok(mut state) = world.resource_mut::<ClientSnapshotReplicationState<TDriver::Snapshot>>()
    {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(snapshot.clone());
        state.snapshot_history.insert(cursor, snapshot);
        prune_client_snapshot_history(&mut state);
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    replay_pending_prediction::<TDriver>(world, tick, "replay predicted input")?;
    Ok(corrected)
}

pub fn apply_authoritative_delta<TDriver>(
    world: &mut World,
    tick: SimulationTick,
    base: SnapshotCursor,
    cursor: SnapshotCursor,
    payload: &[u8],
) -> anyhow::Result<bool>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let delta = TDriver::decode_delta(payload)
        .map_err(|e| map_driver_error::<TDriver>(e, "decode delta"))?;

    let (expected_base, base_snapshot) = {
        let state = world.resource::<ClientSnapshotReplicationState<TDriver::Snapshot>>()?;
        let base_snapshot = state
            .snapshot_history
            .get(&base)
            .cloned()
            .or_else(|| {
                if state.last_acknowledged_cursor == base {
                    state.last_received_snapshot.clone()
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "received delta snapshot with unknown baseline cursor={}",
                    base.0
                )
            })?;
        (state.last_acknowledged_cursor, base_snapshot)
    };

    if base.0 > expected_base.0 {
        anyhow::bail!(
            "delta base cursor mismatch: expected {} got {}",
            expected_base.0,
            base.0
        );
    }

    let rebuilt_snapshot = TDriver::apply_delta_to_snapshot(&base_snapshot, &delta);
    let corrected = if base == expected_base {
        TDriver::apply_delta(world, tick, delta.clone())
            .map_err(|e| map_driver_error::<TDriver>(e, "apply authoritative delta"))?
    } else {
        TDriver::apply_snapshot(world, tick, rebuilt_snapshot.clone())
            .map_err(|e| map_driver_error::<TDriver>(e, "apply delta via snapshot fallback"))?
    };

    if let Ok(mut tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }

    if let Ok(mut state) = world.resource_mut::<ClientSnapshotReplicationState<TDriver::Snapshot>>()
    {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(rebuilt_snapshot.clone());
        state.snapshot_history.insert(cursor, rebuilt_snapshot);
        prune_client_snapshot_history(&mut state);
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    replay_pending_prediction::<TDriver>(world, tick, "replay predicted input after delta")?;
    Ok(corrected)
}

fn replay_pending_prediction<TDriver>(
    world: &mut World,
    tick: SimulationTick,
    error_context: &'static str,
) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq,
{
    let pending_frames = {
        let mut prediction = world.resource_mut::<PredictionState<TDriver::Input>>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > tick.0);
        prediction.pending_frames.clone()
    };

    for frame in pending_frames {
        TDriver::apply_input(world, &frame.commands)
            .map_err(|e| map_driver_error::<TDriver>(e, error_context))?;
    }

    Ok(())
}

fn reset_client_replication_state<TSnapshot>(state: &mut ClientSnapshotReplicationState<TSnapshot>)
where
    TSnapshot: Clone + PartialEq,
{
    state.last_acknowledged_cursor = SnapshotCursor::default();
    state.last_received_tick = SimulationTick::default();
    state.applied_snapshots = 0;
    state.last_received_snapshot = None;
    state.snapshot_history.clear();
}

fn prune_snapshot_history<TSnapshot>(state: &mut ServerSnapshotReplicationState<TSnapshot>)
where
    TSnapshot: Clone + PartialEq,
{
    while state.snapshot_history.len() > MAX_SERVER_SNAPSHOT_HISTORY {
        let Some(oldest_cursor) = state.snapshot_history.keys().next().copied() else {
            break;
        };
        state.snapshot_history.remove(&oldest_cursor);
    }
}

fn prune_snapshot_history_for_connection<TSnapshot>(
    state: &mut ServerSnapshotReplicationState<TSnapshot>,
    connection_id: ConnectionId,
) where
    TSnapshot: Clone + PartialEq,
{
    let Some(history) = state
        .snapshot_history_per_connection
        .get_mut(&connection_id)
    else {
        return;
    };
    while history.len() > MAX_SERVER_SNAPSHOT_HISTORY {
        let Some(oldest_cursor) = history.keys().next().copied() else {
            break;
        };
        history.remove(&oldest_cursor);
    }
}

fn prune_client_snapshot_history<TSnapshot>(state: &mut ClientSnapshotReplicationState<TSnapshot>)
where
    TSnapshot: Clone + PartialEq,
{
    while state.snapshot_history.len() > MAX_CLIENT_SNAPSHOT_HISTORY {
        let Some(oldest_cursor) = state.snapshot_history.keys().next().copied() else {
            break;
        };
        state.snapshot_history.remove(&oldest_cursor);
    }
}
