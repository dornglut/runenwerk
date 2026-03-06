// Owner: Grotto Engine Net - QUIC Tests
mod tests {
    use super::*;
    use engine_net::{
        Ack, ClientCommandEnvelope, ClientMessage, ConnectionId, DeltaSnapshot, InputFrame,
        MoveCommand, ProtocolVersion, Snapshot, SnapshotCursor,
    };

    include!("tests/transport_handshake.rs");
    include!("tests/runtime_exchange.rs");
    include!("tests/server_lifecycle.rs");
    include!("tests/disconnect_reconnect.rs");
}
