use super::PreparedFeatureContributionInspectionEntry;
use crate::plugins::render::features::particle_vfx::PARTICLE_VFX_PAYLOAD_KIND;
use crate::plugins::render::features::world::visuals::WORLD_VISUAL_PAYLOAD_KIND;
use crate::plugins::render::{
    DEFORMATION_RENDER_FEATURE_ID, FeatureContributionStatus, FeatureFallbackPolicy,
    PreparedDeformationFeatureContribution,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderProductVisualFamily {
    ParticleVfx,
    WorldVisual,
    Deformation,
}

impl RenderProductVisualFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParticleVfx => "particle_vfx",
            Self::WorldVisual => "world_visual",
            Self::Deformation => "deformation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderProductVisualEvidenceSeverity {
    Info,
    Warning,
    Error,
}

impl RenderProductVisualEvidenceSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductVisualEvidenceDiagnostic {
    pub severity: RenderProductVisualEvidenceSeverity,
    pub code: String,
    pub message: String,
}

impl RenderProductVisualEvidenceDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderProductVisualEvidenceSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderProductVisualEvidenceSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductVisualFamilyEvidence {
    pub family: RenderProductVisualFamily,
    pub feature_id: String,
    pub payload_kind: String,
    pub status: String,
    pub fallback_policy: String,
    pub prepared_item_count: usize,
    pub residency_request_count: usize,
    pub temporal_input_count: usize,
    pub fallback_count: usize,
    pub over_budget_count: usize,
    pub unsupported_count: usize,
    pub consumed_renderer_handoff: bool,
    pub renderer_owned_product_truth: bool,
}

impl RenderProductVisualFamilyEvidence {
    pub fn new(
        family: RenderProductVisualFamily,
        feature_id: impl Into<String>,
        payload_kind: impl Into<String>,
        status: impl Into<String>,
        fallback_policy: impl Into<String>,
    ) -> Self {
        Self {
            family,
            feature_id: feature_id.into(),
            payload_kind: payload_kind.into(),
            status: status.into(),
            fallback_policy: fallback_policy.into(),
            prepared_item_count: 0,
            residency_request_count: 0,
            temporal_input_count: 0,
            fallback_count: 0,
            over_budget_count: 0,
            unsupported_count: 0,
            consumed_renderer_handoff: false,
            renderer_owned_product_truth: false,
        }
    }

    pub fn with_prepared_item_count(mut self, prepared_item_count: usize) -> Self {
        self.prepared_item_count = prepared_item_count;
        self
    }

    pub fn with_residency_request_count(mut self, residency_request_count: usize) -> Self {
        self.residency_request_count = residency_request_count;
        self
    }

    pub fn with_temporal_input_count(mut self, temporal_input_count: usize) -> Self {
        self.temporal_input_count = temporal_input_count;
        self
    }

    pub fn with_fallback_count(mut self, fallback_count: usize) -> Self {
        self.fallback_count = fallback_count;
        self
    }

    pub fn with_over_budget_count(mut self, over_budget_count: usize) -> Self {
        self.over_budget_count = over_budget_count;
        self
    }

    pub fn with_unsupported_count(mut self, unsupported_count: usize) -> Self {
        self.unsupported_count = unsupported_count;
        self
    }

    pub fn with_consumed_renderer_handoff(mut self, consumed_renderer_handoff: bool) -> Self {
        self.consumed_renderer_handoff = consumed_renderer_handoff;
        self
    }

