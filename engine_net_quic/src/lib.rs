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

async fn run_client_runtime_task(
    transport: QuicTransport,
    bind_addr: SocketAddr,
    server_name: String,
    target: ClientSessionTarget,
    trust_policy: QuicTrustPolicy,
    target_provider: Option<Arc<dyn QuicClientTargetProvider>>,
    mut command_rx: UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
) -> Result<()> {
    let mut current_target = target;
    let mut current_trust_policy = trust_policy.retargeted_for(&current_target);
    let mut reconnect_attempt = 0u32;
    let mut pending_commands = Vec::new();

    loop {
        if reconnect_attempt > 0 {
            let _ = event_tx.send(QuicSessionEvent::Reconnecting {
                attempt: reconnect_attempt,
            });
            if let Some(provider) = &target_provider {
                match provider.refresh_target(&current_target).await {
                    Ok(refreshed_target) => {
                        current_target = refreshed_target;
                        current_trust_policy = current_trust_policy.retargeted_for(&current_target);
                    }
                    Err(error) => {
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message: format!("failed to refresh join grant: {error}"),
                        });
                        reconnect_attempt = reconnect_attempt.saturating_add(1);
                        if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands)
                            .await?
                        {
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: None,
                                reason: None,
                            });
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
        }
        current_trust_policy.validate_expected_fingerprint()?;
        let trusted_certificates = current_trust_policy.trusted_certificates()?;
        let server_addr: SocketAddr =
            current_target.server_endpoint.parse().with_context(|| {
                format!(
                    "invalid server endpoint: {}",
                    current_target.server_endpoint
                )
            })?;

        match transport
            .connect_and_handshake(
                bind_addr,
                server_addr,
                &server_name,
                &trusted_certificates,
                current_target.clone(),
            )
            .await
        {
            Ok(bootstrap) => {
                let _ = event_tx.send(QuicSessionEvent::Connected {
                    connection_id: bootstrap.state.connection_id,
                });
                let _ = event_tx.send(QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                let _ = event_tx.send(QuicSessionEvent::JoinAccepted(bootstrap.accepted.clone()));
                let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                    millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                });
                reconnect_attempt = 0;
                let outcome = run_live_connection_loop(
                    bootstrap.connection,
                    Some(bootstrap.endpoint),
                    &mut command_rx,
                    event_tx.clone(),
                    bootstrap.state.connection_id,
                    &mut pending_commands,
                )
                .await?;
                match outcome {
                    LoopOutcome::Shutdown => return Ok(()),
                    LoopOutcome::ConnectionClosed => {
                        reconnect_attempt = reconnect_attempt.saturating_add(1);
                    }
                }
            }
            Err(error) => {
                let message = error.to_string();
                if let Some(reason) = parse_join_rejection_reason(&message) {
                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: Some(reason),
                    });
                    return Ok(());
                }
                let _ = event_tx.send(QuicSessionEvent::Error { message });
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands).await? {
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: None,
                    });
                    return Ok(());
                }
            }
        }
    }
}

