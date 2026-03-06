use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::{
    SceneResource, SceneSimulationDeltaV1, SceneSimulationSnapshotV1, apply_scene_simulation_delta,
    build_scene_simulation_delta, capture_scene_simulation_snapshot, republish_scene_resources,
    restore_scene_simulation_snapshot,
};
use crate::runtime::{CoreSet, FixedUpdate, FrameEnd, PreUpdate, SystemConfigExt, WorldMut};
use crate::state::SessionRuntimeState;
use anyhow::Context;
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

include!("net_internal/resources_and_plugins.rs");

include!("net_internal/runtime_io.rs");

include!("net_internal/replication_prediction.rs");
