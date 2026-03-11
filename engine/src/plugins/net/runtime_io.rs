use super::*;
use crate::{SessionRuntimeState, WorldMut};
use anyhow::Context;
use ecs::World;
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::{AuthorityRole, SimulationProfileConfig};
use tokio::sync::mpsc::error::TryRecvError;

// engine/src/plugins/net/runtime_io.rs

pub fn map_driver_error<TDriver>(error: TDriver::Error, context: &'static str) -> anyhow::Error
where
    TDriver: ReplicationDriver,
{
    anyhow::Error::new(error).context(context)
}

pub fn network_runtime_receive_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    let Some(mut handle) = world.remove_resource::<NetworkRuntimeHandle>() else {
        if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
            inbound.clear();
        }
        return Ok(());
    };

    if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
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
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = true;
                    status.connection_id = connection_id;
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = true;
                }
            }
            SessionRuntimeEvent::ClientMessage {
                connection_id,
                message,
            } => {
                if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_client(connection_id, message.clone());
                }
                if let Ok(mut inbox) = world.resource_mut::<NetworkServerInbox>() {
                    inbox.push_from(connection_id, message);
                }
            }
            SessionRuntimeEvent::ServerMessage(message) => {
                if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_server(message.clone());
                }
                if let Ok(mut inbox) = world.resource_mut::<NetworkClientInbox>() {
                    inbox.push(message);
                }
            }
            SessionRuntimeEvent::Phase(phase) => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = phase;
                }
            }
            SessionRuntimeEvent::Reconnecting { attempt } => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = false;
                    status.reconnect_attempt = Some(attempt);
                    status.phase = SessionPhase::Handshaking;
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = false;
                    health.reconnect_events = health.reconnect_events.saturating_add(1);
                }
                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.reconnect_attempts =
                        diagnostics.reconnect_attempts.saturating_add(1);
                }
            }
            SessionRuntimeEvent::JoinAccepted(join) => {
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
                    && let Ok(mut session) = world.resource_mut::<ServerSessionState>()
                {
                    let connection = ConnectionId(join.connection_id);
                    session.phase = SessionPhase::Active;
                    session.active_connection = Some(connection);
                    session.active_connections.insert(connection);
                    session.last_disconnect = None;
                    session.last_join_state = Some(join.join_state.clone());

                    if let Ok(mut replication) =
                        world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()
                    {
                        replication
                            .checkpoints
                            .entry(connection)
                            .or_default()
                            .needs_full_resync = true;
                    }
                }

                if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer)
                    && let Ok(mut session) = world.resource_mut::<ClientSessionState>()
                {
                    observe_server_message(
                        &mut session,
                        &ServerMessage::JoinAccepted(join.clone()),
                    );
                }

                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = SessionPhase::Active;
                    status.connected = true;
                    status.connection_id = Some(ConnectionId(join.connection_id));
                    status.last_disconnect = None;
                    status.reconnect_attempt = None;
                }

                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.accepted_connections =
                        diagnostics.accepted_connections.saturating_add(1);
                }

                if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
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
                    if let Ok(mut session) = world.resource_mut::<ClientSessionState>() {
                        observe_server_message(
                            &mut session,
                            &ServerMessage::JoinRejected(engine_net::JoinRejected {
                                reason: reason.clone(),
                            }),
                        );
                    }

                    if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                        status.phase = SessionPhase::Rejected(reason.clone());
                        status.last_disconnect = Some(reason.clone());
                    }

                    clear_session_runtime_state(&mut world);
                }

                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.rejected_connections =
                        diagnostics.rejected_connections.saturating_add(1);
                }
            }
            SessionRuntimeEvent::RttUpdated { millis } => {
                if let Ok(mut metrics) = world.resource_mut::<RoundTripMetrics>() {
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
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.last_error = Some(message);
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.error_events = health.error_events.saturating_add(1);
                }
            }
        }
    }

    world.insert_resource(handle);
    Ok(())
}

pub fn client_receive_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    let Some(len) = world
        .resource::<NetworkClientInbox>()
        .ok()
        .map(|inbox| inbox.len())
    else {
        return Ok(());
    };
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_server_messages_last_frame = len;
    }

    let Some(messages) = world
        .resource_mut::<NetworkClientInbox>()
        .ok()
        .map(|mut inbox| inbox.drain())
    else {
        return Ok(());
    };

    for message in messages {
        let previous_phase = world
            .resource::<ClientSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();

        if let Ok(mut session) = world.resource_mut::<ClientSessionState>() {
            observe_server_message(&mut session, &message);
            let phase = session.phase.clone();
            let connection_id = session.connection_id;
            let last_disconnect = session.last_disconnect.clone();
            drop(session);

            if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                status.phase = phase.clone();
                status.connection_id = connection_id;
                status.last_disconnect = last_disconnect;
                status.connected = matches!(phase, SessionPhase::Active);
            }
        }

        match message {
            ServerMessage::JoinAccepted(join) => {
                if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
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
                        if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
                            outbox.push(ClientMessage::Ack(Ack {
                                cursor: snapshot.cursor,
                                last_received_tick: snapshot.tick,
                            }));
                        }
                        if corrected
                            && let Ok(mut diagnostics) =
                                world.resource_mut::<PredictionDiagnostics>()
                        {
                            diagnostics.corrections_applied =
                                diagnostics.corrections_applied.saturating_add(1);
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
                        if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
                            outbox.push(ClientMessage::Ack(Ack {
                                cursor: snapshot.cursor,
                                last_received_tick: snapshot.tick,
                            }));
                        }
                        if corrected
                            && let Ok(mut diagnostics) =
                                world.resource_mut::<PredictionDiagnostics>()
                        {
                            diagnostics.corrections_applied =
                                diagnostics.corrections_applied.saturating_add(1);
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
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics.accepted_connections.saturating_add(1);
        }

        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    Ok(())
}

