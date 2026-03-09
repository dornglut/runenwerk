use anyhow::Result;
use engine::plugins::NetworkRuntimeHandle;
use engine::prelude::{World, WorldMut};
use engine_net::{ServerMessage, SessionRuntimeCommand};
use grotto_online::{
    AxiomOperatorCommandResult, AxiomOperatorCommandStatus, AxiomOperatorEvent,
    AxiomOperatorOutboundMessage, AxiomOperatorRuntimeHandle,
};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{OperatorOutboundQueue, OperatorRuntimeConfig, ServerRunSignal};

pub(super) fn queue_command_result(
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

pub(super) fn queue_event(
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

pub(super) fn send_runtime_command(
    world: &World,
    command: SessionRuntimeCommand,
) -> std::result::Result<(), String> {
    let handle = world
        .resource::<NetworkRuntimeHandle>()
        .map_err(|_| "network runtime handle is not available".to_string())?;
    handle.send(command)
}

pub(super) fn execute_shutdown_now(world: &mut World) -> Result<()> {
    let _ = send_runtime_command(world, SessionRuntimeCommand::Shutdown);
    if let Ok(run_signal) = world.resource::<ServerRunSignal>() {
        run_signal.running.store(false, Ordering::SeqCst);
    }
    Ok(())
}

pub(super) fn unix_now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

pub(super) fn flush_outbound_messages(world: &mut WorldMut) -> Result<()> {
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

pub(super) fn broadcast_shutdown_notice(world: &World) {
    let _ = send_runtime_command(world, SessionRuntimeCommand::SetDrainMode { enabled: true });
    let _ = send_runtime_command(
        world,
        SessionRuntimeCommand::Server(ServerMessage::Disconnect(
            engine_net::DisconnectReason::ServerShuttingDown,
        )),
    );
}
