use super::RenderPassProvenanceRecord;
use crate::plugins::render::pipelines::{FlowPassKind, PipelineCacheStats};
use crate::plugins::render::shader::{
    ShaderRegistryEvent, ShaderRegistryEventKind, ShaderReloadPollReport, ShaderReloadPollStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPipelineFallbackDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineFallbackDiagnostic {
    pub severity: RenderPipelineFallbackDiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
}

impl RenderPipelineFallbackDiagnostic {
    fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderPipelineFallbackDiagnosticSeverity::Error,
            code,
            message: message.into(),
        }
    }

    fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderPipelineFallbackDiagnosticSeverity::Warning,
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderPipelineFallbackInspectionRequest<'a> {
    pub pass_provenance: &'a [RenderPassProvenanceRecord],
    pub pipeline_cache_stats: Option<PipelineCacheStats>,
    pub shader_reload_poll: Option<ShaderReloadPollReport>,
    pub shader_events: &'a [ShaderRegistryEvent],
    pub require_pipeline_cache_stats: bool,
    pub require_material_exact_shader: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderPipelineFallbackCounts {
    pub pass_count: usize,
    pub pipeline_backed_pass_count: usize,
    pub material_pass_count: usize,
    pub fallback_pass_count: usize,
    pub material_fallback_pass_count: usize,
    pub shader_failure_event_count: usize,
    pub prior_valid_shader_failure_count: usize,
    pub pipeline_cache_hit_count: u64,
    pub pipeline_cache_miss_count: u64,
    pub pipeline_cache_failure_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineFallbackPassEvidence {
    pub flow_id: String,
    pub pass_id: String,
    pub pass_kind: FlowPassKind,
    pub feature_id: Option<String>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_stats_key: String,
    pub bind_group_layout_signature_hash: u64,
    pub material_specialization_fragment_hash: u64,
    pub feature_runtime_version: u64,
    pub consumes_material_resources: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderShaderPriorValidFailureEvidence {
    pub kind: ShaderRegistryEventKind,
    pub id: String,
    pub path: String,
    pub prior_valid_revision: u64,
    pub error: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineFallbackInspection {
    pub counts: RenderPipelineFallbackCounts,
    pub shader_reload_status: Option<ShaderReloadPollStatus>,
    pub passes: Vec<RenderPipelineFallbackPassEvidence>,
    pub shader_failures: Vec<RenderShaderPriorValidFailureEvidence>,
    pub diagnostics: Vec<RenderPipelineFallbackDiagnostic>,
}

impl RenderPipelineFallbackInspection {
    pub fn is_ready(&self) -> bool {
        !self.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == RenderPipelineFallbackDiagnosticSeverity::Error
        })
    }
}

pub fn inspect_render_pipeline_fallback(
    request: RenderPipelineFallbackInspectionRequest<'_>,
) -> RenderPipelineFallbackInspection {
    let mut diagnostics = Vec::new();
    inspect_pipeline_cache_stats(request, &mut diagnostics);
    inspect_shader_reload_poll(request.shader_reload_poll, &mut diagnostics);

    if request.pass_provenance.is_empty() {
        diagnostics.push(RenderPipelineFallbackDiagnostic::error(
            "missing_pass_provenance",
            "renderer pipeline/fallback inspection requires pass provenance evidence",
        ));
    }

    let passes: Vec<_> = request.pass_provenance.iter().map(pass_evidence).collect();
    inspect_passes(request, &passes, &mut diagnostics);

    let shader_failures = shader_failure_evidence(request.shader_events);
    inspect_shader_failures(&shader_failures, &mut diagnostics);

    let stats = request.pipeline_cache_stats.unwrap_or_default();
    RenderPipelineFallbackInspection {
        counts: RenderPipelineFallbackCounts {
            pass_count: passes.len(),
            pipeline_backed_pass_count: passes
                .iter()
                .filter(|pass| is_pipeline_backed_pass_kind(pass.pass_kind))
                .count(),
            material_pass_count: passes
                .iter()
                .filter(|pass| pass.consumes_material_resources)
                .count(),
            fallback_pass_count: passes.iter().filter(|pass| pass.fallback_used).count(),
            material_fallback_pass_count: passes
                .iter()
                .filter(|pass| pass.consumes_material_resources && pass.fallback_used)
                .count(),
            shader_failure_event_count: shader_failures.len(),
            prior_valid_shader_failure_count: shader_failures
                .iter()
                .filter(|failure| failure.prior_valid_revision > 0)
                .count(),
            pipeline_cache_hit_count: stats.hits,
            pipeline_cache_miss_count: stats.misses,
            pipeline_cache_failure_count: stats.failures,
        },
        shader_reload_status: request.shader_reload_poll.map(|report| report.status),
        passes,
        shader_failures,
        diagnostics,
    }
}

fn inspect_pipeline_cache_stats(
    request: RenderPipelineFallbackInspectionRequest<'_>,
    diagnostics: &mut Vec<RenderPipelineFallbackDiagnostic>,
) {
    if !request.require_pipeline_cache_stats {
        return;
    }

    let Some(stats) = request.pipeline_cache_stats else {
        diagnostics.push(RenderPipelineFallbackDiagnostic::error(
            "missing_pipeline_cache_stats",
            "renderer pipeline/fallback inspection requires pipeline cache statistics",
        ));
        return;
    };

    if stats.hits == 0 && stats.misses == 0 && stats.failures == 0 {
        diagnostics.push(RenderPipelineFallbackDiagnostic::error(
            "empty_pipeline_cache_stats",
            "renderer pipeline/fallback inspection requires observed pipeline cache statistics",
        ));
    }
}

fn inspect_shader_reload_poll(
    report: Option<ShaderReloadPollReport>,
    diagnostics: &mut Vec<RenderPipelineFallbackDiagnostic>,
) {
    match report.map(|report| report.status) {
        Some(ShaderReloadPollStatus::Disabled) => {
            diagnostics.push(RenderPipelineFallbackDiagnostic::warning(
                "shader_reload_disabled",
                "shader reload polling is disabled; prior-valid shader evidence may be stale",
            ));
        }
        Some(ShaderReloadPollStatus::Throttled) => {
            diagnostics.push(RenderPipelineFallbackDiagnostic::warning(
                "shader_reload_throttled",
                "shader reload polling was throttled for this frame",
            ));
        }
        Some(ShaderReloadPollStatus::Polled) | None => {}
    }
}

fn inspect_passes(
    request: RenderPipelineFallbackInspectionRequest<'_>,
    passes: &[RenderPipelineFallbackPassEvidence],
    diagnostics: &mut Vec<RenderPipelineFallbackDiagnostic>,
) {
    for pass in passes {
        if is_pipeline_backed_pass_kind(pass.pass_kind) {
            if pass.pipeline_stats_key.trim().is_empty() {
                diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                    "missing_pipeline_stats_key",
                    format!(
                        "pipeline-backed pass '{}' has no pipeline statistics key",
                        pass.pass_id
                    ),
                ));
            }
            if pass.bind_group_layout_signature_hash == 0 {
                diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                    "missing_bind_group_layout_signature",
                    format!(
                        "pipeline-backed pass '{}' has no bind-group layout signature",
                        pass.pass_id
                    ),
                ));
            }
        }

        if pass.consumes_material_resources {
            if request.require_material_exact_shader && pass.fallback_used {
                diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                    "material_shader_fallback_forbidden",
                    format!(
                        "material pass '{}' requires exact generated shader evidence, but fallback '{}' was used",
                        pass.pass_id, pass.shader_id
                    ),
                ));
            }
            if pass.shader_revision == 0 {
                diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                    "missing_material_shader_revision",
                    format!(
                        "material pass '{}' has no loaded generated shader revision",
                        pass.pass_id
                    ),
                ));
            }
            if pass.material_specialization_fragment_hash == 0 {
                diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                    "missing_material_specialization_fragment",
                    format!(
                        "material pass '{}' has no material specialization fragment evidence",
                        pass.pass_id
                    ),
                ));
            }
        } else if pass.fallback_used {
            diagnostics.push(RenderPipelineFallbackDiagnostic::warning(
                "non_material_shader_fallback",
                format!(
                    "pass '{}' used renderer fallback shader '{}'",
                    pass.pass_id, pass.shader_id
                ),
            ));
        }
    }
}

