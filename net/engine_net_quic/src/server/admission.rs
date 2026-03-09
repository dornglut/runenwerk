use engine_net::ServerSessionState;
use quinn::Connection;

#[derive(Debug, Clone)]
pub struct QuicServerBootstrap {
    pub connection: Connection,
    pub state: ServerSessionState,
}
