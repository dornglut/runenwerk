use super::*;
use crate::WorldMut;
use crate::runtime::{WorkQueueDrainer, WorkQueueWriter};
use anyhow::Context;
use ecs::{OwnerRole, WorkQueueEnqueueError, World};
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::{AuthorityRole, SimulationProfileConfig};
use tokio::sync::mpsc::error::TryRecvError;
use world_ops::SyncCursor;

// engine/src/plugins/net/runtime_io.rs

pub fn map_driver_error<TDriver>(error: TDriver::Error, context: &'static str) -> anyhow::Error
where
    TDriver: ReplicationDriver,
{
    anyhow::Error::new(error).context(context)
}

fn enqueue_work_queue_writer_with_backpressure<T: 'static>(
    writer: &mut WorkQueueWriter<T>,
    work_queue_name: &'static str,
    message: T,
) -> Result<(), WorkQueueEnqueueError> {
    let result = writer.enqueue(message);
    if let Err(WorkQueueEnqueueError::Backpressure { capacity, .. }) = &result {
        tracing::warn!(
            work_queue = work_queue_name,
            capacity = *capacity,
            "network queue backpressure; dropping newest message"
        );
    }
    result
}

pub fn network_runtime_receive_system<TDriver>(
    mut world: WorldMut,
    mut client_inbox: WorkQueueWriter<ServerMessage>,
    mut server_inbox: WorkQueueWriter<InboundClientMessage>,
) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    let Some(mut handle) = world.remove_resource::<NetworkRuntimeHandle>() else {
        if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
            inbound.clear();
        }
        return Ok(());
    };

    if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
        inbound.clear();
    }

    loop {
        let event = match handle.try_recv() {
            Ok(Some(event)) => event,
            Ok(None) => break,
            Err(TryRecvError::Disconnected) => {
                update_connection_closed::<TDriver::Snapshot>(&mut world, None, None);
                break;
            }
            Err(TryRecvError::Empty) => break,
        };

        match event {
            SessionRuntimeEvent::Connected { connection_id } => {
                if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = true;
                    status.connection_id = connection_id;
                }
                if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = true;
                }
                if let Some(connection_id) = connection_id {
                    let _ =
                        ensure_owner_for_connection(&mut world, connection_id, OwnerRole::Active);
                }
            }
            SessionRuntimeEvent::ClientMessage {
                connection_id,
                message,
            } => {
                if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_client(connection_id, message.clone());
                }
                if let Err(error) = enqueue_work_queue_writer_with_backpressure(
                    &mut server_inbox,
                    "NetworkServerInbox",
                    InboundClientMessage {
                        connection_id,
                        message,
                    },
                ) {
                    tracing::warn!(error = ?error, "failed to enqueue server inbox message");
                }
            }
            SessionRuntimeEvent::ServerMessage(message) => {
                if let Ok(inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_server(message.clone());
                }
                if let Err(error) = enqueue_work_queue_writer_with_backpressure(
                    &mut client_inbox,
                    "NetworkClientInbox",
                    message,
                ) {
                    tracing::warn!(error = ?error, "failed to enqueue client inbox message");
                }
            }
            SessionRuntimeEvent::Phase(phase) => {
                if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = phase;
                }
            }
            SessionRuntimeEvent::Reconnecting { attempt } => {
                if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = false;
                    status.reconnect_attempt = Some(attempt);
                    status.phase = SessionPhase::Handshaking;
                }
                if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = false;
                    health.reconnect_events = health.reconnect_events.saturating_add(1);
                }
                if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.reconnect_attempts =
                        diagnostics.reconnect_attempts.saturating_add(1);
                }
            }
            SessionRuntimeEvent::JoinAccepted(join) => {
                let connection = ConnectionId(join.connection_id);
                let _ = ensure_owner_for_connection(&mut world, connection, OwnerRole::Active);
                let authority = world
                    .resource::<SimulationProfileConfig>()
                    .map(|config| config.authority)
                    .unwrap_or(AuthorityRole::Local);

                tracing::info!(
                    ?authority,
                    connection_id = join.connection_id,
                    tick_rate_hz = join.tick_rate_hz,
                    "network join accepted"
                );

                if matches!(authority, AuthorityRole::Server)
                    && let Ok(session) = world.resource_mut::<ServerSessionState>()
                {
                    session.phase = SessionPhase::Active;
                    session.active_connection = Some(connection);
                    session.active_connections.insert(connection);
                    session.last_disconnect = None;
                    session.last_join_state = Some(join.join_state.clone());

                    if let Ok(replication) =
                        world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()
                    {
                        replication
                            .checkpoints
                            .entry(connection)
                            .or_default()
                            .needs_full_resync = true;
                    }
                    if let Ok(streaming_interest) =
                        world.resource_mut::<NetStreamingStateResource>()
                    {
                        streaming_interest.mark_needs_full_resync(connection);
                    }
                }

                if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer)
                    && let Ok(session) = world.resource_mut::<ClientSessionState>()
                {
                    observe_server_message(session, &ServerMessage::JoinAccepted(join.clone()));
                }

                if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = SessionPhase::Active;
                    status.connected = true;
                    status.connection_id = Some(ConnectionId(join.connection_id));
                    status.last_disconnect = None;
                    status.reconnect_attempt = None;
                }

                if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.accepted_connections =
                        diagnostics.accepted_connections.saturating_add(1);
                }

                if let Ok(admission) = world.resource_mut::<NetworkAdmissionState>() {
                    admission.authoritative_join = Some(join.join_state.clone());
                }

                apply_session_runtime_join_state(&mut world, &join.join_state);
            }
            SessionRuntimeEvent::JoinRejected(reason) => {
                tracing::warn!(?reason, "network join rejected");

                let authority = world
                    .resource::<SimulationProfileConfig>()
                    .map(|config| config.authority)
                    .unwrap_or(AuthorityRole::Local);

                if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
                    if let Ok(session) = world.resource_mut::<ClientSessionState>() {
                        observe_server_message(
                            session,
                            &ServerMessage::JoinRejected(engine_net::JoinRejected {
                                reason: reason.clone(),
                            }),
                        );
                    }

                    if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                        status.phase = SessionPhase::Rejected(reason.clone());
                        status.last_disconnect = Some(reason.clone());
                    }

                    clear_session_runtime_state(&mut world);
                }

                if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.rejected_connections =
                        diagnostics.rejected_connections.saturating_add(1);
                }
            }
            SessionRuntimeEvent::RttUpdated { millis } => {
                if let Ok(metrics) = world.resource_mut::<RoundTripMetrics>() {
                    metrics.last_rtt_millis = Some(millis);
                    metrics.samples = metrics.samples.saturating_add(1);
                }
            }
            SessionRuntimeEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                tracing::warn!(?connection_id, ?reason, "network connection closed");
                update_connection_closed::<TDriver::Snapshot>(&mut world, connection_id, reason);
            }
            SessionRuntimeEvent::Error { message } => {
                if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.last_error = Some(message);
                }
                if let Ok(health) = world.resource_mut::<ConnectionHealth>() {
                    health.error_events = health.error_events.saturating_add(1);
                }
            }
        }
    }

    world.insert_resource(handle);
    sync_net_diagnostics_view(&mut world);
    Ok(())
}

