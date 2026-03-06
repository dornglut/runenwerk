// Owner: Grotto Engine Net - QUIC Runtime
async fn wait_for_reconnect_backoff(
    command_rx: &mut UnboundedReceiver<QuicSessionCommand>,
    pending_commands: &mut Vec<QuicSessionCommand>,
) -> Result<bool> {
    let sleep = tokio::time::sleep(std::time::Duration::from_millis(250));
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => return Ok(true),
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => return Ok(false),
                    Some(command) => pending_commands.push(command),
                }
            }
        }
    }
}

fn parse_join_rejection_reason(message: &str) -> Option<engine_net::DisconnectReason> {
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
fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}
