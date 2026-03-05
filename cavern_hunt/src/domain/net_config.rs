use crate::domain::resources::{
    InterpolationConfig, ReplicationBudgetConfig, ReplicationCadenceConfig,
    ReplicationKeyframeConfig, ReplicationLoadShedConfig,
};
use anyhow::{Context, Result};
use engine_net::ProtocolVersion;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetProtocolConfigAssetV1 {
    pub protocol_version: u16,
    pub game_content_version: u16,
    pub schema_version: u16,
}

impl Default for NetProtocolConfigAssetV1 {
    fn default() -> Self {
        Self {
            protocol_version: 1,
            game_content_version: 1,
            schema_version: 1,
        }
    }
}

impl From<NetProtocolConfigAssetV1> for ProtocolVersion {
    fn from(value: NetProtocolConfigAssetV1) -> Self {
        ProtocolVersion::new(
            value.protocol_version.into(),
            value.game_content_version.into(),
            value.schema_version.into(),
        )
    }
}

impl From<&NetProtocolConfigAssetV1> for ProtocolVersion {
    fn from(value: &NetProtocolConfigAssetV1) -> Self {
        ProtocolVersion::new(
            value.protocol_version.into(),
            value.game_content_version.into(),
            value.schema_version.into(),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetSyncModeConfig {
    V1,
    #[default]
    V2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDiagnosticsConfigAssetV1 {
    pub enable_periodic_log: bool,
    pub log_interval_ticks: u64,
    pub hud_network_lines_enabled: bool,
}

impl Default for NetDiagnosticsConfigAssetV1 {
    fn default() -> Self {
        Self {
            enable_periodic_log: false,
            log_interval_ticks: 60,
            hud_network_lines_enabled: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetHotReloadConfigAssetV1 {
    pub enabled: bool,
    pub poll_interval_seconds: f32,
}

impl Default for NetHotReloadConfigAssetV1 {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval_seconds: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomOperatorConfigAssetV1 {
    pub enabled: bool,
    pub ws_url: String,
    pub runtime_token: Option<String>,
    pub heartbeat_seconds: u64,
    pub snapshot_interval_ticks: u64,
    pub max_buffered_events: usize,
}

impl Default for AxiomOperatorConfigAssetV1 {
    fn default() -> Self {
        Self {
            enabled: false,
            ws_url: String::new(),
            runtime_token: None,
            heartbeat_seconds: 10,
            snapshot_interval_ticks: 60,
            max_buffered_events: 256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientNetworkConfigAssetV1 {
    pub version: u32,
    pub profile_id: String,
    pub protocol: NetProtocolConfigAssetV1,
    pub server_id: String,
    pub server_name: String,
    pub server_endpoint: String,
    pub join_ticket: String,
    pub cert_path: String,
    pub cert_fingerprint_sha256: Option<String>,
    pub use_axiom_handoff: bool,
    pub axiom_api_base_url: String,
    pub axiom_lobby_id: Option<String>,
    pub axiom_device_id: String,
    pub axiom_access_token: Option<String>,
    pub axiom_refresh_token: Option<String>,
    pub shader_watch: bool,
    pub net_sync_mode: NetSyncModeConfig,
    pub interpolation: InterpolationConfig,
    pub diagnostics: NetDiagnosticsConfigAssetV1,
    pub hot_reload: NetHotReloadConfigAssetV1,
}

impl Default for ClientNetworkConfigAssetV1 {
    fn default() -> Self {
        Self {
            version: 1,
            profile_id: "two_local_balanced".to_string(),
            protocol: NetProtocolConfigAssetV1::default(),
            server_id: "srv-local".to_string(),
            server_name: "localhost".to_string(),
            server_endpoint: "127.0.0.1:7000".to_string(),
            join_ticket: "local-ticket".to_string(),
            cert_path: "var/dev/server-cert.der".to_string(),
            cert_fingerprint_sha256: None,
            use_axiom_handoff: false,
            axiom_api_base_url: "http://api.localhost".to_string(),
            axiom_lobby_id: None,
            axiom_device_id: "grotto-client-local".to_string(),
            axiom_access_token: None,
            axiom_refresh_token: None,
            shader_watch: false,
            net_sync_mode: NetSyncModeConfig::default(),
            interpolation: InterpolationConfig::default(),
            diagnostics: NetDiagnosticsConfigAssetV1::default(),
            hot_reload: NetHotReloadConfigAssetV1::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerNetworkConfigAssetV1 {
    pub version: u32,
    pub profile_id: String,
    pub protocol: NetProtocolConfigAssetV1,
    pub server_id: String,
    pub server_name: String,
    pub bind_endpoint: String,
    pub tick_rate_hz: u16,
    pub cert_output_path: String,
    pub use_axiom_verifier: bool,
    pub axiom_api_base_url: String,
    pub dedicated_server_shared_secret: Option<String>,
    pub shader_watch: bool,
    pub net_sync_mode: NetSyncModeConfig,
    pub replication_budget: ReplicationBudgetConfig,
    pub replication_cadence: ReplicationCadenceConfig,
    pub load_shed: ReplicationLoadShedConfig,
    pub keyframe: ReplicationKeyframeConfig,
    pub diagnostics: NetDiagnosticsConfigAssetV1,
    pub hot_reload: NetHotReloadConfigAssetV1,
    #[serde(default)]
    pub axiom_operator: AxiomOperatorConfigAssetV1,
}

impl Default for ServerNetworkConfigAssetV1 {
    fn default() -> Self {
        Self {
            version: 1,
            profile_id: "two_local_balanced".to_string(),
            protocol: NetProtocolConfigAssetV1::default(),
            server_id: "srv-local".to_string(),
            server_name: "localhost".to_string(),
            bind_endpoint: "127.0.0.1:7000".to_string(),
            tick_rate_hz: 60,
            cert_output_path: "var/dev/server-cert.der".to_string(),
            use_axiom_verifier: false,
            axiom_api_base_url: "http://api.localhost".to_string(),
            dedicated_server_shared_secret: None,
            shader_watch: false,
            net_sync_mode: NetSyncModeConfig::default(),
            replication_budget: ReplicationBudgetConfig::default(),
            replication_cadence: ReplicationCadenceConfig::default(),
            load_shed: ReplicationLoadShedConfig::default(),
            keyframe: ReplicationKeyframeConfig::default(),
            diagnostics: NetDiagnosticsConfigAssetV1::default(),
            hot_reload: NetHotReloadConfigAssetV1::default(),
            axiom_operator: AxiomOperatorConfigAssetV1::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetConfigHotReloadState {
    pub path: PathBuf,
    pub enabled: bool,
    pub poll_interval_seconds: f32,
    pub accumulator_seconds: f32,
    pub last_modified: Option<SystemTime>,
}

impl NetConfigHotReloadState {
    pub fn new(path: PathBuf, enabled: bool, poll_interval_seconds: f32) -> Self {
        Self {
            path,
            enabled,
            poll_interval_seconds: poll_interval_seconds.clamp(0.1, 10.0),
            accumulator_seconds: 0.0,
            last_modified: None,
        }
    }
}

pub fn load_client_network_config_from_path(path: &Path) -> Result<ClientNetworkConfigAssetV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed reading client network config {}", path.display()))?;
    let config: ClientNetworkConfigAssetV1 = ron::from_str(&raw)
        .with_context(|| format!("failed parsing client network config {}", path.display()))?;
    if config.version != 1 {
        anyhow::bail!(
            "unsupported client network config version {} in {}",
            config.version,
            path.display()
        );
    }
    Ok(config)
}

pub fn load_server_network_config_from_path(path: &Path) -> Result<ServerNetworkConfigAssetV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed reading server network config {}", path.display()))?;
    let config: ServerNetworkConfigAssetV1 = ron::from_str(&raw)
        .with_context(|| format!("failed parsing server network config {}", path.display()))?;
    if config.version != 1 {
        anyhow::bail!(
            "unsupported server network config version {} in {}",
            config.version,
            path.display()
        );
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_defaults_operator_block_when_missing_in_ron() {
        let raw = r#"
(
    version: 1,
    profile_id: "legacy",
    protocol: (
        protocol_version: 1,
        game_content_version: 1,
        schema_version: 1,
    ),
    server_id: "srv-local",
    server_name: "localhost",
    bind_endpoint: "127.0.0.1:7000",
    tick_rate_hz: 60,
    cert_output_path: "var/dev/server-cert.der",
    use_axiom_verifier: false,
    axiom_api_base_url: "http://api.localhost",
    dedicated_server_shared_secret: None,
    shader_watch: false,
    net_sync_mode: V2,
    replication_budget: (
        enemy_ops_per_patch_level0: 128,
        enemy_ops_per_patch_level1: 72,
        enemy_ops_per_patch_level2: 36,
        projectile_ops_per_patch_level0: 256,
        projectile_ops_per_patch_level1: 128,
        projectile_ops_per_patch_level2: 64,
        pickup_ops_per_patch_level0: 64,
        pickup_ops_per_patch_level1: 32,
        pickup_ops_per_patch_level2: 16,
        extraction_ops_per_patch_level0: 16,
        extraction_ops_per_patch_level1: 8,
        extraction_ops_per_patch_level2: 4,
    ),
    replication_cadence: (
        patch_emit_interval_level0: 1,
        patch_emit_interval_level1: 2,
        patch_emit_interval_level2: 3,
        enemy_patch_interval_level0: 1,
        enemy_patch_interval_level1: 2,
        enemy_patch_interval_level2: 4,
        projectile_patch_interval_level0: 1,
        projectile_patch_interval_level1: 2,
        projectile_patch_interval_level2: 3,
        pickup_patch_interval_level0: 4,
        pickup_patch_interval_level1: 6,
        pickup_patch_interval_level2: 10,
        extraction_patch_interval_level0: 1,
        extraction_patch_interval_level1: 1,
        extraction_patch_interval_level2: 2,
    ),
    load_shed: (
        bytes_threshold_level1: 60000,
        bytes_threshold_level2: 100000,
        dropped_ops_threshold_level1: 1,
        dropped_ops_threshold_level2: 24,
        connections_force_level1_at_or_above: 3,
        connections_force_level2_bytes_threshold: 45000,
    ),
    keyframe: (
        interval_ticks: 60,
        emit_on_cursor_mismatch: true,
        emit_on_reconnect: true,
    ),
    diagnostics: (
        enable_periodic_log: false,
        log_interval_ticks: 60,
        hud_network_lines_enabled: true,
    ),
    hot_reload: (
        enabled: true,
        poll_interval_seconds: 0.5,
    ),
)
"#;
        let parsed: ServerNetworkConfigAssetV1 =
            ron::from_str(raw).expect("legacy server ron should parse");
        assert!(!parsed.axiom_operator.enabled);
        assert_eq!(parsed.axiom_operator.snapshot_interval_ticks, 60);
        assert_eq!(parsed.axiom_operator.max_buffered_events, 256);
    }
}
