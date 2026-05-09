use anyhow::{Context, Result, anyhow};
use editor_preview::{
    PreviewBootstrap, PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewMode,
    PreviewSessionId, ReloadStatus, RuntimeProductRef, preview_session_id,
};
use engine_net::{ClientMessage, ServerMessage, SessionRuntimeEvent};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use crate::runtime::preview_process::{PreviewProcessConnection, preview_payload_from_typed};

const DEFAULT_PENDING_COMMAND_CAPACITY: usize = 128;
const DEFAULT_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(2);
const POLL_SLEEP: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
pub struct PreviewProcessSpawnConfig {
    pub executable_path: PathBuf,
    pub headless: bool,
    pub bootstrap_timeout: Duration,
    pub shutdown_grace_period: Duration,
}

impl PreviewProcessSpawnConfig {
    pub fn new(executable_path: impl Into<PathBuf>) -> Self {
        Self {
            executable_path: executable_path.into(),
            headless: false,
            bootstrap_timeout: DEFAULT_BOOTSTRAP_TIMEOUT,
            shutdown_grace_period: DEFAULT_SHUTDOWN_GRACE_PERIOD,
        }
    }

    pub fn headless(executable_path: impl Into<PathBuf>) -> Self {
        Self {
            executable_path: executable_path.into(),
            headless: true,
            bootstrap_timeout: DEFAULT_BOOTSTRAP_TIMEOUT,
            shutdown_grace_period: DEFAULT_SHUTDOWN_GRACE_PERIOD,
        }
    }
}

pub struct PreviewProcessManager {
    child: Option<Child>,
    bootstrap: Option<PreviewBootstrap>,
    connection: Option<PreviewProcessConnection>,
    pending_commands: VecDeque<PreviewCommandEnvelope>,
    pending_command_capacity: usize,
    received_statuses: Vec<ReloadStatus>,
    loaded_products: Vec<RuntimeProductRef>,
    last_ready_session: Option<PreviewSessionId>,
    last_heartbeat_session: Option<PreviewSessionId>,
    last_heartbeat_at: Option<Instant>,
    last_mode_ack: Option<(PreviewSessionId, PreviewMode)>,
    last_shutdown_ack: Option<PreviewSessionId>,
    last_error: Option<String>,
    shutdown_grace_period: Duration,
}

