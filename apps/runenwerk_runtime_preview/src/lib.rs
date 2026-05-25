use anyhow::{Result, anyhow};
use editor_preview::{
    PreviewBootstrap, PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewEventEnvelope,
    PreviewMode, PreviewProtocolError, PreviewProtocolPayload, PreviewSessionId, ReloadStatus,
    RuntimeProductPayload, RuntimeProductRef, encode_lower_hex,
    encode_preview_command as encode_preview_command_payload,
    encode_preview_event as encode_preview_event_payload,
};
use engine::app::App;
use engine::plugins::default_plugins;
use engine_net::{
    ClientMessage, ClientSessionTarget, ConnectionId, ProtocolVersion, ServerMessage,
    ServerSessionConfig, SessionRuntimeCommand, SessionRuntimeEvent, TransportKind,
    TypedPayloadMessage,
};
use engine_net_quic::{
    QuicRuntimeServerHandle, QuicSessionCommand, QuicTransport, QuicTrustPolicy,
};
use rustls::pki_types::CertificateDer;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc::{Receiver, Sender};

pub const DEFAULT_PREVIEW_SERVER_ID: &str = "runenwerk-runtime-preview";
pub const DEFAULT_PREVIEW_SERVER_NAME: &str = "runenwerk-runtime-preview.local";
pub const DEFAULT_PREVIEW_JOIN_TICKET: &str = "runenwerk-preview-local";

const WINDOW_TITLE: &str = "Runenwerk Runtime Preview";
const SHUTDOWN_ACK_FLUSH_DELAY: std::time::Duration = std::time::Duration::from_millis(25);

#[derive(Debug, Clone)]
pub struct RuntimePreviewConfig {
    pub headless: bool,
    pub bind_addr: SocketAddr,
    pub server_id: String,
    pub server_name: String,
    pub join_ticket: String,
}

impl RuntimePreviewConfig {
    pub fn headless() -> Self {
        Self {
            headless: true,
            ..Self::default()
        }
    }
}

impl Default for RuntimePreviewConfig {
    fn default() -> Self {
        Self {
            headless: false,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            server_id: DEFAULT_PREVIEW_SERVER_ID.to_string(),
            server_name: DEFAULT_PREVIEW_SERVER_NAME.to_string(),
            join_ticket: DEFAULT_PREVIEW_JOIN_TICKET.to_string(),
        }
    }
}

pub struct RuntimePreviewHost {
    command_tx: Sender<QuicSessionCommand>,
    event_rx: Receiver<SessionRuntimeEvent>,
    certificate: CertificateDer<'static>,
    bootstrap: PreviewBootstrap,
}

impl RuntimePreviewHost {
    pub fn spawn(config: RuntimePreviewConfig) -> Result<Self> {
        let transport = QuicTransport::default();
        let session_config = ServerSessionConfig {
            server_id: config.server_id.clone(),
            protocol: ProtocolVersion::new(1, 1, 1),
            tick_rate_hz: 60,
        };
        let handle = transport.spawn_server_runtime(
            config.bind_addr,
            &config.server_name,
            session_config,
        )?;
        Ok(Self::from_handle(config, handle))
    }

    fn from_handle(config: RuntimePreviewConfig, handle: QuicRuntimeServerHandle) -> Self {
        let bootstrap = PreviewBootstrap {
            endpoint: handle.local_addr.to_string(),
            server_id: config.server_id,
            server_name: handle.server_name.clone(),
            certificate_fingerprint_sha256: handle.certificate_fingerprint_sha256.clone(),
            trusted_certificate_der_hex: certificate_der_hex(&handle.certificate),
            join_ticket: config.join_ticket,
        };
        let certificate = handle.certificate.clone();
        let (command_tx, event_rx) = handle.into_channels();
        Self {
            command_tx,
            event_rx,
            certificate,
            bootstrap,
        }
    }

    pub fn bootstrap(&self) -> &PreviewBootstrap {
        &self.bootstrap
    }