pub fn client_receive_system<TDriver>(
    mut world: WorldMut,
    mut client_inbox: WorkQueueDrainer<ServerMessage>,
    mut client_outbox: WorkQueueWriter<ClientMessage>,
) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let messages = client_inbox.drain();
    if messages.is_empty() {
        return Ok(());
    }

    let len = messages.len();
    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_server_messages_last_frame = len;
    }

    for message in messages {
        let previous_phase = world
            .resource::<ClientSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();

        if let Ok(session) = world.resource_mut::<ClientSessionState>() {
            observe_server_message(session, &message);
            let phase = session.phase.clone();
            let connection_id = session.connection_id;
            let last_disconnect = session.last_disconnect.clone();

            if let Ok(status) = world.resource_mut::<NetworkSessionStatus>() {
                status.phase = phase.clone();
                status.connection_id = connection_id;
                status.last_disconnect = last_disconnect;
                status.connected = matches!(phase, SessionPhase::Active);
            }
        }

        match message {
            ServerMessage::JoinAccepted(join) => {
                let _ = ensure_owner_for_connection(
                    &mut world,
                    ConnectionId(join.connection_id),
                    OwnerRole::Active,
                );
                if let Ok(admission) = world.resource_mut::<NetworkAdmissionState>() {
                    admission.authoritative_join = Some(join.join_state.clone());
                }
                apply_session_runtime_join_state(&mut world, &join.join_state);
            }
            ServerMessage::Snapshot(snapshot) => {
                let result = apply_authoritative_snapshot::<TDriver>(
                    &mut world,
                    snapshot.tick,
                    snapshot.cursor,
                    None,
                    &snapshot.payload,
                )
                .with_context(|| {
                    format!(
                        "failed applying snapshot tick={} cursor={} payload_len={}",
                        snapshot.tick.0,
                        snapshot.cursor.0,
                        snapshot.payload.len()
                    )
                });

                match result {
                    Ok(corrected) => {
                        if let Err(error) = enqueue_work_queue_writer_with_backpressure(
                            &mut client_outbox,
                            "NetworkClientOutbox",
                            ClientMessage::Ack(Ack {
                                cursor: snapshot.cursor,
                                last_received_tick: snapshot.tick,
                            }),
                        ) {
                            tracing::warn!(error = ?error, "failed to enqueue snapshot ack");
                        }
                        if corrected
                            && let Ok(diagnostics) = world.resource_mut::<PredictionDiagnostics>()
                        {
                            diagnostics.corrected = diagnostics.corrected.saturating_add(1);
                        }
                    }
                    Err(error) => {
                        tracing::warn!(
                            error = %format!("{error:#}"),
                            "network snapshot apply failed"
                        );
                    }
                }
            }
            ServerMessage::DeltaSnapshot(snapshot) => {
                let result = apply_authoritative_delta::<TDriver>(
                    &mut world,
                    snapshot.tick,
                    snapshot.base,
                    snapshot.cursor,
                    &snapshot.payload,
                )
                .with_context(|| {
                    format!(
                        "failed applying delta snapshot tick={} cursor={} payload_len={}",
                        snapshot.tick.0,
                        snapshot.cursor.0,
                        snapshot.payload.len()
                    )
                });

                match result {
                    Ok(corrected) => {
                        if let Err(error) = enqueue_work_queue_writer_with_backpressure(
                            &mut client_outbox,
                            "NetworkClientOutbox",
                            ClientMessage::Ack(Ack {
                                cursor: snapshot.cursor,
                                last_received_tick: snapshot.tick,
                            }),
                        ) {
                            tracing::warn!(
                                error = ?error,
                                "failed to enqueue delta snapshot ack"
                            );
                        }
                        if corrected
                            && let Ok(diagnostics) = world.resource_mut::<PredictionDiagnostics>()
                        {
                            diagnostics.corrected = diagnostics.corrected.saturating_add(1);
                        }
                    }
                    Err(error) => {
                        tracing::warn!(
                            error = %format!("{error:#}"),
                            "network delta snapshot apply failed"
                        );
                    }
                }
            }
            _ => {}
        }

        let phase = world
            .resource::<ClientSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();

        if !matches!(previous_phase, SessionPhase::Active)
            && matches!(phase, SessionPhase::Active)
            && let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics.accepted_connections.saturating_add(1);
        }

        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    sync_net_diagnostics_view(&mut world);
    Ok(())
}

