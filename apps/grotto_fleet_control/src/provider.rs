use async_trait::async_trait;
use grotto_online::AxiomLogWindowQuery;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetServerState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetServerStatus {
    pub server_id: String,
    pub state: FleetServerState,
    pub endpoint: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetLogLine {
    pub ts_ms: Option<u64>,
    pub level: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetLogPage {
    pub server_id: String,
    pub lines: Vec<FleetLogLine>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Error)]
pub enum FleetError {
    #[error("server not found: {0}")]
    NotFound(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("operation timed out: {0}")]
    Timeout(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("provider error: {0}")]
    Provider(String),
}

#[async_trait]
pub trait FleetProvider: Send + Sync {
    async fn start_server(&self, server_id: &str) -> Result<FleetServerStatus, FleetError>;

    async fn stop_server(
        &self,
        server_id: &str,
        graceful_timeout_ms: Option<u64>,
    ) -> Result<FleetServerStatus, FleetError>;

    async fn inspect_logs(
        &self,
        server_id: &str,
        query: &AxiomLogWindowQuery,
    ) -> Result<FleetLogPage, FleetError>;
}
