use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::{
    SceneResource, SceneSimulationDeltaV1, SceneSimulationSnapshotV1, apply_scene_simulation_delta,
    build_scene_simulation_delta, capture_scene_simulation_snapshot, republish_scene_resources,
    restore_scene_simulation_snapshot,
};
use crate::runtime::{CoreSet, FixedUpdate, FrameEnd, PreUpdate, SystemConfigExt, WorldMut};
use crate::state::SessionRuntimeState;
use engine_net::{
    Ack, AuthoritativeJoinState, AuthorityRole, ClientCommandEnvelope, ClientMessage,
    ClientSessionState, ConnectionId, DeltaSnapshot, DisconnectReason, InputFrame,
    PlayerCommandBuffer, ServerMessage, ServerSessionConfig, ServerSessionState, SessionPhase,
    SessionRuntimeCommand, SessionRuntimeEvent, SimulationProfile, SimulationProfileConfig,
    SimulationTick, Snapshot, SnapshotCursor, handle_client_message, observe_server_message,
    remove_server_connection,
};
use std::mem;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};

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

fn network_runtime_receive_system(mut world: WorldMut) -> anyhow::Result<()> {
    let Some(mut handle) = world.remove_resource::<NetworkRuntimeHandle>() else {
        if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
            inbound.clear();
        }
        return Ok(());
    };

    if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
        inbound.clear();
    }

    loop {
        let event = match handle.try_recv() {
            Ok(Some(event)) => event,
            Ok(None) => break,
            Err(TryRecvError::Disconnected) => {
                update_connection_closed(&mut world, None, None);
                break;
            }
            Err(TryRecvError::Empty) => break,
        };

        match event {
            SessionRuntimeEvent::Connected { connection_id } => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = true;
                    status.connection_id = connection_id;
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = true;
                }
            }
            SessionRuntimeEvent::ClientMessage {
                connection_id,
                message,
            } => {
                if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_client(connection_id, message.clone());
                }
                if let Ok(mut inbox) = world.resource_mut::<NetworkServerInbox>() {
                    inbox.push(message);
                }
            }
            SessionRuntimeEvent::ServerMessage(message) => {
                if let Ok(mut inbound) = world.resource_mut::<NetworkInboundQueue>() {
                    inbound.push_server(message.clone());
                }
                if let Ok(mut inbox) = world.resource_mut::<NetworkClientInbox>() {
                    inbox.push(message);
                }
            }
            SessionRuntimeEvent::Phase(phase) => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = phase;
                }
            }
            SessionRuntimeEvent::Reconnecting { attempt } => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.connected = false;
                    status.reconnect_attempt = Some(attempt);
                    status.phase = SessionPhase::Handshaking;
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.connected = false;
                    health.reconnect_events = health.reconnect_events.saturating_add(1);
                }
                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.reconnect_attempts =
                        diagnostics.reconnect_attempts.saturating_add(1);
                }
            }
            SessionRuntimeEvent::JoinAccepted(join) => {
                let authority = world
                    .resource::<SimulationProfileConfig>()
                    .map(|config| config.authority)
                    .unwrap_or(AuthorityRole::Local);
                if matches!(authority, AuthorityRole::Server)
                    && let Ok(mut session) = world.resource_mut::<ServerSessionState>()
                {
                    let connection = ConnectionId(join.connection_id);
                    session.phase = SessionPhase::Active;
                    session.active_connection = Some(connection);
                    session.active_connections.insert(connection);
                    session.last_disconnect = None;
                    session.last_join_state = Some(join.join_state.clone());
                }
                if matches!(authority, AuthorityRole::Client | AuthorityRole::Peer)
                    && let Ok(mut session) = world.resource_mut::<ClientSessionState>()
                {
                    observe_server_message(
                        &mut session,
                        &ServerMessage::JoinAccepted(join.clone()),
                    );
                }
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = SessionPhase::Active;
                    status.connected = true;
                    status.connection_id = Some(ConnectionId(join.connection_id));
                    status.last_disconnect = None;
                    status.reconnect_attempt = None;
                }
                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.accepted_connections =
                        diagnostics.accepted_connections.saturating_add(1);
                }
                if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
                    admission.authoritative_join = Some(join.join_state.clone());
                }
                apply_session_runtime_join_state(&mut world, &join.join_state);
            }
            SessionRuntimeEvent::JoinRejected(reason) => {
                if let Ok(mut session) = world.resource_mut::<ClientSessionState>() {
                    observe_server_message(
                        &mut session,
                        &ServerMessage::JoinRejected(engine_net::JoinRejected {
                            reason: reason.clone(),
                        }),
                    );
                }
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.phase = SessionPhase::Rejected(reason.clone());
                    status.last_disconnect = Some(reason.clone());
                }
                clear_session_runtime_state(&mut world);
                if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
                    diagnostics.rejected_connections =
                        diagnostics.rejected_connections.saturating_add(1);
                }
            }
            SessionRuntimeEvent::RttUpdated { millis } => {
                if let Ok(mut metrics) = world.resource_mut::<RoundTripMetrics>() {
                    metrics.last_rtt_millis = Some(millis);
                    metrics.samples = metrics.samples.saturating_add(1);
                }
            }
            SessionRuntimeEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                update_connection_closed(&mut world, connection_id, reason);
            }
            SessionRuntimeEvent::Error { message } => {
                if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                    status.last_error = Some(message);
                }
                if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
                    health.error_events = health.error_events.saturating_add(1);
                }
            }
        }
    }

    world.insert_resource(handle);
    Ok(())
}