pub fn server_receive_system<TDriver>(
    mut world: WorldMut,
    mut server_inbox: WorkQueueDrainer<InboundClientMessage>,
    mut server_outbox: WorkQueueWriter<OutboundServerMessage>,
) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    let config = world
        .resource::<ServerSessionConfig>()
        .cloned()
        .unwrap_or_default();

    if let Ok(session) = world.resource_mut::<ServerSessionState>()
        && session.config != config
    {
        session.config = config;
    }

    let messages = server_inbox.drain();
    if messages.is_empty() {
        return Ok(());
    }

    let len = messages.len();
    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_client_messages_last_frame = len;
    }

    for incoming in messages {
        let connection_id = incoming.connection_id;
        let message = incoming.message;

        if let ClientMessage::Ack(ack) = &message
            && let Some(connection_id) = connection_id
        {
            let ack_outcome = if let Ok(state) =
                world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()
            {
                let baseline_available = state
                    .snapshot_history_per_connection
                    .get(&connection_id)
                    .is_some_and(|history| history.contains_key(&ack.cursor));
                let checkpoint = state.checkpoints.entry(connection_id).or_default();
                checkpoint.mark_snapshot_acknowledged(ack.cursor, baseline_available)
            } else {
                SnapshotAckOutcome::Rejected {
                    cursor: ack.cursor,
                    reason: SnapshotAckRejection::UnsentCursor,
                }
            };
            match ack_outcome {
                SnapshotAckOutcome::Accepted { .. } => {
                    if let Ok(streaming_interest) =
                        world.resource_mut::<NetStreamingStateResource>()
                    {
                        streaming_interest
                            .mark_snapshot_acknowledged(connection_id, SyncCursor(ack.cursor.0));
                    }
                    if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                        diagnostics.acked = diagnostics.acked.saturating_add(1);
                    }
                }
                SnapshotAckOutcome::Rejected { .. } => {
                    if let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
                        diagnostics.rejected_acks = diagnostics.rejected_acks.saturating_add(1);
                    }
                }
            }
        }

        if let ClientMessage::InputFrame(frame) = &message
            && let Some(connection_id) = connection_id
        {
            let decoded = TDriver::decode_input(&frame.payload)
                .map_err(|e| map_driver_error::<TDriver>(e, "decode remote input"))?;
            let controller =
                ensure_owner_for_connection(&mut world, connection_id, OwnerRole::Active);

            let mut lagged = 0u64;
            let current_tick = world.current_buffer_tick();
            for command in decoded {
                if frame.tick.0 < current_tick {
                    lagged = lagged.saturating_add(1);
                    continue;
                }

                if let Err(error) = world.push_buffer_message_for_tick::<TDriver::Input>(
                    frame.tick.0,
                    owner_tick_buffer_provenance(controller),
                    command,
                ) {
                    tracing::warn!(?error, "failed to enqueue remote input into tick buffer");
                }
            }
            if lagged > 0
                && let Ok(diagnostics) = world.resource_mut::<ReplicationDiagnostics>()
            {
                diagnostics.lagged = diagnostics.lagged.saturating_add(lagged);
            }
        }

        let (previous_phase, previous_connection_count) = world
            .resource::<ServerSessionState>()
            .map(|session| (session.phase.clone(), session.active_connections.len()))
            .unwrap_or((SessionPhase::Idle, 0));

        let responses = {
            let session = world.resource_mut::<ServerSessionState>()?;
            handle_client_message(session, &message)
        };

        for response in responses {
            let outbound = if let Some(connection_id) = connection_id {
                OutboundServerMessage::ToConnection {
                    connection_id,
                    message: response,
                }
            } else {
                OutboundServerMessage::Broadcast(response)
            };
            let enqueue_result = enqueue_work_queue_writer_with_backpressure(
                &mut server_outbox,
                "NetworkServerOutbox",
                outbound,
            );
            if let Err(error) = enqueue_result {
                tracing::warn!(error = ?error, "failed to enqueue server response");
            };
        }

        let (phase, connection_count) = world
            .resource::<ServerSessionState>()
            .map(|session| (session.phase.clone(), session.active_connections.len()))
            .unwrap_or_default();

        let latest_join_state = world
            .resource::<ServerSessionState>()
            .ok()
            .and_then(|session| session.last_join_state.clone());

        if let Ok(admission) = world.resource_mut::<NetworkAdmissionState>() {
            admission.authoritative_join = latest_join_state.clone();
        }

        if let Some(join_state) = latest_join_state.as_ref() {
            apply_session_runtime_join_state(&mut world, join_state);
        }

        let session_state = world.resource::<ServerSessionState>().ok().map(|session| {
            (
                session.phase.clone(),
                session.active_connection,
                !session.active_connections.is_empty(),
                session.last_disconnect.clone(),
            )
        });

        if let Some((phase, connection_id, has_active_connections, last_disconnect)) = session_state
            && let Ok(status) = world.resource_mut::<NetworkSessionStatus>()
        {
            status.phase = phase.clone();
            status.connection_id = connection_id;
            status.last_disconnect = last_disconnect;
            status.connected = has_active_connections;
        }

        if connection_count > previous_connection_count
            && let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics
                .accepted_connections
                .saturating_add((connection_count - previous_connection_count) as u64);
        }

        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    sync_net_diagnostics_view(&mut world);
    Ok(())
}