async fn run_server_runtime_task(
    _transport: QuicTransport,
    endpoint: Endpoint,
    session_config: ServerSessionConfig,
    verifier: Option<Arc<dyn QuicServerJoinVerifier>>,
    mut command_rx: UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
) -> Result<()> {
    let mut session_state = ServerSessionState::default();
    engine_net::configure_server_session(&mut session_state, session_config);
    let (peer_event_tx, mut peer_event_rx) = unbounded_channel::<ServerPeerEvent>();
    let mut server_peers =
        BTreeMap::<engine_net::ConnectionId, UnboundedSender<ServerMessage>>::new();
    let mut drain_mode = false;
    loop {
        tokio::select! {
            biased;
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => {
                        endpoint.close(0u32.into(), b"shutdown");
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                            connection_id: None,
                            reason: None,
                        });
                        return Ok(());
                    }
                    Some(QuicSessionCommand::Server(message)) => {
                        let stale_connections = server_peers
                            .iter()
                            .filter_map(|(connection_id, sender)| {
                                sender.send(message.clone()).err().map(|_| *connection_id)
                            })
                            .collect::<Vec<_>>();
                        for connection_id in stale_connections {
                            server_peers.remove(&connection_id);
                            engine_net::remove_server_connection(&mut session_state, connection_id, None);
                        }
                    }
                    Some(QuicSessionCommand::Client(_)) => {}
                    Some(QuicSessionCommand::SetDrainMode { enabled }) => {
                        drain_mode = enabled;
                    }
                    Some(QuicSessionCommand::DisconnectConnection {
                        connection_id,
                        reason,
                    }) => {
                        if let Some(sender) = server_peers.get(&connection_id) {
                            let _ = sender.send(ServerMessage::Disconnect(reason));
                        }
                    }
                }
            }
            peer_event = peer_event_rx.recv() => {
                if let Some(ServerPeerEvent::Closed { connection_id, reason }) = peer_event {
                    server_peers.remove(&connection_id);
                    engine_net::remove_server_connection(&mut session_state, connection_id, reason);
                }
            }
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else {
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: None,
                    });
                    return Ok(());
                };
                let bootstrap =
                    accept_incoming_connection(
                        incoming,
                        &mut session_state,
                        verifier.as_deref(),
                        drain_mode,
                    )
                    .await?;
                let _ = event_tx.send(QuicSessionEvent::Connected {
                    connection_id: bootstrap.state.active_connection,
                });
                let _ = event_tx.send(QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                if let SessionPhase::Rejected(reason) = bootstrap.state.phase.clone() {
                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: Some(reason.clone()),
                    });
                    let close_reason = format!("{reason:?}");
                    let rejected_connection = bootstrap.connection;
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                        rejected_connection.close(0u32.into(), close_reason.as_bytes());
                    });
                    continue;
                }
                if let Some(connection_id) = bootstrap.state.active_connection {
                    let _ = event_tx.send(QuicSessionEvent::JoinAccepted(JoinAccepted {
                        connection_id: connection_id.0,
                        tick_rate_hz: bootstrap.state.config.tick_rate_hz,
                        join_state: bootstrap.state.last_join_state.clone().unwrap_or_default(),
                    }));
                    let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                        millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                    });
                    let (peer_tx, peer_rx) = unbounded_channel();
                    server_peers.insert(connection_id, peer_tx);
                    tokio::spawn(run_server_peer_task(
                        bootstrap.connection,
                        connection_id,
                        peer_rx,
                        event_tx.clone(),
                        peer_event_tx.clone(),
                    ));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ServerPeerEvent {
    Closed {
        connection_id: engine_net::ConnectionId,
        reason: Option<DisconnectReason>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum LoopOutcome {
    Shutdown,
    ConnectionClosed,
}

async fn run_live_connection_loop(
    connection: Connection,
    _endpoint: Option<Endpoint>,
    command_rx: &mut UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
    source_connection_id: Option<engine_net::ConnectionId>,
    pending_commands: &mut Vec<QuicSessionCommand>,
) -> Result<LoopOutcome> {
    for command in pending_commands.drain(..) {
        if dispatch_runtime_command(&connection, &event_tx, command).await? {
            return Ok(LoopOutcome::ConnectionClosed);
        }
    }
    loop {
        tokio::select! {
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => {
                        connection.close(0u32.into(), b"shutdown");
                        return Ok(LoopOutcome::Shutdown);
                    }
                    Some(command) => {
                        if dispatch_runtime_command(&connection, &event_tx, command).await? {
                            return Ok(LoopOutcome::ConnectionClosed);
                        }
                    }
                }
            }
            incoming = connection.read_datagram() => {
                match incoming {
                    Ok(bytes) => {
                        let envelope: MessageEnvelope = decode_message(&bytes)?;
                        match envelope {
                            MessageEnvelope::Client(message) => {
                                let _ = event_tx.send(QuicSessionEvent::ClientMessage {
                                    connection_id: source_connection_id,
                                    message,
                                });
                            }
                            MessageEnvelope::Server(message) => {
                                if let ServerMessage::JoinRejected(JoinRejected { reason }) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                                }
                                if let ServerMessage::JoinAccepted(join) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::JoinAccepted(join.clone()));
                                }
                                if let ServerMessage::Disconnect(reason) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                        connection_id: source_connection_id,
                                        reason: Some(reason.clone()),
                                    });
                                    return Ok(LoopOutcome::ConnectionClosed);
                                }
                                let _ = event_tx.send(QuicSessionEvent::ServerMessage(message));
                            }
                        }
                        let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                            millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                        });
                    }
                    Err(error) => {
                        let message = error.to_string();
                        let reason = parse_join_rejection_reason(&message);
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message,
                        });
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                            connection_id: source_connection_id,
                            reason,
                        });
                        return Ok(LoopOutcome::ConnectionClosed);
                    }
                }
            }
        }
    }
}