impl PreviewProcessManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pending_command_capacity(capacity: usize) -> Self {
        Self {
            pending_command_capacity: capacity.max(1),
            ..Self::default()
        }
    }

    pub fn bootstrap(&self) -> Option<&PreviewBootstrap> {
        self.bootstrap.as_ref()
    }

    pub fn pending_command_count(&self) -> usize {
        self.pending_commands.len()
    }

    pub fn received_statuses(&self) -> &[ReloadStatus] {
        &self.received_statuses
    }

    pub fn loaded_products(&self) -> &[RuntimeProductRef] {
        &self.loaded_products
    }

    pub fn last_ready_session(&self) -> Option<PreviewSessionId> {
        self.last_ready_session
    }

    pub fn last_heartbeat_session(&self) -> Option<PreviewSessionId> {
        self.last_heartbeat_session
    }

    pub fn last_mode_ack(&self) -> Option<(PreviewSessionId, PreviewMode)> {
        self.last_mode_ack
    }

    pub fn last_shutdown_ack(&self) -> Option<PreviewSessionId> {
        self.last_shutdown_ack
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn spawn_child(&mut self, config: &PreviewProcessSpawnConfig) -> Result<&PreviewBootstrap> {
        let mut command = Command::new(&config.executable_path);
        if config.headless {
            command.arg("--headless");
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| {
                format!(
                    "failed to spawn runtime preview process at {}",
                    config.executable_path.display()
                )
            })?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("runtime preview process stdout was not captured"))?;
        let (line_tx, line_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            let result = reader.read_line(&mut line).map(|_| line);
            let _ = line_tx.send(result);
        });
        let line = match line_rx.recv_timeout(config.bootstrap_timeout) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => {
                kill_and_wait(&mut child)?;
                return Err(error).context("failed to read runtime preview bootstrap line");
            }
            Err(RecvTimeoutError::Timeout) => {
                kill_and_wait(&mut child)?;
                return Err(anyhow!(
                    "timed out waiting for runtime preview bootstrap line"
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                kill_and_wait(&mut child)?;
                return Err(anyhow!(
                    "runtime preview bootstrap reader disconnected before a line was read"
                ));
            }
        };
        let bootstrap = match PreviewBootstrap::parse_stdout_line(&line) {
            Ok(bootstrap) => bootstrap,
            Err(error) => {
                kill_and_wait(&mut child)?;
                return Err(error)
                    .context("runtime preview process emitted invalid bootstrap line");
            }
        };
        self.child = Some(child);
        self.bootstrap = Some(bootstrap);
        self.shutdown_grace_period = config.shutdown_grace_period;
        self.bootstrap
            .as_ref()
            .ok_or_else(|| anyhow!("runtime preview bootstrap was not stored"))
    }

    pub async fn connect(&mut self) -> Result<()> {
        let bootstrap = self
            .bootstrap
            .as_ref()
            .ok_or_else(|| anyhow!("runtime preview process has no bootstrap data"))?;
        let connection = PreviewProcessConnection::connect(bootstrap).await?;
        self.connection = Some(connection);
        self.flush_pending_commands().await
    }

    pub async fn start_session(
        &mut self,
        sequence: u64,
        session_id: PreviewSessionId,
        mode: PreviewMode,
    ) -> Result<()> {
        self.queue_or_send(PreviewCommandEnvelope::new(
            sequence,
            PreviewCommand::StartSession { session_id, mode },
        ))
        .await
    }

    pub async fn request_mode(
        &mut self,
        sequence: u64,
        session_id: PreviewSessionId,
        mode: PreviewMode,
    ) -> Result<()> {
        self.queue_or_send(PreviewCommandEnvelope::new(
            sequence,
            editor_preview::PreviewCommand::ChangeMode { session_id, mode },
        ))
        .await
    }

    pub async fn queue_or_send(&mut self, command: PreviewCommandEnvelope) -> Result<()> {
        if let Some(connection) = &self.connection {
            connection.send_preview_command(command).await
        } else {
            if self.pending_commands.len() >= self.pending_command_capacity {
                return Err(anyhow!(
                    "runtime preview pending command queue is full (capacity {})",
                    self.pending_command_capacity
                ));
            }
            self.pending_commands.push_back(command);
            Ok(())
        }
    }

    pub fn ingest_reload_status(&mut self, status: ReloadStatus) {
        self.received_statuses.push(status);
    }

    pub fn poll_events(&mut self) -> Result<usize> {
        let mut drained = 0usize;
        loop {
            let event = match self.connection.as_mut() {
                Some(connection) => connection.try_next_event()?,
                None => None,
            };
            let Some(event) = event else {
                break;
            };
            self.ingest_runtime_event(event)?;
            drained = drained.saturating_add(1);
        }
        Ok(drained)
    }

    pub fn ingest_runtime_event(&mut self, event: SessionRuntimeEvent) -> Result<()> {
        match event {
            SessionRuntimeEvent::ServerMessage(ServerMessage::TypedPayload(payload)) => {
                let event =
                    editor_preview::decode_preview_event(&preview_payload_from_typed(&payload))?;
                self.ingest_preview_event(event.event);
            }
            SessionRuntimeEvent::Error { message } => {
                self.last_error = Some(message);
            }
            SessionRuntimeEvent::ConnectionClosed { reason, .. } => {
                self.last_error =
                    reason.map(|reason| format!("preview connection closed: {reason:?}"));
            }
            SessionRuntimeEvent::ClientMessage {
                message: ClientMessage::TypedPayload(payload),
                ..
            } => {
                let event =
                    editor_preview::decode_preview_event(&preview_payload_from_typed(&payload))?;
                self.ingest_preview_event(event.event);
            }
            _ => {}
        }
        Ok(())
    }

    fn ingest_preview_event(&mut self, event: PreviewEvent) {
        match event {
            PreviewEvent::Ready { session_id } => {
                self.last_ready_session = Some(session_id);
            }
            PreviewEvent::ModeChanged { session_id, mode } => {
                self.last_mode_ack = Some((session_id, mode));
            }
            PreviewEvent::ProductLoaded { product, .. } => {
                self.loaded_products.push(*product);
            }
            PreviewEvent::ReloadStatus { status, .. } => {
                self.received_statuses.push(*status);
            }
            PreviewEvent::Heartbeat { session_id } => {
                self.last_heartbeat_session = Some(session_id);
                self.last_heartbeat_at = Some(Instant::now());
            }
            PreviewEvent::ShutdownAck { session_id } => {
                self.last_shutdown_ack = Some(session_id);
            }
            PreviewEvent::Error { message, .. } => {
                self.last_error = Some(message);
            }
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        let shutdown_session = self.shutdown_session_id();
        if self.connection.is_some() {
            if let Some(connection) = &self.connection {
                let _ = connection
                    .send_preview_command(PreviewCommandEnvelope::new(
                        0,
                        PreviewCommand::Shutdown {
                            session_id: shutdown_session,
                        },
                    ))
                    .await;
            }
            let deadline = Instant::now() + self.shutdown_grace_period;
            while Instant::now() < deadline {
                self.poll_events()?;
                if self.last_shutdown_ack == Some(shutdown_session) {
                    break;
                }
                tokio::time::sleep(POLL_SLEEP).await;
            }
            if let Some(connection) = &self.connection {
                let _ = connection.shutdown().await;
            }
        }
        if let Some(child) = &mut self.child {
            wait_child_until_exit(child, self.shutdown_grace_period).await?;
            if child.try_wait()?.is_none() {
                kill_and_wait(child)?;
            }
        }
        self.child = None;
        self.connection = None;
        Ok(())
    }

    async fn flush_pending_commands(&mut self) -> Result<()> {
        if let Some(connection) = &self.connection {
            while let Some(command) = self.pending_commands.pop_front() {
                if let Err(error) = connection.send_preview_command(command.clone()).await {
                    self.pending_commands.push_front(command);
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn shutdown_session_id(&self) -> PreviewSessionId {
        self.last_heartbeat_session
            .or_else(|| self.last_mode_ack.map(|(session_id, _)| session_id))
            .or(self.last_ready_session)
            .unwrap_or_else(|| preview_session_id(1))
    }
}

impl Default for PreviewProcessManager {
    fn default() -> Self {
        Self {
            child: None,
            bootstrap: None,
            connection: None,
            pending_commands: VecDeque::new(),
            pending_command_capacity: DEFAULT_PENDING_COMMAND_CAPACITY,
            received_statuses: Vec::new(),
            loaded_products: Vec::new(),
            last_ready_session: None,
            last_heartbeat_session: None,
            last_heartbeat_at: None,
            last_mode_ack: None,
            last_shutdown_ack: None,
            last_error: None,
            shutdown_grace_period: DEFAULT_SHUTDOWN_GRACE_PERIOD,
        }
    }
}

async fn wait_child_until_exit(child: &mut Child, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        tokio::time::sleep(POLL_SLEEP).await;
    }
    Ok(())
}

fn kill_and_wait(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_none() {
        child.kill()?;
    }
    let _ = child.wait()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_preview::{
        PreviewEventEnvelope, ReloadDecision, ReloadSubject, ReloadSubjectKind,
        encode_preview_event, preview_session_id,
    };
    use engine_net::{ServerMessage, TypedPayloadMessage};

    #[tokio::test]
    async fn commands_queue_until_connection_exists() {
        let mut manager = PreviewProcessManager::new();

        manager
            .request_mode(1, preview_session_id(1), PreviewMode::Preview)
            .await
            .expect("mode request should queue");

        assert_eq!(manager.pending_command_count(), 1);
    }

    #[tokio::test]
    async fn pending_command_queue_is_bounded() {
        let mut manager = PreviewProcessManager::with_pending_command_capacity(1);

        manager
            .start_session(1, preview_session_id(1), PreviewMode::Preview)
            .await
            .expect("first command should queue");
        let error = manager
            .request_mode(2, preview_session_id(1), PreviewMode::Play)
            .await
            .expect_err("second command should exceed bounded queue");

        assert!(error.to_string().contains("pending command queue is full"));
        assert_eq!(manager.pending_command_count(), 1);
    }

    #[test]
    fn manager_records_reload_statuses_for_existing_surfaces() {
        let mut manager = PreviewProcessManager::new();
        manager.ingest_reload_status(ReloadStatus::new(
            ReloadSubject::new(ReloadSubjectKind::Shader, "shader"),
            ReloadDecision::LiveReload,
            "shader reloaded",
        ));

        assert_eq!(manager.received_statuses().len(), 1);
        assert_eq!(
            manager.received_statuses()[0].decision,
            ReloadDecision::LiveReload
        );
    }

    #[test]
    fn ingest_runtime_event_records_preview_event_state() {
        let mut manager = PreviewProcessManager::new();
        let session_id = preview_session_id(7);
        manager
            .ingest_runtime_event(preview_server_event(PreviewEventEnvelope::new(
                1,
                PreviewEvent::Ready { session_id },
            )))
            .expect("ready event should ingest");
        manager
            .ingest_runtime_event(preview_server_event(PreviewEventEnvelope::new(
                2,
                PreviewEvent::ModeChanged {
                    session_id,
                    mode: PreviewMode::Simulate,
                },
            )))
            .expect("mode event should ingest");
        manager
            .ingest_runtime_event(preview_server_event(PreviewEventEnvelope::new(
                3,
                PreviewEvent::Heartbeat { session_id },
            )))
            .expect("heartbeat event should ingest");
        manager
            .ingest_runtime_event(preview_server_event(PreviewEventEnvelope::new(
                4,
                PreviewEvent::ReloadStatus {
                    session_id,
                    status: Box::new(ReloadStatus::new(
                        ReloadSubject::new(ReloadSubjectKind::Shader, "shader"),
                        ReloadDecision::LiveReload,
                        "shader reloaded",
                    )),
                },
            )))
            .expect("reload event should ingest");
        manager
            .ingest_runtime_event(preview_server_event(PreviewEventEnvelope::new(
                5,
                PreviewEvent::ShutdownAck { session_id },
            )))
            .expect("shutdown event should ingest");

        assert_eq!(manager.last_ready_session(), Some(session_id));
        assert_eq!(
            manager.last_mode_ack(),
            Some((session_id, PreviewMode::Simulate))
        );
        assert_eq!(manager.last_heartbeat_session(), Some(session_id));
        assert_eq!(manager.last_shutdown_ack(), Some(session_id));
        assert_eq!(manager.received_statuses().len(), 1);
    }

    fn preview_server_event(envelope: PreviewEventEnvelope) -> SessionRuntimeEvent {
        let payload = encode_preview_event(&envelope).expect("preview event should encode");
        SessionRuntimeEvent::ServerMessage(ServerMessage::TypedPayload(TypedPayloadMessage::new(
            payload.channel,
            payload.type_name,
            payload.schema_version,
            payload.payload,
        )))
    }
}
