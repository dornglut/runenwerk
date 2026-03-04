mod protocol;
mod replication;
mod session;
mod simulation;
mod transport;

pub use engine_sim::{
    ActorId, AuthorityRole, CommandSource, DeterminismLevel, SimulationCommandFrame,
    SimulationHash, SimulationProfile, SimulationProfileConfig, SimulationRng, SimulationSeed,
    SimulationSessionId, SimulationTick,
};
pub use protocol::{
    Ack, ClientMessage, DeltaSnapshot, DisconnectReason, Hello, InputFrame, JoinAccepted,
    JoinRejected, JoinRequest, MessageEnvelope, ProtocolVersion, RunEvent, RunResult,
    ServerMessage, Snapshot, decode_message, encode_message,
};
pub use replication::{Replicate, Replicated, SnapshotCursor};
pub use session::{
    AuthoritativeJoinState, ClientSessionState, ClientSessionTarget, ServerSessionConfig,
    ServerSessionState, SessionPhase, SessionRuntimeCommand, SessionRuntimeEvent,
    begin_client_session, configure_server_session, handle_client_message, observe_server_message,
    remove_server_connection,
};
#[allow(deprecated)]
pub use simulation::{
    AbilityCommand, AimCommand, Authoritative, CanonicalCommandFrame, ClientClock,
    ClientCommandEnvelope, InteractCommand, Interpolated, MoveCommand, NetworkEntityId,
    NetworkTick, PlayerCommandBuffer, Predicted, RecordedCommand, ReplicationScope, ServerClock,
    SimulationRole,
};
pub use transport::{ConnectionId, Transport, TransportKind};