fn apply_session_runtime_join_state(world: &mut World, join_state: &AuthoritativeJoinState) {
    if let Ok(session) = world.resource_mut::<NetSessionView>() {
        session.apply_authoritative_join(join_state);
    }
}

fn clear_session_runtime_state(world: &mut World) {
    if let Ok(session) = world.resource_mut::<NetSessionView>() {
        session.clear();
    }
}

fn sync_net_diagnostics_view(world: &mut World) {
    let status = world.resource::<NetworkSessionStatus>().ok().cloned();
    let health = world.resource::<ConnectionHealth>().ok().cloned();
    let round_trip = world.resource::<RoundTripMetrics>().ok().copied();
    let network = world.resource::<NetworkDiagnostics>().ok().copied();
    let replication = world.resource::<ReplicationDiagnostics>().ok().copied();
    let prediction = world.resource::<PredictionDiagnostics>().ok().copied();

    if let Ok(view) = world.resource_mut::<NetDiagnosticsView>() {
        if let Some(status) = status {
            view.connected = status.connected;
            view.connection_id = status.connection_id;
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

pub fn client_flush_system(
    mut world: WorldMut,
    mut client_outbox: WorkQueueDrainer<ClientMessage>,
) -> anyhow::Result<()> {
    let messages = client_outbox.drain();
    if messages.is_empty() {
        return Ok(());
    }

    let len = messages.len();
    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_client_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }

    if let Ok(queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_client(message.clone());
        }
    }

    if let Ok(handle) = world.resource::<NetworkRuntimeHandle>() {
        let mut dropped = 0usize;
        for message in &messages {
            if handle
                .send(SessionRuntimeCommand::Client(message.clone()))
                .is_err()
            {
                dropped = dropped.saturating_add(1);
            }
        }

        if dropped > 0 {
            tracing::warn!(
                dropped,
                "network client flush dropped commands because runtime command channel is full"
            );
        }
    }

    Ok(())
}

pub fn server_flush_system(
    mut world: WorldMut,
    mut server_outbox: WorkQueueDrainer<OutboundServerMessage>,
) -> anyhow::Result<()> {
    let messages = server_outbox.drain();
    if messages.is_empty() {
        return Ok(());
    }

    let len = messages.len();
    if let Ok(diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_server_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }

    if let Ok(queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_server(message.clone());
        }
    }

    if let Ok(handle) = world.resource::<NetworkRuntimeHandle>() {
        let mut dropped = 0usize;
        for message in &messages {
            let command = match message {
                OutboundServerMessage::ToConnection {
                    connection_id,
                    message,
                } => SessionRuntimeCommand::ServerToConnection {
                    connection_id: *connection_id,
                    message: message.clone(),
                },
                OutboundServerMessage::Broadcast(message) => {
                    SessionRuntimeCommand::ServerBroadcast(message.clone())
                }
            };
            if handle.send(command).is_err() {
                dropped = dropped.saturating_add(1);
            }
        }

        if dropped > 0 {
            tracing::warn!(
                dropped,
                "network server flush dropped commands because runtime command channel is full"
            );
        }
    }

    Ok(())
}