    pub fn with_renderer_owned_product_truth(mut self, renderer_owned_product_truth: bool) -> Self {
        self.renderer_owned_product_truth = renderer_owned_product_truth;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenderProductVisualEvidenceCounts {
    pub family_count: usize,
    pub ready_family_count: usize,
    pub particle_vfx_batch_count: usize,
    pub world_visual_batch_count: usize,
    pub deformation_stream_count: usize,
    pub residency_request_count: usize,
    pub temporal_input_count: usize,
    pub fallback_count: usize,
    pub over_budget_count: usize,
    pub unsupported_count: usize,
    pub benchmark_command_count: usize,
    pub artifact_path_count: usize,
    pub human_report_path_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderProductVisualEvidenceReport {
    pub counts: RenderProductVisualEvidenceCounts,
    pub family_evidence: Vec<RenderProductVisualFamilyEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub human_report_paths: Vec<String>,
    pub diagnostics: Vec<RenderProductVisualEvidenceDiagnostic>,
}

impl RenderProductVisualEvidenceReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderProductVisualEvidenceSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderProductVisualEvidenceSeverity::Warning
            })
            .count()
    }

    pub fn is_runtime_proven(&self) -> bool {
        self.error_count() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RenderProductVisualEvidenceRequest {
    pub family_evidence: Vec<RenderProductVisualFamilyEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub human_report_paths: Vec<String>,
}

pub fn inspect_render_product_visual_evidence(
    request: RenderProductVisualEvidenceRequest,
) -> RenderProductVisualEvidenceReport {
    let counts = counts_from_request(&request);
    let mut diagnostics = Vec::new();

    validate_required_families(&request.family_evidence, &mut diagnostics);
    validate_family_evidence(&request.family_evidence, &mut diagnostics);
    validate_evidence_references(&request, &mut diagnostics);

    RenderProductVisualEvidenceReport {
        counts,
        family_evidence: request.family_evidence,
        benchmark_commands: request.benchmark_commands,
        artifact_paths: request.artifact_paths,
        human_report_paths: request.human_report_paths,
        diagnostics,
    }
}

pub fn inspect_render_product_visual_prepared_feature(
    entry: &PreparedFeatureContributionInspectionEntry,
) -> Option<RenderProductVisualFamilyEvidence> {
    let family = match entry.payload_kind.as_str() {
        PARTICLE_VFX_PAYLOAD_KIND => RenderProductVisualFamily::ParticleVfx,
        WORLD_VISUAL_PAYLOAD_KIND => RenderProductVisualFamily::WorldVisual,
        _ => return None,
    };

    Some(
        RenderProductVisualFamilyEvidence::new(
            family,
            entry.feature_id.clone(),
            entry.payload_kind.clone(),
            entry.status.clone(),
            entry.fallback_policy.clone(),
        )
        .with_prepared_item_count(field_usize(entry, "batch_count"))
        .with_residency_request_count(field_usize(entry, "residency_request_count"))
        .with_temporal_input_count(field_usize(entry, "temporal_input_count"))
        .with_fallback_count(field_usize(entry, "fallback_batch_count"))
        .with_over_budget_count(field_usize(entry, "over_budget_batch_count"))
        .with_unsupported_count(field_usize(entry, "unsupported_batch_count"))
        .with_consumed_renderer_handoff(true),
    )
}

pub fn inspect_render_product_visual_deformation_handoff(
    contribution: &PreparedDeformationFeatureContribution,
    status: FeatureContributionStatus,
    fallback_policy: FeatureFallbackPolicy,
) -> RenderProductVisualFamilyEvidence {
    let consumed_renderer_handoff = !contribution.streams.is_empty()
        && contribution.streams.iter().all(|stream| {
            !stream.stream_id.trim().is_empty()
                && !stream.input_pose_ref.trim().is_empty()
                && !stream.output_buffer_ref.trim().is_empty()
        });

    RenderProductVisualFamilyEvidence::new(
        RenderProductVisualFamily::Deformation,
        DEFORMATION_RENDER_FEATURE_ID.to_string(),
        "deformation",
        feature_status_label(status),
        feature_fallback_policy_label(fallback_policy),
    )
    .with_prepared_item_count(contribution.streams.len())
    .with_consumed_renderer_handoff(consumed_renderer_handoff)
}

fn counts_from_request(
    request: &RenderProductVisualEvidenceRequest,
) -> RenderProductVisualEvidenceCounts {
    let mut counts = RenderProductVisualEvidenceCounts {
        family_count: request.family_evidence.len(),
        benchmark_command_count: request.benchmark_commands.len(),
        artifact_path_count: request.artifact_paths.len(),
        human_report_path_count: request.human_report_paths.len(),
        ..RenderProductVisualEvidenceCounts::default()
    };

    for family in &request.family_evidence {
        if family.status == "ready" {
            counts.ready_family_count += 1;
        }
        match family.family {
            RenderProductVisualFamily::ParticleVfx => {
                counts.particle_vfx_batch_count += family.prepared_item_count;
            }
            RenderProductVisualFamily::WorldVisual => {
                counts.world_visual_batch_count += family.prepared_item_count;
            }
            RenderProductVisualFamily::Deformation => {
                counts.deformation_stream_count += family.prepared_item_count;
            }
        }
        counts.residency_request_count += family.residency_request_count;
        counts.temporal_input_count += family.temporal_input_count;
        counts.fallback_count += family.fallback_count;
        counts.over_budget_count += family.over_budget_count;
        counts.unsupported_count += family.unsupported_count;
    }

    counts
}

fn validate_required_families(
    families: &[RenderProductVisualFamilyEvidence],
    diagnostics: &mut Vec<RenderProductVisualEvidenceDiagnostic>,
) {
    for required in [
        RenderProductVisualFamily::ParticleVfx,
        RenderProductVisualFamily::WorldVisual,
        RenderProductVisualFamily::Deformation,
    ] {
        if !families.iter().any(|evidence| evidence.family == required) {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "missing_product_visual_family",
                format!(
                    "product visual runtime evidence requires {} family evidence",
                    required.as_str()
                ),
            ));
        }
    }
}