fn inspect_shader_failures(
    failures: &[RenderShaderPriorValidFailureEvidence],
    diagnostics: &mut Vec<RenderPipelineFallbackDiagnostic>,
) {
    for failure in failures {
        if failure.prior_valid_revision == 0 {
            diagnostics.push(RenderPipelineFallbackDiagnostic::error(
                "shader_failure_without_prior_valid_revision",
                format!(
                    "shader '{}' failed without prior-valid revision evidence",
                    failure.id
                ),
            ));
        } else {
            diagnostics.push(RenderPipelineFallbackDiagnostic::warning(
                "shader_failure_preserved_prior_valid_revision",
                format!(
                    "shader '{}' failed while prior-valid revision {} remains observable",
                    failure.id, failure.prior_valid_revision
                ),
            ));
        }
    }
}

fn pass_evidence(record: &RenderPassProvenanceRecord) -> RenderPipelineFallbackPassEvidence {
    RenderPipelineFallbackPassEvidence {
        flow_id: record.flow_id.clone(),
        pass_id: record.pass_id.clone(),
        pass_kind: record.pass_kind,
        feature_id: record.feature_id.clone(),
        shader_id: record.shader_id.clone(),
        shader_revision: record.shader_revision,
        fallback_used: record.fallback_used,
        pipeline_stats_key: record.pipeline_stats_key.clone(),
        bind_group_layout_signature_hash: record.bind_group_layout_signature_hash,
        material_specialization_fragment_hash: record.material_specialization_fragment_hash,
        feature_runtime_version: record.feature_runtime_version,
        consumes_material_resources: record.material_binding.consumes_material_resources,
    }
}

fn shader_failure_evidence(
    events: &[ShaderRegistryEvent],
) -> Vec<RenderShaderPriorValidFailureEvidence> {
    events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                ShaderRegistryEventKind::Failed | ShaderRegistryEventKind::SkippedEmpty
            )
        })
        .map(|event| RenderShaderPriorValidFailureEvidence {
            kind: event.kind,
            id: event.id.clone(),
            path: event.path.clone(),
            prior_valid_revision: event.revision,
            error: event.error.clone(),
            details: event.details.clone(),
        })
        .collect()
}

fn is_pipeline_backed_pass_kind(kind: FlowPassKind) -> bool {
    matches!(
        kind,
        FlowPassKind::Compute | FlowPassKind::Fullscreen | FlowPassKind::Graphics
    )
}
