use super::*;
use crate::{App, CoreSet, FixedUpdate, FrameEnd, PreUpdate, SessionRuntimeState, SystemConfigExt};
use ecs::{QueueConfig, QueueEnqueueError, World};
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::SimulationTick;
use std::collections::BTreeMap;
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

// engine/src/plugins/net/resources.rs

const NETWORK_MESSAGE_QUEUE_CAPACITY: usize = 4_096;

fn configure_network_message_queues(world: &mut World) {
    let config = QueueConfig {
        capacity: Some(NETWORK_MESSAGE_QUEUE_CAPACITY),
    };
    world.configure_queue::<ServerMessage>(config);
    world.configure_queue::<InboundClientMessage>(config);
    world.configure_queue::<ClientMessage>(config);
    world.configure_queue::<OutboundServerMessage>(config);
}

fn enqueue_queue_with_backpressure<T: 'static>(
    world: &mut World,
    queue_name: &'static str,
    message: T,
) -> Result<(), QueueEnqueueError> {
    let result = world.queue_enqueue(message);
    if let Err(QueueEnqueueError::Backpressure { capacity, .. }) = &result {
        tracing::warn!(
            queue = queue_name,
            capacity = *capacity,
            "network queue backpressure; dropping newest message"
        );
    }
    result
}

pub fn enqueue_client_inbox(
    world: &mut World,
    message: ServerMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_queue_with_backpressure(world, "NetworkClientInbox", message)
}

pub fn client_inbox_len(world: &World) -> usize {
    world.queue_pending_count::<ServerMessage>()
}

pub fn client_inbox_is_empty(world: &World) -> bool {
    client_inbox_len(world) == 0
}

pub fn drain_client_inbox(world: &mut World) -> Vec<ServerMessage> {
    world.queue_drain::<ServerMessage>()
}

pub fn enqueue_server_inbox(
    world: &mut World,
    message: ClientMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_server_inbox_from(world, None, message)
}

pub fn enqueue_server_inbox_from(
    world: &mut World,
    connection_id: Option<ConnectionId>,
    message: ClientMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_queue_with_backpressure(
        world,
        "NetworkServerInbox",
        InboundClientMessage {
            connection_id,
            message,
        },
    )
}

pub fn server_inbox_len(world: &World) -> usize {
    world.queue_pending_count::<InboundClientMessage>()
}

pub fn server_inbox_is_empty(world: &World) -> bool {
    server_inbox_len(world) == 0
}

pub fn drain_server_inbox(world: &mut World) -> Vec<InboundClientMessage> {
    world.queue_drain::<InboundClientMessage>()
}

pub fn enqueue_client_outbox(
    world: &mut World,
    message: ClientMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_queue_with_backpressure(world, "NetworkClientOutbox", message)
}

pub fn client_outbox_len(world: &World) -> usize {
    world.queue_pending_count::<ClientMessage>()
}

pub fn client_outbox_is_empty(world: &World) -> bool {
    client_outbox_len(world) == 0
}

pub fn drain_client_outbox(world: &mut World) -> Vec<ClientMessage> {
    world.queue_drain::<ClientMessage>()
}

pub fn enqueue_server_outbox(
    world: &mut World,
    message: OutboundServerMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_queue_with_backpressure(world, "NetworkServerOutbox", message)
}

pub fn enqueue_server_outbox_broadcast(
    world: &mut World,
    message: ServerMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_server_outbox(world, OutboundServerMessage::Broadcast(message))
}

pub fn enqueue_server_outbox_to(
    world: &mut World,
    connection_id: ConnectionId,
    message: ServerMessage,
) -> Result<(), QueueEnqueueError> {
    enqueue_server_outbox(
        world,
        OutboundServerMessage::ToConnection {
            connection_id,
            message,
        },
    )
}

pub fn server_outbox_len(world: &World) -> usize {
    world.queue_pending_count::<OutboundServerMessage>()
}

pub fn server_outbox_is_empty(world: &World) -> bool {
    server_outbox_len(world) == 0
}

pub fn drain_server_outbox(world: &mut World) -> Vec<OutboundServerMessage> {
    world.queue_drain::<OutboundServerMessage>()
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
    configure_network_message_queues(app.world_mut());
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
    configure_network_message_queues(app.world_mut());
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

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct NetworkClientInbox;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct NetworkServerInbox;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct NetworkClientOutbox;

#[derive(Debug, Clone, PartialEq)]
pub enum OutboundServerMessage {
    ToConnection {
        connection_id: ConnectionId,
        message: ServerMessage,
    },
    Broadcast(ServerMessage),
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct NetworkServerOutbox;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
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
        self.client_messages.push(InboundClientMessage {
            connection_id,
            message,
        });
    }

    pub fn push_server(&mut self, message: ServerMessage) {
        self.server_messages.push(message);
    }

    pub fn client_messages(&self) -> &[InboundClientMessage] {
        &self.client_messages
    }

    pub fn server_messages(&self) -> &[ServerMessage] {
        &self.server_messages
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
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
        self.client_messages.push(message);
    }

    pub fn push_server(&mut self, message: OutboundServerMessage) {
        self.server_messages.push(message);
    }

    pub fn client_messages(&self) -> &[ClientMessage] {
        &self.client_messages
    }

    pub fn server_messages(&self) -> &[OutboundServerMessage] {
        &self.server_messages
    }
}

#[derive(ecs::Component, ecs::Resource)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct NetworkSessionStatus {
    pub phase: SessionPhase,
    pub connection_id: Option<ConnectionId>,
    pub last_disconnect: Option<DisconnectReason>,
    pub last_error: Option<String>,
    pub connected: bool,
    pub reconnect_attempt: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct NetworkAdmissionState {
    pub authoritative_join: Option<AuthoritativeJoinState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct ConnectionHealth {
    pub connected: bool,
    pub close_events: u64,
    pub error_events: u64,
    pub reconnect_events: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
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

#[derive(Debug, Clone, PartialEq, ecs::Component, ecs::Resource)]
pub struct ServerSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    pub checkpoints: BTreeMap<ConnectionId, ConnectionBaselineCheckpoint>,
    pub snapshot_history: BTreeMap<SnapshotCursor, TSnapshot>,
    pub snapshot_history_per_connection:
        BTreeMap<ConnectionId, BTreeMap<SnapshotCursor, TSnapshot>>,
    pub latest_snapshot: Option<TSnapshot>,
    pub latest_snapshot_per_connection: BTreeMap<ConnectionId, TSnapshot>,
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
            snapshot_history_per_connection: BTreeMap::new(),
            latest_snapshot: None,
            latest_snapshot_per_connection: BTreeMap::new(),
            latest_tick: SimulationTick::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, ecs::Component, ecs::Resource)]
pub struct ClientSnapshotReplicationState<TSnapshot>
where
    TSnapshot: Clone + PartialEq + 'static,
{
    pub last_acknowledged_cursor: SnapshotCursor,
    pub last_received_tick: SimulationTick,
    pub applied_snapshots: u64,
    pub last_received_snapshot: Option<TSnapshot>,
    pub snapshot_history: BTreeMap<SnapshotCursor, TSnapshot>,
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
            snapshot_history: BTreeMap::new(),
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

#[derive(Debug, Clone, PartialEq, ecs::Component, ecs::Resource)]
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct ReplicationDiagnostics {
    pub fixed_steps_observed: u64,
    pub last_snapshot_cursor: u64,
    pub emitted_snapshots: u64,
    pub applied_snapshots: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct PredictionDiagnostics {
    pub fixed_steps_observed: u64,
    pub commands_applied: u64,
    pub corrections_applied: u64,
}