    pub fn certificate(&self) -> CertificateDer<'static> {
        self.certificate.clone()
    }

    pub fn client_target(&self) -> ClientSessionTarget {
        client_target_from_bootstrap(&self.bootstrap)
    }

    pub fn trust_policy(&self) -> QuicTrustPolicy {
        QuicTrustPolicy::PinnedServer {
            expected_fingerprint_sha256: self.bootstrap.certificate_fingerprint_sha256.clone(),
            trusted_certificates: vec![self.certificate()],
        }
    }

    pub async fn next_event(&mut self) -> Option<SessionRuntimeEvent> {
        self.event_rx.recv().await
    }

    pub async fn run_command_loop(&mut self) -> Result<RuntimePreviewLoopExit> {
        let mut state = RuntimePreviewLoopState::default();
        while let Some(event) = self.next_event().await {
            match event {
                SessionRuntimeEvent::ClientMessage {
                    connection_id: Some(connection_id),
                    message: ClientMessage::TypedPayload(payload),
                } => {
                    let command = match editor_preview::decode_preview_command(
                        &preview_payload_from_typed(&payload),
                    ) {
                        Ok(command) => command,
                        Err(error) => {
                            self.send_preview_event(
                                connection_id,
                                PreviewEventEnvelope::new(
                                    0,
                                    PreviewEvent::Error {
                                        session_id: None,
                                        message: error.to_string(),
                                    },
                                ),
                            )
                            .await?;
                            continue;
                        }
                    };
                    let (events, should_shutdown) = state.handle_command(command);
                    let shutdown_session = events.iter().find_map(|event| match &event.event {
                        PreviewEvent::ShutdownAck { session_id } => Some(*session_id),
                        _ => None,
                    });
                    for event in events {
                        self.send_preview_event(connection_id, event).await?;
                    }
                    if should_shutdown {
                        tokio::time::sleep(SHUTDOWN_ACK_FLUSH_DELAY).await;
                        return Ok(RuntimePreviewLoopExit::ShutdownRequested {
                            session_id: shutdown_session,
                        });
                    }
                }
                SessionRuntimeEvent::ConnectionClosed {
                    connection_id: None,
                    ..
                } => return Ok(RuntimePreviewLoopExit::TransportClosed),
                SessionRuntimeEvent::Error { .. } => {}
                _ => {}
            }
        }
        Ok(RuntimePreviewLoopExit::EventStreamClosed)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(SessionRuntimeCommand::Shutdown)
            .await
            .map_err(|_| anyhow!("runtime preview server command channel closed"))
    }

    async fn send_preview_event(
        &self,
        connection_id: ConnectionId,
        event: PreviewEventEnvelope,
    ) -> Result<()> {
        let payload = encode_preview_event(&event)?;
        self.command_tx
            .send(SessionRuntimeCommand::ServerToConnection {
                connection_id,
                message: ServerMessage::TypedPayload(payload),
            })
            .await
            .map_err(|_| anyhow!("runtime preview server command channel closed"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewLoopExit {
    ShutdownRequested {
        session_id: Option<PreviewSessionId>,
    },
    TransportClosed,
    EventStreamClosed,
}

#[derive(Debug, Default)]
pub struct RuntimePreviewLoopState {
    modes: BTreeMap<PreviewSessionId, PreviewMode>,
    loaded_products: Vec<RuntimeProductRef>,
    reload_statuses: Vec<ReloadStatus>,
}

impl RuntimePreviewLoopState {
    pub fn mode(&self, session_id: PreviewSessionId) -> Option<PreviewMode> {
        self.modes.get(&session_id).copied()
    }

    pub fn loaded_products(&self) -> &[RuntimeProductRef] {
        &self.loaded_products
    }

    pub fn reload_statuses(&self) -> &[ReloadStatus] {
        &self.reload_statuses
    }

    fn handle_command(
        &mut self,
        envelope: PreviewCommandEnvelope,
    ) -> (Vec<PreviewEventEnvelope>, bool) {
        let sequence = envelope.sequence;
        match envelope.command {
            PreviewCommand::StartSession { session_id, mode } => {
                self.modes.insert(session_id, mode);
                (
                    vec![
                        PreviewEventEnvelope::new(sequence, PreviewEvent::Ready { session_id }),
                        PreviewEventEnvelope::new(
                            sequence,
                            PreviewEvent::ModeChanged { session_id, mode },
                        ),
                    ],
                    false,
                )
            }
            PreviewCommand::ChangeMode { session_id, mode } => {
                self.modes.insert(session_id, mode);
                (
                    vec![PreviewEventEnvelope::new(
                        sequence,
                        PreviewEvent::ModeChanged { session_id, mode },
                    )],
                    false,
                )
            }
            PreviewCommand::PublishProduct {
                session_id,
                payload,
            } => {
                let product = product_ref_from_payload(*payload);
                self.loaded_products.push(product.clone());
                (
                    vec![PreviewEventEnvelope::new(
                        sequence,
                        PreviewEvent::ProductLoaded {
                            session_id,
                            product: Box::new(product),
                        },
                    )],
                    false,
                )
            }
            PreviewCommand::ApplyReload { session_id, status } => {
                let status = *status;
                self.reload_statuses.push(status.clone());
                (
                    vec![PreviewEventEnvelope::new(
                        sequence,
                        PreviewEvent::ReloadStatus {
                            session_id,
                            status: Box::new(status),
                        },
                    )],
                    false,
                )
            }
            PreviewCommand::Heartbeat { session_id } => (
                vec![PreviewEventEnvelope::new(
                    sequence,
                    PreviewEvent::Heartbeat { session_id },
                )],
                false,
            ),
            PreviewCommand::Shutdown { session_id } => (
                vec![PreviewEventEnvelope::new(
                    sequence,
                    PreviewEvent::ShutdownAck { session_id },
                )],
                true,
            ),
        }
    }
}

fn product_ref_from_payload(payload: RuntimeProductPayload) -> RuntimeProductRef {
    match payload {
        RuntimeProductPayload::Descriptor(product) => product,
        RuntimeProductPayload::WorldSdf(package) => package.product_ref,
    }
}

pub fn client_target_from_bootstrap(bootstrap: &PreviewBootstrap) -> ClientSessionTarget {
    ClientSessionTarget {
        server_id: bootstrap.server_id.clone(),
        server_endpoint: bootstrap.endpoint.clone(),
        transport: TransportKind::Quic,
        protocol: ProtocolVersion::new(1, 1, 1),
        server_cert_fingerprint_sha256: bootstrap.certificate_fingerprint_sha256.clone(),
        ticket: bootstrap.join_ticket.clone(),
    }
}

pub fn certificate_der_hex(certificate: &CertificateDer<'static>) -> String {
    encode_lower_hex(certificate.as_ref())
}

pub fn build_preview_app(headless: bool) -> App {
    let mut app = if headless {
        App::headless()
    } else {
        App::new()
    };
    app.set_title(WINDOW_TITLE);
    app.add_plugins(default_plugins());
    app
}

pub fn encode_preview_command(
    command: &PreviewCommandEnvelope,
) -> Result<TypedPayloadMessage, PreviewProtocolError> {
    Ok(typed_payload_from_preview(encode_preview_command_payload(
        command,
    )?))
}

pub fn encode_preview_event(
    event: &PreviewEventEnvelope,
) -> Result<TypedPayloadMessage, PreviewProtocolError> {
    Ok(typed_payload_from_preview(encode_preview_event_payload(
        event,
    )?))
}

pub fn decode_preview_command(
    payload: &TypedPayloadMessage,
) -> Result<PreviewCommandEnvelope, PreviewProtocolError> {
    editor_preview::decode_preview_command(&preview_payload_from_typed(payload))
}

pub fn decode_preview_event(
    payload: &TypedPayloadMessage,
) -> Result<PreviewEventEnvelope, PreviewProtocolError> {
    editor_preview::decode_preview_event(&preview_payload_from_typed(payload))
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

pub async fn wait_for_join_acceptance(
    events: &mut Receiver<SessionRuntimeEvent>,
    timeout: std::time::Duration,
) -> Result<engine_net::ConnectionId> {
    let started = std::time::Instant::now();
    loop {
        let remaining = timeout
            .checked_sub(started.elapsed())
            .ok_or_else(|| anyhow!("timed out waiting for runtime preview join"))?;
        let event = tokio::time::timeout(remaining, events.recv())
            .await
            .map_err(|_| anyhow!("timed out waiting for runtime preview join"))?
            .ok_or_else(|| anyhow!("runtime preview event channel closed"))?;
        if let SessionRuntimeEvent::JoinAccepted(join) = event {
            return Ok(engine_net::ConnectionId(join.connection_id));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_preview::{
        PreviewCommand, PreviewCommandEnvelope, ReloadDecision, ReloadSubject, ReloadSubjectKind,
        preview_session_id,
    };
    use engine_net::{ClientMessage, SessionRuntimeCommand};
    use engine_net_quic::default_client_bind_addr;
    use std::io::ErrorKind;
    use tokio::sync::mpsc::Sender;

    #[test]
    fn bootstrap_line_contains_connection_material() {
        let bootstrap = PreviewBootstrap {
            endpoint: "127.0.0.1:7777".to_string(),
            server_id: "srv".to_string(),
            server_name: "preview.local".to_string(),
            certificate_fingerprint_sha256: "abc".to_string(),
            trusted_certificate_der_hex: "010203".to_string(),
            join_ticket: "ticket".to_string(),
        };
        let line = bootstrap
            .to_stdout_line()
            .expect("bootstrap line should encode");
        assert!(line.starts_with(editor_preview::PREVIEW_BOOTSTRAP_PREFIX));
        assert_eq!(
            PreviewBootstrap::parse_stdout_line(&line).expect("bootstrap line should decode"),
            bootstrap
        );
    }

    #[test]
    fn preview_command_payload_round_trips_through_generic_typed_payload() {
        let command = PreviewCommandEnvelope::new(
            9,
            PreviewCommand::Heartbeat {
                session_id: preview_session_id(3),
            },
        );
        let payload = encode_preview_command(&command).expect("command should encode");
        assert_eq!(payload.channel, editor_preview::PREVIEW_CHANNEL);
        assert_eq!(payload.type_name, editor_preview::PREVIEW_COMMAND_TYPE);
        assert_eq!(
            decode_preview_command(&payload).expect("command should decode"),
            command
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn command_loop_handles_session_heartbeat_reload_and_shutdown() {
        let mut host = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
            Ok(host) => host,
            Err(error) if is_permission_denied(&error) => {
                eprintln!(
                    "skipping runtime preview command-loop transport test: local socket bind is denied"
                );
                return;
            }
            Err(error) => panic!("preview server should spawn: {error:?}"),
        };
        let client_target = host.client_target();
        let trust_policy = host.trust_policy();
        let server_name = host.bootstrap().server_name.clone();
        let loop_task = tokio::spawn(async move { host.run_command_loop().await });
        let transport = QuicTransport::default();
        let client = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                client_target,
                trust_policy,
            )
            .expect("preview client should spawn");
        let (client_tx, mut client_events) = client.into_channels();
        let connection_id =
            wait_for_join_acceptance(&mut client_events, std::time::Duration::from_secs(5))
                .await
                .expect("client should join preview server");

        send_preview_command(
            &client_tx,
            PreviewCommandEnvelope::new(
                1,
                PreviewCommand::StartSession {
                    session_id: preview_session_id(1),
                    mode: PreviewMode::Preview,
                },
            ),
        )
        .await;
        assert!(matches!(
            next_preview_event(&mut client_events).await.event,
            PreviewEvent::Ready { .. }
        ));
        assert!(matches!(
            next_preview_event(&mut client_events).await.event,
            PreviewEvent::ModeChanged {
                mode: PreviewMode::Preview,
                ..
            }
        ));

        send_preview_command(
            &client_tx,
            PreviewCommandEnvelope::new(
                2,
                PreviewCommand::Heartbeat {
                    session_id: preview_session_id(1),
                },
            ),
        )
        .await;
        assert!(matches!(
            next_preview_event(&mut client_events).await.event,
            PreviewEvent::Heartbeat { session_id: _ }
        ));

        send_preview_command(
            &client_tx,
            PreviewCommandEnvelope::new(
                3,
                PreviewCommand::ApplyReload {
                    session_id: preview_session_id(1),
                    status: Box::new(ReloadStatus::new(
                        ReloadSubject::new(ReloadSubjectKind::Shader, "shader"),
                        ReloadDecision::LiveReload,
                        "shader reloaded",
                    )),
                },
            ),
        )
        .await;
        assert!(matches!(
            next_preview_event(&mut client_events).await.event,
            PreviewEvent::ReloadStatus { .. }
        ));

        send_preview_command(
            &client_tx,
            PreviewCommandEnvelope::new(
                4,
                PreviewCommand::Shutdown {
                    session_id: preview_session_id(1),
                },
            ),
        )
        .await;
        assert!(matches!(
            next_preview_event(&mut client_events).await.event,
            PreviewEvent::ShutdownAck { .. }
        ));
        assert_eq!(
            loop_task
                .await
                .expect("loop task should join")
                .expect("loop should exit cleanly"),
            RuntimePreviewLoopExit::ShutdownRequested {
                session_id: Some(preview_session_id(1))
            }
        );
        client_tx
            .send(SessionRuntimeCommand::Shutdown)
            .await
            .expect("client command channel should be open");
        assert_eq!(connection_id, engine_net::ConnectionId(1));
    }

    fn is_permission_denied(error: &anyhow::Error) -> bool {
        error.chain().any(|cause| {
            cause
                .downcast_ref::<std::io::Error>()
                .is_some_and(|io_error| io_error.kind() == ErrorKind::PermissionDenied)
        })
    }

    async fn send_preview_command(
        client_tx: &Sender<QuicSessionCommand>,
        command: PreviewCommandEnvelope,
    ) {
        let payload = encode_preview_command(&command).expect("preview command should encode");
        client_tx
            .send(SessionRuntimeCommand::Client(ClientMessage::TypedPayload(
                payload,
            )))
            .await
            .expect("client command channel should be open");
    }

    async fn next_preview_event(
        client_events: &mut Receiver<SessionRuntimeEvent>,
    ) -> PreviewEventEnvelope {
        let started = std::time::Instant::now();
        loop {
            let remaining = std::time::Duration::from_secs(5)
                .checked_sub(started.elapsed())
                .expect("timed out waiting for preview event");
            let event = tokio::time::timeout(remaining, client_events.recv())
                .await
                .expect("timed out waiting for preview event")
                .expect("client event channel should stay open");
            if let SessionRuntimeEvent::ServerMessage(ServerMessage::TypedPayload(payload)) = event
            {
                return decode_preview_event(&payload).expect("preview event should decode");
            }
        }
    }
}
