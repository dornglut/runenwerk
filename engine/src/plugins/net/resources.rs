use super::*;
use crate::{App, CoreSet, FixedUpdate, FrameEnd, PreUpdate, SystemConfigExt};
use ecs::{
    OwnerId, OwnerRole, OwnershipTarget, TickBufferConfig, TickBufferProvenance, WorkQueueConfig,
    WorkQueueEnqueueError, World,
};
use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::*;
use engine_sim::SimulationTick;
use std::collections::{BTreeMap, BTreeSet};
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

// engine/src/plugins/net/resources.rs

const NETWORK_MESSAGE_QUEUE_CAPACITY: usize = 4_096;
const MAX_TRACKED_SENT_BASELINE_CURSORS: usize = 256;
const TICK_BUFFER_PROVENANCE_DOMAIN_SERVER: u32 = 1;
const TICK_BUFFER_PROVENANCE_DOMAIN_OWNER: u32 = 2;

fn configure_network_message_queues(world: &mut World) {
    let config = WorkQueueConfig {
        capacity: Some(NETWORK_MESSAGE_QUEUE_CAPACITY),
    };
    world.configure_work_queue::<ServerMessage>(config);
    world.configure_work_queue::<InboundClientMessage>(config);
    world.configure_work_queue::<ClientMessage>(config);
    world.configure_work_queue::<OutboundServerMessage>(config);
}

fn enqueue_work_queue_with_backpressure<T: 'static>(
    world: &mut World,
    work_queue_name: &'static str,
    message: T,
) -> Result<(), WorkQueueEnqueueError> {
    let result = world.work_queue_enqueue(message);
    if let Err(WorkQueueEnqueueError::Backpressure { capacity, .. }) = &result {
        tracing::warn!(
            work_queue = work_queue_name,
            capacity = *capacity,
            "network queue backpressure; dropping newest message"
        );
    }
    result
}

pub fn enqueue_client_inbox(
    world: &mut World,
    message: ServerMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_work_queue_with_backpressure(world, "NetworkClientInbox", message)
}

pub fn client_inbox_len(world: &World) -> usize {
    world.work_queue_pending_count::<ServerMessage>()
}

pub fn client_inbox_is_empty(world: &World) -> bool {
    client_inbox_len(world) == 0
}

pub fn drain_client_inbox(world: &mut World) -> Vec<ServerMessage> {
    world.work_queue_drain::<ServerMessage>()
}

pub fn enqueue_server_inbox(
    world: &mut World,
    message: ClientMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_server_inbox_from(world, None, message)
}

pub fn enqueue_server_inbox_from(
    world: &mut World,
    connection_id: Option<ConnectionId>,
    message: ClientMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_work_queue_with_backpressure(
        world,
        "NetworkServerInbox",
        InboundClientMessage {
            connection_id,
            message,
        },
    )
}

pub fn server_inbox_len(world: &World) -> usize {
    world.work_queue_pending_count::<InboundClientMessage>()
}

pub fn server_inbox_is_empty(world: &World) -> bool {
    server_inbox_len(world) == 0
}

pub fn drain_server_inbox(world: &mut World) -> Vec<InboundClientMessage> {
    world.work_queue_drain::<InboundClientMessage>()
}

pub fn enqueue_client_outbox(
    world: &mut World,
    message: ClientMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_work_queue_with_backpressure(world, "NetworkClientOutbox", message)
}

pub fn client_outbox_len(world: &World) -> usize {
    world.work_queue_pending_count::<ClientMessage>()
}

pub fn client_outbox_is_empty(world: &World) -> bool {
    client_outbox_len(world) == 0
}

pub fn drain_client_outbox(world: &mut World) -> Vec<ClientMessage> {
    world.work_queue_drain::<ClientMessage>()
}

pub fn enqueue_server_outbox(
    world: &mut World,
    message: OutboundServerMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_work_queue_with_backpressure(world, "NetworkServerOutbox", message)
}

pub fn enqueue_server_outbox_broadcast(
    world: &mut World,
    message: ServerMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_server_outbox(world, OutboundServerMessage::Broadcast(message))
}

pub fn enqueue_server_outbox_to(
    world: &mut World,
    connection_id: ConnectionId,
    message: ServerMessage,
) -> Result<(), WorkQueueEnqueueError> {
    enqueue_server_outbox(
        world,
        OutboundServerMessage::ToConnection {
            connection_id,
            message,
        },
    )
}

