use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use engine_net::{
    AuthoritativeJoinState, ClientMessage, ClientSessionState, ClientSessionTarget,
    DisconnectReason, JoinAccepted, JoinRejected, JoinRequest, MessageEnvelope, ServerMessage,
    ServerSessionConfig, ServerSessionState, SessionPhase, SessionRuntimeCommand,
    SessionRuntimeEvent, Transport, TransportKind, begin_client_session, decode_message,
    encode_message, handle_client_message, observe_server_message,
};
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuicTransportConfig {
    pub alpn_protocols: Vec<Vec<u8>>,
    pub max_concurrent_sessions: u32,
}

impl Default for QuicTransportConfig {
    fn default() -> Self {
        Self {
            alpn_protocols: vec![b"grottoq/1".to_vec()],
            max_concurrent_sessions: 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuicTransport {
    config: QuicTransportConfig,
}

#[derive(Debug, Clone)]
pub struct QuicServerEndpoint {
    pub endpoint: Endpoint,
    pub certificate: CertificateDer<'static>,
    pub certificate_fingerprint_sha256: String,
    pub server_name: String,
}

#[derive(Debug, Clone)]
pub struct QuicClientBootstrap {
    pub endpoint: Endpoint,
    pub connection: Connection,
    pub state: ClientSessionState,
    pub accepted: JoinAccepted,
}

#[derive(Debug, Clone)]
pub struct QuicServerBootstrap {
    pub connection: Connection,
    pub state: ServerSessionState,
}

#[derive(Debug, Clone)]
pub enum QuicTrustPolicy {
    DirectRoots(Vec<CertificateDer<'static>>),
    PinnedServer {
        expected_fingerprint_sha256: String,
        trusted_certificates: Vec<CertificateDer<'static>>,
    },
}

impl QuicTrustPolicy {
    fn retargeted_for(&self, target: &ClientSessionTarget) -> Self {
        match self {
            Self::DirectRoots(certificates) => Self::DirectRoots(certificates.clone()),
            Self::PinnedServer {
                trusted_certificates,
                ..
            } => Self::PinnedServer {
                expected_fingerprint_sha256: target.server_cert_fingerprint_sha256.clone(),
                trusted_certificates: trusted_certificates.clone(),
            },
        }
    }
}

#[async_trait]
pub trait QuicClientTargetProvider: Send + Sync {
    async fn refresh_target(&self, previous: &ClientSessionTarget) -> Result<ClientSessionTarget>;
}

#[derive(Debug, Error)]
pub enum QuicJoinVerificationError {
    #[error("join request rejected: {0:?}")]
    Rejected(DisconnectReason),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait QuicServerJoinVerifier: Send + Sync {
    async fn verify_join_request(
        &self,
        request: &JoinRequest,
        config: &ServerSessionConfig,
    ) -> std::result::Result<AuthoritativeJoinState, QuicJoinVerificationError>;
}

impl QuicTrustPolicy {
    fn trusted_certificates(&self) -> Result<Vec<CertificateDer<'static>>> {
        let certificates = match self {
            Self::DirectRoots(certificates) => certificates.clone(),
            Self::PinnedServer {
                trusted_certificates,
                ..
            } => trusted_certificates.clone(),
        };
        if certificates.is_empty() {
            return Err(anyhow!(
                "QUIC trust policy requires at least one trusted certificate"
            ));
        }
        Ok(certificates)
    }

    fn validate_expected_fingerprint(&self) -> Result<()> {
        if let Self::PinnedServer {
            expected_fingerprint_sha256,
            trusted_certificates,
        } = self
        {
            let matches = trusted_certificates.iter().any(|certificate| {
                certificate_fingerprint_sha256(certificate) == *expected_fingerprint_sha256
            });
            if !matches {
                return Err(anyhow!(
                    "trusted server certificate does not match the expected fingerprint"
                ));
            }
        }
        Ok(())
    }
}

pub type QuicSessionCommand = SessionRuntimeCommand;
pub type QuicSessionEvent = SessionRuntimeEvent;

pub struct QuicRuntimeClientHandle {
    command_tx: UnboundedSender<QuicSessionCommand>,
    event_rx: UnboundedReceiver<QuicSessionEvent>,
}

impl QuicRuntimeClientHandle {
    pub fn into_channels(
        self,
    ) -> (
        UnboundedSender<QuicSessionCommand>,
        UnboundedReceiver<QuicSessionEvent>,
    ) {
        (self.command_tx, self.event_rx)
    }
}

pub struct QuicRuntimeServerHandle {
    command_tx: UnboundedSender<QuicSessionCommand>,
    event_rx: UnboundedReceiver<QuicSessionEvent>,
    pub local_addr: SocketAddr,
    pub certificate: CertificateDer<'static>,
    pub certificate_fingerprint_sha256: String,
    pub server_name: String,
}

impl QuicRuntimeServerHandle {
    pub fn into_channels(
        self,
    ) -> (
        UnboundedSender<QuicSessionCommand>,
        UnboundedReceiver<QuicSessionEvent>,
    ) {
        (self.command_tx, self.event_rx)
    }
}

impl QuicTransport {
    pub fn new(config: QuicTransportConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &QuicTransportConfig {
        &self.config
    }

    pub fn build_transport_config(&self) -> quinn::TransportConfig {
        let mut config = quinn::TransportConfig::default();
        config.max_concurrent_bidi_streams(self.config.max_concurrent_sessions.into());
        config.datagram_receive_buffer_size(Some(64 * 1024));
        config.datagram_send_buffer_size(64 * 1024);
        config
    }

    pub fn bind_server_endpoint(&self, bind_addr: SocketAddr) -> Result<QuicServerEndpoint> {
        self.bind_server_endpoint_named(bind_addr, "localhost")
    }

    pub fn bind_server_endpoint_named(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
    ) -> Result<QuicServerEndpoint> {
        let cert = rcgen::generate_simple_self_signed(vec![server_name.to_string()])?;
        let cert_der = CertificateDer::from(cert.cert.der().clone());
        let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
        let mut server_config =
            ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
        server_config.transport = Arc::new(self.build_transport_config());
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        Ok(QuicServerEndpoint {
            endpoint,
            certificate_fingerprint_sha256: certificate_fingerprint_sha256(&cert_der),
            certificate: cert_der,
            server_name: server_name.to_string(),
        })
    }

    pub fn bind_client_endpoint(
        &self,
        bind_addr: SocketAddr,
        trusted_certificates: &[CertificateDer<'static>],
    ) -> Result<Endpoint> {
        let mut roots = RootCertStore::empty();
        for cert in trusted_certificates {
            roots.add(cert.clone())?;
        }
        let mut client_config = ClientConfig::with_root_certificates(Arc::new(roots))?;
        client_config.transport_config(Arc::new(self.build_transport_config()));
        let mut endpoint = Endpoint::client(bind_addr)?;
        endpoint.set_default_client_config(client_config);
        Ok(endpoint)
    }

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
        let mut state = ClientSessionState::default();
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

    pub fn spawn_client_runtime(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        target: ClientSessionTarget,
        trust_policy: QuicTrustPolicy,
    ) -> Result<QuicRuntimeClientHandle> {
        self.spawn_client_runtime_with_provider(bind_addr, server_name, target, trust_policy, None)
    }

    pub fn spawn_client_runtime_with_provider(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        target: ClientSessionTarget,
        trust_policy: QuicTrustPolicy,
        target_provider: Option<Arc<dyn QuicClientTargetProvider>>,
    ) -> Result<QuicRuntimeClientHandle> {
        trust_policy.validate_expected_fingerprint()?;
        let (command_tx, command_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();
        let transport = self.clone();
        let server_name = server_name.to_string();
        tokio::spawn(async move {
            if let Err(error) = run_client_runtime_task(
                transport,
                bind_addr,
                server_name,
                target,
                trust_policy,
                target_provider,
                command_rx,
                event_tx.clone(),
            )
            .await
            {
                let _ = event_tx.send(QuicSessionEvent::Error {
                    message: error.to_string(),
                });
                let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                    connection_id: None,
                    reason: None,
                });
            }
        });
        Ok(QuicRuntimeClientHandle {
            command_tx,
            event_rx,
        })
    }

    pub fn spawn_server_runtime(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        session_config: ServerSessionConfig,
    ) -> Result<QuicRuntimeServerHandle> {
        self.spawn_server_runtime_with_verifier(bind_addr, server_name, session_config, None)
    }

    pub fn spawn_server_runtime_with_verifier(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        session_config: ServerSessionConfig,
        verifier: Option<Arc<dyn QuicServerJoinVerifier>>,
    ) -> Result<QuicRuntimeServerHandle> {
        let server = self.bind_server_endpoint_named(bind_addr, server_name)?;
        let local_addr = server.endpoint.local_addr()?;
        let (command_tx, command_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();
        let transport = self.clone();
        let endpoint = server.endpoint.clone();
        tokio::spawn(async move {
            if let Err(error) = run_server_runtime_task(
                transport,
                endpoint,
                session_config,
                verifier,
                command_rx,
                event_tx.clone(),
            )
            .await
            {
                let _ = event_tx.send(QuicSessionEvent::Error {
                    message: error.to_string(),
                });
                let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                    connection_id: None,
                    reason: None,
                });
            }
        });
        Ok(QuicRuntimeServerHandle {
            command_tx,
            event_rx,
            local_addr,
            certificate: server.certificate,
            certificate_fingerprint_sha256: server.certificate_fingerprint_sha256,
            server_name: server.server_name,
        })
    }
}

impl Default for QuicTransport {
    fn default() -> Self {
        Self::new(QuicTransportConfig::default())
    }
}

impl Transport for QuicTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Quic
    }
}

pub fn default_client_bind_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

pub fn certificate_fingerprint_sha256(cert: &CertificateDer<'_>) -> String {
    let digest = Sha256::digest(cert.as_ref());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

pub async fn write_message(stream: &mut SendStream, envelope: &MessageEnvelope) -> Result<()> {
    let bytes = encode_message(envelope)?;
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    Ok(())
}

pub async fn read_message(stream: &mut RecvStream) -> Result<Option<MessageEnvelope>> {
    let length = match stream.read_u32().await {
        Ok(length) => length,
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = vec![0u8; length as usize];
    stream.read_exact(&mut bytes).await?;
    Ok(Some(decode_message(&bytes)?))
}

// Owner: Grotto Engine Net - QUIC Runtime
include!("runtime_impl.rs");

#[cfg(test)]
include!("tests.rs");
