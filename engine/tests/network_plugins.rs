use engine::net::prelude::*;
use engine::plugins::net::{
    ClientSnapshotReplicationState, NetworkAdmissionState, NetworkClientInbox, NetworkClientOutbox,
    NetworkDiagnostics, NetworkOutboundQueue, NetworkRuntimeHandle, NetworkServerInbox,
    NetworkServerOutbox, NetworkSessionStatus, OutboundServerMessage, PredictionDiagnostics,
    PredictionState as NetPredictionState, ReplicationDiagnostics, ServerSnapshotReplicationState,
    client_inbox_is_empty, client_outbox_len, enqueue_client_inbox, enqueue_client_outbox,
    enqueue_server_inbox, enqueue_server_inbox_from, enqueue_server_outbox_broadcast,
    server_inbox_is_empty, server_outbox_len,
};
use engine::plugins::{ScenePlugin, default_plugins};
use engine::prelude::*;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct MoveCommand {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AbilityCommand {
    slot: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum ClientCommandEnvelope {
    Move(MoveCommand),
    Ability(AbilityCommand),
}

impl Default for ClientCommandEnvelope {
    fn default() -> Self {
        Self::Move(MoveCommand { x: 0.0, y: 0.0 })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Component, ecs::Resource)]
struct PlayerCommandBuffer {
    commands: Vec<ClientCommandEnvelope>,
}

impl PlayerCommandBuffer {
    fn push(&mut self, command: ClientCommandEnvelope) {
        self.commands.push(command);
    }

    fn drain(&mut self) -> Vec<ClientCommandEnvelope> {
        std::mem::take(&mut self.commands)
    }

    fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestSnapshot {
    context: TestSnapshotContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestSnapshotContext {
    world_scene_label: String,
}

impl Default for TestSnapshot {
    fn default() -> Self {
        Self {
            context: TestSnapshotContext {
                world_scene_label: "gameplay_stub".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct TestDelta {
    changed: bool,
}

struct TestReplicationDriver;

impl ReplicationDriver for TestReplicationDriver {
    type Snapshot = TestSnapshot;
    type Delta = TestDelta;
    type Input = ClientCommandEnvelope;
    type Error = io::Error;

    fn capture_snapshot(_world: &World) -> Result<Option<Self::Snapshot>, Self::Error> {
        Ok(Some(TestSnapshot::default()))
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        TestDelta {
            changed: previous != current,
        }
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        if delta.changed {
            Self::Snapshot::default()
        } else {
            base.clone()
        }
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}

impl SnapshotApplyDriver for TestReplicationDriver {
    fn apply_snapshot(
        _world: &mut World,
        _tick: engine_sim::SimulationTick,
        _snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn apply_delta(
        _world: &mut World,
        _tick: engine_sim::SimulationTick,
        _delta: Self::Delta,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl InputDriver for TestReplicationDriver {
    fn receive_remote_input(
        _world: &mut World,
        _connection_id: ConnectionId,
        _tick: engine_sim::SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_local_input(world: &mut World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(world
            .resource_mut::<PlayerCommandBuffer>()
            .map(|commands| commands.drain())
            .unwrap_or_default())
    }

    fn apply_input(_world: &mut World, _input: &[Self::Input]) -> Result<(), Self::Error> {
        Ok(())
    }
}

type PredictionState = NetPredictionState<ClientCommandEnvelope>;
type ClientSnapshotState = ClientSnapshotReplicationState<TestSnapshot>;
type ServerSnapshotState = ServerSnapshotReplicationState<TestSnapshot>;

struct NetworkClientPlugin;

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(NetPlugin::<TestReplicationDriver>::new(NetRole::Client));
    }
}

struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(NetPlugin::<TestReplicationDriver>::new(NetRole::Server));
    }
}

struct NetworkHostPlugin;

impl Plugin for NetworkHostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCommandBuffer>();
        app.add_plugin(NetPlugin::<TestReplicationDriver>::new(NetRole::Host));
    }
}

include!("network_plugins/basic_flow.rs");

include!("network_plugins/runtime_and_replication.rs");

include!("network_plugins/delta_and_reconnect.rs");
