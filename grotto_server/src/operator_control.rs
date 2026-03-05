use anyhow::Result;
use cavern_hunt::domain::{ReplicationRuntimeMetrics, ServerNetworkConfigAssetV1};
use engine::plugins::{
    ConnectionHealth, NetworkDiagnostics, NetworkRuntimeHandle, NetworkSessionStatus,
    RoundTripMetrics,
};
use engine::prelude::{App, FrameEnd, PreUpdate, SimulationTick, Update, World, WorldMut};
use engine::state::SessionRuntimeState;
use engine_net::{ConnectionId, ServerMessage, ServerSessionState, SessionRuntimeCommand};
use grotto_online::{
    AxiomOperatorBridgeConfig, AxiomOperatorCommand, AxiomOperatorCommandKind,
    AxiomOperatorCommandResult, AxiomOperatorCommandStatus, AxiomOperatorEvent,
    AxiomOperatorOutboundMessage, AxiomOperatorRuntimeHandle, AxiomOperatorSnapshot,
    spawn_axiom_operator_bridge,
};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::error::TryRecvError;

pub fn try_install_operator_control(
    app: &mut App,
    running: Arc<AtomicBool>,
    config: &ServerNetworkConfigAssetV1,
) -> Result<()> {
    if !config.axiom_operator.enabled {
        return Ok(());
    }

    let runtime_token = config
        .axiom_operator
        .runtime_token
        .clone()
        .filter(|value| !value.trim().is_empty());
    let Some(runtime_token) = runtime_token else {
        eprintln!(
            "Axiom operator bridge is enabled but runtime_token is missing; bridge is disabled"
        );
        return Ok(());
    };
    if config.axiom_operator.ws_url.trim().is_empty() {
        eprintln!("Axiom operator bridge is enabled but ws_url is empty; bridge is disabled");
        return Ok(());
    }

    let handle = spawn_axiom_operator_bridge(AxiomOperatorBridgeConfig {
        ws_url: config.axiom_operator.ws_url.clone(),
        runtime_token,
        server_id: config.server_id.clone(),
        heartbeat_seconds: config.axiom_operator.heartbeat_seconds,
        reconnect_backoff_ms: 500,
        max_buffered_events: config.axiom_operator.max_buffered_events,
    })?;

    app.world_mut().insert_resource(handle);
    app.world_mut()
        .insert_resource(OperatorRuntimeConfig::from_config(config));
    app.world_mut().insert_resource(OperatorControlState {
        dedupe_capacity: config.axiom_operator.max_buffered_events.max(32),
        snapshot_interval_ticks: config.axiom_operator.snapshot_interval_ticks.max(1),
        ..OperatorControlState::default()
    });
    app.world_mut()
        .insert_resource(OperatorObservedState::default());
    app.world_mut()
        .insert_resource(OperatorOutboundQueue::default());
    app.world_mut().insert_resource(ServerRunSignal { running });

    app.add_systems(PreUpdate, operator_receive_commands_system);
    app.add_systems(
        Update,
        (
            operator_shutdown_tick_system,
            operator_emit_runtime_events_system,
            operator_emit_snapshot_system,
        ),
    );
    app.add_systems(FrameEnd, operator_flush_outbound_system);

    Ok(())
}

#[derive(Debug, Clone)]
struct OperatorRuntimeConfig {
    server_id: String,
}

impl OperatorRuntimeConfig {
    fn from_config(config: &ServerNetworkConfigAssetV1) -> Self {
        Self {
            server_id: config.server_id.clone(),
        }
    }
}

#[derive(Default)]
struct OperatorOutboundQueue {
    messages: Vec<AxiomOperatorOutboundMessage>,
}

impl OperatorOutboundQueue {
    fn push(&mut self, message: AxiomOperatorOutboundMessage) {
        self.messages.push(message);
    }

    fn drain(&mut self) -> Vec<AxiomOperatorOutboundMessage> {
        std::mem::take(&mut self.messages)
    }
}

#[derive(Default, Clone)]
struct OperatorControlState {
    recent_command_ids: VecDeque<String>,
    dedupe_capacity: usize,
    drain_mode_enabled: bool,
    pending_shutdown_deadline: Option<Instant>,
    force_snapshot: bool,
    last_snapshot_tick: u64,
    snapshot_interval_ticks: u64,
}

#[derive(Default)]
struct OperatorObservedState {
    accepted_connections: u64,
    rejected_connections: u64,
    close_events: u64,
    reconnect_attempts: u64,
    last_phase: Option<String>,
    last_error: Option<String>,
}

struct ServerRunSignal {
    running: Arc<AtomicBool>,
}