fn validate_family_evidence(
    families: &[RenderProductVisualFamilyEvidence],
    diagnostics: &mut Vec<RenderProductVisualEvidenceDiagnostic>,
) {
    for family in families {
        if family.feature_id.trim().is_empty() {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "missing_product_visual_feature_id",
                format!(
                    "{} evidence is missing the renderer feature id",
                    family.family.as_str()
                ),
            ));
        }
        if family.payload_kind.trim().is_empty() {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "missing_product_visual_payload_kind",
                format!(
                    "{} evidence is missing the renderer payload kind",
                    family.family.as_str()
                ),
            ));
        }
        if family.status != "ready" {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "product_visual_family_not_ready",
                format!(
                    "{} evidence status is '{}', expected ready",
                    family.family.as_str(),
                    family.status
                ),
            ));
        }
        if family.prepared_item_count == 0 {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "missing_product_visual_prepared_items",
                format!(
                    "{} evidence has no prepared batches or streams",
                    family.family.as_str()
                ),
            ));
        }
        if !family.consumed_renderer_handoff {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "product_visual_handoff_not_consumed",
                format!(
                    "{} evidence did not consume a renderer handoff inspection",
                    family.family.as_str()
                ),
            ));
        }
        if family.renderer_owned_product_truth {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "renderer_owned_product_truth",
                format!(
                    "{} evidence attempted to make renderer inspection product truth",
                    family.family.as_str()
                ),
            ));
        }
        if family.fallback_count >= family.prepared_item_count && family.fallback_count > 0 {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "fallback_only_product_visual_claim",
                format!(
                    "{} evidence is fallback-only and cannot prove runtime visual production",
                    family.family.as_str()
                ),
            ));
        } else if family.fallback_count > 0 {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "fallback_product_visual_state_present",
                format!(
                    "{} evidence reports {} fallback prepared items",
                    family.family.as_str(),
                    family.fallback_count
                ),
            ));
        }
        if family.over_budget_count > 0 {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "over_budget_product_visual_state_present",
                format!(
                    "{} evidence reports {} over-budget prepared items",
                    family.family.as_str(),
                    family.over_budget_count
                ),
            ));
        }
        if family.unsupported_count > 0 {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                "unsupported_product_visual_state_present",
                format!(
                    "{} evidence reports {} unsupported prepared items",
                    family.family.as_str(),
                    family.unsupported_count
                ),
            ));
        }
    }
}

fn validate_evidence_references(
    request: &RenderProductVisualEvidenceRequest,
    diagnostics: &mut Vec<RenderProductVisualEvidenceDiagnostic>,
) {
    validate_non_empty_strings(
        "benchmark_command",
        "product visual runtime evidence requires at least one benchmark command",
        &request.benchmark_commands,
        diagnostics,
    );
    validate_non_empty_strings(
        "artifact_path",
        "product visual runtime evidence requires at least one raw artifact path",
        &request.artifact_paths,
        diagnostics,
    );
    validate_non_empty_strings(
        "human_report_path",
        "product visual runtime evidence requires at least one human report path",
        &request.human_report_paths,
        diagnostics,
    );
}

fn validate_non_empty_strings(
    label: &'static str,
    empty_message: &'static str,
    values: &[String],
    diagnostics: &mut Vec<RenderProductVisualEvidenceDiagnostic>,
) {
    if values.is_empty() {
        diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
            format!("missing_{label}"),
            empty_message,
        ));
    }
    for (index, value) in values.iter().enumerate() {
        if value.trim().is_empty() {
            diagnostics.push(RenderProductVisualEvidenceDiagnostic::error(
                format!("blank_{label}"),
                format!("product visual runtime evidence has a blank {label} at index {index}"),
            ));
        }
    }
}

fn field_usize(entry: &PreparedFeatureContributionInspectionEntry, key: &str) -> usize {
    entry
        .registered_payload_fields
        .iter()
        .find_map(|(field_key, value)| {
            (field_key == key)
                .then(|| value.parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
}

fn feature_status_label(status: FeatureContributionStatus) -> &'static str {
    match status {
        FeatureContributionStatus::Ready => "ready",
        FeatureContributionStatus::Stale => "stale",
        FeatureContributionStatus::Disabled => "disabled",
        FeatureContributionStatus::Missing => "missing",
    }
}

fn feature_fallback_policy_label(policy: FeatureFallbackPolicy) -> &'static str {
    match policy {
        FeatureFallbackPolicy::ReuseLastGood => "reuse_last_good",
        FeatureFallbackPolicy::EmptyContribution => "empty_contribution",
        FeatureFallbackPolicy::SkipFeaturePasses => "skip_feature_passes",
        FeatureFallbackPolicy::FailFrame => "fail_frame",
    }
}
