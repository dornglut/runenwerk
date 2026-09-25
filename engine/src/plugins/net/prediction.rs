use super::*;
use crate::WorldMut;
use anyhow::Context;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::{AuthorityRole, SimulationProfileConfig, SimulationTick};
use runen_ecs::World;
use runen_net::identity::{ConnectionHandle, SimulationTick as RunenNetSimulationTick};
use runen_net::input::{
    PredictionInputOutcome, PredictionInvalidationReason, PredictionLineage,
    PredictionReconciliationError, PredictionReconciliationOutcome,
    PredictionState as RunenNetPredictionState,
};
use runen_net::replication::ClientReplicationSet;
use world_ops::SyncCursor;

// engine/src/plugins/net/prediction.rs

#[derive(Debug, runen_ecs::Resource)]
pub(crate) struct ClientPredictionIntegration {
    semantic: PredictionLineage<Vec<u8>>,
}

impl ClientPredictionIntegration {
    fn new(
        lineage: runen_net::replication::ReplicationLineageKey,
        policy: ClientPredictionPolicy,
    ) -> Self {
        Self {
            semantic: PredictionLineage::new(lineage, policy.limits()),
        }
    }

    pub(crate) fn require_connection_replacement<S>(
        &mut self,
        replication: &mut ClientReplicationSet<S>,
    ) -> Result<(), runen_net::replication::ClientSetError> {
        self.semantic
            .require_connection_replacement_full(replication)
    }
}

pub(crate) fn configure_client_prediction(
    app: &mut crate::App,
    prediction_policy: Option<ClientPredictionPolicy>,
    replication_policy: Option<ClientReplicationPolicy>,
) {
    let Some(prediction_policy) = prediction_policy else {
        return;
    };
    let replication_policy = replication_policy
        .expect("tracked client prediction requires explicit ClientReplicationPolicy");
    app.insert_resource(ClientPredictionIntegration::new(
        replication_policy.lineage(),
        prediction_policy,
    ));
}

pub fn client_prediction_state(world: &World) -> Option<RunenNetPredictionState> {
    world
        .resource::<ClientPredictionIntegration>()
        .ok()
        .map(|i| i.semantic.state())
}

pub fn client_prediction_pending_count(world: &World) -> Option<usize> {
    world
        .resource::<ClientPredictionIntegration>()
        .ok()
        .map(|i| i.semantic.pending_count())
}

pub fn client_prediction_pending_bytes(world: &World) -> Option<usize> {
    world
        .resource::<ClientPredictionIntegration>()
        .ok()
        .map(|i| i.semantic.pending_bytes())
}

pub fn client_prediction_connection_lost(world: &mut World) -> anyhow::Result<()> {
    world
        .resource_mut::<ClientPredictionIntegration>()
        .context("client prediction requires explicit ClientPredictionPolicy")?
        .semantic
        .connection_lost();
    Ok(())
}

pub fn client_prediction_participant_membership_ended(world: &mut World) -> anyhow::Result<()> {
    world
        .resource_mut::<ClientPredictionIntegration>()
        .context("client prediction requires explicit ClientPredictionPolicy")?
        .semantic
        .participant_membership_ended();
    Ok(())
}

pub fn client_prediction_session_closed(world: &mut World) -> anyhow::Result<()> {
    world
        .resource_mut::<ClientPredictionIntegration>()
        .context("client prediction requires explicit ClientPredictionPolicy")?
        .semantic
        .session_closed();
    Ok(())
}

fn admit_client_prediction(
    world: &mut World,
    tick: SimulationTick,
    payload: &[u8],
) -> Option<PredictionInputOutcome> {
    world
        .resource_mut::<ClientPredictionIntegration>()
        .ok()
        .map(|i| {
            i.semantic.admit_local(
                RunenNetSimulationTick::new(tick.0),
                &payload.to_vec(),
                payload.len(),
            )
        })
}

pub(crate) fn mark_client_prediction_local_application_failed(world: &mut World) {
    if let Ok(i) = world.resource_mut::<ClientPredictionIntegration>() {
        i.semantic.local_application_failed();
    }
}

