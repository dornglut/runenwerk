// Owner: Engine Network Plugin - Resources and Plugin Wiring
#[derive(Debug, Clone, Default)]
pub struct NetworkClientInbox {
    messages: Vec<ServerMessage>,
}

impl NetworkClientInbox {
    pub fn push(&mut self, message: ServerMessage) {
        self.messages.push(message);
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

#[derive(Debug, Clone, Default)]
pub struct NetworkServerInbox {
    messages: Vec<ClientMessage>,
}

impl NetworkServerInbox {
    pub fn push(&mut self, message: ClientMessage) {
        self.messages.push(message);
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

#[derive(Debug, Clone, Default)]
pub struct NetworkClientOutbox {
    messages: Vec<ClientMessage>,
}

impl NetworkClientOutbox {
    pub fn push(&mut self, message: ClientMessage) {
        self.messages.push(message);
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn drain(&mut self) -> Vec<ClientMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkServerOutbox {
    messages: Vec<ServerMessage>,
}

impl NetworkServerOutbox {
    pub fn push(&mut self, message: ServerMessage) {
        self.messages.push(message);
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn drain(&mut self) -> Vec<ServerMessage> {
        mem::take(&mut self.messages)
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub struct NetworkOutboundQueue {
    client_messages: Vec<ClientMessage>,
    server_messages: Vec<ServerMessage>,
}

impl NetworkOutboundQueue {
    pub fn clear(&mut self) {
        self.client_messages.clear();
        self.server_messages.clear();
    }

    pub fn push_client(&mut self, message: ClientMessage) {
        self.client_messages.push(message);
    }

    pub fn push_server(&mut self, message: ServerMessage) {
        self.server_messages.push(message);
    }

    pub fn client_messages(&self) -> &[ClientMessage] {
        &self.client_messages
    }

    pub fn server_messages(&self) -> &[ServerMessage] {
        &self.server_messages
    }
}

pub struct NetworkRuntimeHandle {
    command_tx: UnboundedSender<SessionRuntimeCommand>,
    event_rx: UnboundedReceiver<SessionRuntimeEvent>,
}

impl NetworkRuntimeHandle {
    pub fn new(
        command_tx: UnboundedSender<SessionRuntimeCommand>,
        event_rx: UnboundedReceiver<SessionRuntimeEvent>,
    ) -> Self {
        Self {
            command_tx,
            event_rx,
        }
    }

    pub fn send(&self, command: SessionRuntimeCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .map_err(|error| format!("network runtime send failed: {error}"))
    }

    fn try_recv(&mut self) -> Result<Option<SessionRuntimeEvent>, TryRecvError> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkSessionStatus {
    pub phase: SessionPhase,
    pub connection_id: Option<ConnectionId>,
    pub last_disconnect: Option<DisconnectReason>,
    pub last_error: Option<String>,
    pub connected: bool,
    pub reconnect_attempt: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkAdmissionState {
    pub authoritative_join: Option<AuthoritativeJoinState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionHealth {
    pub connected: bool,
    pub close_events: u64,
    pub error_events: u64,
    pub reconnect_events: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct RoundTripMetrics {
    pub last_rtt_millis: Option<u32>,
    pub samples: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SnapshotReplicationState {
    pub active_connection: Option<ConnectionId>,
    pub initial_snapshot_sent: bool,
    pub last_sent_cursor: SnapshotCursor,
    pub last_acknowledged_cursor: SnapshotCursor,
    pub last_received_tick: SimulationTick,
    pub applied_snapshots: u64,
    pub last_sent_snapshot: Option<SceneSimulationSnapshotV1>,
    pub last_received_snapshot: Option<SceneSimulationSnapshotV1>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PredictionState {
    pending_frames: Vec<InputFrame>,
}

impl PredictionState {
    pub fn pending_frames_len(&self) -> usize {
        self.pending_frames.len()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct ReplicationDiagnostics {
    pub fixed_steps_observed: u64,
    pub last_snapshot_cursor: u64,
    pub emitted_snapshots: u64,
    pub applied_snapshots: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PredictionDiagnostics {
    pub fixed_steps_observed: u64,
    pub commands_applied: u64,
    pub corrections_applied: u64,
}

pub struct NetworkClientPlugin;
pub struct NetworkServerPlugin;
pub struct ReplicationPlugin;
pub struct PredictionPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(mut config) = app.world_mut().resource_mut::<SimulationProfileConfig>() {
            config.authority = AuthorityRole::Client;
            if matches!(config.profile, SimulationProfile::LocalSinglePlayer) {
                config.profile = SimulationProfile::DedicatedAuthority;
            }
        }
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
        app.add_systems(
            PreUpdate,
            network_runtime_receive_system.in_set(CoreSet::NetReceive),
        );
        app.add_systems(PreUpdate, client_receive_system.in_set(CoreSet::NetReceive));
        app.add_systems(FrameEnd, client_flush_system.in_set(CoreSet::FrameEnd));
    }
}

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        if let Ok(mut config) = app.world_mut().resource_mut::<SimulationProfileConfig>() {
            config.authority = AuthorityRole::Server;
            if matches!(config.profile, SimulationProfile::LocalSinglePlayer) {
                config.profile = SimulationProfile::DedicatedAuthority;
            }
        }
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
        app.add_systems(
            PreUpdate,
            network_runtime_receive_system.in_set(CoreSet::NetReceive),
        );
        app.add_systems(PreUpdate, server_receive_system.in_set(CoreSet::NetReceive));
        app.add_systems(FrameEnd, server_flush_system.in_set(CoreSet::FrameEnd));
    }
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnapshotCursor>();
        app.init_resource::<SnapshotReplicationState>();
        app.init_resource::<ReplicationDiagnostics>();
        app.add_systems(
            FixedUpdate,
            replication_step_system.in_set(CoreSet::Replication),
        );
    }
}

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.init_resource::<PredictionState>();
        app.init_resource::<PredictionDiagnostics>();
        app.add_systems(
            FixedUpdate,
            prediction_step_system.in_set(CoreSet::Simulation),
        );
    }
}
