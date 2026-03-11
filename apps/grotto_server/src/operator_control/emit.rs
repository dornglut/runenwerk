use anyhow::Result;
use cavern_hunt::ReplicationRuntimeMetrics;
use engine::plugins::net::{
    ConnectionHealth, NetworkDiagnostics, NetworkSessionStatus, RoundTripMetrics,
};
use engine::prelude::{SimulationTick, WorldMut};
use engine::state::SessionRuntimeState;
use engine_net::ServerSessionState;
use grotto_online::{AxiomOperatorOutboundMessage, AxiomOperatorSnapshot};
use serde_json::json;
use std::time::Instant;

use super::common::{execute_shutdown_now, flush_outbound_messages, queue_event, unix_now_millis};
use super::types::{
    OperatorControlState, OperatorObservedState, OperatorOutboundQueue, OperatorRuntimeConfig,
};

pub(super) fn operator_shutdown_tick_system(mut world: WorldMut) -> Result<()> {
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

pub(super) fn operator_emit_runtime_events_system(mut world: WorldMut) -> Result<()> {
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

pub(super) fn operator_emit_snapshot_system(mut world: WorldMut) -> Result<()> {
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

pub(super) fn operator_flush_outbound_system(mut world: WorldMut) -> Result<()> {
    flush_outbound_messages(&mut world)
}