pub fn server_receive_system<TDriver>(mut world: WorldMut) -> anyhow::Result<()>
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
{
    let config = world
        .resource::<ServerSessionConfig>()
        .map(|resource| resource.clone())
        .unwrap_or_default();

    if let Ok(mut session) = world.resource_mut::<ServerSessionState>()
        && session.config != config
    {
        session.config = config;
    }

    let Some(len) = world
        .resource::<NetworkServerInbox>()
        .ok()
        .map(|inbox| inbox.len())
    else {
        return Ok(());
    };
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_client_messages_last_frame = len;
    }

    let Some(messages) = world
        .resource_mut::<NetworkServerInbox>()
        .ok()
        .map(|mut inbox| inbox.drain())
    else {
        return Ok(());
    };

    for incoming in messages {
        let connection_id = incoming.connection_id;
        let message = incoming.message;

        if let ClientMessage::Ack(ack) = &message
            && let Ok(mut state) =
                world.resource_mut::<ServerSnapshotReplicationState<TDriver::Snapshot>>()
            && let Some(connection_id) = connection_id
        {
            let checkpoint = state.checkpoints.entry(connection_id).or_default();
            checkpoint.last_ack_cursor = ack.cursor;
            checkpoint.needs_full_resync = false;
        }

        if let ClientMessage::InputFrame(frame) = &message
            && let Some(connection_id) = connection_id
        {
            let decoded = TDriver::decode_input(&frame.payload)
                .map_err(|e| map_driver_error::<TDriver>(e, "decode remote input"))?;

            TDriver::receive_remote_input(&mut world, connection_id, frame.tick, decoded)
                .map_err(|e| map_driver_error::<TDriver>(e, "receive remote input"))?;
        }

        let (previous_phase, previous_connection_count) = world
            .resource::<ServerSessionState>()
            .map(|session| (session.phase.clone(), session.active_connections.len()))
            .unwrap_or((SessionPhase::Idle, 0));

        let responses = {
            let mut session = world.resource_mut::<ServerSessionState>()?;
            handle_client_message(&mut session, &message)
        };

        if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
            for response in responses {
                if let Some(connection_id) = connection_id {
                    outbox.push_to(connection_id, response);
                } else {
                    outbox.push_broadcast(response);
                }
            }
        }

        let (phase, connection_count) = world
            .resource::<ServerSessionState>()
            .map(|session| (session.phase.clone(), session.active_connections.len()))
            .unwrap_or_default();

        let latest_join_state = world
            .resource::<ServerSessionState>()
            .ok()
            .and_then(|session| session.last_join_state.clone());

        if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
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
            && let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>()
        {
            status.phase = phase.clone();
            status.connection_id = connection_id;
            status.last_disconnect = last_disconnect;
            status.connected = has_active_connections;
        }

        if connection_count > previous_connection_count
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics
                .accepted_connections
                .saturating_add((connection_count - previous_connection_count) as u64);
        }

        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    Ok(())
}

fn apply_session_runtime_join_state(world: &mut World, join_state: &AuthoritativeJoinState) {
    if let Ok(mut session) = world.resource_mut::<SessionRuntimeState>() {
        session.apply_authoritative_join(join_state);
    }
}

fn clear_session_runtime_state(world: &mut World) {
    if let Ok(mut session) = world.resource_mut::<SessionRuntimeState>() {
        session.clear();
    }
}

pub fn client_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    let Some(len) = world
        .resource::<NetworkClientOutbox>()
        .ok()
        .map(|outbox| outbox.len())
    else {
        return Ok(());
    };

    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_client_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }

    let Some(messages) = world
        .resource_mut::<NetworkClientOutbox>()
        .ok()
        .map(|mut outbox| outbox.drain())
    else {
        return Ok(());
    };

    if let Ok(mut queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_client(message.clone());
        }
    }

    if let Some(handle) = world.resource::<NetworkRuntimeHandle>().ok() {
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

pub fn server_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    let Some(len) = world
        .resource::<NetworkServerOutbox>()
        .ok()
        .map(|outbox| outbox.len())
    else {
        return Ok(());
    };

    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_server_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }

    let Some(messages) = world
        .resource_mut::<NetworkServerOutbox>()
        .ok()
        .map(|mut outbox| outbox.drain())
    else {
        return Ok(());
    };

    if let Ok(mut queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_server(message.clone());
        }
    }

    if let Some(handle) = world.resource::<NetworkRuntimeHandle>().ok() {
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
