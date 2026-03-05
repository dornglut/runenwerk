use crate::config::FleetKubernetesConfig;
use crate::provider::{
    FleetError, FleetLogLine, FleetLogPage, FleetProvider, FleetServerState, FleetServerStatus,
};
use async_trait::async_trait;
use grotto_online::{AxiomLogLevelFilter, AxiomLogWindowQuery};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{Container, Pod, PodSpec, PodTemplateSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use kube::api::{Api, ListParams, LogParams, Patch, PatchParams, PostParams};
use kube::{Client, ResourceExt};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, Instant, sleep};

pub struct KubernetesFleetProvider {
    client: Client,
    config: FleetKubernetesConfig,
}

impl KubernetesFleetProvider {
    pub fn new(client: Client, config: FleetKubernetesConfig) -> Self {
        Self { client, config }
    }

    pub async fn from_default(config: FleetKubernetesConfig) -> Result<Self, FleetError> {
        let client = Client::try_default()
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?;
        Ok(Self::new(client, config))
    }

    fn deployment_name(&self, server_id: &str) -> String {
        let mut normalized = String::with_capacity(server_id.len());
        for ch in server_id.chars() {
            if ch.is_ascii_alphanumeric() {
                normalized.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' {
                normalized.push('-');
            }
        }
        if normalized.is_empty() {
            normalized.push_str("unknown");
        }
        format!("{}{}", self.config.deployment_name_prefix, normalized)
    }