pub(crate) fn observe_client_prediction<TDriver>(
    world: &mut World,
    replication: &ClientReplicationSet<ClientReplicatedStateProduct>,
) -> anyhow::Result<bool>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq,
{
    let Some(mut integration) = world.remove_resource::<ClientPredictionIntegration>() else {
        return Ok(false);
    };
    let mut replayed_commands = 0u64;
    let observation = integration
        .semantic
        .observe_replication(replication, |target, payload| {
            let commands = TDriver::decode_input(payload)
                .map_err(anyhow::Error::new)
                .context("decode RunenNet retained prediction batch")?;
            TDriver::apply_input(world, SimulationTick(target.get()), &commands)
                .map_err(anyhow::Error::new)
                .context("replay RunenNet retained prediction batch")?;
            replayed_commands = replayed_commands.saturating_add(commands.len() as u64);
            Ok::<_, anyhow::Error>(())
        });
    world.insert_resource(integration);
    match observation {
        Ok(PredictionReconciliationOutcome::ReconciledReplay { .. }) => {
            if replayed_commands > 0
                && let Ok(d) = world.resource_mut::<PredictionDiagnostics>()
            {
                d.replayed = d.replayed.saturating_add(replayed_commands);
            }
            Ok(false)
        }
        Ok(_) => Ok(false),
        Err(PredictionReconciliationError::ReplayFailed { source, .. }) => {
            tracing::warn!(error=%format!("{source:#}"), "client prediction replay failed; authoritative host restoration required");
            Ok(true)
        }
        Err(error) => Err(anyhow::anyhow!(
            "RunenNet client prediction reconciliation failed: {error:?}"
        )),
    }
}

pub(crate) fn confirm_client_prediction_host_restored_if_needed(
    world: &mut World,
    replication: &ClientReplicationSet<ClientReplicatedStateProduct>,
) -> anyhow::Result<()> {
    let needs_restoration_confirmation = world
        .resource::<ClientPredictionIntegration>()
        .ok()
        .is_some_and(|integration| {
            matches!(
                integration.semantic.state(),
                RunenNetPredictionState::Invalidated {
                    reason: PredictionInvalidationReason::LocalApplicationFailure
                        | PredictionInvalidationReason::ReplayFailure,
                    ..
                }
            )
        });
    if needs_restoration_confirmation {
        confirm_client_prediction_host_restored(world, replication)?;
    }
    Ok(())
}

pub(crate) fn confirm_client_prediction_host_restored(
    world: &mut World,
    replication: &ClientReplicationSet<ClientReplicatedStateProduct>,
) -> anyhow::Result<()> {
    let Some(mut integration) = world.remove_resource::<ClientPredictionIntegration>() else {
        return Ok(());
    };
    let result = integration
        .semantic
        .confirm_host_restored_after_prediction_failure(replication)
        .map_err(|e| {
            anyhow::anyhow!("RunenNet prediction host-restoration confirmation failed: {e:?}")
        });
    world.insert_resource(integration);
    result.map(|_| ())
}

const FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 30;
const MAX_SERVER_SNAPSHOT_HISTORY: usize = 256;

fn active_connections(world: &World) -> Vec<ConnectionHandle> {
    let mut connections = world
        .resource::<RunenNetSessionProjection>()
        .map(|projection| projection.active_connections().collect::<Vec<_>>())
        .unwrap_or_default();
    connections.sort_by_key(|connection| connection.get());
    connections
}

