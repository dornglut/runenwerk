use engine::plugins::scene::SceneSimulationDeltaV1;
use engine::plugins::{
    NetworkAdmissionState, NetworkClientInbox, NetworkClientOutbox, NetworkClientPlugin,
    NetworkDiagnostics, NetworkOutboundQueue, NetworkRuntimeHandle, NetworkServerInbox,
    NetworkServerOutbox, NetworkServerPlugin, NetworkSessionStatus, PredictionDiagnostics,
    PredictionPlugin, ReplicationDiagnostics, ReplicationPlugin, ScenePlugin, default_plugins,
};
use engine::prelude::*;
use engine_net::{
    ClientCommandEnvelope, ClientMessage, ClientSessionState, ClientSessionTarget, Hello,
    MoveCommand, PlayerCommandBuffer, ProtocolVersion, ServerMessage, ServerSessionConfig,
    SessionPhase, SnapshotCursor, TransportKind, begin_client_session,
};

include!("network_plugins/basic_flow.rs");

include!("network_plugins/runtime_and_replication.rs");

include!("network_plugins/delta_and_reconnect.rs");
