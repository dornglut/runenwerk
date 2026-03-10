pub(crate) fn parse_join_rejection_reason(message: &str) -> Option<engine_net::DisconnectReason> {
    if message.contains("WrongServer") {
        return Some(engine_net::DisconnectReason::WrongServer);
    }
    if message.contains("VersionMismatch") {
        return Some(engine_net::DisconnectReason::VersionMismatch);
    }
    if message.contains("InvalidTicket") {
        return Some(engine_net::DisconnectReason::InvalidTicket);
    }
    if message.contains("TicketExpired") {
        return Some(engine_net::DisconnectReason::TicketExpired);
    }
    if message.contains("ServerShuttingDown") {
        return Some(engine_net::DisconnectReason::ServerShuttingDown);
    }
    if message.contains("TimedOut") {
        return Some(engine_net::DisconnectReason::TimedOut);
    }
    None
}