fn operator_receive_commands_system(mut world: WorldMut) -> Result<()> {
    let Some(mut handle) = world.remove_resource::<AxiomOperatorRuntimeHandle>() else {
        return Ok(());
    };
    loop {
        let command = match handle.try_recv_command() {
            Ok(Some(command)) => command,
            Ok(None) | Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        };
        process_operator_command(&mut world, command)?;
    }
    world.insert_resource(handle);
    Ok(())
}

fn process_operator_command(world: &mut World, command: AxiomOperatorCommand) -> Result<()> {
    let server_id = world
        .resource::<OperatorRuntimeConfig>()
        .map(|cfg| cfg.server_id.clone())
        .unwrap_or_else(|_| "srv-local".to_string());
    let command_id = command.command_id.clone();

    if !command.targets_server(&server_id) {
        return Ok(());
    }

    let duplicate = {
        let mut state = world.resource_mut::<OperatorControlState>()?;
        let duplicate = state
            .recent_command_ids
            .iter()
            .any(|id| id == &command.command_id);
        if !duplicate {
            state
                .recent_command_ids
                .push_back(command.command_id.clone());
            while state.recent_command_ids.len() > state.dedupe_capacity {
                let _ = state.recent_command_ids.pop_front();
            }
        }
        duplicate
    };
    if duplicate {
        queue_command_result(
            world,
            command_id,
            AxiomOperatorCommandStatus::Rejected,
            Some("duplicate command_id".to_string()),
        )?;
        return Ok(());
    }

    match command.kind {
        AxiomOperatorCommandKind::SetDrainMode { enabled } => {
            let result =
                send_runtime_command(world, SessionRuntimeCommand::SetDrainMode { enabled });
            match result {
                Ok(()) => {
                    if let Ok(mut state) = world.resource_mut::<OperatorControlState>() {
                        state.drain_mode_enabled = enabled;
                    }
                    queue_event(
                        world,
                        "operator.drain_mode_changed",
                        None,
                        json!({ "enabled": enabled }),
                    )?;
                    queue_command_result(
                        world,
                        command_id,
                        AxiomOperatorCommandStatus::Accepted,
                        None,
                    )?;
                }
                Err(error) => {
                    queue_command_result(
                        world,
                        command_id,
                        AxiomOperatorCommandStatus::Failed,
                        Some(error),
                    )?;
                }
            }
        }
        AxiomOperatorCommandKind::DisconnectConnection {
            connection_id,
            reason,
        } => {
            let result = send_runtime_command(
                world,
                SessionRuntimeCommand::DisconnectConnection {
                    connection_id: ConnectionId(connection_id),
                    reason,
                },
            );
            match result {
                Ok(()) => {
                    queue_event(
                        world,
                        "operator.disconnect_connection",
                        None,
                        json!({ "connection_id": connection_id }),
                    )?;
                    queue_command_result(
                        world,
                        command_id,
                        AxiomOperatorCommandStatus::Accepted,
                        None,
                    )?;
                }
                Err(error) => {
                    queue_command_result(
                        world,
                        command_id,
                        AxiomOperatorCommandStatus::Failed,
                        Some(error),
                    )?;
                }
            }
        }
        AxiomOperatorCommandKind::Shutdown { grace_ms } => {
            let _ =
                send_runtime_command(world, SessionRuntimeCommand::SetDrainMode { enabled: true });
            let _ = send_runtime_command(
                world,
                SessionRuntimeCommand::Server(ServerMessage::Disconnect(
                    engine_net::DisconnectReason::ServerShuttingDown,
                )),
            );
            let grace_duration = grace_ms.unwrap_or(0);
            if grace_duration == 0 {
                execute_shutdown_now(world)?;
            } else if let Ok(mut state) = world.resource_mut::<OperatorControlState>() {
                state.pending_shutdown_deadline =
                    Some(Instant::now() + Duration::from_millis(grace_duration));
            }
            queue_event(
                world,
                "operator.shutdown_requested",
                Some(command_id.clone()),
                json!({ "grace_ms": grace_duration }),
            )?;
            queue_command_result(
                world,
                command_id,
                AxiomOperatorCommandStatus::Accepted,
                None,
            )?;
        }
        AxiomOperatorCommandKind::SnapshotNow => {
            if let Ok(mut state) = world.resource_mut::<OperatorControlState>() {
                state.force_snapshot = true;
            }
            queue_command_result(
                world,
                command_id,
                AxiomOperatorCommandStatus::Accepted,
                None,
            )?;
        }
        AxiomOperatorCommandKind::StartServer { .. }
        | AxiomOperatorCommandKind::StopServer { .. }
        | AxiomOperatorCommandKind::InspectLogs { .. } => {
            queue_command_result(
                world,
                command_id,
                AxiomOperatorCommandStatus::Rejected,
                Some("command is not supported on runtime bridge (Phase 2 only)".to_string()),
            )?;
        }
    }

    Ok(())
}

