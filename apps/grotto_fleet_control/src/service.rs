use crate::config::FleetServiceConfig;
use crate::provider::{FleetError, FleetProvider};
use crate::router::{FleetCommandContext, FleetCommandOutput, execute_fleet_command};
use anyhow::{Context, Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use grotto_online::{
    AxiomOperatorCommand, AxiomOperatorCommandKind, AxiomOperatorCommandResult,
    AxiomOperatorCommandStatus, AxiomOperatorEvent, AxiomOperatorInboundMessage,
    AxiomOperatorOutboundMessage,
};
use serde_json::json;
use std::collections::{BTreeSet, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, HeaderValue, USER_AGENT};
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};

#[derive(Debug, Clone)]
pub struct FleetServiceRuntime {
    ws_url: String,
    command_token: String,
    service_id: String,
    heartbeat_seconds: u64,
    reconnect_backoff_ms: u64,
    max_buffered_events: usize,
    runtime_graceful_stop_enabled: bool,
    runtime_graceful_default_timeout_ms: u64,
    runtime_force_stop_timeout_ms: u64,
    command_context: FleetCommandContext,
}

impl FleetServiceRuntime {
    pub fn from_config(config: &FleetServiceConfig) -> Result<Self> {
        if !config.axiom.enabled {
            return Err(anyhow!("axiom bridge is disabled in fleet config"));
        }
        if config.axiom.ws_url.trim().is_empty() {
            return Err(anyhow!("fleet axiom.ws_url must not be empty"));
        }
        if config.axiom.command_token.trim().is_empty() {
            return Err(anyhow!("fleet axiom.command_token must not be empty"));
        }

        let authorized_server_ids = (!config.axiom.allowed_server_ids.is_empty()).then(|| {
            config
                .axiom
                .allowed_server_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        });

        Ok(Self {
            ws_url: config.axiom.ws_url.clone(),
            command_token: config.axiom.command_token.clone(),
            service_id: config.axiom.service_id.clone(),
            heartbeat_seconds: config.axiom.heartbeat_seconds.max(1),
            reconnect_backoff_ms: config.axiom.reconnect_backoff_ms.max(100),
            max_buffered_events: config.axiom.max_buffered_events.max(1),
            runtime_graceful_stop_enabled: config.axiom.runtime_graceful_stop_enabled,
            runtime_graceful_default_timeout_ms: config.axiom.runtime_graceful_default_timeout_ms,
            runtime_force_stop_timeout_ms: config.axiom.runtime_force_stop_timeout_ms,
            command_context: FleetCommandContext {
                authorized_server_ids,
            },
        })
    }

