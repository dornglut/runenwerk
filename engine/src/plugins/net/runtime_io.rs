use super::*;
use crate::WorldMut;
use anyhow::Context;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use runen_ecs::World;
use runen_net::identity::ConnectionHandle;
use std::collections::HashSet;
use world_ops::SyncCursor;

// engine/src/plugins/net/runtime_io.rs

#[derive(Debug, Default, runen_ecs::Resource)]
struct NetworkSessionDiagnosticsCursor {
    active_connections: HashSet<ConnectionHandle>,
}

pub fn map_driver_error<TDriver>(error: TDriver::Error, context: &'static str) -> anyhow::Error
where
    TDriver: ReplicationDriver,
{
    anyhow::Error::new(error).context(context)
}

pub fn client_receive_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    if world.resource::<NetworkClientInbox>().is_ok() {
        world
            .resource::<NetworkClientOutbox>()
            .context("NetworkClientOutbox should be installed by the client network role")?;
    }

    let messages = drain_client_inbox(&mut world);
    if messages.is_empty() {
        return Ok(());
    }

    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_server_messages_last_frame = messages.len();
    }

    if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
        inbound.clear();
        for message in &messages {
            inbound.push_server(message.clone());
        }
    }

    for message in messages {
        let result = match &message {
            ServerMessage::Snapshot(snapshot) => {
                process_client_full_snapshot::<TDriver>(&mut world, snapshot).with_context(|| {
                    format!(
                        "failed processing snapshot tick={} cursor={} payload_len={}",
                        snapshot.tick.0,
                        snapshot.cursor.0,
                        snapshot.payload.len()
                    )
                })
            }
            ServerMessage::DeltaSnapshot(snapshot) => {
                process_client_delta_snapshot::<TDriver>(&mut world, snapshot).with_context(|| {
                    format!(
                        "failed processing delta snapshot tick={} cursor={} payload_len={}",
                        snapshot.tick.0,
                        snapshot.cursor.0,
                        snapshot.payload.len()
                    )
                })
            }
            ServerMessage::RunEvent(_)
            | ServerMessage::RunResult(_)
            | ServerMessage::TypedPayload(_) => continue,
        };

        match result {
            Ok(processed) => {
                if let Some(ack) = processed.acknowledgement {
                    match enqueue_client_outbox(&mut world, ClientMessage::Ack(ack)) {
                        Ok(()) => {}
                        Err(NetworkPendingEnqueueError::Unavailable { endpoint, .. }) => {
                            anyhow::bail!("{endpoint} should be installed by NetPlugin");
                        }
                        Err(NetworkPendingEnqueueError::Backpressure { capacity, .. }) => {
                            tracing::warn!(capacity, "failed to enqueue client replication ack");
                        }
                    }
                }
                tracing::trace!(
                    outcome = ?processed.outcome,
                    corrected = processed.corrected,
                    "processed RunenNet client replication message"
                );
            }
            Err(error) => {
                tracing::warn!(
                    error = %format!("{error:#}"),
                    "client replication processing failed"
                );
            }
        }
    }

    sync_net_diagnostics_view(&mut world);
    Ok(())
}

fn connection_is_admitted(world: &World, connection: ConnectionHandle) -> bool {
    world
        .resource::<RunenNetSessionProjection>()
        .ok()
        .and_then(|projection| projection.participant_for_connection(connection))
        .is_some()
}

pub fn server_receive_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    if world.resource::<NetworkServerInbox>().is_ok() {
        world
            .resource::<NetworkInputStaging<TDriver::Input>>()
            .context("NetworkInputStaging should be installed by NetPlugin")?;
    }

    let messages = drain_server_inbox(&mut world);
    if messages.is_empty() {
        return Ok(());
    }

    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_client_messages_last_frame = messages.len();
    }

    if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
        inbound.clear();
        for incoming in &messages {
            inbound.push_client(incoming.connection, incoming.message.clone());
        }
    }

    for incoming in messages {
        let connection = incoming.connection;
        let message = incoming.message;

        if matches!(message, ClientMessage::Ack(_)) {
            let Some(connection) = connection else {
                tracing::warn!("ignoring replication ACK without a RunenNet connection handle");
                continue;
            };
            if !connection_is_admitted(&world, connection) {
                tracing::warn!(
                    connection = connection.get(),
                    "ignoring replication ACK from a connection not admitted by RunenNet session"
                );
                continue;
            }
        }

        if let ClientMessage::Ack(ack) = &message
            && let Some(connection) = connection
        {
            let ack_outcome =
                acknowledge_authority_replication(&mut world, connection, ack.cursor)?;
            if matches!(ack_outcome, AuthorityAckOutcome::Confirmed) {
                if let Ok(streaming_state) = world.resource_mut::<NetStreamingStateResource>() {
                    streaming_state
                        .mark_snapshot_acknowledged(connection, SyncCursor(ack.cursor.0));
                }
                if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                    diagnostics.acked = diagnostics.acked.saturating_add(1);
                }
            } else if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                diagnostics.rejected_acks = diagnostics.rejected_acks.saturating_add(1);
            }
        }

        if let ClientMessage::InputFrame(frame) = &message {
            let Some(connection) = connection else {
                if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                    diagnostics.unauthorized_inputs =
                        diagnostics.unauthorized_inputs.saturating_add(1);
                }
                tracing::warn!("ignoring authority input without a RunenNet connection handle");
                continue;
            };
            process_authority_input_frame::<TDriver>(&mut world, connection, frame)?;
        }
    }

    sync_net_diagnostics_view(&mut world);
    Ok(())
}

