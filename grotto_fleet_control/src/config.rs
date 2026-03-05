use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetKubernetesConfig {
    pub namespace: String,
    pub server_id_label_key: String,
    pub deployment_name_prefix: String,
    pub image: String,
    pub container_name: String,
    pub server_args: Vec<String>,
    pub startup_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
    pub log_default_limit: u32,
    pub log_max_limit: u32,
}

impl Default for FleetKubernetesConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            server_id_label_key: "grotto.server_id".to_string(),
            deployment_name_prefix: "grotto-server-".to_string(),
            image: "ghcr.io/example/grotto-server:latest".to_string(),
            container_name: "grotto-server".to_string(),
            server_args: Vec::new(),
            startup_timeout_seconds: 90,
            shutdown_timeout_seconds: 30,
            log_default_limit: 200,
            log_max_limit: 2000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FleetAxiomBridgeConfig {
    pub enabled: bool,
    pub ws_url: String,
    pub command_token: String,
    pub service_id: String,
    pub heartbeat_seconds: u64,
    pub reconnect_backoff_ms: u64,
    pub max_buffered_events: usize,
    pub runtime_graceful_stop_enabled: bool,
    pub runtime_graceful_default_timeout_ms: u64,
    pub runtime_force_stop_timeout_ms: u64,
    #[serde(default)]
    pub allowed_server_ids: Vec<String>,
}

impl Default for FleetAxiomBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ws_url: String::new(),
            command_token: String::new(),
            service_id: "fleet-control".to_string(),
            heartbeat_seconds: 10,
            reconnect_backoff_ms: 500,
            max_buffered_events: 512,
            runtime_graceful_stop_enabled: true,
            runtime_graceful_default_timeout_ms: 8_000,
            runtime_force_stop_timeout_ms: 15_000,
            allowed_server_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FleetServiceConfig {
    #[serde(default)]
    pub kubernetes: FleetKubernetesConfig,
    #[serde(default)]
    pub axiom: FleetAxiomBridgeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum FleetConfigFile {
    Service(FleetServiceConfig),
    Kubernetes(FleetKubernetesConfig),
}

fn load_fleet_config_file(path: &Path) -> Result<FleetConfigFile> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed reading fleet kubernetes config {}", path.display()))?;
    ron::from_str(&raw)
        .with_context(|| format!("failed parsing fleet kubernetes config {}", path.display()))
}

pub fn load_fleet_kubernetes_config_from_path(path: &Path) -> Result<FleetKubernetesConfig> {
    match load_fleet_config_file(path)? {
        FleetConfigFile::Service(config) => Ok(config.kubernetes),
        FleetConfigFile::Kubernetes(config) => Ok(config),
    }
}

pub fn load_fleet_service_config_from_path(path: &Path) -> Result<FleetServiceConfig> {
    match load_fleet_config_file(path)? {
        FleetConfigFile::Service(config) => Ok(config),
        FleetConfigFile::Kubernetes(kubernetes) => Ok(FleetServiceConfig {
            kubernetes,
            axiom: FleetAxiomBridgeConfig::default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_config_parses_legacy_kubernetes_shape() {
        let legacy = r#"
            (
                namespace: "ops",
                server_id_label_key: "grotto.server_id",
                deployment_name_prefix: "grotto-",
                image: "image:v1",
                container_name: "grotto-server",
                server_args: [],
                startup_timeout_seconds: 60,
                shutdown_timeout_seconds: 20,
                log_default_limit: 200,
                log_max_limit: 1000,
            )
        "#;
        let parsed: FleetConfigFile = ron::from_str(legacy).expect("legacy shape should parse");
        let config = match parsed {
            FleetConfigFile::Kubernetes(kubernetes) => FleetServiceConfig {
                kubernetes,
                axiom: FleetAxiomBridgeConfig::default(),
            },
            FleetConfigFile::Service(service) => service,
        };
        assert_eq!(config.kubernetes.namespace, "ops");
        assert!(!config.axiom.enabled);
    }

    #[test]
    fn service_config_parses_full_wrapped_shape() {
        let wrapped = r#"
            (
                kubernetes: (
                    namespace: "ops",
                    server_id_label_key: "grotto.server_id",
                    deployment_name_prefix: "grotto-",
                    image: "image:v1",
                    container_name: "grotto-server",
                    server_args: [],
                    startup_timeout_seconds: 60,
                    shutdown_timeout_seconds: 20,
                    log_default_limit: 200,
                    log_max_limit: 1000,
                ),
                axiom: (
                    enabled: true,
                    ws_url: "wss://example.invalid/fleet",
                    command_token: "secret",
                    service_id: "fleet-a",
                    heartbeat_seconds: 5,
                    reconnect_backoff_ms: 250,
                    max_buffered_events: 42,
                    allowed_server_ids: ["srv-a", "srv-b"],
                ),
            )
        "#;
        let parsed: FleetConfigFile = ron::from_str(wrapped).expect("wrapped shape should parse");
        let config = match parsed {
            FleetConfigFile::Service(service) => service,
            FleetConfigFile::Kubernetes(_) => panic!("expected wrapped service config"),
        };
        assert!(config.axiom.enabled);
        assert_eq!(config.axiom.service_id, "fleet-a");
        assert_eq!(config.axiom.allowed_server_ids.len(), 2);
    }
}