    pub async fn run(&self, provider: &dyn FleetProvider) -> Result<()> {
        let mut pending = VecDeque::<AxiomOperatorOutboundMessage>::new();
        let reconnect_delay = Duration::from_millis(self.reconnect_backoff_ms);

        loop {
            let mut request = self
                .ws_url
                .as_str()
                .into_client_request()
                .with_context(|| format!("invalid fleet websocket url {}", self.ws_url))?;
            request.headers_mut().insert(
                USER_AGENT,
                HeaderValue::from_static("grotto-quest-demo-fleet-control/1"),
            );
            let bearer = format!("Bearer {}", self.command_token);
            let bearer = HeaderValue::from_str(&bearer)
                .context("fleet command_token produced an invalid auth header")?;
            request.headers_mut().insert(AUTHORIZATION, bearer);

            match tokio_tungstenite::connect_async(request).await {
                Ok((mut stream, _)) => {
                    push_with_limit(
                        &mut pending,
                        AxiomOperatorOutboundMessage::Hello {
                            server_id: self.service_id.clone(),
                            started_at_ms: unix_now_millis(),
                            capabilities: vec![
                                "start_server".to_string(),
                                "stop_server".to_string(),
                                "inspect_logs".to_string(),
                            ],
                        },
                        self.max_buffered_events,
                    );

                    let mut heartbeat =
                        tokio::time::interval(Duration::from_secs(self.heartbeat_seconds));
                    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                    'connected: loop {
                        while let Some(message) = pending.pop_front() {
                            let text = match serde_json::to_string(&message) {
                                Ok(text) => text,
                                Err(error) => {
                                    eprintln!("failed to encode fleet outbound message: {error}");
                                    continue;
                                }
                            };
                            if let Err(error) = stream.send(Message::Text(text.into())).await {
                                push_front_with_limit(
                                    &mut pending,
                                    message,
                                    self.max_buffered_events,
                                );
                                eprintln!("fleet websocket send failed: {error}");
                                break 'connected;
                            }
                        }

                        tokio::select! {
                            incoming = stream.next() => {
                                match incoming {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Err(error) = self
                                            .handle_incoming_text(provider, &mut pending, text.as_ref())
                                            .await
                                        {
                                            eprintln!("fleet inbound message handling failed: {error}");
                                        }
                                    }
                                    Some(Ok(Message::Binary(bytes))) => {
                                        match String::from_utf8(bytes.to_vec()) {
                                            Ok(text) => {
                                                if let Err(error) = self
                                                    .handle_incoming_text(provider, &mut pending, &text)
                                                    .await
                                                {
                                                    eprintln!("fleet inbound binary message handling failed: {error}");
                                                }
                                            }
                                            Err(error) => {
                                                eprintln!("fleet inbound binary payload was not UTF-8: {error}");
                                            }
                                        }
                                    }
                                    Some(Ok(Message::Ping(payload))) => {
                                        if let Err(error) = stream.send(Message::Pong(payload)).await {
                                            eprintln!("fleet websocket pong failed: {error}");
                                            break 'connected;
                                        }
                                    }
                                    Some(Ok(Message::Pong(_))) => {}
                                    Some(Ok(Message::Frame(_))) => {}
                                    Some(Ok(Message::Close(_))) => break 'connected,
                                    Some(Err(error)) => {
                                        eprintln!("fleet websocket read failed: {error}");
                                        break 'connected;
                                    }
                                    None => break 'connected,
                                }
                            }
                            _ = heartbeat.tick() => {
                                if let Err(error) = stream.send(Message::Ping(Vec::new().into())).await {
                                    eprintln!("fleet heartbeat ping failed: {error}");
                                    break 'connected;
                                }
                            }
                            _ = tokio::signal::ctrl_c() => {
                                let _ = stream.close(None).await;
                                return Ok(());
                            }
                        }
                    }
                }
                Err(error) => {
                    eprintln!("fleet websocket connect failed: {error}");
                }
            }

