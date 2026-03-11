use super::*;
use crate::{App, CoreSet, FixedUpdate, FrameEnd, PreUpdate, SessionRuntimeState, SystemConfigExt};
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::SimulationTick;
use std::collections::BTreeMap;
use std::mem;
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

// engine/src/plugins/net/resources.rs

const NETWORK_MESSAGE_QUEUE_CAPACITY: usize = 4_096;

fn push_bounded<T>(queue: &mut Vec<T>, message: T, queue_name: &str) {
    if queue.len() >= NETWORK_MESSAGE_QUEUE_CAPACITY {
        queue.remove(0);
        tracing::warn!(
            queue = queue_name,
            capacity = NETWORK_MESSAGE_QUEUE_CAPACITY,
            "network queue overflow; dropping oldest message"
        );
    }
    queue.push(message);
}

pub(crate) fn configure_runtime_bridge<TDriver>(app: &mut App)
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    app.add_systems(
        PreUpdate,
        network_runtime_receive_system::<TDriver>.in_set(CoreSet::NetReceive),
    );
    app.add_systems(
        PreUpdate,
        client_receive_system::<TDriver>.in_set(CoreSet::NetReceive),
    );
    app.add_systems(
        PreUpdate,
        server_receive_system::<TDriver>.in_set(CoreSet::NetReceive),
    );
}

pub(crate) fn configure_client_role(app: &mut App) {
    app.init_resource::<NetworkClientInbox>();
    app.init_resource::<NetworkClientOutbox>();
    app.init_resource::<NetworkInboundQueue>();
    app.init_resource::<NetworkOutboundQueue>();
    app.init_resource::<NetworkSessionStatus>();
    app.init_resource::<NetworkAdmissionState>();
    app.init_resource::<SessionRuntimeState>();
    app.init_resource::<ConnectionHealth>();
    app.init_resource::<RoundTripMetrics>();
    app.init_resource::<ClientSessionState>();
    app.init_resource::<NetworkDiagnostics>();
    app.add_systems(FrameEnd, client_flush_system.in_set(CoreSet::FrameEnd));
}

pub(crate) fn configure_server_role(app: &mut App) {
    app.init_resource::<NetworkServerInbox>();
    app.init_resource::<NetworkServerOutbox>();
    app.init_resource::<NetworkInboundQueue>();
    app.init_resource::<NetworkOutboundQueue>();
    app.init_resource::<NetworkSessionStatus>();
    app.init_resource::<NetworkAdmissionState>();
    app.init_resource::<SessionRuntimeState>();
    app.init_resource::<ConnectionHealth>();
    app.init_resource::<RoundTripMetrics>();
    app.init_resource::<ServerSessionConfig>();
    app.init_resource::<ServerSessionState>();
    app.init_resource::<NetworkDiagnostics>();
    app.add_systems(FrameEnd, server_flush_system.in_set(CoreSet::FrameEnd));
}

pub(crate) fn configure_replication<TDriver>(app: &mut App)
where
    TDriver: ReplicationDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq + 'static,
{
    app.init_resource::<SnapshotCursor>();
    app.init_resource::<ServerSnapshotReplicationState<TDriver::Snapshot>>();
    app.init_resource::<ClientSnapshotReplicationState<TDriver::Snapshot>>();
    app.init_resource::<ReplicationDiagnostics>();
    app.add_systems(
        FixedUpdate,
        replication_step_system::<TDriver>
            .in_set(CoreSet::Replication)
            .after(CoreSet::Simulation),
    );
}

