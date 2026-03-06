use grotto_online::AxiomOperatorOutboundMessage;
use std::collections::VecDeque;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;

use cavern_hunt::domain::ServerNetworkConfigAssetV1;

// Owner: Grotto Server - Operator Control Types
#[derive(Debug, Clone)]
pub(super) struct OperatorRuntimeConfig {
    pub(super) server_id: String,
}

impl OperatorRuntimeConfig {
    pub(super) fn from_config(config: &ServerNetworkConfigAssetV1) -> Self {
        Self {
            server_id: config.server_id.clone(),
        }
    }
}

#[derive(Default)]
pub(super) struct OperatorOutboundQueue {
    messages: Vec<AxiomOperatorOutboundMessage>,
}

impl OperatorOutboundQueue {
    pub(super) fn push(&mut self, message: AxiomOperatorOutboundMessage) {
        self.messages.push(message);
    }

    pub(super) fn drain(&mut self) -> Vec<AxiomOperatorOutboundMessage> {
        std::mem::take(&mut self.messages)
    }
}

#[derive(Default, Clone)]
pub(super) struct OperatorControlState {
    pub(super) recent_command_ids: VecDeque<String>,
    pub(super) dedupe_capacity: usize,
    pub(super) drain_mode_enabled: bool,
    pub(super) pending_shutdown_deadline: Option<Instant>,
    pub(super) force_snapshot: bool,
    pub(super) last_snapshot_tick: u64,
    pub(super) snapshot_interval_ticks: u64,
}

#[derive(Default)]
pub(super) struct OperatorObservedState {
    pub(super) accepted_connections: u64,
    pub(super) rejected_connections: u64,
    pub(super) close_events: u64,
    pub(super) reconnect_attempts: u64,
    pub(super) last_phase: Option<String>,
    pub(super) last_error: Option<String>,
}

pub(super) struct ServerRunSignal {
    pub(super) running: Arc<AtomicBool>,
}