            tokio::select! {
                _ = tokio::signal::ctrl_c() => return Ok(()),
                _ = tokio::time::sleep(reconnect_delay) => {}
            }
        }
    }

    async fn handle_incoming_text(
        &self,
        provider: &dyn FleetProvider,
        pending: &mut VecDeque<AxiomOperatorOutboundMessage>,
        text: &str,
    ) -> Result<()> {
        let message: AxiomOperatorInboundMessage = serde_json::from_str(text)
            .with_context(|| format!("invalid fleet inbound payload: {text}"))?;
        match message {
            AxiomOperatorInboundMessage::Ping { ts_ms } => {
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::Pong { ts_ms },
                    self.max_buffered_events,
                );
            }
            AxiomOperatorInboundMessage::Command(command) => {
                self.handle_command(provider, pending, command).await?;
            }
        }
        Ok(())
    }

    async fn handle_command(
        &self,
        provider: &dyn FleetProvider,
        pending: &mut VecDeque<AxiomOperatorOutboundMessage>,
        command: AxiomOperatorCommand,
    ) -> Result<()> {
        let provider_command = if let AxiomOperatorCommandKind::StopServer {
            graceful_timeout_ms,
        } = &command.kind
        {
            if self.runtime_graceful_stop_enabled {
                let graceful_timeout_ms = graceful_timeout_ms
                    .unwrap_or(self.runtime_graceful_default_timeout_ms)
                    .max(1);
                self.queue_runtime_graceful_stop(pending, &command, graceful_timeout_ms);
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::Event(AxiomOperatorEvent {
                        server_id: command.target_server_id.clone(),
                        event_type: "fleet.runtime_graceful_stop_requested".to_string(),
                        ts_ms: unix_now_millis(),
                        command_id: Some(command.command_id.clone()),
                        payload: json!({
                            "graceful_timeout_ms": graceful_timeout_ms,
                        }),
                    }),
                    self.max_buffered_events,
                );
                tokio::time::sleep(Duration::from_millis(graceful_timeout_ms)).await;
            }
            AxiomOperatorCommand {
                command_id: command.command_id.clone(),
                target_server_id: command.target_server_id.clone(),
                issued_at_ms: command.issued_at_ms,
                kind: AxiomOperatorCommandKind::StopServer {
                    graceful_timeout_ms: Some(self.runtime_force_stop_timeout_ms.max(1)),
                },
            }
        } else {
            command.clone()
        };

        match execute_fleet_command(provider, &provider_command, &self.command_context).await {
            Ok(Some(output)) => {
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::CommandResult(AxiomOperatorCommandResult {
                        command_id: command.command_id.clone(),
                        server_id: command.target_server_id.clone(),
                        status: AxiomOperatorCommandStatus::Accepted,
                        message: None,
                        ts_ms: unix_now_millis(),
                    }),
                    self.max_buffered_events,
                );
                let (event_type, payload) = match output {
                    FleetCommandOutput::Start(status) => (
                        "fleet.start_server".to_string(),
                        json!({ "status": status }),
                    ),
                    FleetCommandOutput::Stop(status) => {
                        ("fleet.stop_server".to_string(), json!({ "status": status }))
                    }
                    FleetCommandOutput::Logs(page) => {
                        ("fleet.inspect_logs".to_string(), json!({ "page": page }))
                    }
                };
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::Event(AxiomOperatorEvent {
                        server_id: command.target_server_id,
                        event_type,
                        ts_ms: unix_now_millis(),
                        command_id: Some(command.command_id),
                        payload,
                    }),
                    self.max_buffered_events,
                );
            }
            Ok(None) => {
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::CommandResult(AxiomOperatorCommandResult {
                        command_id: command.command_id.clone(),
                        server_id: command.target_server_id.clone(),
                        status: AxiomOperatorCommandStatus::Rejected,
                        message: Some(
                            "command is not supported by fleet control service".to_string(),
                        ),
                        ts_ms: unix_now_millis(),
                    }),
                    self.max_buffered_events,
                );
            }
            Err(error) => {
                let status = map_fleet_error_status(&error);
                let error_message = error.to_string();
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::CommandResult(AxiomOperatorCommandResult {
                        command_id: command.command_id.clone(),
                        server_id: command.target_server_id.clone(),
                        status,
                        message: Some(error_message.clone()),
                        ts_ms: unix_now_millis(),
                    }),
                    self.max_buffered_events,
                );
                push_with_limit(
                    pending,
                    AxiomOperatorOutboundMessage::Event(AxiomOperatorEvent {
                        server_id: command.target_server_id,
                        event_type: "fleet.command_failed".to_string(),
                        ts_ms: unix_now_millis(),
                        command_id: Some(command.command_id),
                        payload: json!({ "error": error_message }),
                    }),
                    self.max_buffered_events,
                );
            }
        }
        Ok(())
    }

    fn queue_runtime_graceful_stop(
        &self,
        pending: &mut VecDeque<AxiomOperatorOutboundMessage>,
        command: &AxiomOperatorCommand,
        graceful_timeout_ms: u64,
    ) {
        let drain_command = AxiomOperatorCommand {
            command_id: format!("{}:runtime-drain", command.command_id),
            target_server_id: command.target_server_id.clone(),
            issued_at_ms: Some(unix_now_millis()),
            kind: AxiomOperatorCommandKind::SetDrainMode { enabled: true },
        };
        let shutdown_command = AxiomOperatorCommand {
            command_id: format!("{}:runtime-shutdown", command.command_id),
            target_server_id: command.target_server_id.clone(),
            issued_at_ms: Some(unix_now_millis()),
            kind: AxiomOperatorCommandKind::Shutdown {
                grace_ms: Some(graceful_timeout_ms),
            },
        };
        push_with_limit(
            pending,
            AxiomOperatorOutboundMessage::DispatchCommand {
                command: drain_command,
            },
            self.max_buffered_events,
        );
        push_with_limit(
            pending,
            AxiomOperatorOutboundMessage::DispatchCommand {
                command: shutdown_command,
            },
            self.max_buffered_events,
        );
    }
}

