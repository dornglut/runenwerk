#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRayQueryCapabilityState {
    Supported,
    Unsupported,
    Disabled,
    Pending,
}

impl RenderRayQueryCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::Disabled => "disabled",
            Self::Pending => "pending",
        }
    }

    fn is_available(self) -> bool {
        self == Self::Supported
    }

    fn requires_fallback_evidence(self) -> bool {
        matches!(self, Self::Unsupported | Self::Disabled | Self::Pending)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryCapabilityProfile {
    pub backend: Option<String>,
    pub ray_query: RenderRayQueryCapabilityState,
    pub raytracing_pipeline: RenderRayQueryCapabilityState,
    pub acceleration_structure_build: RenderRayQueryCapabilityState,
    pub shader_table: RenderRayQueryCapabilityState,
    pub timestamp_query: RenderRayQueryCapabilityState,
    pub readback: RenderRayQueryCapabilityState,
    pub required_capabilities: Vec<String>,
    pub unsupported_reason: Option<String>,
    pub native_fallback_visible: bool,
}

impl RenderRayQueryCapabilityProfile {
    pub fn portable_unsupported(reason: impl Into<String>) -> Self {
        Self {
            backend: None,
            ray_query: RenderRayQueryCapabilityState::Unsupported,
            raytracing_pipeline: RenderRayQueryCapabilityState::Unsupported,
            acceleration_structure_build: RenderRayQueryCapabilityState::Unsupported,
            shader_table: RenderRayQueryCapabilityState::Unsupported,
            timestamp_query: RenderRayQueryCapabilityState::Unsupported,
            readback: RenderRayQueryCapabilityState::Unsupported,
            required_capabilities: vec![
                "ray_query".to_string(),
                "acceleration_structure".to_string(),
            ],
            unsupported_reason: Some(reason.into()),
            native_fallback_visible: true,
        }
    }

    pub fn supports_ray_query_invocation(&self) -> bool {
        self.ray_query.is_available() && self.acceleration_structure_build.is_available()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRayQueryAccelerationResourceKind {
    BottomLevel,
    TopLevel,
}

impl RenderRayQueryAccelerationResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BottomLevel => "bottom_level",
            Self::TopLevel => "top_level",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRayQueryAccelerationResourceStatus {
    Ready,
    MissingSource,
    StaleSource,
    Unsupported,
    Disabled,
    OverBudget,
}

impl RenderRayQueryAccelerationResourceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::MissingSource => "missing_source",
            Self::StaleSource => "stale_source",
            Self::Unsupported => "unsupported",
            Self::Disabled => "disabled",
            Self::OverBudget => "over_budget",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryAccelerationSourceLineage {
    pub source_kind: String,
    pub source_id: String,
    pub product_id: Option<u64>,
    pub generation: Option<u64>,
    pub cache_id: Option<String>,
}

impl RenderRayQueryAccelerationSourceLineage {
    fn is_complete(&self) -> bool {
        !self.source_kind.trim().is_empty()
            && !self.source_id.trim().is_empty()
            && self.product_id.is_some()
            && self.generation.is_some()
            && self
                .cache_id
                .as_deref()
                .is_some_and(|cache_id| !cache_id.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryAccelerationResourceEvidence {
    pub kind: RenderRayQueryAccelerationResourceKind,
    pub debug_label: String,
    pub status: RenderRayQueryAccelerationResourceStatus,
    pub source_lineage: Vec<RenderRayQueryAccelerationSourceLineage>,
    pub memory_bytes: u64,
    pub build_version: Option<u64>,
    pub invalidation_reason: Option<String>,
    pub exposes_backend_handle: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderRayQueryAccelerationResourceCounts {
    pub resource_count: usize,
    pub ready_resource_count: usize,
    pub ready_bottom_level_count: usize,
    pub ready_top_level_count: usize,
    pub stale_resource_count: usize,
    pub missing_source_count: usize,
    pub unsupported_resource_count: usize,
    pub disabled_resource_count: usize,
    pub over_budget_resource_count: usize,
    pub total_memory_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRayQueryDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryDiagnostic {
    pub severity: RenderRayQueryDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl RenderRayQueryDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderRayQueryDiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderRayQueryDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryInspectionRequest {
    pub capability_profile: RenderRayQueryCapabilityProfile,
    pub acceleration_resources: Vec<RenderRayQueryAccelerationResourceEvidence>,
    pub max_acceleration_resource_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayQueryInspection {
    pub capability_profile: RenderRayQueryCapabilityProfile,
    pub acceleration_resource_counts: RenderRayQueryAccelerationResourceCounts,
    pub acceleration_resources: Vec<RenderRayQueryAccelerationResourceEvidence>,
    pub ray_query_invocation_allowed: bool,
    pub diagnostics: Vec<RenderRayQueryDiagnostic>,
}

impl RenderRayQueryInspection {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderRayQueryDiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderRayQueryDiagnosticSeverity::Warning)
            .count()
    }

    pub fn is_ready(&self) -> bool {
        self.error_count() == 0
    }
}

pub fn inspect_render_ray_query_capability(
    request: RenderRayQueryInspectionRequest,
) -> RenderRayQueryInspection {
    let counts = count_acceleration_resources(&request.acceleration_resources);
    let mut diagnostics = Vec::new();

    validate_capability_profile(&request.capability_profile, &mut diagnostics);
    validate_acceleration_resources(&request, &counts, &mut diagnostics);

    let ray_query_invocation_allowed = request.capability_profile.supports_ray_query_invocation()
        && counts.ready_bottom_level_count > 0
        && counts.ready_top_level_count > 0
        && !has_errors(&diagnostics);

    RenderRayQueryInspection {
        capability_profile: request.capability_profile,
        acceleration_resource_counts: counts,
        acceleration_resources: request.acceleration_resources,
        ray_query_invocation_allowed,
        diagnostics,
    }
}

fn count_acceleration_resources(
    resources: &[RenderRayQueryAccelerationResourceEvidence],
) -> RenderRayQueryAccelerationResourceCounts {
    let mut counts = RenderRayQueryAccelerationResourceCounts {
        resource_count: resources.len(),
        ..RenderRayQueryAccelerationResourceCounts::default()
    };

    for resource in resources {
        counts.total_memory_bytes = counts
            .total_memory_bytes
            .saturating_add(resource.memory_bytes);
        match resource.status {
            RenderRayQueryAccelerationResourceStatus::Ready => {
                counts.ready_resource_count += 1;
                match resource.kind {
                    RenderRayQueryAccelerationResourceKind::BottomLevel => {
                        counts.ready_bottom_level_count += 1
                    }
                    RenderRayQueryAccelerationResourceKind::TopLevel => {
                        counts.ready_top_level_count += 1
                    }
                }
            }
            RenderRayQueryAccelerationResourceStatus::MissingSource => {
                counts.missing_source_count += 1;
            }
            RenderRayQueryAccelerationResourceStatus::StaleSource => {
                counts.stale_resource_count += 1;
            }
            RenderRayQueryAccelerationResourceStatus::Unsupported => {
                counts.unsupported_resource_count += 1;
            }
            RenderRayQueryAccelerationResourceStatus::Disabled => {
                counts.disabled_resource_count += 1;
            }
            RenderRayQueryAccelerationResourceStatus::OverBudget => {
                counts.over_budget_resource_count += 1;
            }
        }
    }

    counts
}

fn validate_capability_profile(
    profile: &RenderRayQueryCapabilityProfile,
    diagnostics: &mut Vec<RenderRayQueryDiagnostic>,
) {
    let required_capability_missing = profile
        .required_capabilities
        .iter()
        .any(|capability| capability.trim().is_empty());
    if profile.required_capabilities.is_empty() || required_capability_missing {
        diagnostics.push(RenderRayQueryDiagnostic::error(
            "invalid_required_capabilities",
            "ray-query capability evidence requires non-empty capability names",
        ));
    }

    let required_fallback = [
        profile.ray_query,
        profile.raytracing_pipeline,
        profile.acceleration_structure_build,
        profile.shader_table,
    ]
    .into_iter()
    .any(RenderRayQueryCapabilityState::requires_fallback_evidence);

    if required_fallback {
        if profile
            .unsupported_reason
            .as_deref()
            .is_none_or(|reason| reason.trim().is_empty())
        {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "unsupported_capability_missing_reason",
                "unsupported, disabled, or pending ray-query capabilities require a typed reason",
            ));
        }
        if !profile.native_fallback_visible {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "native_fallback_not_visible",
                "ray-query unsupported or disabled states require visible non-RT fallback evidence",
            ));
        }
    }

    if profile.ray_query == RenderRayQueryCapabilityState::Unsupported {
        diagnostics.push(RenderRayQueryDiagnostic::warning(
            "ray_query_unsupported",
            "backend reports ray-query unsupported; renderer must use explicit non-RT fallback",
        ));
    }
    if profile.acceleration_structure_build == RenderRayQueryCapabilityState::Unsupported {
        diagnostics.push(RenderRayQueryDiagnostic::warning(
            "acceleration_structure_build_unsupported",
            "backend reports acceleration-resource builds unsupported",
        ));
    }
    if profile.timestamp_query != RenderRayQueryCapabilityState::Supported {
        diagnostics.push(RenderRayQueryDiagnostic::warning(
            "timestamp_query_unavailable",
            "GPU timing for ray-query work is unavailable or degraded on this profile",
        ));
    }
}

fn validate_acceleration_resources(
    request: &RenderRayQueryInspectionRequest,
    counts: &RenderRayQueryAccelerationResourceCounts,
    diagnostics: &mut Vec<RenderRayQueryDiagnostic>,
) {
    if let Some(max_bytes) = request.max_acceleration_resource_bytes {
        if counts.total_memory_bytes > max_bytes {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "acceleration_resource_budget_exceeded",
                format!(
                    "derived acceleration resources use {} bytes over the {} byte budget",
                    counts.total_memory_bytes, max_bytes
                ),
            ));
        }
    }

    if request.capability_profile.supports_ray_query_invocation() {
        if counts.ready_bottom_level_count == 0 {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "missing_ready_bottom_level_acceleration_resource",
                "supported ray-query invocation requires at least one ready bottom-level acceleration resource",
            ));
        }
        if counts.ready_top_level_count == 0 {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "missing_ready_top_level_acceleration_resource",
                "supported ray-query invocation requires at least one ready top-level acceleration resource",
            ));
        }
    }

    for resource in &request.acceleration_resources {
        validate_acceleration_resource(resource, diagnostics);
    }
}

