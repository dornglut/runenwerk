use engine::plugins::render::inspect::{
    RenderScaleVisibilityCandidate, RenderScaleVisibilityCapabilities,
    RenderScaleVisibilityCapabilityStatus, RenderScaleVisibilityConfig,
    inspect_render_scale_visibility,
};

fn candidate(
    product_id: u64,
    center: [f32; 3],
    screen_size_px: f32,
) -> RenderScaleVisibilityCandidate {
    RenderScaleVisibilityCandidate {
        product_id,
        cache_id: format!("render-gpu-cache:{product_id}"),
        center,
        radius: 0.1,
        screen_size_px,
        resident_bytes: 128,
    }
}

#[test]
fn render_scale_visibility_reports_visible_culled_lod_and_indirect_counts() {
    let config = RenderScaleVisibilityConfig {
        frustum_extent: [1.0, 1.0, 1.0],
        min_screen_size_px: 4.0,
        near_lod_screen_size_px: 96.0,
        medium_lod_screen_size_px: 24.0,
        max_visible_candidates: 8,
    };

    let inspection = inspect_render_scale_visibility(
        &[
            candidate(1, [0.0, 0.0, 0.0], 128.0),
            candidate(2, [4.0, 0.0, 0.0], 64.0),
            candidate(3, [0.0, 0.0, 0.0], 2.0),
        ],
        config,
        RenderScaleVisibilityCapabilities::supported(),
    );

    assert_eq!(inspection.resident_count, 3);
    assert_eq!(inspection.visible_count, 1);
    assert_eq!(inspection.culled_count, 2);
    assert_eq!(inspection.compacted_count, 1);
    assert_eq!(inspection.submitted_draw_count, 1);
    assert_eq!(inspection.indirect_command_count, 1);
    assert_eq!(inspection.records[0].culling_reason, "visible");
    assert_eq!(inspection.records[0].lod_band, "near");
    assert!(inspection.records[0].submitted);
    assert_eq!(inspection.records[1].culling_reason, "outside_frustum");
    assert_eq!(inspection.records[2].culling_reason, "below_screen_size");
}

#[test]
fn render_scale_visibility_fails_closed_when_indirect_submission_is_unsupported() {
    let inspection = inspect_render_scale_visibility(
        &[candidate(1, [0.0, 0.0, 0.0], 128.0)],
        RenderScaleVisibilityConfig::default(),
        RenderScaleVisibilityCapabilities {
            storage_compaction: RenderScaleVisibilityCapabilityStatus::Supported,
            indirect_submission: RenderScaleVisibilityCapabilityStatus::Unsupported,
        },
    );

    assert_eq!(inspection.visible_count, 1);
    assert_eq!(inspection.compacted_count, 1);
    assert_eq!(inspection.submitted_draw_count, 0);
    assert_eq!(inspection.indirect_command_count, 0);
    assert_eq!(inspection.indirect_submission_status, "unsupported");
    assert!(inspection.records.iter().all(|record| !record.submitted));
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "indirect_submission_unsupported")
    );
}

#[test]
fn render_scale_visibility_compaction_budget_bounds_submitted_work() {
    let config = RenderScaleVisibilityConfig {
        max_visible_candidates: 1,
        ..RenderScaleVisibilityConfig::default()
    };

    let inspection = inspect_render_scale_visibility(
        &[
            candidate(1, [0.0, 0.0, 0.0], 32.0),
            candidate(2, [0.0, 0.0, 0.0], 32.0),
        ],
        config,
        RenderScaleVisibilityCapabilities::supported(),
    );

    assert_eq!(inspection.visible_count, 2);
    assert_eq!(inspection.compacted_count, 1);
    assert_eq!(inspection.submitted_draw_count, 1);
    assert_eq!(
        inspection.records[1].culling_reason,
        "compaction_budget_exceeded"
    );
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "visible_compaction_budget_exceeded")
    );
}