fn client_receive_system(mut world: WorldMut) -> anyhow::Result<()> {
    let len = world.resource::<NetworkClientInbox>()?.len();
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_server_messages_last_frame = len;
    }
    let messages = world.resource_mut::<NetworkClientInbox>()?.drain();

    for message in messages {
        let previous_phase = world
            .resource::<ClientSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();
        if let Ok(mut session) = world.resource_mut::<ClientSessionState>() {
            observe_server_message(&mut session, &message);
            let phase = session.phase.clone();
            let connection_id = session.connection_id;
            let last_disconnect = session.last_disconnect.clone();
            drop(session);
            if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
                status.phase = phase.clone();
                status.connection_id = connection_id;
                status.last_disconnect = last_disconnect;
                status.connected = matches!(phase, SessionPhase::Active);
            }
        }

        match message {
            ServerMessage::JoinAccepted(join) => {
                if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
                    admission.authoritative_join = Some(join.join_state.clone());
                }
                apply_session_runtime_join_state(&mut world, &join.join_state);
            }
            ServerMessage::Snapshot(snapshot) => {
                let corrected = apply_authoritative_snapshot(
                    &mut world,
                    snapshot.tick,
                    snapshot.cursor,
                    None,
                    &snapshot.payload,
                )?;
                if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
                    outbox.push(ClientMessage::Ack(Ack {
                        cursor: snapshot.cursor,
                        last_received_tick: snapshot.tick,
                    }));
                }
                if corrected
                    && let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>()
                {
                    diagnostics.corrections_applied =
                        diagnostics.corrections_applied.saturating_add(1);
                }
            }
            ServerMessage::DeltaSnapshot(snapshot) => {
                let corrected = apply_authoritative_delta(
                    &mut world,
                    snapshot.tick,
                    snapshot.cursor,
                    &snapshot.payload,
                )?;
                if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
                    outbox.push(ClientMessage::Ack(Ack {
                        cursor: snapshot.cursor,
                        last_received_tick: snapshot.tick,
                    }));
                }
                if corrected
                    && let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>()
                {
                    diagnostics.corrections_applied =
                        diagnostics.corrections_applied.saturating_add(1);
                }
            }
            _ => {}
        }

        let phase = world
            .resource::<ClientSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();
        if !matches!(previous_phase, SessionPhase::Active)
            && matches!(phase, SessionPhase::Active)
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics.accepted_connections.saturating_add(1);
        }
        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    Ok(())
}