pub(crate) fn configure_prediction<TDriver>(app: &mut App)
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq + 'static,
{
    app.init_resource::<PredictionState<TDriver::Input>>();
    app.init_resource::<PredictionDiagnostics>();
    app.add_systems(
        FixedUpdate,
        prediction_step_system::<TDriver>
            .in_set(CoreSet::Simulation)
            .before(CoreSet::Replication),
    );
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkClientInbox {
    messages: Vec<ServerMessage>,
}

impl NetworkClientInbox {
    pub fn push(&mut self, message: ServerMessage) {
        push_bounded(&mut self.messages, message, "NetworkClientInbox");
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn drain(&mut self) -> Vec<ServerMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkServerInbox {
    messages: Vec<InboundClientMessage>,
}

impl NetworkServerInbox {
    pub fn push(&mut self, message: ClientMessage) {
        self.push_from(None, message);
    }

    pub fn push_from(&mut self, connection_id: Option<ConnectionId>, message: ClientMessage) {
        push_bounded(
            &mut self.messages,
            InboundClientMessage {
                connection_id,
                message,
            },
            "NetworkServerInbox",
        );
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn drain(&mut self) -> Vec<InboundClientMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkClientOutbox {
    messages: Vec<ClientMessage>,
}

impl NetworkClientOutbox {
    pub fn push(&mut self, message: ClientMessage) {
        push_bounded(&mut self.messages, message, "NetworkClientOutbox");
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn drain(&mut self) -> Vec<ClientMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutboundServerMessage {
    ToConnection {
        connection_id: ConnectionId,
        message: ServerMessage,
    },
    Broadcast(ServerMessage),
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkServerOutbox {
    messages: Vec<OutboundServerMessage>,
}

impl NetworkServerOutbox {
    pub fn push(&mut self, message: ServerMessage) {
        self.push_broadcast(message);
    }

    pub fn push_broadcast(&mut self, message: ServerMessage) {
        push_bounded(
            &mut self.messages,
            OutboundServerMessage::Broadcast(message),
            "NetworkServerOutbox",
        );
    }

    pub fn push_to(&mut self, connection_id: ConnectionId, message: ServerMessage) {
        push_bounded(
            &mut self.messages,
            OutboundServerMessage::ToConnection {
                connection_id,
                message,
            },
            "NetworkServerOutbox",
        );
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn drain(&mut self) -> Vec<OutboundServerMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkInboundQueue {
    client_messages: Vec<InboundClientMessage>,
    server_messages: Vec<ServerMessage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InboundClientMessage {
    pub connection_id: Option<ConnectionId>,
    pub message: ClientMessage,
}

impl NetworkInboundQueue {
    pub fn clear(&mut self) {
        self.client_messages.clear();
        self.server_messages.clear();
    }

    pub fn push_client(&mut self, connection_id: Option<ConnectionId>, message: ClientMessage) {
        push_bounded(
            &mut self.client_messages,
            InboundClientMessage {
                connection_id,
                message,
            },
            "NetworkInboundQueue.client_messages",
        );
    }

    pub fn push_server(&mut self, message: ServerMessage) {
        push_bounded(
            &mut self.server_messages,
            message,
            "NetworkInboundQueue.server_messages",
        );
    }

    pub fn client_messages(&self) -> &[InboundClientMessage] {
        &self.client_messages
    }

    pub fn server_messages(&self) -> &[ServerMessage] {
        &self.server_messages
    }
}

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct NetworkOutboundQueue {
    client_messages: Vec<ClientMessage>,
    server_messages: Vec<OutboundServerMessage>,
}

impl NetworkOutboundQueue {
    pub fn clear(&mut self) {
        self.client_messages.clear();
        self.server_messages.clear();
    }

    pub fn push_client(&mut self, message: ClientMessage) {
        push_bounded(
            &mut self.client_messages,
            message,
            "NetworkOutboundQueue.client_messages",
        );
    }

    pub fn push_server(&mut self, message: OutboundServerMessage) {
        push_bounded(
            &mut self.server_messages,
            message,
            "NetworkOutboundQueue.server_messages",
        );
    }

    pub fn client_messages(&self) -> &[ClientMessage] {
        &self.client_messages
    }

    pub fn server_messages(&self) -> &[OutboundServerMessage] {
        &self.server_messages
    }
}

#[derive(ecs::Component)]
pub struct NetworkRuntimeHandle {
    command_tx: Sender<SessionRuntimeCommand>,
    event_rx: Receiver<SessionRuntimeEvent>,
}

impl NetworkRuntimeHandle {
    pub fn new(
        command_tx: Sender<SessionRuntimeCommand>,
        event_rx: Receiver<SessionRuntimeEvent>,
    ) -> Self {
        Self {
            command_tx,
            event_rx,
        }
    }

    pub fn send(&self, command: SessionRuntimeCommand) -> Result<(), String> {
        self.command_tx
            .try_send(command)
            .map_err(|error| format!("network runtime send failed: {error}"))
    }

    pub fn try_recv(&mut self) -> Result<Option<SessionRuntimeEvent>, TryRecvError> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct NetworkSessionStatus {
    pub phase: SessionPhase,
    pub connection_id: Option<ConnectionId>,
    pub last_disconnect: Option<DisconnectReason>,
    pub last_error: Option<String>,
    pub connected: bool,
    pub reconnect_attempt: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct NetworkAdmissionState {
    pub authoritative_join: Option<AuthoritativeJoinState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct ConnectionHealth {
    pub connected: bool,
    pub close_events: u64,
    pub error_events: u64,
    pub reconnect_events: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct RoundTripMetrics {
    pub last_rtt_millis: Option<u32>,
    pub samples: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ConnectionBaselineCheckpoint {
    pub last_ack_cursor: SnapshotCursor,
    pub last_sent_cursor: SnapshotCursor,
    pub last_full_snapshot_cursor: SnapshotCursor,
    pub last_full_snapshot_tick: SimulationTick,
    pub needs_full_resync: bool,
}

impl Default for ConnectionBaselineCheckpoint {
    fn default() -> Self {
        Self {
            last_ack_cursor: SnapshotCursor::default(),
            last_sent_cursor: SnapshotCursor::default(),
            last_full_snapshot_cursor: SnapshotCursor::default(),
            last_full_snapshot_tick: SimulationTick::default(),
            needs_full_resync: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ecs::Component)]
pub struct ServerSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    pub checkpoints: BTreeMap<ConnectionId, ConnectionBaselineCheckpoint>,
    pub snapshot_history: BTreeMap<SnapshotCursor, TSnapshot>,
    pub latest_snapshot: Option<TSnapshot>,
    pub latest_tick: SimulationTick,
}

impl<TSnapshot> Default for ServerSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            checkpoints: BTreeMap::new(),
            snapshot_history: BTreeMap::new(),
            latest_snapshot: None,
            latest_tick: SimulationTick::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, ecs::Component)]
pub struct ClientSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    pub last_acknowledged_cursor: SnapshotCursor,
    pub last_received_tick: SimulationTick,
    pub applied_snapshots: u64,
    pub last_received_snapshot: Option<TSnapshot>,
}

impl<TSnapshot> Default for ClientSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            last_acknowledged_cursor: SnapshotCursor::default(),
            last_received_tick: SimulationTick::default(),
            applied_snapshots: 0,
            last_received_snapshot: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingInputFrame<TInput>
where
    TInput: Clone + PartialEq + 'static,
{
    pub tick: SimulationTick,
    pub commands: Vec<TInput>,
}

#[derive(Debug, Clone, PartialEq, ecs::Component)]
pub struct PredictionState<TInput>
where
    TInput: Clone + PartialEq + 'static,
{
    pub pending_frames: Vec<PendingInputFrame<TInput>>,
}

impl<TInput> Default for PredictionState<TInput>
where
    TInput: Clone + PartialEq + 'static,
{
    fn default() -> Self {
        Self {
            pending_frames: Vec::new(),
        }
    }
}

impl<TInput> PredictionState<TInput>
where
    TInput: Clone + PartialEq + 'static,
{
    pub fn pending_frames_len(&self) -> usize {
        self.pending_frames.len()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct NetworkDiagnostics {
    pub processed_client_messages_last_frame: usize,
    pub processed_server_messages_last_frame: usize,
    pub flushed_client_messages_last_frame: usize,
    pub flushed_server_messages_last_frame: usize,
    pub flush_count: u64,
    pub accepted_connections: u64,
    pub rejected_connections: u64,
    pub reconnect_attempts: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct ReplicationDiagnostics {
    pub fixed_steps_observed: u64,
    pub last_snapshot_cursor: u64,
    pub emitted_snapshots: u64,
    pub applied_snapshots: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component)]
pub struct PredictionDiagnostics {
    pub fixed_steps_observed: u64,
    pub commands_applied: u64,
    pub corrections_applied: u64,
}
