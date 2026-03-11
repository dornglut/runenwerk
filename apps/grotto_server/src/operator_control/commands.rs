use anyhow::Result;
use engine::prelude::{World, WorldMut};
use engine_net::{ConnectionId, SessionRuntimeCommand};
use grotto_online::{AxiomOperatorCommand, AxiomOperatorCommandKind, AxiomOperatorCommandStatus};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::error::TryRecvError;

use super::common::{
    broadcast_shutdown_notice, execute_shutdown_now, queue_command_result, queue_event,
    send_runtime_command,
};
use super::types::{OperatorControlState, OperatorRuntimeConfig};
use super::types::OperatorRuntimeBridgeHandle;

pub(super) fn operator_receive_commands_system(mut world: WorldMut) -> Result<()> {
    let Some(mut handle) = world.remove_resource::<OperatorRuntimeBridgeHandle>() else {
        return Ok(());
    };
    loop {
        let command = match handle.handle.try_recv_command() {
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
            broadcast_shutdown_notice(world);
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