fn validate_acceleration_resource(
    resource: &RenderRayQueryAccelerationResourceEvidence,
    diagnostics: &mut Vec<RenderRayQueryDiagnostic>,
) {
    if resource.debug_label.trim().is_empty() {
        diagnostics.push(RenderRayQueryDiagnostic::error(
            "missing_acceleration_resource_debug_label",
            "derived acceleration resources require stable debug labels",
        ));
    }
    if resource.exposes_backend_handle {
        diagnostics.push(RenderRayQueryDiagnostic::error(
            "backend_handle_exposed",
            "ray-query inspection must not expose mutable backend handles as public authority",
        ));
    }

    match resource.status {
        RenderRayQueryAccelerationResourceStatus::Ready => {
            if resource.source_lineage.is_empty()
                || resource
                    .source_lineage
                    .iter()
                    .any(|lineage| !lineage.is_complete())
            {
                diagnostics.push(RenderRayQueryDiagnostic::error(
                    "missing_source_lineage",
                    "ready acceleration resources require complete product/source lineage",
                ));
            }
            if resource.memory_bytes == 0 {
                diagnostics.push(RenderRayQueryDiagnostic::error(
                    "missing_acceleration_resource_memory",
                    "ready acceleration resources require nonzero memory evidence",
                ));
            }
            if resource.build_version.is_none() {
                diagnostics.push(RenderRayQueryDiagnostic::error(
                    "missing_acceleration_resource_build_version",
                    "ready acceleration resources require a build version",
                ));
            }
        }
        RenderRayQueryAccelerationResourceStatus::MissingSource => {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "acceleration_resource_missing_source",
                "acceleration resource cannot be derived without source lineage",
            ));
        }
        RenderRayQueryAccelerationResourceStatus::StaleSource => {
            if resource
                .invalidation_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
            {
                diagnostics.push(RenderRayQueryDiagnostic::error(
                    "stale_acceleration_resource_missing_invalidation_reason",
                    "stale acceleration resources require an invalidation reason",
                ));
            } else {
                diagnostics.push(RenderRayQueryDiagnostic::warning(
                    "stale_acceleration_resource",
                    "acceleration resource is stale and must not be used for ray-query invocation",
                ));
            }
        }
        RenderRayQueryAccelerationResourceStatus::Unsupported => {
            diagnostics.push(RenderRayQueryDiagnostic::warning(
                "acceleration_resource_unsupported",
                "acceleration resource is unsupported on this backend",
            ));
        }
        RenderRayQueryAccelerationResourceStatus::Disabled => {
            diagnostics.push(RenderRayQueryDiagnostic::warning(
                "acceleration_resource_disabled",
                "acceleration resource is disabled by renderer policy",
            ));
        }
        RenderRayQueryAccelerationResourceStatus::OverBudget => {
            diagnostics.push(RenderRayQueryDiagnostic::error(
                "acceleration_resource_over_budget",
                "acceleration resource exceeded its memory budget",
            ));
        }
    }
}

fn has_errors(diagnostics: &[RenderRayQueryDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == RenderRayQueryDiagnosticSeverity::Error)
}
