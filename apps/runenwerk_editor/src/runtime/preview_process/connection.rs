use anyhow::{Result, anyhow};
use editor_preview::{PreviewBootstrap, PreviewCommandEnvelope, PreviewProtocolPayload};
use engine_net::{
    ClientMessage, ConnectionId, SessionRuntimeCommand, SessionRuntimeEvent, TypedPayloadMessage,
};
use engine_net_quic::{
    QuicSessionCommand, QuicSessionEvent, QuicTransport, QuicTrustPolicy, default_client_bind_addr,
};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::runtime::preview_process::trusted_certificate_from_bootstrap;

pub struct PreviewProcessConnection {
    command_tx: Sender<QuicSessionCommand>,
    event_rx: Receiver<QuicSessionEvent>,
    connection_id: ConnectionId,
}

impl PreviewProcessConnection {
    pub async fn connect(bootstrap: &PreviewBootstrap) -> Result<Self> {
        let certificate = trusted_certificate_from_bootstrap(bootstrap)?;
        let transport = QuicTransport::default();
        let client = transport.spawn_client_runtime(
            default_client_bind_addr(),
            &bootstrap.server_name,
            engine_net::ClientSessionTarget {
                server_id: bootstrap.server_id.clone(),
                server_endpoint: bootstrap.endpoint.clone(),
                transport: engine_net::TransportKind::Quic,
                protocol: engine_net::ProtocolVersion::new(1, 1, 1),
                server_cert_fingerprint_sha256: bootstrap.certificate_fingerprint_sha256.clone(),
                ticket: bootstrap.join_ticket.clone(),
            },
            QuicTrustPolicy::PinnedServer {
                expected_fingerprint_sha256: bootstrap.certificate_fingerprint_sha256.clone(),
                trusted_certificates: vec![certificate],
            },
        )?;
        let (command_tx, mut event_rx) = client.into_channels();
        let connection_id = wait_for_join_acceptance(&mut event_rx).await?;
        Ok(Self {
            command_tx,
            event_rx,
            connection_id,
        })
    }

    pub const fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub async fn send_preview_command(&self, command: PreviewCommandEnvelope) -> Result<()> {
        let payload = typed_payload_from_preview(editor_preview::encode_preview_command(&command)?);
        self.command_tx
            .send(SessionRuntimeCommand::Client(ClientMessage::TypedPayload(
                payload,
            )))
            .await
            .map_err(|_| anyhow!("preview process command channel closed"))
    }

    pub async fn next_event(&mut self) -> Option<SessionRuntimeEvent> {
        self.event_rx.recv().await
    }

    pub fn try_next_event(&mut self) -> Result<Option<SessionRuntimeEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(anyhow!("preview process event channel closed")),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(SessionRuntimeCommand::Shutdown)
            .await
            .map_err(|_| anyhow!("preview process command channel closed"))
    }
}

pub fn typed_payload_from_preview(payload: PreviewProtocolPayload) -> TypedPayloadMessage {
    TypedPayloadMessage::new(
        payload.channel,
        payload.type_name,
        payload.schema_version,
        payload.payload,
    )
}

pub fn preview_payload_from_typed(payload: &TypedPayloadMessage) -> PreviewProtocolPayload {
    PreviewProtocolPayload::new(
        payload.channel.clone(),
        payload.type_name.clone(),
        payload.schema_version,
        payload.payload.clone(),
    )
}

async fn wait_for_join_acceptance(events: &mut Receiver<QuicSessionEvent>) -> Result<ConnectionId> {
    loop {
        let event = tokio::time::timeout(std::time::Duration::from_secs(5), events.recv())
            .await
            .map_err(|_| anyhow!("timed out waiting for runtime preview process join"))?
            .ok_or_else(|| anyhow!("runtime preview process event channel closed"))?;
        if let SessionRuntimeEvent::JoinAccepted(join) = event {
            return Ok(ConnectionId(join.connection_id));
        }
    }
}