fn operator_shutdown_tick_system(mut world: WorldMut) -> Result<()> {
    let should_shutdown = {
        let mut state = world.resource_mut::<OperatorControlState>()?;
        if let Some(deadline) = state.pending_shutdown_deadline {
            if Instant::now() >= deadline {
                state.pending_shutdown_deadline = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if should_shutdown {
        execute_shutdown_now(&mut world)?;
        queue_event(
            &mut world,
            "operator.shutdown_executed",
            None,
            json!({ "source": "grace_deadline_elapsed" }),
        )?;
    }
    Ok(())
}

fn operator_emit_runtime_events_system(mut world: WorldMut) -> Result<()> {
    let diagnostics = world.resource::<NetworkDiagnostics>().ok().copied();
    let health = world.resource::<ConnectionHealth>().ok().cloned();
    let status = world.resource::<NetworkSessionStatus>().ok().cloned();

    let mut events_to_emit: Vec<(String, serde_json::Value)> = Vec::new();
    {
        let mut observed = world.resource_mut::<OperatorObservedState>()?;
        if let Some(diagnostics) = diagnostics {
            if diagnostics.accepted_connections > observed.accepted_connections {
                events_to_emit.push((
                    "network.join_accepted".to_string(),
                    json!({ "accepted_connections": diagnostics.accepted_connections }),
                ));
            }
            if diagnostics.rejected_connections > observed.rejected_connections {
                events_to_emit.push((
                    "network.join_rejected".to_string(),
                    json!({ "rejected_connections": diagnostics.rejected_connections }),
                ));
            }
            if diagnostics.reconnect_attempts > observed.reconnect_attempts {
                events_to_emit.push((
                    "network.reconnecting".to_string(),
                    json!({ "reconnect_attempts": diagnostics.reconnect_attempts }),
                ));
            }
            observed.accepted_connections = diagnostics.accepted_connections;
            observed.rejected_connections = diagnostics.rejected_connections;
            observed.reconnect_attempts = diagnostics.reconnect_attempts;
        }
        if let Some(health) = health {
            if health.close_events > observed.close_events {
                events_to_emit.push((
                    "network.connection_closed".to_string(),
                    json!({ "close_events": health.close_events }),
                ));
            }
            observed.close_events = health.close_events;
        }
        if let Some(status) = status {
            let phase = format!("{:?}", status.phase);
            if observed.last_phase.as_deref() != Some(phase.as_str()) {
                events_to_emit.push((
                    "network.phase_changed".to_string(),
                    json!({ "phase": phase }),
                ));
                observed.last_phase = Some(phase);
            }
            if observed.last_error != status.last_error {
                if let Some(error) = status.last_error.clone() {
                    events_to_emit.push((
                        "runtime.error_excerpt".to_string(),
                        json!({ "message": error }),
                    ));
                }
                observed.last_error = status.last_error;
            }
        }
    }

    for (event_type, payload) in events_to_emit {
        queue_event(&mut world, event_type.as_str(), None, payload)?;
    }
    Ok(())
}

fn operator_emit_snapshot_system(mut world: WorldMut) -> Result<()> {
    let tick = world
        .resource::<SimulationTick>()
        .ok()
        .map(|tick| tick.0)
        .unwrap_or(0);
    let should_emit = {
        let mut state = world.resource_mut::<OperatorControlState>()?;
        if state.force_snapshot
            || tick
                >= state
                    .last_snapshot_tick
                    .saturating_add(state.snapshot_interval_ticks)
        {
            state.force_snapshot = false;
            state.last_snapshot_tick = tick;
            true
        } else {
            false
        }
    };
    if !should_emit {
        return Ok(());
    }

    let server_id = world
        .resource::<OperatorRuntimeConfig>()
        .map(|config| config.server_id.clone())
        .unwrap_or_else(|_| "srv-local".to_string());
    let session_status = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .cloned()
        .unwrap_or_default();
    let diagnostics = world
        .resource::<NetworkDiagnostics>()
        .ok()
        .copied()
        .unwrap_or_default();
    let health = world
        .resource::<ConnectionHealth>()
        .ok()
        .cloned()
        .unwrap_or_default();
    let rtt = world
        .resource::<RoundTripMetrics>()
        .ok()
        .copied()
        .unwrap_or_default();
    let session_runtime = world
        .resource::<SessionRuntimeState>()
        .ok()
        .cloned()
        .unwrap_or_default();
    let server_session = world
        .resource::<ServerSessionState>()
        .ok()
        .cloned()
        .unwrap_or_default();
    let replication = world
        .resource::<ReplicationRuntimeMetrics>()
        .ok()
        .copied()
        .unwrap_or_default();
    let operator_state = world
        .resource::<OperatorControlState>()
        .ok()
        .cloned()
        .unwrap_or_default();

    let payload = json!({
        "tick": tick,
        "drain_mode_enabled": operator_state.drain_mode_enabled,
        "network_session": {
            "phase": format!("{:?}", session_status.phase),
            "connected": session_status.connected,
            "connection_id": session_status.connection_id.map(|id| id.0),
            "reconnect_attempt": session_status.reconnect_attempt,
            "last_disconnect": session_status.last_disconnect.map(|reason| format!("{reason:?}")),
        },
        "network_diagnostics": {
            "accepted_connections": diagnostics.accepted_connections,
            "rejected_connections": diagnostics.rejected_connections,
            "reconnect_attempts": diagnostics.reconnect_attempts,
            "flush_count": diagnostics.flush_count,
        },
        "connection_health": {
            "connected": health.connected,
            "close_events": health.close_events,
            "error_events": health.error_events,
        },
        "round_trip": {
            "last_rtt_millis": rtt.last_rtt_millis,
            "samples": rtt.samples,
        },
        "session_runtime": {
            "admitted": session_runtime.admitted,
            "lobby_id": session_runtime.lobby_id,
            "max_players": session_runtime.max_players,
            "ai_fill_target": session_runtime.ai_fill_target,
            "roster_size": session_runtime.roster_player_codes.len(),
        },
        "server_session": {
            "phase": format!("{:?}", server_session.phase),
            "active_connections": server_session.active_connections.len(),
            "active_connection": server_session.active_connection.map(|id| id.0),
        },
        "replication": {
            "bytes_sent_total": replication.bytes_sent_total,
            "bytes_received_total": replication.bytes_received_total,
            "patches_applied": replication.patches_applied,
            "keyframes_applied": replication.keyframes_applied,
            "load_shed_level_last_tick": replication.load_shed_level_last_tick,
        },
    });
    world
        .resource_mut::<OperatorOutboundQueue>()?
        .push(AxiomOperatorOutboundMessage::Snapshot(
            AxiomOperatorSnapshot {
                server_id,
                ts_ms: unix_now_millis(),
                payload,
            },
        ));
    Ok(())
}

fn operator_flush_outbound_system(mut world: WorldMut) -> Result<()> {
    let messages = world.resource_mut::<OperatorOutboundQueue>()?.drain();
    if messages.is_empty() {
        return Ok(());
    }

    let Some(handle) = world.resource::<AxiomOperatorRuntimeHandle>().ok() else {
        return Ok(());
    };
    for message in messages {
        if let Err(error) = handle.send_outbound(message) {
            eprintln!("failed sending operator outbound message: {error}");
        }
    }
    Ok(())
}

fn queue_command_result(
    world: &mut World,
    command_id: String,
    status: AxiomOperatorCommandStatus,
    message: Option<String>,
) -> Result<()> {
    let server_id = world
        .resource::<OperatorRuntimeConfig>()
        .map(|config| config.server_id.clone())
        .unwrap_or_else(|_| "srv-local".to_string());
    world.resource_mut::<OperatorOutboundQueue>()?.push(
        AxiomOperatorOutboundMessage::CommandResult(AxiomOperatorCommandResult {
            command_id,
            server_id,
            status,
            message,
            ts_ms: unix_now_millis(),
        }),
    );
    Ok(())
}

fn queue_event(
    world: &mut World,
    event_type: &str,
    command_id: Option<String>,
    payload: serde_json::Value,
) -> Result<()> {
    let server_id = world
        .resource::<OperatorRuntimeConfig>()
        .map(|config| config.server_id.clone())
        .unwrap_or_else(|_| "srv-local".to_string());
    world
        .resource_mut::<OperatorOutboundQueue>()?
        .push(AxiomOperatorOutboundMessage::Event(AxiomOperatorEvent {
            server_id,
            event_type: event_type.to_string(),
            ts_ms: unix_now_millis(),
            command_id,
            payload,
        }));
    Ok(())
}

fn send_runtime_command(
    world: &World,
    command: SessionRuntimeCommand,
) -> std::result::Result<(), String> {
    let handle = world
        .resource::<NetworkRuntimeHandle>()
        .map_err(|_| "network runtime handle is not available".to_string())?;
    handle.send(command)
}

fn execute_shutdown_now(world: &mut World) -> Result<()> {
    let _ = send_runtime_command(world, SessionRuntimeCommand::Shutdown);
    if let Ok(run_signal) = world.resource::<ServerRunSignal>() {
        run_signal.running.store(false, Ordering::SeqCst);
    }
    Ok(())
}

fn unix_now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}
