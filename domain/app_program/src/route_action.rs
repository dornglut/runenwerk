//! Route-to-action mapping and fail-closed route resolution.

use crate::action::{
    AppAction, AppActionCapability, AppActionPayload, AppActionPayloadShape, AppActionSource,
};
use crate::ids::{AppActionId, AppActionVersion, AppProgramId, AppProgramVersion};
use crate::report::{
    AppDiagnostic, NAMESPACE_HOST_COMPATIBILITY, NAMESPACE_REPORT_BUDGET,
    NAMESPACE_ROUTE_ACTION_RESOLVE, NAMESPACE_VERSION_COMPATIBILITY, RouteActionResolutionReport,
};

const PAYLOAD_SUMMARY_BUDGET: usize = 96;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteActionRequest {
    pub route_id: String,
    pub route_schema_version: u32,
    pub payload: AppActionPayload,
    pub capabilities: Vec<AppActionCapability>,
    pub source_control_id: Option<String>,
    pub source_map: Vec<String>,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl RouteActionRequest {
    pub fn new(
        route_id: impl Into<String>,
        route_schema_version: u32,
        payload: AppActionPayload,
    ) -> Self {
        Self {
            route_id: route_id.into(),
            route_schema_version,
            payload,
            capabilities: Vec::new(),
            source_control_id: None,
            source_map: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: AppActionCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_source_control(mut self, source_control_id: impl Into<String>) -> Self {
        self.source_control_id = Some(source_control_id.into());
        self
    }

    pub fn with_source_map(mut self, source_map: impl Into<String>) -> Self {
        self.source_map.push(source_map.into());
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: AppDiagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    fn has_capability(&self, capability: &AppActionCapability) -> bool {
        self.capabilities
            .iter()
            .any(|candidate| candidate == capability)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteActionMap {
    pub program_id: AppProgramId,
    pub program_version: AppProgramVersion,
    pub mappings: Vec<RouteActionMapping>,
}

impl RouteActionMap {
    pub fn new(program_id: AppProgramId, program_version: AppProgramVersion) -> Self {
        Self {
            program_id,
            program_version,
            mappings: Vec::new(),
        }
    }

    pub fn with_mapping(mut self, mapping: RouteActionMapping) -> Self {
        self.mappings.push(mapping);
        self.mappings
            .sort_by(|left, right| left.route_id.cmp(&right.route_id));
        self
    }

    pub fn resolve(&self, request: &RouteActionRequest) -> RouteActionResolution {
        let payload_summary = request.payload.summary(PAYLOAD_SUMMARY_BUDGET);
        if request.diagnostics.iter().any(AppDiagnostic::is_error) {
            return RouteActionResolution::rejected(
                request,
                RouteActionResolutionStatus::RejectedRouteDiagnostics,
                payload_summary,
                vec![AppDiagnostic::new(
                    NAMESPACE_ROUTE_ACTION_RESOLVE,
                    "app.route_action.resolve.route_diagnostics",
                    "route event contains diagnostics and is rejected",
                )],
            );
        }

        let Some(mapping) = self
            .mappings
            .iter()
            .find(|mapping| mapping.route_id == request.route_id)
        else {
            return RouteActionResolution::rejected(
                request,
                RouteActionResolutionStatus::MissingRoute,
                payload_summary,
                vec![AppDiagnostic::new(
                    NAMESPACE_ROUTE_ACTION_RESOLVE,
                    "app.route_action.resolve.unknown_route",
                    format!("route {} is not mapped to an app action", request.route_id),
                )],
            );
        };

        if mapping.route_schema_version != request.route_schema_version {
            return RouteActionResolution::rejected(
                request,
                RouteActionResolutionStatus::WrongRouteSchemaVersion,
                payload_summary,
                vec![AppDiagnostic::new(
                    NAMESPACE_VERSION_COMPATIBILITY,
                    "app.version.compatibility.route_schema",
                    format!(
                        "route {} schema version {} is not compatible with expected version {}",
                        request.route_id,
                        request.route_schema_version,
                        mapping.route_schema_version
                    ),
                )],
            );
        }

        if let Err(diagnostic) = request.payload.validate_against(&mapping.payload_shape) {
            return RouteActionResolution::rejected(
                request,
                RouteActionResolutionStatus::InvalidPayload,
                payload_summary,
                vec![diagnostic],
            );
        }

        if let Some(missing) = mapping
            .required_capabilities
            .iter()
            .find(|capability| !request.has_capability(capability))
        {
            return RouteActionResolution::rejected(
                request,
                RouteActionResolutionStatus::MissingCapability,
                payload_summary,
                vec![AppDiagnostic::new(
                    NAMESPACE_HOST_COMPATIBILITY,
                    "app.host.compatibility.missing_capability",
                    format!(
                        "route {} is missing capability {}",
                        request.route_id, missing
                    ),
                )],
            );
        }

        let mut action = AppAction::new(
            mapping.action_id.clone(),
            mapping.action_version,
            request.payload.clone(),
            AppActionSource::local_headless(
                Some(request.route_id.clone()),
                request.source_control_id.clone(),
                request.source_map.clone(),
            ),
        );
        for capability in mapping.required_capabilities.iter().cloned() {
            action = action.with_required_capability(capability);
        }

        RouteActionResolution {
            route_id: request.route_id.clone(),
            route_schema_version: request.route_schema_version,
            status: RouteActionResolutionStatus::Accepted,
            action: Some(action),
            payload_summary,
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteActionMapping {
    pub route_id: String,
    pub route_schema_version: u32,
    pub action_id: AppActionId,
    pub action_version: AppActionVersion,
    pub payload_shape: AppActionPayloadShape,
    pub required_capabilities: Vec<AppActionCapability>,
}

impl RouteActionMapping {
    pub fn new(
        route_id: impl Into<String>,
        route_schema_version: u32,
        action_id: AppActionId,
        action_version: AppActionVersion,
        payload_shape: AppActionPayloadShape,
    ) -> Self {
        Self {
            route_id: route_id.into(),
            route_schema_version,
            action_id,
            action_version,
            payload_shape,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_required_capability(mut self, capability: AppActionCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteActionResolutionStatus {
    Accepted,
    MissingRoute,
    WrongRouteSchemaVersion,
    InvalidPayload,
    MissingCapability,
    RejectedRouteDiagnostics,
}

impl RouteActionResolutionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::MissingRoute => "missing_route",
            Self::WrongRouteSchemaVersion => "wrong_route_schema_version",
            Self::InvalidPayload => "invalid_payload",
            Self::MissingCapability => "missing_capability",
            Self::RejectedRouteDiagnostics => "rejected_route_diagnostics",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteActionResolution {
    pub route_id: String,
    pub route_schema_version: u32,
    pub status: RouteActionResolutionStatus,
    pub action: Option<AppAction>,
    pub payload_summary: crate::action::AppActionPayloadSummary,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl RouteActionResolution {
    fn rejected(
        request: &RouteActionRequest,
        status: RouteActionResolutionStatus,
        payload_summary: crate::action::AppActionPayloadSummary,
        diagnostics: Vec<AppDiagnostic>,
    ) -> Self {
        Self {
            route_id: request.route_id.clone(),
            route_schema_version: request.route_schema_version,
            status,
            action: None,
            payload_summary,
            diagnostics,
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.status == RouteActionResolutionStatus::Accepted
    }

    pub fn report(&self) -> RouteActionResolutionReport {
        let mut diagnostics = self.diagnostics.clone();
        if self.payload_summary.truncated {
            diagnostics.push(AppDiagnostic::warning(
                NAMESPACE_REPORT_BUDGET,
                "app.report.budget.payload_summary_truncated",
                "payload summary was truncated to the configured report budget",
            ));
        }
        RouteActionResolutionReport {
            route_id: self.route_id.clone(),
            route_schema_version: self.route_schema_version,
            status: self.status.as_str().to_owned(),
            action_id: self
                .action
                .as_ref()
                .map(|action| action.action_id.as_str().to_owned()),
            action_version: self
                .action
                .as_ref()
                .map(|action| action.action_version.value()),
            payload_summary: self.payload_summary.text.clone(),
            payload_summary_truncated: self.payload_summary.truncated,
            diagnostics,
        }
    }
}