fn server_receive_system(mut world: WorldMut) -> anyhow::Result<()> {
    let config = world
        .resource::<ServerSessionConfig>()
        .map(|resource| resource.clone())
        .unwrap_or_default();
    if let Ok(mut session) = world.resource_mut::<ServerSessionState>()
        && session.config != config
    {
        session.config = config;
    }

    let len = world.resource::<NetworkServerInbox>()?.len();
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.processed_client_messages_last_frame = len;
    }
    let messages = world.resource_mut::<NetworkServerInbox>()?.drain();

    for message in messages {
        if let ClientMessage::Ack(ack) = &message
            && let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>()
        {
            state.last_acknowledged_cursor = ack.cursor;
            state.last_received_tick = ack.last_received_tick;
        }
        if let ClientMessage::InputFrame(frame) = &message
            && let Ok(mut buffer) = world.resource_mut::<PlayerCommandBuffer>()
        {
            for command in &frame.commands {
                buffer.push(command.clone());
            }
        }

        let (previous_phase, previous_connection) = world
            .resource::<ServerSessionState>()
            .map(|session| (session.phase.clone(), session.active_connection))
            .unwrap_or((SessionPhase::Idle, None));
        let responses = {
            let mut session = world.resource_mut::<ServerSessionState>()?;
            handle_client_message(&mut session, &message)
        };
        if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
            for response in responses {
                outbox.push(response);
            }
        }
        let phase = world
            .resource::<ServerSessionState>()
            .map(|session| session.phase.clone())
            .unwrap_or_default();
        let current_connection = world
            .resource::<ServerSessionState>()
            .ok()
            .and_then(|session| session.active_connection);
        if current_connection != previous_connection
            && current_connection.is_some()
            && let Ok(mut replication) = world.resource_mut::<SnapshotReplicationState>()
        {
            reset_replication_for_connection(&mut replication, current_connection);
        }
        let latest_join_state = world
            .resource::<ServerSessionState>()
            .ok()
            .and_then(|session| session.last_join_state.clone());
        if let Ok(mut admission) = world.resource_mut::<NetworkAdmissionState>() {
            admission.authoritative_join = latest_join_state.clone();
        }
        if let Some(join_state) = latest_join_state.as_ref() {
            apply_session_runtime_join_state(&mut world, join_state);
        }
        let session_state = world.resource::<ServerSessionState>().ok().map(|session| {
            (
                session.phase.clone(),
                session.active_connection,
                !session.active_connections.is_empty(),
                session.last_disconnect.clone(),
            )
        });
        if let Some((phase, connection_id, has_active_connections, last_disconnect)) = session_state
            && let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>()
        {
            status.phase = phase.clone();
            status.connection_id = connection_id;
            status.last_disconnect = last_disconnect;
            status.connected = has_active_connections;
        }
        if !matches!(previous_phase, SessionPhase::Active)
            && matches!(phase, SessionPhase::Active)
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.accepted_connections = diagnostics.accepted_connections.saturating_add(1);
        }
        if !matches!(previous_phase, SessionPhase::Rejected(_))
            && matches!(phase, SessionPhase::Rejected(_))
            && let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>()
        {
            diagnostics.rejected_connections = diagnostics.rejected_connections.saturating_add(1);
        }
    }

    Ok(())
}

fn apply_session_runtime_join_state(world: &mut ecs::World, join_state: &AuthoritativeJoinState) {
    if let Ok(mut session) = world.resource_mut::<SessionRuntimeState>() {
        session.apply_authoritative_join(join_state);
    }
}

fn clear_session_runtime_state(world: &mut ecs::World) {
    if let Ok(mut session) = world.resource_mut::<SessionRuntimeState>() {
        session.clear();
    }
}

fn client_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    let len = world.resource::<NetworkClientOutbox>()?.len();
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_client_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }
    let messages = world.resource_mut::<NetworkClientOutbox>()?.drain();
    if let Ok(mut queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_client(message.clone());
        }
    }
    if let Some(handle) = world.resource::<NetworkRuntimeHandle>().ok() {
        for message in &messages {
            let _ = handle.send(SessionRuntimeCommand::Client(message.clone()));
        }
    }
    Ok(())
}

fn server_flush_system(mut world: WorldMut) -> anyhow::Result<()> {
    let len = world.resource::<NetworkServerOutbox>()?.len();
    if let Ok(mut diagnostics) = world.resource_mut::<NetworkDiagnostics>() {
        diagnostics.flushed_server_messages_last_frame = len;
        if len > 0 {
            diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
        }
    }
    let messages = world.resource_mut::<NetworkServerOutbox>()?.drain();
    if let Ok(mut queue) = world.resource_mut::<NetworkOutboundQueue>() {
        queue.clear();
        for message in &messages {
            queue.push_server(message.clone());
        }
    }
    if let Some(handle) = world.resource::<NetworkRuntimeHandle>().ok() {
        for message in &messages {
            let _ = handle.send(SessionRuntimeCommand::Server(message.clone()));
        }
    }
    Ok(())
}

