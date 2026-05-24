use engine::plugins::render::features::particle_vfx::PARTICLE_VFX_PAYLOAD_KIND;
use engine::plugins::render::features::world::WORLD_VISUAL_PAYLOAD_KIND;
use engine::plugins::render::inspect::{
    PreparedFeatureContributionInspectionEntry, RenderProductVisualEvidenceReport,
    RenderProductVisualEvidenceRequest, inspect_render_product_visual_deformation_handoff,
    inspect_render_product_visual_evidence, inspect_render_product_visual_prepared_feature,
};
use engine::plugins::render::{
    FeatureContributionStatus, FeatureFallbackPolicy, PARTICLE_VFX_RENDER_FEATURE_ID,
    PreparedDeformationFeatureContribution, PreparedDeformationStream,
    WORLD_VISUAL_RENDER_FEATURE_ID,
};

fn main() {
    let report = build_report();
    println!(
        "render product visual evidence runtime_proven={} errors={} warnings={} families={}",
        report.is_runtime_proven(),
        report.error_count(),
        report.warning_count(),
        report.counts.family_count
    );
    println!(
        "particle_batches={} world_batches={} deformation_streams={} residency_requests={} temporal_inputs={}",
        report.counts.particle_vfx_batch_count,
        report.counts.world_visual_batch_count,
        report.counts.deformation_stream_count,
        report.counts.residency_request_count,
        report.counts.temporal_input_count
    );
}

fn build_report() -> RenderProductVisualEvidenceReport {
    inspect_render_product_visual_evidence(RenderProductVisualEvidenceRequest {
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
    })
}

fn particle_evidence() -> engine::plugins::render::inspect::RenderProductVisualFamilyEvidence {
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
        .expect("particle/VFX payload should inspect")
}

fn world_visual_evidence() -> engine::plugins::render::inspect::RenderProductVisualFamilyEvidence {
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
        .expect("world visual payload should inspect")
}

fn deformation_evidence() -> engine::plugins::render::inspect::RenderProductVisualFamilyEvidence {
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
        registered_payload_summary: Some("product visual prepared payload".to_string()),
        registered_payload_fields: fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_product_visual_evidence_report() {
        let report = build_report();

        assert!(report.is_runtime_proven());
        assert_eq!(report.counts.family_count, 3);
        assert_eq!(report.counts.deformation_stream_count, 1);
    }
}
