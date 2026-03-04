use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FixedUpdate, FrameEnd, PreUpdate, Res, ResMut, SystemConfigExt};
use engine_net::{
    ClientMessage, PlayerCommandBuffer, ServerMessage, SimulationRole, SnapshotCursor,
};

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
        std::mem::take(&mut self.messages)
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
        std::mem::take(&mut self.messages)
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
        std::mem::take(&mut self.messages)
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
        std::mem::take(&mut self.messages)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct NetworkDiagnostics {
    pub processed_client_messages_last_frame: usize,
    pub processed_server_messages_last_frame: usize,
    pub flushed_client_messages_last_frame: usize,
    pub flushed_server_messages_last_frame: usize,
    pub flush_count: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct ReplicationDiagnostics {
    pub fixed_steps_observed: u64,
    pub last_snapshot_cursor: u64,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PredictionDiagnostics {
    pub fixed_steps_observed: u64,
    pub commands_applied: u64,
}

pub struct NetworkClientPlugin;
pub struct NetworkServerPlugin;
pub struct ReplicationPlugin;
pub struct PredictionPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationRole::Client);
        app.init_resource::<NetworkClientInbox>();
        app.init_resource::<NetworkClientOutbox>();
        app.init_resource::<NetworkDiagnostics>();
        app.add_systems(PreUpdate, client_receive_system.in_set(CoreSet::NetReceive));
        app.add_systems(FrameEnd, client_flush_system.in_set(CoreSet::FrameEnd));
    }
}

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationRole::Server);
        app.init_resource::<NetworkServerInbox>();
        app.init_resource::<NetworkServerOutbox>();
        app.init_resource::<NetworkDiagnostics>();
        app.add_systems(PreUpdate, server_receive_system.in_set(CoreSet::NetReceive));
        app.add_systems(FrameEnd, server_flush_system.in_set(CoreSet::FrameEnd));
    }
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnapshotCursor>();
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
        app.init_resource::<PredictionDiagnostics>();
        app.add_systems(
            FixedUpdate,
            prediction_step_system.in_set(CoreSet::Simulation),
        );
    }
}

fn client_receive_system(
    mut inbox: ResMut<NetworkClientInbox>,
    mut diagnostics: ResMut<NetworkDiagnostics>,
) {
    diagnostics.processed_server_messages_last_frame = inbox.len();
    inbox.drain();
}

fn server_receive_system(
    mut inbox: ResMut<NetworkServerInbox>,
    mut diagnostics: ResMut<NetworkDiagnostics>,
) {
    diagnostics.processed_client_messages_last_frame = inbox.len();
    inbox.drain();
}

fn client_flush_system(
    mut outbox: ResMut<NetworkClientOutbox>,
    mut diagnostics: ResMut<NetworkDiagnostics>,
) {
    diagnostics.flushed_client_messages_last_frame = outbox.len();
    if diagnostics.flushed_client_messages_last_frame > 0 {
        diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
    }
    outbox.drain();
}

fn server_flush_system(
    mut outbox: ResMut<NetworkServerOutbox>,
    mut diagnostics: ResMut<NetworkDiagnostics>,
) {
    diagnostics.flushed_server_messages_last_frame = outbox.len();
    if diagnostics.flushed_server_messages_last_frame > 0 {
        diagnostics.flush_count = diagnostics.flush_count.saturating_add(1);
    }
    outbox.drain();
}

fn replication_step_system(
    role: Res<SimulationRole>,
    mut cursor: ResMut<SnapshotCursor>,
    mut diagnostics: ResMut<ReplicationDiagnostics>,
) {
    diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    if matches!(*role, SimulationRole::Server) {
        cursor.0 = cursor.0.saturating_add(1);
    }
    diagnostics.last_snapshot_cursor = cursor.0;
}

fn prediction_step_system(
    mut commands: ResMut<PlayerCommandBuffer>,
    mut diagnostics: ResMut<PredictionDiagnostics>,
) {
    diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    diagnostics.commands_applied = diagnostics
        .commands_applied
        .saturating_add(commands.drain().len() as u64);
}
