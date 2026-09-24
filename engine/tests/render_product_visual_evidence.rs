use engine::plugins::render::features::particle_vfx::PARTICLE_VFX_PAYLOAD_KIND;
use engine::plugins::render::features::world::WORLD_VISUAL_PAYLOAD_KIND;
use engine::plugins::render::inspect::{
    PreparedFeatureContributionInspectionEntry, RenderProductVisualEvidenceReport,
    RenderProductVisualEvidenceRequest, RenderProductVisualFamily,
    RenderProductVisualFamilyEvidence, inspect_render_product_visual_deformation_handoff,
    inspect_render_product_visual_evidence, inspect_render_product_visual_prepared_feature,
};
use engine::plugins::render::{
    DEFORMATION_RENDER_FEATURE_ID, FeatureContributionStatus, FeatureFallbackPolicy,
    PARTICLE_VFX_RENDER_FEATURE_ID, PreparedDeformationFeatureContribution,
    PreparedDeformationStream, WORLD_VISUAL_RENDER_FEATURE_ID,
};

fn prepared_entry(
    feature_id: String,
    payload_kind: &str,
    fields: &[(&str, &str)],
) -> PreparedFeatureContributionInspectionEntry {
    PreparedFeatureContributionInspectionEntry {
        feature_id,
        status: "ready".to_string(),
        fallback_policy: "skip_feature_passes".to_string(),
        payload_kind: payload_kind.to_string(),
        registered_payload_summary: Some("registered product visual payload".to_string()),
        registered_payload_fields: fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

fn particle_evidence() -> RenderProductVisualFamilyEvidence {
    let entry = prepared_entry(
        PARTICLE_VFX_RENDER_FEATURE_ID.to_string(),
        PARTICLE_VFX_PAYLOAD_KIND,
        &[
            ("batch_count", "2"),
            ("residency_request_count", "1"),
            ("temporal_input_count", "3"),
            ("fallback_batch_count", "0"),
            ("over_budget_batch_count", "0"),
            ("unsupported_batch_count", "0"),
        ],
    );
    inspect_render_product_visual_prepared_feature(&entry)
        .expect("particle/VFX registered payload should be product visual evidence")
}

fn world_visual_evidence() -> RenderProductVisualFamilyEvidence {
    let entry = prepared_entry(
        WORLD_VISUAL_RENDER_FEATURE_ID.to_string(),
        WORLD_VISUAL_PAYLOAD_KIND,
        &[
            ("batch_count", "2"),
            ("residency_request_count", "1"),
            ("temporal_input_count", "4"),
            ("fallback_batch_count", "0"),
            ("over_budget_batch_count", "0"),
            ("unsupported_batch_count", "0"),
        ],
    );
    inspect_render_product_visual_prepared_feature(&entry)
        .expect("world visual registered payload should be product visual evidence")
}

fn deformation_evidence() -> RenderProductVisualFamilyEvidence {
    inspect_render_product_visual_deformation_handoff(
        &PreparedDeformationFeatureContribution {
            streams: vec![PreparedDeformationStream {
                stream_id: "hero.animation.skinning".to_string(),
                input_pose_ref: "pose.palette.hero".to_string(),
                output_buffer_ref: "gpu.skinning.output.hero".to_string(),
            }],
        },
        FeatureContributionStatus::Ready,
        FeatureFallbackPolicy::SkipFeaturePasses,
    )
}

fn ready_request() -> RenderProductVisualEvidenceRequest {
    RenderProductVisualEvidenceRequest {
        family_evidence: vec![
            particle_evidence(),
            world_visual_evidence(),
            deformation_evidence(),
        ],
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-product-visual-evidence/README.md".to_string(),
        ],
        human_report_paths: vec![
            "docs-site/src/content/docs/reports/benchmarks/render/product-visual-evidence.md"
                .to_string(),
        ],
    }
}

fn ready_report() -> RenderProductVisualEvidenceReport {
    inspect_render_product_visual_evidence(ready_request())
}

fn has_error(report: &RenderProductVisualEvidenceReport, code: &str) -> bool {
    report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

fn error_codes(report: &RenderProductVisualEvidenceReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

#[test]
fn render_product_visual_evidence_ready_report_covers_cross_family_runtime_claim() {
    let report = ready_report();

    assert!(report.is_runtime_proven(), "{:?}", error_codes(&report));
    assert_eq!(report.counts.family_count, 3);
    assert_eq!(report.counts.ready_family_count, 3);
    assert_eq!(report.counts.particle_vfx_batch_count, 2);
    assert_eq!(report.counts.world_visual_batch_count, 2);
    assert_eq!(report.counts.deformation_stream_count, 1);
    assert_eq!(report.counts.residency_request_count, 2);
    assert_eq!(report.counts.temporal_input_count, 7);
    assert_eq!(report.counts.benchmark_command_count, 1);
    assert_eq!(report.counts.artifact_path_count, 1);
    assert_eq!(report.counts.human_report_path_count, 1);
}

#[test]
fn render_product_visual_evidence_missing_family_fails_closed() {
    let mut request = ready_request();
    request
        .family_evidence
        .retain(|family| family.family != RenderProductVisualFamily::WorldVisual);

    let report = inspect_render_product_visual_evidence(request);

    assert!(!report.is_runtime_proven());
    assert!(has_error(&report, "missing_product_visual_family"));
}

#[test]
fn render_product_visual_evidence_fallback_only_and_ownership_leaks_fail_closed() {
    let request = RenderProductVisualEvidenceRequest {
        family_evidence: vec![
            RenderProductVisualFamilyEvidence::new(
                RenderProductVisualFamily::ParticleVfx,
                PARTICLE_VFX_RENDER_FEATURE_ID.to_string(),
                PARTICLE_VFX_PAYLOAD_KIND,
                "ready",
                "reuse_last_good",
            )
            .with_prepared_item_count(2)
            .with_fallback_count(2)
            .with_consumed_renderer_handoff(true),
            world_visual_evidence().with_renderer_owned_product_truth(true),
            deformation_evidence(),
        ],
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-product-visual-evidence/README.md".to_string(),
        ],
        human_report_paths: vec![
            "docs-site/src/content/docs/reports/benchmarks/render/product-visual-evidence.md"
                .to_string(),
        ],
    };

    let report = inspect_render_product_visual_evidence(request);

    assert!(!report.is_runtime_proven());
    assert!(has_error(&report, "fallback_only_product_visual_claim"));
    assert!(has_error(&report, "renderer_owned_product_truth"));
}

#[test]
fn render_product_visual_evidence_requires_artifacts_reports_and_deformation_streams() {
    let request = RenderProductVisualEvidenceRequest {
        family_evidence: vec![
            particle_evidence(),
            world_visual_evidence(),
            inspect_render_product_visual_deformation_handoff(
                &PreparedDeformationFeatureContribution {
                    streams: vec![PreparedDeformationStream {
                        stream_id: String::new(),
                        input_pose_ref: "pose.palette.hero".to_string(),
                        output_buffer_ref: "gpu.skinning.output.hero".to_string(),
                    }],
                },
                FeatureContributionStatus::Ready,
                FeatureFallbackPolicy::SkipFeaturePasses,
            ),
        ],
        benchmark_commands: Vec::new(),
        artifact_paths: vec![" ".to_string()],
        human_report_paths: Vec::new(),
    };

    let report = inspect_render_product_visual_evidence(request);

    assert!(!report.is_runtime_proven());
    assert!(has_error(&report, "product_visual_handoff_not_consumed"));
    assert!(has_error(&report, "missing_benchmark_command"));
    assert!(has_error(&report, "blank_artifact_path"));
    assert!(has_error(&report, "missing_human_report_path"));
    assert!(
        report.family_evidence.iter().any(|family| {
            family.family == RenderProductVisualFamily::Deformation
                && family.feature_id == DEFORMATION_RENDER_FEATURE_ID.to_string()
        }),
        "deformation handoff evidence should keep the renderer feature identity"
    );
}
