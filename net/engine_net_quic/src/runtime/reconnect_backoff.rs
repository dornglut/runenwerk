use anyhow::Result;
use tokio::sync::mpsc::Receiver;

use crate::QuicSessionCommand;

pub(crate) async fn wait_for_reconnect_backoff(
    command_rx: &mut Receiver<QuicSessionCommand>,
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
