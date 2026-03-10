use ecs::World;
use engine_net::*;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_sim::{AuthorityRole, SimulationProfileConfig, SimulationTick};
use crate::plugins::*;
use crate::WorldMut;

// engine/src/plugins/net/prediction.rs

pub fn replication_step_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
  TDriver: ReplicationDriver + Send + Sync + 'static,
  TDriver::Snapshot: Clone + PartialEq,
{
    const FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 30;

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

    let tick = world.resource::<SimulationTick>().copied().unwrap_or_default();

    let cursor = {
        let mut cursor = world.resource_mut::<SnapshotCursor>()?;
        cursor.0 = cursor.0.saturating_add(1);
        *cursor
    };

    let captured_snapshot = TDriver::capture_snapshot(&world)
      .map_err(|e| map_driver_error::<TDriver>(e, "capture snapshot"))?;

    let last_ack = world
      .resource::<SnapshotReplicationState<TDriver::Snapshot>>()
      .map(|state| state.last_acknowledged_cursor)
      .unwrap_or_default();

    let initial_snapshot_sent = world
      .resource::<SnapshotReplicationState<TDriver::Snapshot>>()
      .map(|state| state.initial_snapshot_sent)
      .unwrap_or(false);

    let last_sent_snapshot = world
      .resource::<SnapshotReplicationState<TDriver::Snapshot>>()
      .ok()
      .and_then(|state| state.last_sent_snapshot.clone());

    if let Some(snapshot) = captured_snapshot
      && let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>()
    {
        let should_send_full_snapshot =
          !initial_snapshot_sent || cursor.0 % FULL_SNAPSHOT_INTERVAL_TICKS == 0;

        if should_send_full_snapshot {
            let payload = TDriver::encode_snapshot(&snapshot)
              .map_err(|e| map_driver_error::<TDriver>(e, "encode snapshot"))?;

            outbox.push(ServerMessage::Snapshot(Snapshot {
                tick,
                cursor,
                last_applied: last_ack,
                entity_ids: Vec::new(),
                payload,
            }));
        } else {
            let base_snapshot = last_sent_snapshot.unwrap_or_else(|| snapshot.clone());
            let delta = TDriver::build_delta(&base_snapshot, &snapshot);
            let payload = TDriver::encode_delta(&delta)
              .map_err(|e| map_driver_error::<TDriver>(e, "encode delta"))?;

            outbox.push(ServerMessage::DeltaSnapshot(DeltaSnapshot {
                tick,
                base: last_ack,
                cursor,
                entity_ids: Vec::new(),
                payload,
            }));
        }

        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState<TDriver::Snapshot>>() {
            state.initial_snapshot_sent = true;
            state.last_sent_cursor = cursor;
            state.last_sent_snapshot = Some(snapshot);
        }

        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.emitted_snapshots = diagnostics.emitted_snapshots.saturating_add(1);
        }
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.last_snapshot_cursor = cursor.0;
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

    let tick = world.resource::<SimulationTick>().copied().unwrap_or_default();

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

        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState<TSnapshot>>()
          && (connection_id.is_none() || state.active_connection == connection_id)
        {
            reset_replication_for_connection(&mut state, active_connection);
        }
    } else if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
        status.connected = false;
        status.phase = SessionPhase::Closed;
        status.last_disconnect = reason.clone();
    }

    if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
        health.connected = false;
        health.close_events = health.close_events.saturating_add(1);
    }
}

pub fn reset_replication_for_connection<TSnapshot>(
    state: &mut SnapshotReplicationState<TSnapshot>,
    connection: Option<ConnectionId>,
) where
  TSnapshot: Clone + PartialEq,
{
    state.active_connection = connection;
    state.initial_snapshot_sent = false;
    state.last_sent_cursor = SnapshotCursor::default();
    state.last_acknowledged_cursor = SnapshotCursor::default();
    state.last_received_tick = SimulationTick::default();
    state.last_sent_snapshot = None;
    state.last_received_snapshot = None;
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

    if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState<TDriver::Snapshot>>() {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(snapshot);
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    let pending_frames = {
        let mut prediction = world.resource_mut::<PredictionState<TDriver::Input>>()?;
        prediction.pending_frames.retain(|frame| frame.tick.0 > tick.0);
        prediction.pending_frames.clone()
    };

    for frame in pending_frames {
        TDriver::apply_input(world, &frame.commands)
          .map_err(|e| map_driver_error::<TDriver>(e, "replay predicted input"))?;
    }

    Ok(corrected)
}

pub fn apply_authoritative_delta<TDriver>(
    world: &mut World,
    tick: SimulationTick,
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

    let base_snapshot = world
      .resource::<SnapshotReplicationState<TDriver::Snapshot>>()
      .ok()
      .and_then(|state| state.last_received_snapshot.clone())
      .ok_or_else(|| anyhow::anyhow!("received delta snapshot without a baseline snapshot"))?;

    let rebuilt_snapshot = TDriver::apply_delta_to_snapshot(&base_snapshot, &delta);

    let corrected = TDriver::apply_snapshot(world, tick, rebuilt_snapshot.clone())
      .map_err(|e| map_driver_error::<TDriver>(e, "apply rebuilt delta snapshot"))?;

    if let Ok(mut tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }

    if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState<TDriver::Snapshot>>() {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(rebuilt_snapshot);
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    let pending_frames = {
        let mut prediction = world.resource_mut::<PredictionState<TDriver::Input>>()?;
        prediction.pending_frames.retain(|frame| frame.tick.0 > tick.0);
        prediction.pending_frames.clone()
    };

    for frame in pending_frames {
        TDriver::apply_input(world, &frame.commands)
          .map_err(|e| map_driver_error::<TDriver>(e, "replay predicted input after delta"))?;
    }

    Ok(corrected)
}