    fn server_labels(&self, server_id: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                self.config.server_id_label_key.clone(),
                server_id.to_string(),
            ),
            (
                "app.kubernetes.io/name".to_string(),
                "grotto-server".to_string(),
            ),
        ])
    }

    fn build_deployment(&self, server_id: &str, deployment_name: &str) -> Deployment {
        let labels = self.server_labels(server_id);
        Deployment {
            metadata: ObjectMeta {
                name: Some(deployment_name.to_string()),
                namespace: Some(self.config.namespace.clone()),
                labels: Some(labels.clone()),
                ..ObjectMeta::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..LabelSelector::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..ObjectMeta::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: self.config.container_name.clone(),
                            image: Some(self.config.image.clone()),
                            args: (!self.config.server_args.is_empty())
                                .then_some(self.config.server_args.clone()),
                            ..Container::default()
                        }],
                        ..PodSpec::default()
                    }),
                },
                ..DeploymentSpec::default()
            }),
            ..Deployment::default()
        }
    }

    async fn wait_for_available_replicas(
        &self,
        deployments: &Api<Deployment>,
        deployment_name: &str,
        expected_min_available: i32,
        timeout_seconds: u64,
    ) -> Result<(), FleetError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_seconds.max(1));
        loop {
            if Instant::now() >= deadline {
                return Err(FleetError::Timeout(format!(
                    "deployment/{deployment_name} did not become ready in {timeout_seconds}s"
                )));
            }
            let deployment = deployments
                .get(deployment_name)
                .await
                .map_err(|error| FleetError::Provider(error.to_string()))?;
            let available = deployment
                .status
                .as_ref()
                .and_then(|status| status.available_replicas)
                .unwrap_or(0);
            if available >= expected_min_available {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn wait_for_scaled_down(
        &self,
        deployments: &Api<Deployment>,
        deployment_name: &str,
        timeout_seconds: u64,
    ) -> Result<(), FleetError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_seconds.max(1));
        loop {
            if Instant::now() >= deadline {
                return Err(FleetError::Timeout(format!(
                    "deployment/{deployment_name} did not scale down in {timeout_seconds}s"
                )));
            }
            let deployment = deployments
                .get(deployment_name)
                .await
                .map_err(|error| FleetError::Provider(error.to_string()))?;
            let replicas = deployment
                .status
                .as_ref()
                .and_then(|status| status.replicas)
                .unwrap_or(0);
            let available = deployment
                .status
                .as_ref()
                .and_then(|status| status.available_replicas)
                .unwrap_or(0);
            if replicas == 0 && available == 0 {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}

#[async_trait]
impl FleetProvider for KubernetesFleetProvider {
    async fn start_server(&self, server_id: &str) -> Result<FleetServerStatus, FleetError> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &self.config.namespace);
        let deployment_name = self.deployment_name(server_id);
        let existing = deployments
            .get_opt(&deployment_name)
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?;

        if existing.is_none() {
            let manifest = self.build_deployment(server_id, &deployment_name);
            deployments
                .create(&PostParams::default(), &manifest)
                .await
                .map_err(|error| FleetError::Provider(error.to_string()))?;
        } else {
            let patch = json!({ "spec": { "replicas": 1 } });
            deployments
                .patch(
                    &deployment_name,
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
                .map_err(|error| FleetError::Provider(error.to_string()))?;
        }

        self.wait_for_available_replicas(
            &deployments,
            &deployment_name,
            1,
            self.config.startup_timeout_seconds,
        )
        .await?;

        Ok(FleetServerStatus {
            server_id: server_id.to_string(),
            state: FleetServerState::Running,
            endpoint: None,
            details: Some(format!("deployment/{deployment_name} is ready")),
        })
    }

    async fn stop_server(
        &self,
        server_id: &str,
        graceful_timeout_ms: Option<u64>,
    ) -> Result<FleetServerStatus, FleetError> {
        let deployments: Api<Deployment> =
            Api::namespaced(self.client.clone(), &self.config.namespace);
        let deployment_name = self.deployment_name(server_id);
        let Some(_) = deployments
            .get_opt(&deployment_name)
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?
        else {
            return Err(FleetError::NotFound(format!(
                "deployment/{deployment_name} not found"
            )));
        };

        let patch = json!({ "spec": { "replicas": 0 } });
        deployments
            .patch(
                &deployment_name,
                &PatchParams::default(),
                &Patch::Merge(&patch),
            )
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?;
        let timeout_seconds = graceful_timeout_ms
            .and_then(|ms| u64::try_from((ms / 1000).max(1)).ok())
            .unwrap_or(self.config.shutdown_timeout_seconds);
        self.wait_for_scaled_down(&deployments, &deployment_name, timeout_seconds)
            .await?;

        Ok(FleetServerStatus {
            server_id: server_id.to_string(),
            state: FleetServerState::Stopped,
            endpoint: None,
            details: Some(format!("deployment/{deployment_name} is scaled to zero")),
        })
    }

    async fn inspect_logs(
        &self,
        server_id: &str,
        query: &AxiomLogWindowQuery,
    ) -> Result<FleetLogPage, FleetError> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.config.namespace);
        let selector = format!("{}={server_id}", self.config.server_id_label_key);
        let listed = pods
            .list(&ListParams::default().labels(&selector))
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?;
        let Some(pod) = listed.items.into_iter().next() else {
            return Err(FleetError::NotFound(format!(
                "no pod found with selector {selector}"
            )));
        };

        let requested_limit = query.limit.unwrap_or(self.config.log_default_limit);
        let limit = requested_limit
            .max(1)
            .min(self.config.log_max_limit.max(self.config.log_default_limit));
        let since_seconds = query.from_ts_ms.and_then(|from| {
            let now_ms = unix_now_millis();
            let delta = now_ms.saturating_sub(from);
            i64::try_from((delta / 1000).max(1)).ok()
        });
        let raw_logs = pods
            .logs(
                &pod.name_any(),
                &LogParams {
                    follow: false,
                    previous: false,
                    since_seconds,
                    tail_lines: Some(i64::from(limit)),
                    timestamps: true,
                    ..LogParams::default()
                },
            )
            .await
            .map_err(|error| FleetError::Provider(error.to_string()))?;

        let cursor = query
            .cursor
            .as_deref()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let filtered = raw_logs
            .lines()
            .filter(|line| level_matches(*line, query.level.as_ref()))
            .collect::<Vec<_>>();
        let start = cursor.min(filtered.len());
        let end = (start + limit as usize).min(filtered.len());
        let lines = filtered[start..end]
            .iter()
            .map(|line| FleetLogLine {
                ts_ms: None,
                level: infer_level(line).map(ToOwned::to_owned),
                message: (*line).to_string(),
            })
            .collect::<Vec<_>>();
        let next_cursor = (end < filtered.len()).then(|| end.to_string());

        Ok(FleetLogPage {
            server_id: server_id.to_string(),
            lines,
            next_cursor,
        })
    }
}

fn level_matches(line: &str, level: Option<&AxiomLogLevelFilter>) -> bool {
    let Some(level) = level else {
        return true;
    };
    let upper = line.to_ascii_uppercase();
    match level {
        AxiomLogLevelFilter::Trace => upper.contains("TRACE"),
        AxiomLogLevelFilter::Debug => upper.contains("DEBUG"),
        AxiomLogLevelFilter::Info => upper.contains("INFO"),
        AxiomLogLevelFilter::Warn => upper.contains("WARN"),
        AxiomLogLevelFilter::Error => upper.contains("ERROR"),
    }
}

fn infer_level(line: &str) -> Option<&'static str> {
    let upper = line.to_ascii_uppercase();
    if upper.contains("TRACE") {
        Some("trace")
    } else if upper.contains("DEBUG") {
        Some("debug")
    } else if upper.contains("INFO") {
        Some("info")
    } else if upper.contains("WARN") {
        Some("warn")
    } else if upper.contains("ERROR") {
        Some("error")
    } else {
        None
    }
}

fn unix_now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}