pub fn server_outbox_len(world: &World) -> usize {
    world.work_queue_pending_count::<OutboundServerMessage>()
}

pub fn server_outbox_is_empty(world: &World) -> bool {
    server_outbox_len(world) == 0
}

pub fn drain_server_outbox(world: &mut World) -> Vec<OutboundServerMessage> {
    world.work_queue_drain::<OutboundServerMessage>()
}

pub fn ensure_owner_for_connection(
    world: &mut World,
    connection_id: ConnectionId,
    role: OwnerRole,
) -> OwnerId {
    if let Ok(routing) = world.resource::<NetworkOwnerRouting>()
        && let Some(owner_id) = routing.by_connection.get(&connection_id).copied()
    {
        world.set_owner_role(owner_id, role);
        return owner_id;
    }

    let owner_id = world.create_owner(role);
    if let Ok(routing) = world.resource_mut::<NetworkOwnerRouting>() {
        routing.by_connection.insert(connection_id, owner_id);
        routing.by_owner.insert(owner_id, connection_id);
    }
    owner_id
}

pub fn owner_for_connection(world: &World, connection_id: ConnectionId) -> Option<OwnerId> {
    world
        .resource::<NetworkOwnerRouting>()
        .ok()
        .and_then(|routing| routing.by_connection.get(&connection_id).copied())
}

pub fn server_tick_buffer_provenance() -> TickBufferProvenance {
    TickBufferProvenance::new(TICK_BUFFER_PROVENANCE_DOMAIN_SERVER, 1)
}

pub fn owner_tick_buffer_provenance(owner_id: OwnerId) -> TickBufferProvenance {
    TickBufferProvenance::new(TICK_BUFFER_PROVENANCE_DOMAIN_OWNER, owner_id.as_raw())
}

pub fn remove_owner_for_connection(
    world: &mut World,
    connection_id: ConnectionId,
) -> Option<OwnerId> {
    let mut removed = None;
    if let Ok(routing) = world.resource_mut::<NetworkOwnerRouting>()
        && let Some(owner_id) = routing.by_connection.remove(&connection_id)
    {
        routing.by_owner.remove(&owner_id);
        removed = Some(owner_id);
    }
    removed
}

pub fn route_connection_targets(
    world: &World,
    connection_id: ConnectionId,
) -> Vec<OwnershipTarget> {
    let Some(owner_id) = owner_for_connection(world, connection_id) else {
        return Vec::new();
    };
    world.route_owner_targets(owner_id)
}

pub(crate) fn configure_runtime_bridge<TDriver>(app: &mut App)
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    app.add_systems(
        PreUpdate,
        network_runtime_receive_system::<TDriver>.in_set(NetPreUpdateSet::Receive),
    );
    app.add_systems(
        PreUpdate,
        client_receive_system::<TDriver>.in_set(NetPreUpdateSet::Receive),
    );
    app.add_systems(
        PreUpdate,
        server_receive_system::<TDriver>.in_set(NetPreUpdateSet::Receive),
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
    app.init_resource::<NetSessionView>();
    app.init_resource::<NetDiagnosticsView>();
    app.init_resource::<ConnectionHealth>();
    app.init_resource::<RoundTripMetrics>();
    app.init_resource::<ClientSessionState>();
    app.init_resource::<NetworkOwnerRouting>();
    app.init_resource::<NetworkReplicationMetadata>();
    app.init_resource::<NetStreamingStateResource>();
    app.init_resource::<NetworkDiagnostics>();
    app.add_systems(FrameEnd, client_flush_system.in_set(CoreSet::FrameEnd));
    app.add_systems(
        FrameEnd,
        sync_net_diagnostics_view_system.in_set(CoreSet::FrameEnd),
    );
}