pub fn record_reconnect_attempt(world: &mut World, attempt: u32) {
    if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
        status.reconnect_attempt = Some(attempt);
    }
    if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
        health.reconnect_events = health.reconnect_events.saturating_add(1);
    }
    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.reconnect_attempts = diagnostics.reconnect_attempts.saturating_add(1);
    }
}

pub fn record_network_error(world: &mut World, message: impl Into<String>) {
    if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
        status.last_error = Some(message.into());
    }
    if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
        health.error_events = health.error_events.saturating_add(1);
    }
}

/// Reconcile engine-owned status/diagnostic projections from RunenNet-authorized session bindings.
///
/// This function never decides admission or loss. The authoritative input is
/// [`RunenNetSessionProjection`], which is itself updated only after accepted RunenNet Core
/// lifecycle operations. A private cursor remembers the previously observed connection set only
/// to count host-facing transition diagnostics; it is not an identity or routing authority.
pub fn sync_runennet_session_projection(world: &mut World) {
    let mut active_connections = world
        .resource::<RunenNetSessionProjection>()
        .map(|projection| projection.active_connections().collect::<Vec<_>>())
        .unwrap_or_default();
    active_connections.sort_by_key(|connection| connection.get());
    let active_set = active_connections.iter().copied().collect::<HashSet<_>>();
    let previous_set = world
        .resource::<NetworkSessionDiagnosticsCursor>()
        .map(|cursor| cursor.active_connections.clone())
        .unwrap_or_default();

    let newly_active = active_set.difference(&previous_set).count();
    let stale_count = previous_set.difference(&active_set).count();

    if world.has_resource::<NetworkSessionDiagnosticsCursor>() {
        world
            .resource_mut::<NetworkSessionDiagnosticsCursor>()
            .expect("diagnostics cursor should exist after presence check")
            .active_connections = active_set;
    } else {
        world.insert_resource(NetworkSessionDiagnosticsCursor {
            active_connections: active_set,
        });
    }

    let active_connection_count = active_connections.len();
    let connected = active_connection_count > 0;
    if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
        status.connected = connected;
        status.active_connection_count = active_connection_count;
    }
    if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
        health.connected = connected;
        health.close_events = health.close_events.saturating_add(stale_count as u64);
    }
    if newly_active > 0
        && let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>()
    {
        diagnostics.accepted_connections = diagnostics
            .accepted_connections
            .saturating_add(newly_active as u64);
    }
}

pub fn sync_runennet_session_projection_system(mut world: WorldMut) {
    sync_runennet_session_projection(&mut world);
}

fn sync_net_diagnostics_view(world: &mut World) {
    sync_runennet_session_projection(world);

    let status = world.resource::<NetworkSessionStatus>().ok().cloned();
    let health = world.resource::<ConnectionHealth>().ok().cloned();
    let round_trip = world.resource::<RoundTripMetrics>().ok().copied();
    let network = world.resource::<NetworkDiagnostics>().ok().copied();
    let replication = world.resource::<ReplicationDiagnostics>().ok().copied();
    let prediction = world.resource::<PredictionDiagnostics>().ok().copied();

    if let Ok(view) = world.resource_mut::<NetDiagnosticsView>() {
        if let Some(status) = status {
            view.connected = status.connected;
            view.active_connection_count = status.active_connection_count;
        }
        if let Some(health) = health {
            view.close_events = health.close_events;
            view.error_events = health.error_events;
            view.reconnect_events = health.reconnect_events;
        }
        if let Some(round_trip) = round_trip {
            view.last_rtt_millis = round_trip.last_rtt_millis;
        }
        if let Some(network) = network {
            view.accepted_connections = network.accepted_connections;
            view.rejected_connections = network.rejected_connections;
            view.reconnect_attempts = network.reconnect_attempts;
        }
        if let Some(replication) = replication {
            view.emitted_snapshots = replication.emitted_snapshots;
            view.applied_snapshots = replication.applied_snapshots;
            view.acked_snapshots = replication.acked;
            view.lagged_inputs = replication.lagged;
        }
        if let Some(prediction) = prediction {
            view.corrected_predictions = prediction.corrected;
        }
    }
}

pub fn sync_net_diagnostics_view_system(mut world: WorldMut) {
    sync_net_diagnostics_view(&mut world);
}

pub fn client_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    world
        .resource::<NetworkClientOutbox>()
        .context("NetworkClientOutbox should be installed by the client network role")?;
    let messages = drain_client_outbox(&mut world);
    if messages.is_empty() {
        return Ok(());
    }

    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_client_messages_last_frame = messages.len();
        diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
    }

    if let Ok(queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in messages {
            queue.push_client(message);
        }
    }

    Ok(())
}

pub fn server_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    world
        .resource::<NetworkServerOutbox>()
        .context("NetworkServerOutbox should be installed by the server network role")?;
    let messages = drain_server_outbox(&mut world);
    if messages.is_empty() {
        return Ok(());
    }

    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_server_messages_last_frame = messages.len();
        diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
    }

    if let Ok(queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in messages {
            queue.push_server(message);
        }
    }

    Ok(())
}
