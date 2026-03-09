use crate::provider::{FleetError, FleetLogPage, FleetProvider, FleetServerStatus};
use grotto_online::{AxiomOperatorCommand, AxiomOperatorCommandKind};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FleetCommandContext {
    pub authorized_server_ids: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FleetCommandOutput {
    Start(FleetServerStatus),
    Stop(FleetServerStatus),
    Logs(FleetLogPage),
}

pub async fn execute_fleet_command(
    provider: &dyn FleetProvider,
    command: &AxiomOperatorCommand,
    context: &FleetCommandContext,
) -> Result<Option<FleetCommandOutput>, FleetError> {
    if let Some(authorized) = &context.authorized_server_ids
        && !authorized.contains(&command.target_server_id)
    {
        return Err(FleetError::Unauthorized(format!(
            "server_id {} is not authorized",
            command.target_server_id
        )));
    }

    match &command.kind {
        AxiomOperatorCommandKind::StartServer { .. } => {
            let status = provider.start_server(&command.target_server_id).await?;
            Ok(Some(FleetCommandOutput::Start(status)))
        }
        AxiomOperatorCommandKind::StopServer {
            graceful_timeout_ms,
        } => {
            let status = provider
                .stop_server(&command.target_server_id, *graceful_timeout_ms)
                .await?;
            Ok(Some(FleetCommandOutput::Stop(status)))
        }
        AxiomOperatorCommandKind::InspectLogs { query } => {
            let page = provider
                .inspect_logs(&command.target_server_id, query)
                .await?;
            Ok(Some(FleetCommandOutput::Logs(page)))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{FleetLogLine, FleetServerState};
    use async_trait::async_trait;
    use grotto_online::{AxiomLogWindowQuery, AxiomOperatorCommandKind};

    struct TestProvider;

    #[async_trait]
    impl FleetProvider for TestProvider {
        async fn start_server(&self, server_id: &str) -> Result<FleetServerStatus, FleetError> {
            if server_id == "missing" {
                return Err(FleetError::NotFound(server_id.to_string()));
            }
            Ok(FleetServerStatus {
                server_id: server_id.to_string(),
                state: FleetServerState::Running,
                endpoint: Some("127.0.0.1:7000".to_string()),
                details: None,
            })
        }

        async fn stop_server(
            &self,
            server_id: &str,
            _graceful_timeout_ms: Option<u64>,
        ) -> Result<FleetServerStatus, FleetError> {
            if server_id == "missing" {
                return Err(FleetError::NotFound(server_id.to_string()));
            }
            Ok(FleetServerStatus {
                server_id: server_id.to_string(),
                state: FleetServerState::Stopped,
                endpoint: None,
                details: None,
            })
        }

        async fn inspect_logs(
            &self,
            server_id: &str,
            _query: &AxiomLogWindowQuery,
        ) -> Result<FleetLogPage, FleetError> {
            if server_id == "missing" {
                return Err(FleetError::NotFound(server_id.to_string()));
            }
            Ok(FleetLogPage {
                server_id: server_id.to_string(),
                lines: vec![FleetLogLine {
                    ts_ms: Some(1),
                    level: Some("info".to_string()),
                    message: "ready".to_string(),
                }],
                next_cursor: None,
            })
        }
    }

    #[tokio::test]
    async fn router_rejects_unauthorized_server_id() {
        let mut authorized = BTreeSet::new();
        authorized.insert("srv-allowed".to_string());
        let ctx = FleetCommandContext {
            authorized_server_ids: Some(authorized),
        };
        let command = AxiomOperatorCommand {
            command_id: "cmd-1".to_string(),
            target_server_id: "srv-other".to_string(),
            issued_at_ms: None,
            kind: AxiomOperatorCommandKind::StartServer { profile: None },
        };

        let error = execute_fleet_command(&TestProvider, &command, &ctx)
            .await
            .expect_err("unauthorized command should fail");
        assert!(matches!(error, FleetError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn router_returns_not_found_for_missing_server() {
        let ctx = FleetCommandContext::default();
        let command = AxiomOperatorCommand {
            command_id: "cmd-2".to_string(),
            target_server_id: "missing".to_string(),
            issued_at_ms: None,
            kind: AxiomOperatorCommandKind::StartServer { profile: None },
        };

        let error = execute_fleet_command(&TestProvider, &command, &ctx)
            .await
            .expect_err("missing server should fail");
        assert!(matches!(error, FleetError::NotFound(_)));
    }
}