pub(crate) fn configure_server_role(app: &mut App) {
    configure_network_message_queues(app.world_mut());
    app.init_resource::<NetworkServerInbox>();
    app.init_resource::<NetworkServerOutbox>();
    app.init_resource::<NetworkInboundQueue>();
    app.init_resource::<NetworkOutboundQueue>();
    app.init_resource::<NetworkSessionStatus>();
    app.init_resource::<NetworkAdmissionState>();
    app.init_resource::<NetSessionView>();
    app.init_resource::<NetDiagnosticsView>();
    app.init_resource::<ConnectionHealth>();
    app.init_resource::<RoundTripMetrics>();
    app.init_resource::<ServerSessionConfig>();
    app.init_resource::<ServerSessionState>();
    app.init_resource::<NetworkOwnerRouting>();
    app.init_resource::<NetworkReplicationMetadata>();
    app.init_resource::<NetStreamingStateResource>();
    app.init_resource::<NetworkDiagnostics>();
    app.add_systems(FrameEnd, server_flush_system.in_set(CoreSet::FrameEnd));
    app.add_systems(
        FrameEnd,
        sync_net_diagnostics_view_system.in_set(CoreSet::FrameEnd),
    );
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
        sync_connection_streaming_state_system
            .after(CoreSet::Simulation)
            .before(NetFixedSet::Prediction),
    );
    app.add_systems(
        FixedUpdate,
        replication_step_system::<TDriver>
            .in_set(NetFixedSet::Replication)
            .after(CoreSet::Simulation)
            .after(NetFixedSet::Prediction),
    );
}

