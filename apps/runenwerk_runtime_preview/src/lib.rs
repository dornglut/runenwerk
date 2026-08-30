mod network;

use anyhow::{Result, anyhow};
use editor_preview::{
    PreviewBootstrap, PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewEventEnvelope,
    PreviewMode, PreviewSessionId, ReloadStatus, RuntimeProductPayload, RuntimeProductRef,
};
use engine::app::App;
use engine::plugins::default_plugins;
use network::{ServerNetworkCommand, ServerNetworkEvent};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};

pub const DEFAULT_PREVIEW_SERVER_NAME: &str = "runenwerk-runtime-preview.local";

const WINDOW_TITLE: &str = "Runenwerk Runtime Preview";

#[derive(Debug, Clone)]
pub struct RuntimePreviewConfig {
    pub headless: bool,
    pub bind_addr: SocketAddr,
    pub server_name: String,
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
            server_name: DEFAULT_PREVIEW_SERVER_NAME.to_string(),
        }
    }
}

pub struct RuntimePreviewHost {
    command_tx: Sender<ServerNetworkCommand>,
    event_rx: Receiver<ServerNetworkEvent>,
    bootstrap: PreviewBootstrap,
    network_task: Mutex<Option<JoinHandle<Result<()>>>>,
}

impl RuntimePreviewHost {
    pub fn spawn(config: RuntimePreviewConfig) -> Result<Self> {
        let (command_tx, event_rx, bootstrap, network_task) =
            network::spawn(config.bind_addr, &config.server_name)?;
        Ok(Self {
            command_tx,
            event_rx,
            bootstrap,
            network_task: Mutex::new(Some(network_task)),
        })
    }

    pub fn bootstrap(&self) -> &PreviewBootstrap {
        &self.bootstrap
    }

    pub async fn run_command_loop(&mut self) -> Result<RuntimePreviewLoopExit> {
        let mut state = RuntimePreviewLoopState::default();
        let mut shutdown_session = None;
        while let Some(event) = self.event_rx.recv().await {
            match event {
                ServerNetworkEvent::Command(command) => {
                    if shutdown_session.is_some() {
                        return Err(anyhow!(
                            "runtime preview received a command after shutdown was acknowledged"
                        ));
                    }
                    let (events, should_shutdown) = state.handle_command(command);
                    let acknowledged_shutdown =
                        events.iter().find_map(|event| match &event.event {
                            PreviewEvent::ShutdownAck { session_id } => Some(*session_id),
                            _ => None,
                        });
                    for event in events {
                        self.send_preview_event(event).await?;
                    }
                    if should_shutdown {
                        shutdown_session = acknowledged_shutdown;
                    }
                }
                ServerNetworkEvent::Closed => {
                    return Ok(match shutdown_session {
                        Some(session_id) => RuntimePreviewLoopExit::ShutdownRequested {
                            session_id: Some(session_id),
                        },
                        None => RuntimePreviewLoopExit::TransportClosed,
                    });
                }
                ServerNetworkEvent::Error(message) => {
                    return Err(anyhow!("runtime preview transport failed: {message}"));
                }
            }
        }
        Ok(match shutdown_session {
            Some(session_id) => RuntimePreviewLoopExit::ShutdownRequested {
                session_id: Some(session_id),
            },
            None => RuntimePreviewLoopExit::EventStreamClosed,
        })
    }

    pub async fn shutdown(&self) -> Result<()> {
        let task = self.network_task.lock().await.take();
        let Some(task) = task else {
            return Ok(());
        };
        let _ = self.command_tx.send(ServerNetworkCommand::Shutdown).await;
        task.await
            .map_err(|error| anyhow!("runtime preview server task failed: {error}"))?
    }

    async fn send_preview_event(&self, event: PreviewEventEnvelope) -> Result<()> {
        self.command_tx
            .send(ServerNetworkCommand::Send(event))
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

#[cfg(test)]
mod tests {
    use super::*;
    use editor_preview::{
        PreviewCommand, ReloadDecision, ReloadSubject, ReloadSubjectKind, preview_session_id,
    };

    #[test]
    fn bootstrap_line_contains_connection_material() {
        let bootstrap = PreviewBootstrap {
            endpoint: "127.0.0.1:7777".to_string(),
            server_name: "preview.local".to_string(),
            trusted_certificate_der_hex: "010203".to_string(),
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
    fn loop_state_handles_session_heartbeat_reload_and_shutdown() {
        let mut state = RuntimePreviewLoopState::default();
        let session_id = preview_session_id(1);

        let (events, should_shutdown) = state.handle_command(PreviewCommandEnvelope::new(
            1,
            PreviewCommand::StartSession {
                session_id,
                mode: PreviewMode::Preview,
            },
        ));
        assert!(!should_shutdown);
        assert!(matches!(events[0].event, PreviewEvent::Ready { .. }));
        assert!(matches!(
            events[1].event,
            PreviewEvent::ModeChanged {
                mode: PreviewMode::Preview,
                ..
            }
        ));

        let (events, should_shutdown) = state.handle_command(PreviewCommandEnvelope::new(
            2,
            PreviewCommand::Heartbeat { session_id },
        ));
        assert!(!should_shutdown);
        assert!(matches!(events[0].event, PreviewEvent::Heartbeat { .. }));

        let (events, should_shutdown) = state.handle_command(PreviewCommandEnvelope::new(
            3,
            PreviewCommand::ApplyReload {
                session_id,
                status: Box::new(ReloadStatus::new(
                    ReloadSubject::new(ReloadSubjectKind::Shader, "shader"),
                    ReloadDecision::LiveReload,
                    "shader reloaded",
                )),
            },
        ));
        assert!(!should_shutdown);
        assert!(matches!(events[0].event, PreviewEvent::ReloadStatus { .. }));

        let (events, should_shutdown) = state.handle_command(PreviewCommandEnvelope::new(
            4,
            PreviewCommand::Shutdown { session_id },
        ));
        assert!(should_shutdown);
        assert!(matches!(events[0].event, PreviewEvent::ShutdownAck { .. }));
    }
}