fn replication_step_system(mut world: WorldMut) -> anyhow::Result<()> {
    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        let cursor = world
            .resource::<SnapshotCursor>()
            .map(|cursor| cursor.0)
            .unwrap_or(0);
        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.last_snapshot_cursor = cursor;
        }
        return Ok(());
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let cursor = {
        let mut cursor = world.resource_mut::<SnapshotCursor>()?;
        cursor.0 = cursor.0.saturating_add(1);
        *cursor
    };
    let captured_snapshot = capture_scene_snapshot(&world)?;

    let last_ack = world
        .resource::<SnapshotReplicationState>()
        .map(|state| state.last_acknowledged_cursor)
        .unwrap_or_default();
    let initial_snapshot_sent = world
        .resource::<SnapshotReplicationState>()
        .map(|state| state.initial_snapshot_sent)
        .unwrap_or(false);
    let last_sent_snapshot = world
        .resource::<SnapshotReplicationState>()
        .ok()
        .and_then(|state| state.last_sent_snapshot.clone());

    if let Some(snapshot) = captured_snapshot
        && let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>()
    {
        if initial_snapshot_sent {
            let base_snapshot = last_sent_snapshot.unwrap_or_else(|| snapshot.clone());
            let delta = build_scene_simulation_delta(&base_snapshot, &snapshot);
            outbox.push(ServerMessage::DeltaSnapshot(DeltaSnapshot {
                tick,
                base: last_ack,
                cursor,
                entity_ids: Vec::new(),
                payload: postcard::to_allocvec(&delta)?,
            }));
        } else {
            outbox.push(ServerMessage::Snapshot(Snapshot {
                tick,
                cursor,
                last_applied: last_ack,
                entity_ids: Vec::new(),
                payload: postcard::to_allocvec(&snapshot)?,
            }));
        }

        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>() {
            state.initial_snapshot_sent = true;
            state.last_sent_cursor = cursor;
            state.last_sent_snapshot = Some(snapshot);
        }
        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.emitted_snapshots = diagnostics.emitted_snapshots.saturating_add(1);
        }
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.last_snapshot_cursor = cursor.0;
    }
    Ok(())
}

fn prediction_step_system(mut world: WorldMut) -> anyhow::Result<()> {
    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let commands = {
        let mut commands = world.resource_mut::<PlayerCommandBuffer>()?;
        commands.drain()
    };
    if commands.is_empty() {
        return Ok(());
    }

    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.commands_applied = diagnostics
            .commands_applied
            .saturating_add(commands.len() as u64);
    }

    if matches!(authority, AuthorityRole::Client) {
        if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
            outbox.push(ClientMessage::InputFrame(InputFrame {
                tick,
                commands: commands.clone(),
            }));
        }
        if let Ok(mut prediction) = world.resource_mut::<PredictionState>() {
            prediction.pending_frames.push(InputFrame {
                tick,
                commands: commands.clone(),
            });
        }
    }

    apply_commands_to_scene(&mut world, &commands)?;
    let _ = republish_scene_resources(&mut world);
    Ok(())
}

fn update_connection_closed(
    world: &mut ecs::World,
    connection_id: Option<ConnectionId>,
    reason: Option<DisconnectReason>,
) {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    if matches!(authority, AuthorityRole::Server) {
        let mut active_connection = None;
        let mut has_active_connections = false;
        if let Ok(mut session) = world.resource_mut::<ServerSessionState>() {
            match connection_id {
                Some(connection_id) => {
                    remove_server_connection(&mut session, connection_id, reason.clone());
                }
                None => {
                    session.active_connections.clear();
                    session.active_connection = None;
                    session.phase = SessionPhase::Closed;
                    session.last_disconnect = reason.clone();
                }
            }
            active_connection = session.active_connection;
            has_active_connections = !session.active_connections.is_empty();
        }
        if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
            status.connected = has_active_connections;
            status.phase = if has_active_connections {
                SessionPhase::Active
            } else {
                SessionPhase::Closed
            };
            status.connection_id = active_connection;
            status.last_disconnect = reason.clone();
        }
        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>()
            && (connection_id.is_none() || state.active_connection == connection_id)
        {
            reset_replication_for_connection(&mut state, active_connection);
        }
    } else if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
        status.connected = false;
        status.phase = SessionPhase::Closed;
        status.last_disconnect = reason.clone();
    }
    if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
        health.connected = false;
        health.close_events = health.close_events.saturating_add(1);
    }
}

fn reset_replication_for_connection(
    state: &mut SnapshotReplicationState,
    connection: Option<ConnectionId>,
) {
    state.active_connection = connection;
    state.initial_snapshot_sent = false;
    state.last_sent_cursor = SnapshotCursor::default();
    state.last_acknowledged_cursor = SnapshotCursor::default();
    state.last_received_tick = SimulationTick::default();
    state.last_sent_snapshot = None;
    state.last_received_snapshot = None;
}

