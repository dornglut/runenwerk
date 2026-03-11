use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::error::TrySendError;

use crate::QuicSessionEvent;
use crate::server::runtime::ServerPeerEvent;

pub(crate) fn send_runtime_event(event_tx: &Sender<QuicSessionEvent>, event: QuicSessionEvent) {
    send_event_with_backpressure("runtime", event_tx, event);
}

pub(crate) fn send_peer_event(event_tx: &Sender<ServerPeerEvent>, event: ServerPeerEvent) {
    send_event_with_backpressure("peer", event_tx, event);
}

fn send_event_with_backpressure<T>(channel_name: &str, event_tx: &Sender<T>, event: T)
where
    T: Send + 'static,
{
    match event_tx.try_send(event) {
        Ok(()) => {}
        Err(TrySendError::Full(event)) => {
            let event_tx = event_tx.clone();
            let channel_name = channel_name.to_string();
            tokio::spawn(async move {
                if event_tx.send(event).await.is_err() {
                    eprintln!("quic {channel_name} event dropped: channel closed");
                }
            });
        }
        Err(TrySendError::Closed(_)) => {
            eprintln!("quic {channel_name} event dropped: channel closed");
        }
    }
}