pub(crate) fn configure_prediction<TDriver>(app: &mut App)
where
    TDriver: ReplicationDriver + InputDriver + Send + Sync + 'static,
    TDriver::Input: Clone + PartialEq + 'static,
{
    app.world_mut()
        .configure_tick_buffer::<TDriver::Input>(TickBufferConfig {
            capacity: Some(NETWORK_MESSAGE_QUEUE_CAPACITY),
            retain_finalized_ticks: false,
        });
    app.init_resource::<PredictionState<TDriver::Input>>();
    app.init_resource::<PredictionDiagnostics>();
    app.add_systems(
        FixedUpdate,
        prediction_step_system::<TDriver>
            .in_set(NetFixedSet::Prediction)
            .after(CoreSet::Simulation),
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
pub struct NetworkOwnerRouting {
    pub by_connection: BTreeMap<ConnectionId, OwnerId>,
    pub by_owner: BTreeMap<OwnerId, ConnectionId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct NetworkAdmissionState {
    pub authoritative_join: Option<AuthoritativeJoinState>,
}

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct NetSessionView {
    pub admitted: bool,
    pub lobby_id: Option<String>,
    pub roster_player_codes: Vec<String>,
    pub max_players: u8,
    pub ai_fill_target: u8,
    pub settings_json: Option<String>,
}

impl Default for NetSessionView {
    fn default() -> Self {
        Self {
            admitted: false,
            lobby_id: None,
            roster_player_codes: Vec::new(),
            max_players: 1,
            ai_fill_target: 1,
            settings_json: None,
        }
    }
}

impl NetSessionView {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn apply_authoritative_join(&mut self, join: &AuthoritativeJoinState) {
        let roster_size = join.roster_player_codes.len().clamp(1, u8::MAX as usize) as u8;
        let max_players = join.max_players.max(roster_size).max(1);
        let ai_fill_target = if join.ai_fill_target == 0 {
            max_players
        } else {
            join.ai_fill_target.clamp(roster_size, max_players)
        };

        self.admitted = true;
        self.lobby_id = join.lobby_id.clone();
        self.roster_player_codes = join.roster_player_codes.clone();
        self.max_players = max_players;
        self.ai_fill_target = ai_fill_target;
        self.settings_json = join.settings_json.clone();
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionBaselineCheckpoint {
    pub last_ack_cursor: SnapshotCursor,
    pub last_sent_cursor: SnapshotCursor,
    pub last_full_snapshot_cursor: SnapshotCursor,
    pub last_full_snapshot_tick: SimulationTick,
    pub needs_full_resync: bool,
    pub sent_cursors: BTreeSet<SnapshotCursor>,
}

impl Default for ConnectionBaselineCheckpoint {
    fn default() -> Self {
        Self {
            last_ack_cursor: SnapshotCursor::default(),
            last_sent_cursor: SnapshotCursor::default(),
            last_full_snapshot_cursor: SnapshotCursor::default(),
            last_full_snapshot_tick: SimulationTick::default(),
            needs_full_resync: true,
            sent_cursors: BTreeSet::new(),
        }
    }
}

impl ConnectionBaselineCheckpoint {
    pub fn mark_snapshot_sent(
        &mut self,
        cursor: SnapshotCursor,
        tick: SimulationTick,
        sent_full_snapshot: bool,
    ) {
        if cursor.0 >= self.last_sent_cursor.0 {
            self.last_sent_cursor = cursor;
        }
        self.sent_cursors.insert(cursor);
        while self.sent_cursors.len() > MAX_TRACKED_SENT_BASELINE_CURSORS {
            let Some(oldest_cursor) = self.sent_cursors.first().copied() else {
                break;
            };
            self.sent_cursors.remove(&oldest_cursor);
        }
        if sent_full_snapshot {
            self.last_full_snapshot_cursor = cursor;
            self.last_full_snapshot_tick = tick;
            self.needs_full_resync = false;
        }
    }

    pub fn mark_snapshot_acknowledged(
        &mut self,
        cursor: SnapshotCursor,
        baseline_available: bool,
    ) -> SnapshotAckOutcome {
        let outcome = self.validate_snapshot_ack(cursor, baseline_available);
        if matches!(outcome, SnapshotAckOutcome::Accepted { .. }) {
            self.last_ack_cursor = cursor;
            self.needs_full_resync = false;
        }
        outcome
    }

    fn validate_snapshot_ack(
        &self,
        cursor: SnapshotCursor,
        baseline_available: bool,
    ) -> SnapshotAckOutcome {
        if self.last_ack_cursor.0 != 0 && cursor <= self.last_ack_cursor {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::StaleCursor {
                    last_acknowledged: self.last_ack_cursor,
                },
            };
        }
        if self.last_sent_cursor.0 != 0 && cursor > self.last_sent_cursor {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::FutureCursor {
                    latest_cursor: self.last_sent_cursor,
                },
            };
        }
        if !self.sent_cursors.contains(&cursor) {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::UnsentCursor,
            };
        }
        if !baseline_available {
            return SnapshotAckOutcome::Rejected {
                cursor,
                reason: SnapshotAckRejection::PrunedCursor,
            };
        }
        SnapshotAckOutcome::Accepted { cursor }
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
    pub acked: u64,
    pub rejected_acks: u64,
    pub lagged: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct PredictionDiagnostics {
    pub fixed_steps_observed: u64,
    pub commands_applied: u64,
    pub replayed: u64,
    pub corrected: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, ecs::Component, ecs::Resource)]
pub struct NetDiagnosticsView {
    pub connected: bool,
    pub connection_id: Option<ConnectionId>,
    pub accepted_connections: u64,
    pub rejected_connections: u64,
    pub reconnect_attempts: u64,
    pub close_events: u64,
    pub error_events: u64,
    pub reconnect_events: u64,
    pub last_rtt_millis: Option<u32>,
    pub emitted_snapshots: u64,
    pub applied_snapshots: u64,
    pub acked_snapshots: u64,
    pub lagged_inputs: u64,
    pub corrected_predictions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_accepts_only_sent_and_available_baselines() {
        let mut checkpoint = ConnectionBaselineCheckpoint::default();
        checkpoint.mark_snapshot_sent(SnapshotCursor(1), SimulationTick(1), true);

        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(1), true),
            SnapshotAckOutcome::Accepted {
                cursor: SnapshotCursor(1)
            }
        );
        assert_eq!(checkpoint.last_ack_cursor, SnapshotCursor(1));
        assert!(!checkpoint.needs_full_resync);
    }

    #[test]
    fn checkpoint_rejects_stale_future_unsent_and_pruned_acks() {
        let mut checkpoint = ConnectionBaselineCheckpoint::default();
        checkpoint.mark_snapshot_sent(SnapshotCursor(1), SimulationTick(1), true);
        checkpoint.mark_snapshot_sent(SnapshotCursor(3), SimulationTick(3), false);
        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(1), true),
            SnapshotAckOutcome::Accepted {
                cursor: SnapshotCursor(1)
            }
        );

        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(1), true),
            SnapshotAckOutcome::Rejected {
                cursor: SnapshotCursor(1),
                reason: SnapshotAckRejection::StaleCursor {
                    last_acknowledged: SnapshotCursor(1)
                }
            }
        );
        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(99), false),
            SnapshotAckOutcome::Rejected {
                cursor: SnapshotCursor(99),
                reason: SnapshotAckRejection::FutureCursor {
                    latest_cursor: SnapshotCursor(3)
                }
            }
        );
        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(2), false),
            SnapshotAckOutcome::Rejected {
                cursor: SnapshotCursor(2),
                reason: SnapshotAckRejection::UnsentCursor
            }
        );
        checkpoint.mark_snapshot_sent(SnapshotCursor(4), SimulationTick(4), false);
        assert_eq!(
            checkpoint.mark_snapshot_acknowledged(SnapshotCursor(4), false),
            SnapshotAckOutcome::Rejected {
                cursor: SnapshotCursor(4),
                reason: SnapshotAckRejection::PrunedCursor
            }
        );
    }
}
