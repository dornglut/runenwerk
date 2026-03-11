use anyhow::Result;
use cavern_hunt::ServerNetworkConfigAssetV1;
use engine::prelude::{App, FrameEnd, PreUpdate, Update};
use grotto_online::{AxiomOperatorBridgeConfig, spawn_axiom_operator_bridge};
use std::sync::{Arc, atomic::AtomicBool};

use super::commands::operator_receive_commands_system;
use super::emit::{
    operator_emit_runtime_events_system, operator_emit_snapshot_system,
    operator_flush_outbound_system, operator_shutdown_tick_system,
};
use super::types::{
    OperatorControlState, OperatorObservedState, OperatorOutboundQueue,
    OperatorRuntimeBridgeHandle, OperatorRuntimeConfig, ServerRunSignal,
};

// Owner: Grotto Server - Operator Control Install
pub(super) fn try_install_operator_control(
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

    app.world_mut()
        .insert_resource(OperatorRuntimeBridgeHandle { handle });
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
