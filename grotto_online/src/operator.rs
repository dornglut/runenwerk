use anyhow::{Context, Result, anyhow};
use engine_net::DisconnectReason;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};
use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, HeaderValue, USER_AGENT};
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};

const DEFAULT_HEARTBEAT_SECONDS: u64 = 10;
const DEFAULT_RECONNECT_BACKOFF_MS: u64 = 500;
const DEFAULT_BUFFERED_EVENTS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomOperatorBridgeConfig {
    pub ws_url: String,
    pub runtime_token: String,
    pub server_id: String,
    pub heartbeat_seconds: u64,
    pub reconnect_backoff_ms: u64,
    pub max_buffered_events: usize,
}

impl Default for AxiomOperatorBridgeConfig {
    fn default() -> Self {
        Self {
            ws_url: String::new(),
            runtime_token: String::new(),
            server_id: "srv-local".to_string(),
            heartbeat_seconds: DEFAULT_HEARTBEAT_SECONDS,
            reconnect_backoff_ms: DEFAULT_RECONNECT_BACKOFF_MS,
            max_buffered_events: DEFAULT_BUFFERED_EVENTS,
        }
    }
}

impl AxiomOperatorBridgeConfig {
    fn validated(&self) -> Result<Self> {
        if self.ws_url.trim().is_empty() {
            return Err(anyhow!("Axiom operator ws_url must not be empty"));
        }
        if self.runtime_token.trim().is_empty() {
            return Err(anyhow!("Axiom operator runtime_token must not be empty"));
        }
        if self.server_id.trim().is_empty() {
            return Err(anyhow!("Axiom operator server_id must not be empty"));
        }
        let mut copy = self.clone();
        copy.heartbeat_seconds = copy.heartbeat_seconds.max(1);
        copy.reconnect_backoff_ms = copy.reconnect_backoff_ms.max(100);
        copy.max_buffered_events = copy.max_buffered_events.max(1);
        Ok(copy)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxiomLogLevelFilter {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomLogWindowQuery {
    pub from_ts_ms: Option<u64>,
    pub to_ts_ms: Option<u64>,
    pub level: Option<AxiomLogLevelFilter>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AxiomOperatorCommandKind {
    SetDrainMode {
        enabled: bool,
    },
    DisconnectConnection {
        connection_id: u64,
        reason: DisconnectReason,
    },
    Shutdown {
        grace_ms: Option<u64>,
    },
    SnapshotNow,
    StartServer {
        profile: Option<String>,
    },
    StopServer {
        graceful_timeout_ms: Option<u64>,
    },
    InspectLogs {
        query: AxiomLogWindowQuery,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomOperatorCommand {
    pub command_id: String,
    pub target_server_id: String,
    pub issued_at_ms: Option<u64>,
    #[serde(flatten)]
    pub kind: AxiomOperatorCommandKind,
}

impl AxiomOperatorCommand {
    pub fn targets_server(&self, server_id: &str) -> bool {
        self.target_server_id == server_id
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxiomOperatorCommandStatus {
    Accepted,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomOperatorCommandResult {
    pub command_id: String,
    pub server_id: String,
    pub status: AxiomOperatorCommandStatus,
    pub message: Option<String>,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxiomOperatorSnapshot {
    pub server_id: String,
    pub ts_ms: u64,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxiomOperatorEvent {
    pub server_id: String,
    pub event_type: String,
    pub ts_ms: u64,
    pub command_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AxiomOperatorInboundMessage {
    Command(AxiomOperatorCommand),
    Ping { ts_ms: Option<u64> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AxiomOperatorOutboundMessage {
    Hello {
        server_id: String,
        started_at_ms: u64,
        capabilities: Vec<String>,
    },
    Snapshot(AxiomOperatorSnapshot),
    Event(AxiomOperatorEvent),
    CommandResult(AxiomOperatorCommandResult),
    DispatchCommand {
        command: AxiomOperatorCommand,
    },
    Pong {
        ts_ms: Option<u64>,
    },
}

pub struct AxiomOperatorRuntimeHandle {
    command_rx: Receiver<AxiomOperatorCommand>,
    outbound_tx: Sender<AxiomOperatorOutboundMessage>,
}

impl AxiomOperatorRuntimeHandle {
    pub fn send_outbound(&self, message: AxiomOperatorOutboundMessage) -> Result<(), String> {
        self.outbound_tx
            .try_send(message)
            .map_err(|error| format!("axiom operator outbound queue send failed: {error}"))
    }

    pub fn try_recv_command(&mut self) -> Result<Option<AxiomOperatorCommand>, TryRecvError> {
        match self.command_rx.try_recv() {
            Ok(command) => Ok(Some(command)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

pub fn spawn_axiom_operator_bridge(
    config: AxiomOperatorBridgeConfig,
) -> Result<AxiomOperatorRuntimeHandle> {
    let config = config.validated()?;
    let (outbound_tx, outbound_rx) = tokio::sync::mpsc::channel(config.max_buffered_events);
    let (command_tx, command_rx) = tokio::sync::mpsc::channel(config.max_buffered_events);
    tokio::spawn(async move {
        if let Err(error) = run_axiom_operator_bridge(config, outbound_rx, command_tx).await {
            eprintln!("axiom operator bridge stopped: {error:#}");
        }
    });
    Ok(AxiomOperatorRuntimeHandle {
        command_rx,
        outbound_tx,
    })
}

async fn run_axiom_operator_bridge(
    config: AxiomOperatorBridgeConfig,
    mut outbound_rx: Receiver<AxiomOperatorOutboundMessage>,
    command_tx: Sender<AxiomOperatorCommand>,
) -> Result<()> {
    let mut pending = VecDeque::<AxiomOperatorOutboundMessage>::new();
    let reconnect_delay = Duration::from_millis(config.reconnect_backoff_ms);
    loop {
        let mut request = config
            .ws_url
            .as_str()
            .into_client_request()
            .with_context(|| format!("invalid operator websocket url {}", config.ws_url))?;
        request.headers_mut().insert(
            USER_AGENT,
            HeaderValue::from_static("grotto-quest-operator/1"),
        );
        let bearer = format!("Bearer {}", config.runtime_token);
        let bearer = HeaderValue::from_str(&bearer)
            .context("axiom operator runtime_token produced an invalid auth header")?;
        request.headers_mut().insert(AUTHORIZATION, bearer);

        match tokio_tungstenite::connect_async(request).await {
            Ok((mut stream, _)) => {
                push_with_limit(
                    &mut pending,
                    AxiomOperatorOutboundMessage::Hello {
                        server_id: config.server_id.clone(),
                        started_at_ms: unix_now_millis(),
                        capabilities: vec![
                            "set_drain_mode".to_string(),
                            "disconnect_connection".to_string(),
                            "shutdown".to_string(),
                            "snapshot_now".to_string(),
                            "start_server".to_string(),
                            "stop_server".to_string(),
                            "inspect_logs".to_string(),
                        ],
                    },
                    config.max_buffered_events,
                );

                let mut heartbeat =
                    tokio::time::interval(Duration::from_secs(config.heartbeat_seconds));
                heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                'connected: loop {
                    while let Some(message) = pending.pop_front() {
                        let text = match serde_json::to_string(&message) {
                            Ok(text) => text,
                            Err(error) => {
                                eprintln!(
                                    "failed to encode axiom operator outbound message: {error}"
                                );
                                continue;
                            }
                        };
                        if let Err(error) = stream.send(Message::Text(text.into())).await {
                            push_front_with_limit(
                                &mut pending,
                                message,
                                config.max_buffered_events,
                            );
                            eprintln!("axiom operator websocket send failed: {error}");
                            break 'connected;
                        }
                    }

                    tokio::select! {
                        maybe_outbound = outbound_rx.recv() => {
                            match maybe_outbound {
                                Some(outbound) => {
                                    push_with_limit(&mut pending, outbound, config.max_buffered_events);
                                }
                                None => {
                                    let _ = stream.close(None).await;
                                    return Ok(());
                                }
                            }
                        }
                        incoming = stream.next() => {
                            match incoming {
                                Some(Ok(Message::Text(text))) => {
                                    if let Err(error) =
                                        handle_incoming_text(&config, &command_tx, &mut pending, text.as_ref()).await
                                    {
                                        eprintln!("axiom operator inbound message handling failed: {error}");
                                    }
                                }
                                Some(Ok(Message::Binary(bytes))) => {
                                    match String::from_utf8(bytes.to_vec()) {
                                        Ok(text) => {
                                            if let Err(error) =
                                                handle_incoming_text(&config, &command_tx, &mut pending, &text).await
                                            {
                                                eprintln!("axiom operator inbound binary message handling failed: {error}");
                                            }
                                        }
                                        Err(error) => {
                                            eprintln!("axiom operator inbound binary message was not valid UTF-8: {error}");
                                        }
                                    }
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    if let Err(error) = stream.send(Message::Pong(payload)).await {
                                        eprintln!("axiom operator websocket pong failed: {error}");
                                        break 'connected;
                                    }
                                }
                                Some(Ok(Message::Pong(_))) => {}
                                Some(Ok(Message::Frame(_))) => {}
                                Some(Ok(Message::Close(_))) => break 'connected,
                                Some(Err(error)) => {
                                    eprintln!("axiom operator websocket read failed: {error}");
                                    break 'connected;
                                }
                                None => break 'connected,
                            }
                        }
                        _ = heartbeat.tick() => {
                            if let Err(error) = stream.send(Message::Ping(Vec::new().into())).await {
                                eprintln!("axiom operator heartbeat ping failed: {error}");
                                break 'connected;
                            }
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("axiom operator websocket connect failed: {error}");
            }
        }

        tokio::time::sleep(reconnect_delay).await;
    }
}

async fn handle_incoming_text(
    config: &AxiomOperatorBridgeConfig,
    command_tx: &Sender<AxiomOperatorCommand>,
    pending: &mut VecDeque<AxiomOperatorOutboundMessage>,
    text: &str,
) -> Result<()> {
    let message: AxiomOperatorInboundMessage = serde_json::from_str(text)
        .with_context(|| format!("invalid operator inbound payload: {text}"))?;
    match message {
        AxiomOperatorInboundMessage::Command(command) => {
            if !command.targets_server(&config.server_id) {
                return Ok(());
            }
            if let Err(error) = command_tx.try_send(command) {
                return Err(anyhow!("operator command queue send failed: {error}"));
            }
        }
        AxiomOperatorInboundMessage::Ping { ts_ms } => {
            push_with_limit(
                pending,
                AxiomOperatorOutboundMessage::Pong { ts_ms },
                config.max_buffered_events,
            );
        }
    }
    Ok(())
}

fn push_with_limit<T>(queue: &mut VecDeque<T>, value: T, max: usize) {
    if queue.len() >= max {
        let _ = queue.pop_front();
    }
    queue.push_back(value);
}

fn push_front_with_limit<T>(queue: &mut VecDeque<T>, value: T, max: usize) {
    if queue.len() >= max {
        let _ = queue.pop_back();
    }
    queue.push_front(value);
}

fn unix_now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn command_round_trip_parses_expected_variant() {
        let value = json!({
            "type": "command",
            "command_id": "cmd-1",
            "target_server_id": "srv-a",
            "issued_at_ms": 123,
            "op": "set_drain_mode",
            "enabled": true
        });
        let parsed: AxiomOperatorInboundMessage =
            serde_json::from_value(value).expect("command should parse");
        let AxiomOperatorInboundMessage::Command(command) = parsed else {
            panic!("expected command");
        };
        assert_eq!(command.command_id, "cmd-1");
        assert!(command.targets_server("srv-a"));
        assert!(matches!(
            command.kind,
            AxiomOperatorCommandKind::SetDrainMode { enabled: true }
        ));
    }

    #[test]
    fn pending_queue_respects_limit() {
        let mut queue = VecDeque::new();
        push_with_limit(&mut queue, 1, 2);
        push_with_limit(&mut queue, 2, 2);
        push_with_limit(&mut queue, 3, 2);
        assert_eq!(queue, VecDeque::from([2, 3]));
    }
}