pub async fn run_fleet_service(
    provider: &dyn FleetProvider,
    config: FleetServiceConfig,
) -> Result<()> {
    let runtime = FleetServiceRuntime::from_config(&config)?;
    runtime.run(provider).await
}

fn map_fleet_error_status(error: &FleetError) -> AxiomOperatorCommandStatus {
    match error {
        FleetError::Unauthorized(_) | FleetError::Invalid(_) | FleetError::NotFound(_) => {
            AxiomOperatorCommandStatus::Rejected
        }
        FleetError::Timeout(_) | FleetError::Provider(_) => AxiomOperatorCommandStatus::Failed,
    }
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
    use crate::config::{FleetAxiomBridgeConfig, FleetKubernetesConfig};
    use crate::provider::{FleetLogPage, FleetServerState, FleetServerStatus};
    use crate::router::FleetCommandOutput;

    #[test]
    fn maps_expected_status_for_fleet_errors() {
        assert_eq!(
            map_fleet_error_status(&FleetError::Unauthorized("nope".to_string())),
            AxiomOperatorCommandStatus::Rejected
        );
        assert_eq!(
            map_fleet_error_status(&FleetError::Provider("boom".to_string())),
            AxiomOperatorCommandStatus::Failed
        );
    }

    #[test]
    fn runtime_builds_authorization_context_from_allowed_servers() {
        let config = FleetServiceConfig {
            kubernetes: FleetKubernetesConfig::default(),
            axiom: FleetAxiomBridgeConfig {
                enabled: true,
                ws_url: "ws://localhost:1234".to_string(),
                command_token: "token".to_string(),
                service_id: "fleet-control-a".to_string(),
                heartbeat_seconds: 10,
                reconnect_backoff_ms: 500,
                max_buffered_events: 128,
                runtime_graceful_stop_enabled: true,
                runtime_graceful_default_timeout_ms: 6_000,
                runtime_force_stop_timeout_ms: 10_000,
                allowed_server_ids: vec!["srv-a".to_string(), "srv-b".to_string()],
            },
        };
        let runtime = FleetServiceRuntime::from_config(&config).expect("config should validate");
        let authorized = runtime
            .command_context
            .authorized_server_ids
            .as_ref()
            .expect("authorized ids should be present");
        assert!(authorized.contains("srv-a"));
        assert!(authorized.contains("srv-b"));
    }

    #[test]
    fn push_with_limit_evicts_oldest_entries() {
        let mut queue = VecDeque::new();
        push_with_limit(&mut queue, 1u32, 2);
        push_with_limit(&mut queue, 2u32, 2);
        push_with_limit(&mut queue, 3u32, 2);
        assert_eq!(queue.into_iter().collect::<Vec<_>>(), vec![2, 3]);
    }

    #[test]
    fn fleet_output_serializes_to_expected_event_payload_shapes() {
        let start_output = FleetCommandOutput::Start(FleetServerStatus {
            server_id: "srv-a".to_string(),
            state: FleetServerState::Running,
            endpoint: Some("10.0.0.1:7000".to_string()),
            details: Some("ready".to_string()),
        });
        let logs_output = FleetCommandOutput::Logs(FleetLogPage {
            server_id: "srv-a".to_string(),
            lines: Vec::new(),
            next_cursor: Some("5".to_string()),
        });
        let start_payload = match start_output {
            FleetCommandOutput::Start(status) => json!({ "status": status }),
            _ => unreachable!(),
        };
        let logs_payload = match logs_output {
            FleetCommandOutput::Logs(page) => json!({ "page": page }),
            _ => unreachable!(),
        };
        assert!(start_payload.get("status").is_some());
        assert!(logs_payload.get("page").is_some());
    }
}
