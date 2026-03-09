use engine_net::{ClientSessionState, JoinAccepted};
use quinn::{Connection, Endpoint};

#[derive(Debug, Clone)]
pub struct QuicClientBootstrap {
    pub endpoint: Endpoint,
    pub connection: Connection,
    pub state: ClientSessionState,
    pub accepted: JoinAccepted,
}