pub fn replication_step_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    if matches!(authority, AuthorityRole::Server | AuthorityRole::Peer) {
        world
            .resource::<NetworkServerOutbox>()
            .context("NetworkServerOutbox should be installed by the server network role")?;
    }

    if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    if !matches!(authority, AuthorityRole::Server | AuthorityRole::Peer) {
        let cursor = world
            .resource::<SnapshotCursor>()
            .map(|cursor| cursor.0)
            .unwrap_or(0);

        if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.last_snapshot_cursor = cursor;
        }
        return Ok(());
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let cursor = {
        let cursor = world.resource_mut::<SnapshotCursor>()?;
        cursor.0 = cursor.0.saturating_add(1);
        *cursor
    };

    let active_connections = active_connections(&world);

    let mut outbound = Vec::<(OutboundServerMessage, ConnectionHandle, bool)>::new();
    if !active_connections.is_empty() {
        let mut snapshots_for_connections = Vec::<(ConnectionHandle, TDriver::Snapshot)>::new();
        for connection in &active_connections {
            let captured_snapshot = TDriver::capture_snapshot_for_connection(&world, *connection)
                .map_err(|error| {
                map_driver_error::<TDriver>(error, "capture snapshot for connection")
            })?;
            if let Some(snapshot) = captured_snapshot {
                snapshots_for_connections.push((*connection, snapshot));
            }
        }

        let state = world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()?;
        state.latest_tick = tick;
        state
            .checkpoints
            .retain(|connection, _| active_connections.contains(connection));
        state
            .snapshot_history_per_connection
            .retain(|connection, _| active_connections.contains(connection));
        state
            .latest_snapshot_per_connection
            .retain(|connection, _| active_connections.contains(connection));

        let mut first_snapshot_for_tick: Option<TDriver::Snapshot> = None;

        for (connection, snapshot) in snapshots_for_connections {
            if first_snapshot_for_tick.is_none() {
                first_snapshot_for_tick = Some(snapshot.clone());
            }

            state
                .latest_snapshot_per_connection
                .insert(connection, snapshot.clone());
            state
                .snapshot_history_per_connection
                .entry(connection)
                .or_default()
                .insert(cursor, snapshot.clone());
            prune_snapshot_history_for_connection(state, connection);

            let (last_ack_cursor, needs_full_resync) = {
                let checkpoint = state.checkpoints.entry(connection).or_default();
                (checkpoint.last_ack_cursor, checkpoint.needs_full_resync)
            };

            let scheduled_full = cursor.0 % FULL_SNAPSHOT_INTERVAL_TICKS == 0;
            let mut send_full = needs_full_resync || scheduled_full || last_ack_cursor.0 == 0;

            let message = if send_full {
                let payload = TDriver::encode_snapshot(&snapshot)
                    .map_err(|error| map_driver_error::<TDriver>(error, "encode snapshot"))?;
                ServerMessage::Snapshot(Snapshot {
                    tick,
                    cursor,
                    last_applied: last_ack_cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            } else if let Some(base_snapshot) = state
                .snapshot_history_per_connection
                .get(&connection)
                .and_then(|history| history.get(&last_ack_cursor))
            {
                let delta = TDriver::build_delta(base_snapshot, &snapshot);
                let payload = TDriver::encode_delta(&delta)
                    .map_err(|error| map_driver_error::<TDriver>(error, "encode delta"))?;
                ServerMessage::DeltaSnapshot(DeltaSnapshot {
                    tick,
                    base: last_ack_cursor,
                    cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            } else {
                send_full = true;
                let payload = TDriver::encode_snapshot(&snapshot).map_err(|error| {
                    map_driver_error::<TDriver>(error, "encode fallback snapshot")
                })?;
                ServerMessage::Snapshot(Snapshot {
                    tick,
                    cursor,
                    last_applied: last_ack_cursor,
                    entity_ids: Vec::new(),
                    payload,
                })
            };

            outbound.push((
                OutboundServerMessage::ToConnection {
                    connection,
                    message,
                },
                connection,
                send_full,
            ));
        }

        if let Some(snapshot) = first_snapshot_for_tick {
            state.latest_snapshot = Some(snapshot.clone());
            state.snapshot_history.insert(cursor, snapshot);
            prune_snapshot_history(state);
        }
    }

    let mut accepted_outbound = Vec::<(ConnectionHandle, bool)>::new();
    for (message, connection, sent_full_snapshot) in outbound {
        match enqueue_server_outbox(&mut world, message) {
            Ok(()) => accepted_outbound.push((connection, sent_full_snapshot)),
            Err(NetworkPendingEnqueueError::Unavailable { endpoint, .. }) => {
                anyhow::bail!("{endpoint} should be installed by NetPlugin");
            }
            Err(NetworkPendingEnqueueError::Backpressure { capacity, .. }) => {
                tracing::warn!(
                    capacity,
                    "failed to enqueue replication server outbox message"
                );
            }
        }
    }

    if !accepted_outbound.is_empty() {
        let state = world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()?;
        for (connection, sent_full_snapshot) in &accepted_outbound {
            state
                .checkpoints
                .entry(*connection)
                .or_default()
                .mark_snapshot_sent(cursor, tick, *sent_full_snapshot);
        }
    }

    if !accepted_outbound.is_empty()
        && let Ok(streaming_state) = world.resource_mut::<NetStreamingStateResource>()
    {
        for (connection, sent_full_snapshot) in &accepted_outbound {
            streaming_state.mark_snapshot_sent(
                *connection,
                SyncCursor(cursor.0),
                *sent_full_snapshot,
            );
        }
    }

    if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.last_snapshot_cursor = cursor.0;
        diagnostics.emitted_snapshots = diagnostics
            .emitted_snapshots
            .saturating_add(accepted_outbound.len() as u64);
    }

    Ok(())
}

pub fn prediction_step_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    world
        .resource::<NetworkInputStaging<TDriver::Input>>()
        .context("NetworkInputStaging should be installed by NetPlugin")?;
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|c| c.authority)
        .unwrap_or(AuthorityRole::Local);
    if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
        world
            .resource::<NetworkClientOutbox>()
            .context("NetworkClientOutbox should be installed by the client network role")?;
    }
    if let Ok(d) = world.resource_mut::<PredictionDiagnostics>() {
        d.fixed_steps_observed = d.fixed_steps_observed.saturating_add(1);
    }
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let mut authority_inputs = drain_authority_input_for_tick::<TDriver>(&mut world, tick)?;
    let commands = TDriver::take_local_input(&mut world)
        .map_err(|e| map_driver_error::<TDriver>(e, "take local input"))?;
    let mut staged_commands = Vec::with_capacity(commands.len());
    if !commands.is_empty() {
        let staging = world.resource_mut::<NetworkInputStaging<TDriver::Input>>()?;
        for command in commands {
            match staging.stage(tick, command.clone()) {
                Ok(()) => staged_commands.push(command),
                Err(NetworkInputStageError::Backpressure { capacity, .. }) => {
                    tracing::warn!(
                        capacity,
                        tick = tick.0,
                        "network input staging backpressure; rejecting local input"
                    );
                }
            }
        }
    }

    if matches!(authority, AuthorityRole::Client) {
        if staged_commands.is_empty() {
            return Ok(());
        }
        let payload = TDriver::encode_input(&staged_commands)
            .map_err(|e| map_driver_error::<TDriver>(e, "encode input"))?;
        let outcome = admit_client_prediction(&mut world, tick, &payload);
        let conflicting = matches!(outcome, Some(PredictionInputOutcome::ConflictingInput));
        if !conflicting {
            match enqueue_client_outbox(
                &mut world,
                ClientMessage::InputFrame(InputFrame {
                    tick,
                    payload: payload.clone(),
                }),
            ) {
                Ok(()) => {}
                Err(NetworkPendingEnqueueError::Unavailable { endpoint, .. }) => {
                    anyhow::bail!("{endpoint} should be installed by NetPlugin")
                }
                Err(NetworkPendingEnqueueError::Backpressure { capacity, .. }) => tracing::warn!(
                    capacity,
                    "failed to enqueue local input frame; admitted prediction remains pending"
                ),
            }
        }
        if matches!(outcome, Some(PredictionInputOutcome::InputAccepted)) {
            if let Err(error) = TDriver::apply_input(&mut world, tick, &staged_commands) {
                mark_client_prediction_local_application_failed(&mut world);
                let original = map_driver_error::<TDriver>(
                    error,
                    "apply newly admitted local prediction batch",
                );
                restore_client_prediction_host_after_failure::<TDriver>(&mut world)
                    .context("restore authoritative host state after local prediction failure")?;
                return Err(original);
            }
            if let Ok(d) = world.resource_mut::<PredictionDiagnostics>() {
                d.commands_applied = d
                    .commands_applied
                    .saturating_add(staged_commands.len() as u64);
            }
        }
        return Ok(());
    }

    if matches!(authority, AuthorityRole::Peer) && !staged_commands.is_empty() {
        let payload = TDriver::encode_input(&staged_commands)
            .map_err(|e| map_driver_error::<TDriver>(e, "encode peer input"))?;
        match enqueue_client_outbox(
            &mut world,
            ClientMessage::InputFrame(InputFrame { tick, payload }),
        ) {
            Ok(()) => {}
            Err(NetworkPendingEnqueueError::Unavailable { endpoint, .. }) => {
                anyhow::bail!("{endpoint} should be installed by NetPlugin")
            }
            Err(NetworkPendingEnqueueError::Backpressure { capacity, .. }) => {
                tracing::warn!(capacity, "failed to enqueue local peer input frame")
            }
        }
    }

    let local = world
        .resource_mut::<NetworkInputStaging<TDriver::Input>>()?
        .drain_tick(tick);
    authority_inputs.extend(local);
    if authority_inputs.is_empty() {
        return Ok(());
    }
    if let Ok(d) = world.resource_mut::<PredictionDiagnostics>() {
        d.commands_applied = d
            .commands_applied
            .saturating_add(authority_inputs.len() as u64);
    }
    TDriver::apply_input(&mut world, tick, &authority_inputs)
        .map_err(|e| map_driver_error::<TDriver>(e, "apply streamed input"))?;
    Ok(())
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
    connection: ConnectionHandle,
) where
    TSnapshot: Clone + PartialEq,
{
    let Some(history) = state.snapshot_history_per_connection.get_mut(&connection) else {
        return;
    };
    while history.len() > MAX_SERVER_SNAPSHOT_HISTORY {
        let Some(oldest_cursor) = history.keys().next().copied() else {
            break;
        };
        history.remove(&oldest_cursor);
    }
}