async fn run_server_peer_task(
    connection: Connection,
    connection_id: engine_net::ConnectionId,
    mut message_rx: UnboundedReceiver<ServerMessage>,
    event_tx: UnboundedSender<QuicSessionEvent>,
    peer_event_tx: UnboundedSender<ServerPeerEvent>,
) {
    loop {
        tokio::select! {
            message = message_rx.recv() => {
                match message {
                    Some(message) => {
                        let should_close = matches!(message, ServerMessage::Disconnect(_));
                        let disconnect_reason = if let ServerMessage::Disconnect(reason) = &message {
                            Some(reason.clone())
                        } else {
                            None
                        };
                        let send_result: Result<()> = (|| {
                            let bytes = encode_message(&MessageEnvelope::Server(message))?;
                            connection
                                .send_datagram(bytes.into())
                                .context("failed to send server datagram")?;
                            Ok(())
                        })();
                        if let Err(error) = send_result {
                            let _ = event_tx.send(QuicSessionEvent::Error {
                                message: error.to_string(),
                            });
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            let _ = peer_event_tx.send(ServerPeerEvent::Closed {
                                connection_id,
                                reason: disconnect_reason,
                            });
                            return;
                        }
                        if should_close {
                            let close_reason = disconnect_reason
                                .as_ref()
                                .map(|reason| format!("{reason:?}"))
                                .unwrap_or_else(|| "server disconnect".to_string());
                            connection.close(0u32.into(), close_reason.as_bytes());
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            let _ = peer_event_tx.send(ServerPeerEvent::Closed {
                                connection_id,
                                reason: disconnect_reason,
                            });
                            return;
                        }
                    }
                    None => {
                        connection.close(0u32.into(), b"server peer dropped");
                        let _ = peer_event_tx.send(ServerPeerEvent::Closed {
                            connection_id,
                            reason: None,
                        });
                        return;
                    }
                }
            }
            incoming = connection.read_datagram() => {
                match incoming {
                    Ok(bytes) => {
                        match decode_message::<MessageEnvelope>(&bytes) {
                            Ok(MessageEnvelope::Client(message)) => {
                                let _ = event_tx.send(QuicSessionEvent::ClientMessage {
                                    connection_id: Some(connection_id),
                                    message,
                                });
                                let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                                    millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                                });
                            }
                            Ok(MessageEnvelope::Server(_)) => {}
                            Err(error) => {
                                let _ = event_tx.send(QuicSessionEvent::Error {
                                    message: error.to_string(),
                                });
                            }
                        }
                    }
                    Err(error) => {
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message: error.to_string(),
                        });
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                            connection_id: Some(connection_id),
                            reason: None,
                        });
                        let _ = peer_event_tx.send(ServerPeerEvent::Closed {
                            connection_id,
                            reason: None,
                        });
                        return;
                    }
                }
            }
        }
    }
}

async fn dispatch_runtime_command(
    connection: &Connection,
    _event_tx: &UnboundedSender<QuicSessionEvent>,
    command: QuicSessionCommand,
) -> Result<bool> {
    match command {
        QuicSessionCommand::Client(message) => {
            let bytes = encode_message(&MessageEnvelope::Client(message))?;
            connection
                .send_datagram(bytes.into())
                .context("failed to send client datagram")?;
            Ok(false)
        }
        _ => Ok(false),
    }
}

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

