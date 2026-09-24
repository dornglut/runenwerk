use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, PreparedRenderFrameInspection, RenderCapturePointIdentity,
};
use engine::plugins::render::inspect::{
    RenderReadinessBudgetKind, RenderReadinessBudgetMeasurements, RenderReadinessBudgetThreshold,
    RenderReadinessReportRequest, RenderReplayArtifactReference, RenderReplayManifest,
    evaluate_render_readiness_budgets, inspect_render_readiness, validate_render_replay_manifest,
};
use std::path::PathBuf;

fn build_readiness_report() -> engine::plugins::render::inspect::RenderReadinessReport {
    let prepared = PreparedRenderFrameInspection {
        frame_index: 1,
        prepare_epoch: 1,
        render_surface_id: 1,
        native_window_id: Some(1),
        surface_size: (1280, 720),
        views: Vec::new(),
        flow_invocations: Vec::new(),
        feature_contributions: Vec::new(),
        dynamic_texture_targets: Vec::new(),
        product_selections: Vec::new(),
    };
    let capture_point = RenderCapturePointIdentity {
        flow_id: "example.readiness".to_string(),
        pass_id: "compose".to_string(),
        stage: CaptureStage::After,
        resource_id: "surface.color".to_string(),
        texture_class: CaptureTextureClass::ImportedTexture,
    };
    let replay = RenderReplayManifest::new("example.readiness.replay", 1)
        .with_capability_profile("wgpu-portable-v1")
        .with_prepared_frame_digest("prepared-frame:example")
        .with_artifact(RenderReplayArtifactReference {
            capture_point,
            artifact_path: Some(PathBuf::from("target/render-debug/example.png")),
            format: Some("Rgba8Unorm".to_string()),
        });
    let budget_report = evaluate_render_readiness_budgets(
        &RenderReadinessBudgetMeasurements {
            frame_total_ms: Some(4.0),
            ..RenderReadinessBudgetMeasurements::default()
        },
        &[RenderReadinessBudgetThreshold::max(
            RenderReadinessBudgetKind::FrameTotalMillis,
            16.0,
        )],
    );

    inspect_render_readiness(RenderReadinessReportRequest {
        prepared_frame: Some(prepared),
        budget_report,
        replay_validation: Some(validate_render_replay_manifest(&replay)),
        ..RenderReadinessReportRequest::default()
    })
}

fn main() {
    let report = build_readiness_report();
    println!(
        "render readiness frame={:?} errors={} warnings={}",
        report.frame_index,
        report.error_count(),
        report.warning_count()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_renderer_report_from_public_inspection_dtos() {
        let report = build_readiness_report();

        assert!(report.is_ready());
        assert_eq!(report.frame_index, Some(1));
        assert_eq!(report.source_reports.budget_result_count, 1);
    }
}