fn capture_scene_snapshot(world: &ecs::World) -> anyhow::Result<Option<SceneSimulationSnapshotV1>> {
    let Some(scene_resource) = world.resource::<SceneResource>().ok() else {
        return Ok(None);
    };
    let Some(manager) = scene_resource.manager.as_ref() else {
        return Ok(None);
    };
    Ok(Some(capture_scene_simulation_snapshot(manager)?))
}

fn apply_authoritative_snapshot(
    world: &mut ecs::World,
    tick: SimulationTick,
    cursor: SnapshotCursor,
    snapshot: Option<SceneSimulationSnapshotV1>,
    payload: &[u8],
) -> anyhow::Result<bool> {
    let snapshot = match snapshot {
        Some(snapshot) => snapshot,
        None => postcard::from_bytes(payload)?,
    };
    let previous_prediction = world.resource::<SceneResource>().ok().and_then(|scene| {
        scene.manager.as_ref().map(|manager| {
            (
                manager.world_runtime.ctx.player_move_x,
                manager.world_runtime.ctx.player_move_y,
                manager.world_runtime.ctx.camera_yaw,
                manager.world_runtime.ctx.camera_pitch,
            )
        })
    });

    ensure_scene_manager(world)?;
    {
        let mut scene_resource = world.resource_mut::<SceneResource>()?;
        let manager = scene_resource
            .manager
            .as_mut()
            .expect("scene manager should exist after initialization");
        restore_scene_simulation_snapshot(manager, &snapshot)?;
    }
    republish_scene_resources(world)?;

    if let Ok(mut tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }
    if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>() {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(snapshot.clone());
    }
    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    let corrected = previous_prediction
        .map(|prediction| {
            prediction
                != (
                    snapshot.context.player_move_x,
                    snapshot.context.player_move_y,
                    snapshot.context.camera_yaw,
                    snapshot.context.camera_pitch,
                )
        })
        .unwrap_or(false);

    let pending_frames = {
        let mut prediction = world.resource_mut::<PredictionState>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > tick.0);
        prediction.pending_frames.clone()
    };
    for frame in pending_frames {
        apply_commands_to_scene(world, &frame.commands)?;
    }
    republish_scene_resources(world)?;
    Ok(corrected)
}

fn apply_authoritative_delta(
    world: &mut ecs::World,
    tick: SimulationTick,
    cursor: SnapshotCursor,
    payload: &[u8],
) -> anyhow::Result<bool> {
    let delta: SceneSimulationDeltaV1 = postcard::from_bytes(payload)?;
    let base_snapshot = world
        .resource::<SnapshotReplicationState>()
        .ok()
        .and_then(|state| state.last_received_snapshot.clone())
        .ok_or_else(|| anyhow::anyhow!("received delta snapshot without a baseline snapshot"))?;
    let snapshot = apply_scene_simulation_delta(&base_snapshot, &delta);
    apply_authoritative_snapshot(world, tick, cursor, Some(snapshot), payload)
}

fn ensure_scene_manager(world: &mut ecs::World) -> anyhow::Result<()> {
    let has_manager = world
        .resource::<SceneResource>()
        .ok()
        .map(|resource| resource.manager.is_some())
        .unwrap_or(false);
    if has_manager {
        return Ok(());
    }

    let window = world.resource::<crate::runtime::WindowState>()?.clone();
    let mut scene_resource = world.resource_mut::<SceneResource>()?;
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(crate::plugins::scene::SceneManager::new(&window)?);
    }
    Ok(())
}

fn apply_commands_to_scene(
    world: &mut ecs::World,
    commands: &[ClientCommandEnvelope],
) -> anyhow::Result<()> {
    let Ok(mut scene_resource) = world.resource_mut::<SceneResource>() else {
        return Ok(());
    };
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    for command in commands {
        match command {
            ClientCommandEnvelope::Move(move_cmd) => {
                manager.world_runtime.ctx.player_move_x = move_cmd.x;
                manager.world_runtime.ctx.player_move_y = move_cmd.y;
            }
            ClientCommandEnvelope::Aim(aim_cmd) => {
                manager.world_runtime.ctx.camera_yaw = aim_cmd.x;
                manager.world_runtime.ctx.camera_pitch = aim_cmd.y;
            }
            ClientCommandEnvelope::Ability(_) | ClientCommandEnvelope::Interact(_) => {}
        }
    }
    Ok(())
}
