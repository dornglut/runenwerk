use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::features::FeatureContributionStatus;
use crate::plugins::render::{
    RenderFeatureContributionCollectorId, RenderFeatureContributionPayloadKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedFeatureContributionDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFeatureContributionDiagnostic {
    pub feature_id: RenderFeatureId,
    pub collector_id: Option<RenderFeatureContributionCollectorId>,
    pub payload_kind: Option<RenderFeatureContributionPayloadKind>,
    pub resource_type_name: Option<String>,
    pub status: FeatureContributionStatus,
    pub severity: PreparedFeatureContributionDiagnosticSeverity,
    pub message: String,
}

impl PreparedFeatureContributionDiagnostic {
    pub fn error(feature_id: RenderFeatureId, message: impl Into<String>) -> Self {
        Self {
            feature_id,
            collector_id: None,
            payload_kind: None,
            resource_type_name: None,
            status: FeatureContributionStatus::Missing,
            severity: PreparedFeatureContributionDiagnosticSeverity::Error,
            message: message.into(),
        }
    }

    pub fn with_collector_id(mut self, collector_id: RenderFeatureContributionCollectorId) -> Self {
        self.collector_id = Some(collector_id);
        self
    }

    pub fn with_payload_kind(mut self, payload_kind: RenderFeatureContributionPayloadKind) -> Self {
        self.payload_kind = Some(payload_kind);
        self
    }

    pub fn with_resource_type_name(mut self, resource_type_name: impl Into<String>) -> Self {
        self.resource_type_name = Some(resource_type_name.into());
        self
    }

    pub fn with_status(mut self, status: FeatureContributionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_severity(
        mut self,
        severity: PreparedFeatureContributionDiagnosticSeverity,
    ) -> Self {
        self.severity = severity;
        self
    }
}