async fn accept_incoming_connection(
    incoming: quinn::Incoming,
    state: &mut ServerSessionState,
    verifier: Option<&dyn QuicServerJoinVerifier>,
    drain_mode: bool,
) -> Result<QuicServerBootstrap> {
    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
    let connection = incoming.await.context("server accept failed")?;
    let (mut send, mut recv) = connection.accept_bi().await?;
    while let Some(message) = read_message(&mut recv).await? {
        let MessageEnvelope::Client(client_message) = message else {
            continue;
        };
        if drain_mode && let ClientMessage::JoinRequest(request) = &client_message {
            let reason = DisconnectReason::ServerShuttingDown;
            state.last_join_request = Some(request.clone());
            state.last_join_state = None;
            state.phase = SessionPhase::Rejected(reason.clone());
            state.last_disconnect = Some(reason.clone());
            write_message(
                &mut send,
                &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected { reason })),
            )
            .await?;
            break;
        }
        if let ClientMessage::JoinRequest(request) = &client_message
            && let Some(verifier) = verifier
        {
            match verifier.verify_join_request(request, &state.config).await {
                Ok(join_state) => {
                    state.last_join_state = Some(join_state);
                }
                Err(QuicJoinVerificationError::Rejected(reason)) => {
                    state.last_join_request = Some(request.clone());
                    state.last_join_state = None;
                    state.phase = SessionPhase::Rejected(reason.clone());
                    state.last_disconnect = Some(reason.clone());
                    write_message(
                        &mut send,
                        &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected {
                            reason,
                        })),
                    )
                    .await?;
                    break;
                }
                Err(QuicJoinVerificationError::Other(error)) => return Err(error),
            }
        }
        for response in handle_client_message(state, &client_message) {
            write_message(&mut send, &MessageEnvelope::Server(response)).await?;
        }
        if matches!(state.phase, SessionPhase::Active)
            || matches!(state.phase, SessionPhase::Rejected(_))
        {
            break;
        }
    }
    send.finish()?;
    Ok(QuicServerBootstrap {
        connection,
        state: state.clone(),
    })
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_net::{
        Ack, ClientCommandEnvelope, ClientMessage, ConnectionId, DeltaSnapshot, InputFrame,
        MoveCommand, ProtocolVersion, Snapshot, SnapshotCursor,
    };

    #[test]
    fn quic_transport_defaults_to_quic_and_grotto_alpn() {
        let transport = QuicTransport::default();
        assert_eq!(transport.kind(), TransportKind::Quic);
        assert_eq!(
            transport.config().alpn_protocols,
            vec![b"grottoq/1".to_vec()]
        );
        let _config = transport.build_transport_config();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn certificate_fingerprint_is_sha256_hex() {
        let transport = QuicTransport::default();
        let server = transport
            .bind_server_endpoint(default_client_bind_addr())
            .expect("server endpoint should build");
        assert_eq!(server.certificate_fingerprint_sha256.len(), 64);
        assert!(
            server
                .certificate_fingerprint_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn loopback_quic_handshake_reaches_join_accepted() {
        let transport = QuicTransport::default();
        let server = transport
            .bind_server_endpoint(default_client_bind_addr())
            .expect("server endpoint should bind");
        let server_addr = server
            .endpoint
            .local_addr()
            .expect("server addr should exist");
        let server_cert = server.certificate.clone();
        let server_name = server.server_name.clone();

        let server_task = tokio::spawn({
            let endpoint = server.endpoint.clone();
            let transport = transport.clone();
            async move {
                transport
                    .accept_and_handshake(
                        &endpoint,
                        ServerSessionConfig {
                            server_id: "srv-local".to_string(),
                            protocol: ProtocolVersion::new(1, 1, 1),
                            tick_rate_hz: 60,
                        },
                    )
                    .await
            }
        });

        let client_endpoint = transport
            .bind_client_endpoint(default_client_bind_addr(), &[server_cert])
            .expect("client endpoint should bind");
        let connection = client_endpoint
            .connect(server_addr, &server_name)
            .expect("client connect should start")
            .await
            .expect("client connect should complete");
        let target = ClientSessionTarget {
            server_id: "srv-local".to_string(),
            server_endpoint: server_addr.to_string(),
            transport: TransportKind::Quic,
            protocol: ProtocolVersion::new(1, 1, 1),
            server_cert_fingerprint_sha256: "unused-for-direct-root-store".to_string(),
            ticket: "ticket-1".to_string(),
        };
        let mut state = ClientSessionState::default();
        let outbound = begin_client_session(&mut state, target);
        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .expect("client control stream should open");
        for message in outbound {
            write_message(&mut send, &MessageEnvelope::Client(message))
                .await
                .expect("client message should write");
        }
        send.finish().expect("handshake stream should finish");

        let mut accepted = None;
        while let Some(message) = read_message(&mut recv)
            .await
            .expect("client should read server response")
        {
            let MessageEnvelope::Server(server_message) = message else {
                continue;
            };
            observe_server_message(&mut state, &server_message);
            if let ServerMessage::JoinAccepted(join) = server_message {
                accepted = Some(join);
                break;
            }
        }
        let server_result = server_task
            .await
            .expect("server task should join")
            .expect("server handshake should succeed");
        assert_eq!(state.phase, SessionPhase::Active);
        assert_eq!(accepted.expect("join accepted").connection_id, 1);
        assert_eq!(server_result.state.phase, SessionPhase::Active);
        assert_eq!(server_result.state.active_connection, Some(ConnectionId(1)));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn runtime_handles_exchange_datagrams_after_handshake() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, mut server_events) = server_handle.into_channels();

        let client_handle = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint.clone(),
                    ticket: "ticket-1".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: fingerprint,
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("client runtime should spawn");
        let (client_tx, mut client_events) = client_handle.into_channels();

        let mut client_joined = false;
        let mut server_connected = false;
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
            {
                if matches!(event, Some(QuicSessionEvent::JoinAccepted(_))) {
                    client_joined = true;
                    break;
                }
            }
        }
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
            {
                if matches!(event, Some(QuicSessionEvent::Connected { .. })) {
                    server_connected = true;
                    break;
                }
            }
        }
        assert!(client_joined, "client should reach JoinAccepted");
        assert!(
            server_connected,
            "server should accept the client connection"
        );

        client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(3),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: -1.0 })],
                },
            )))
            .expect("client command should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(3),
                    cursor: SnapshotCursor(1),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![1, 2, 3],
                },
            )))
            .expect("server snapshot should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::DeltaSnapshot(
                DeltaSnapshot {
                    tick: engine_net::SimulationTick(4),
                    base: SnapshotCursor(0),
                    cursor: SnapshotCursor(1),
                    entity_ids: Vec::new(),
                    payload: vec![4, 5, 6],
                },
            )))
            .expect("server delta should send");
        client_tx
            .send(QuicSessionCommand::Client(ClientMessage::Ack(Ack {
                cursor: SnapshotCursor(1),
                last_received_tick: engine_net::SimulationTick(4),
            })))
            .expect("client ack should send");

        let mut saw_input = false;
        let mut saw_ack = false;
        let mut saw_snapshot = false;
        let mut saw_delta = false;
        for _ in 0..40 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ClientMessage {
                        message: ClientMessage::InputFrame(_),
                        ..
                    } => {
                        saw_input = true;
                    }
                    QuicSessionEvent::ClientMessage {
                        message: ClientMessage::Ack(_),
                        ..
                    } => {
                        saw_ack = true;
                    }
                    _ => {}
                }
            }
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(_)) => {
                        saw_snapshot = true;
                    }
                    QuicSessionEvent::ServerMessage(ServerMessage::DeltaSnapshot(_)) => {
                        saw_delta = true;
                    }
                    _ => {}
                }
            }
            if saw_input && saw_ack && saw_snapshot && saw_delta {
                break;
            }
        }

        assert!(saw_input, "server should receive input datagrams");
        assert!(saw_ack, "server should receive ack datagrams");
        assert!(
            saw_snapshot,
            "client should receive server snapshot datagrams"
        );
        assert!(saw_delta, "client should receive delta snapshot datagrams");

        client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_accepts_a_second_client_after_disconnect() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, mut server_events) = server_handle.into_channels();

        let first_client = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint.clone(),
                    ticket: "ticket-1".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: fingerprint.clone(),
                    trusted_certificates: vec![server_cert.clone()],
                },
            )
            .expect("first client runtime should spawn");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();

        let mut first_join = None;
        for _ in 0..20 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                first_client_events.recv(),
            )
            .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(first_join, Some(1));

        first_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("first client shutdown should send");

        let mut saw_close = false;
        for _ in 0..40 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(QuicSessionEvent::ConnectionClosed { .. }) = event
            {
                saw_close = true;
                break;
            }
        }
        assert!(saw_close, "server should observe the first disconnect");

        let second_client = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint,
                    ticket: "ticket-2".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: certificate_fingerprint_sha256(&server_cert),
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("second client runtime should spawn");
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut second_join = None;
        for _ in 0..20 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                second_client_events.recv(),
            )
            .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                second_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(second_join, Some(2));

        second_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("second client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_broadcasts_to_multiple_connected_clients() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, mut server_events) = server_handle.into_channels();

        let spawn_client = |ticket: &str| {
            transport
                .spawn_client_runtime(
                    default_client_bind_addr(),
                    &server_name,
                    ClientSessionTarget {
                        server_id: "srv-local".to_string(),
                        server_endpoint: local_addr.to_string(),
                        transport: TransportKind::Quic,
                        protocol: ProtocolVersion::new(1, 1, 1),
                        server_cert_fingerprint_sha256: fingerprint.clone(),
                        ticket: ticket.to_string(),
                    },
                    QuicTrustPolicy::PinnedServer {
                        expected_fingerprint_sha256: fingerprint.clone(),
                        trusted_certificates: vec![server_cert.clone()],
                    },
                )
                .expect("client runtime should spawn")
        };

        let first_client = spawn_client("ticket-1");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();
        let second_client = spawn_client("ticket-2");
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut first_join = None;
        let mut second_join = None;
        for _ in 0..30 {
            if first_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    first_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
            }
            if second_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                second_join = Some(join.connection_id);
            }
            if first_join.is_some() && second_join.is_some() {
                break;
            }
        }
        assert!(matches!(first_join, Some(1 | 2)));
        assert!(matches!(second_join, Some(1 | 2)));
        assert_ne!(first_join, second_join);

        first_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(10),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 })],
                },
            )))
            .expect("first client input should send");
        second_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(10),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.0, y: 1.0 })],
                },
            )))
            .expect("second client input should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(11),
                    cursor: SnapshotCursor(3),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![9, 9, 9],
                },
            )))
            .expect("server snapshot should send");

        let mut saw_input_1 = false;
        let mut saw_input_2 = false;
        for _ in 0..60 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ClientMessage {
                        connection_id: Some(ConnectionId(connection_id)),
                        message: ClientMessage::InputFrame(_),
                    } if Some(connection_id) == first_join => {
                        saw_input_1 = true;
                    }
                    QuicSessionEvent::ClientMessage {
                        connection_id: Some(ConnectionId(connection_id)),
                        message: ClientMessage::InputFrame(_),
                    } if Some(connection_id) == second_join => {
                        saw_input_2 = true;
                    }
                    _ => {}
                }
            }
            if saw_input_1 && saw_input_2 {
                break;
            }
        }
        assert!(saw_input_1, "server should receive input from connection 1");
        assert!(saw_input_2, "server should receive input from connection 2");

        let mut first_saw_snapshot = false;
        let mut second_saw_snapshot = false;
        for _ in 0..40 {
            if !first_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    first_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(snapshot))) =
                    event
            {
                first_saw_snapshot = snapshot.cursor == SnapshotCursor(3);
            }
            if !second_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(snapshot))) =
                    event
            {
                second_saw_snapshot = snapshot.cursor == SnapshotCursor(3);
            }
            if first_saw_snapshot && second_saw_snapshot {
                break;
            }
        }

        assert!(
            first_saw_snapshot,
            "first client should receive broadcast snapshot"
        );
        assert!(
            second_saw_snapshot,
            "second client should receive broadcast snapshot"
        );

        first_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("first client shutdown should send");
        second_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("second client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_drain_mode_rejects_new_join_requests() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, _server_events) = server_handle.into_channels();

        server_tx
            .send(QuicSessionCommand::SetDrainMode { enabled: true })
            .expect("drain mode command should send");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client_handle = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint,
                    ticket: "ticket-drain".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: certificate_fingerprint_sha256(&server_cert),
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("client runtime should spawn");
        let (client_tx, mut client_events) = client_handle.into_channels();

        let mut join_rejected = None;
        for _ in 0..30 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(QuicSessionEvent::JoinRejected(reason)) = event
            {
                join_rejected = Some(reason);
                break;
            }
        }
        assert_eq!(join_rejected, Some(DisconnectReason::ServerShuttingDown));

        let _ = client_tx.send(QuicSessionCommand::Shutdown);
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_disconnect_connection_targets_only_one_peer() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, mut server_events) = server_handle.into_channels();

        let spawn_client = |ticket: &str| {
            transport
                .spawn_client_runtime(
                    default_client_bind_addr(),
                    &server_name,
                    ClientSessionTarget {
                        server_id: "srv-local".to_string(),
                        server_endpoint: local_addr.to_string(),
                        transport: TransportKind::Quic,
                        protocol: ProtocolVersion::new(1, 1, 1),
                        server_cert_fingerprint_sha256: fingerprint.clone(),
                        ticket: ticket.to_string(),
                    },
                    QuicTrustPolicy::PinnedServer {
                        expected_fingerprint_sha256: fingerprint.clone(),
                        trusted_certificates: vec![server_cert.clone()],
                    },
                )
                .expect("client runtime should spawn")
        };

        let first_client = spawn_client("ticket-a");
        let second_client = spawn_client("ticket-b");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut first_join = None;
        let mut second_join = None;
        for _ in 0..40 {
            if first_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    first_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
            }
            if second_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                second_join = Some(join.connection_id);
            }
            if first_join.is_some() && second_join.is_some() {
                break;
            }
        }
        assert!(first_join.is_some(), "first client should join");
        assert!(second_join.is_some(), "second client should join");
        assert_ne!(first_join, second_join);

        let disconnect_connection_id = first_join.expect("first join connection id");
        server_tx
            .send(QuicSessionCommand::DisconnectConnection {
                connection_id: ConnectionId(disconnect_connection_id),
                reason: DisconnectReason::TimedOut,
            })
            .expect("targeted disconnect should send");

        let mut first_disconnected = false;
        for _ in 0..30 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                first_client_events.recv(),
            )
            .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ConnectionClosed {
                        reason: Some(DisconnectReason::TimedOut),
                        ..
                    }
                    | QuicSessionEvent::ServerMessage(ServerMessage::Disconnect(
                        DisconnectReason::TimedOut,
                    )) => {
                        first_disconnected = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
        assert!(
            first_disconnected,
            "disconnected client should observe timed-out disconnect"
        );

        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(99),
                    cursor: SnapshotCursor(9),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![7, 7, 7],
                },
            )))
            .expect("server snapshot should send");
        second_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(99),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.5, y: 0.0 })],
                },
            )))
            .expect("second client input should send");

        let mut second_saw_snapshot = false;
        let mut server_saw_second_input = false;
        for _ in 0..40 {
            if !second_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(_))) = event
            {
                second_saw_snapshot = true;
            }
            if !server_saw_second_input
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    server_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ClientMessage {
                    connection_id: Some(ConnectionId(id)),
                    message: ClientMessage::InputFrame(_),
                }) = event
                && Some(id) == second_join
            {
                server_saw_second_input = true;
            }
            if second_saw_snapshot && server_saw_second_input {
                break;
            }
        }
        assert!(
            second_saw_snapshot,
            "remaining client should still receive snapshots"
        );
        assert!(
            server_saw_second_input,
            "server should still receive input from remaining client"
        );

        first_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("first client shutdown should send");
        second_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("second client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn client_runtime_reconnects_after_server_disconnect() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, _server_events) = server_handle.into_channels();

        let client_handle = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint.clone(),
                    ticket: "ticket-1".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: fingerprint,
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("client runtime should spawn");
        let (client_tx, mut client_events) = client_handle.into_channels();

        let mut first_join = None;
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(first_join, Some(1));

        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Disconnect(
                engine_net::DisconnectReason::TimedOut,
            )))
            .expect("server disconnect should send");

        let mut saw_reconnecting = false;
        let mut second_join = None;
        for _ in 0..80 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::Reconnecting { attempt } => {
                        saw_reconnecting = saw_reconnecting || attempt >= 1;
                    }
                    QuicSessionEvent::JoinAccepted(join) => {
                        if join.connection_id > 1 {
                            second_join = Some(join.connection_id);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        assert!(saw_reconnecting, "client should emit reconnect attempts");
        assert_eq!(second_join, Some(2));

        client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }
}
