use tokio::sync::mpsc::Sender;

use crate::QuicSessionEvent;
use crate::server::runtime::ServerPeerEvent;

pub(crate) fn send_runtime_event(event_tx: &Sender<QuicSessionEvent>, event: QuicSessionEvent) {
    let _ = event_tx.try_send(event);
}

pub(crate) fn send_peer_event(event_tx: &Sender<ServerPeerEvent>, event: ServerPeerEvent) {
    let _ = event_tx.try_send(event);
}
