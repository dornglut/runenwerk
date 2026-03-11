use anyhow::{Context, Result, anyhow};
use engine_net::{
    ClientSessionTarget, JoinRejected, MessageEnvelope, ServerMessage, ServerSessionConfig,
    ServerSessionState, SessionPhase, begin_client_session, handle_client_message,
    observe_server_message,
};
use quinn::Endpoint;
use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;

use crate::{QuicClientBootstrap, QuicServerBootstrap, QuicTransport, read_message, write_message};

impl QuicTransport {
    pub async fn connect_and_handshake(
        &self,
        bind_addr: SocketAddr,
        server_addr: SocketAddr,
        server_name: &str,
        trusted_certificates: &[CertificateDer<'static>],
        target: ClientSessionTarget,
    ) -> Result<QuicClientBootstrap> {
        let client_endpoint = self.bind_client_endpoint(bind_addr, trusted_certificates)?;
        let connection = client_endpoint
            .connect(server_addr, server_name)?
            .await
            .context("quic client connect failed")?;
        let mut state = engine_net::ClientSessionState::default();
        let outbound = begin_client_session(&mut state, target);
        let (mut send, mut recv) = connection.open_bi().await?;
        for message in outbound {
            write_message(&mut send, &MessageEnvelope::Client(message)).await?;
        }
        send.finish()?;

        let mut accepted = None;
        while let Some(message) = read_message(&mut recv).await? {
            let MessageEnvelope::Server(server_message) = message else {
                continue;
            };
            observe_server_message(&mut state, &server_message);
            if let ServerMessage::JoinAccepted(join) = server_message {
                accepted = Some(join);
                break;
            }
            if let ServerMessage::JoinRejected(JoinRejected { reason }) = server_message {
                return Err(anyhow!("server rejected join handshake: {reason:?}"));
            }
        }

        Ok(QuicClientBootstrap {
            endpoint: client_endpoint,
            connection,
            state,
            accepted: accepted.ok_or_else(|| anyhow!("server closed without JoinAccepted"))?,
        })
    }

    pub async fn accept_and_handshake(
        &self,
        endpoint: &Endpoint,
        session_config: ServerSessionConfig,
    ) -> Result<QuicServerBootstrap> {
        let mut state = ServerSessionState::default();
        engine_net::configure_server_session(&mut state, session_config);
        let incoming = endpoint
            .accept()
            .await
            .ok_or_else(|| anyhow!("server endpoint closed before accepting a connection"))?;
        let connection = incoming.await.context("server accept failed")?;
        let (mut send, mut recv) = connection.accept_bi().await?;
        while let Some(message) = read_message(&mut recv).await? {
            let MessageEnvelope::Client(client_message) = message else {
                continue;
            };
            for response in handle_client_message(&mut state, &client_message) {
                write_message(&mut send, &MessageEnvelope::Server(response)).await?;
            }
            if matches!(state.phase, SessionPhase::Active)
                || matches!(state.phase, SessionPhase::Rejected(_))
            {
                break;
            }
        }
        send.finish()?;
        Ok(QuicServerBootstrap { connection, state })
    }
}
