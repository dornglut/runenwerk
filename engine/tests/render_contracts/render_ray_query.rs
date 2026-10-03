use engine::plugins::render::inspect::{
    RenderRayQueryAccelerationResourceEvidence, RenderRayQueryAccelerationResourceKind,
    RenderRayQueryAccelerationResourceStatus, RenderRayQueryAccelerationSourceLineage,
    RenderRayQueryCapabilityProfile, RenderRayQueryCapabilityState,
    RenderRayQueryDiagnosticSeverity, RenderRayQueryInspectionRequest,
    inspect_render_ray_query_capability,
};

fn supported_profile() -> RenderRayQueryCapabilityProfile {
    RenderRayQueryCapabilityProfile {
        backend: Some("test-backend".to_string()),
        ray_query: RenderRayQueryCapabilityState::Supported,
        raytracing_pipeline: RenderRayQueryCapabilityState::Supported,
        acceleration_structure_build: RenderRayQueryCapabilityState::Supported,
        shader_table: RenderRayQueryCapabilityState::Supported,
        timestamp_query: RenderRayQueryCapabilityState::Supported,
        readback: RenderRayQueryCapabilityState::Supported,
        required_capabilities: vec![
            "ray_query".to_string(),
            "acceleration_structure".to_string(),
        ],
        unsupported_reason: None,
        native_fallback_visible: false,
    }
}

fn lineage(product_id: u64) -> RenderRayQueryAccelerationSourceLineage {
    RenderRayQueryAccelerationSourceLineage {
        source_kind: "prepared_mesh".to_string(),
        source_id: format!("mesh:{product_id}"),
        product_id: Some(product_id),
        generation: Some(7),
        cache_id: Some(format!("render-gpu-cache:{product_id}")),
    }
}

fn resource(
    kind: RenderRayQueryAccelerationResourceKind,
    product_id: u64,
) -> RenderRayQueryAccelerationResourceEvidence {
    RenderRayQueryAccelerationResourceEvidence {
        kind,
        debug_label: format!("rt-accel-{product_id}"),
        status: RenderRayQueryAccelerationResourceStatus::Ready,
        source_lineage: vec![lineage(product_id)],
        memory_bytes: 4096,
        build_version: Some(1),
        invalidation_reason: None,
        exposes_backend_handle: false,
    }
}

#[test]
fn render_ray_query_reports_supported_capability_and_ready_acceleration_resources() {
    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: supported_profile(),
        acceleration_resources: vec![
            resource(RenderRayQueryAccelerationResourceKind::BottomLevel, 1),
            resource(RenderRayQueryAccelerationResourceKind::TopLevel, 2),
        ],
        max_acceleration_resource_bytes: Some(16_384),
    });

    assert!(inspection.is_ready());
    assert!(inspection.ray_query_invocation_allowed);
    assert_eq!(
        inspection.acceleration_resource_counts.ready_resource_count,
        2
    );
    assert_eq!(
        inspection
            .acceleration_resource_counts
            .ready_bottom_level_count,
        1
    );
    assert_eq!(
        inspection
            .acceleration_resource_counts
            .ready_top_level_count,
        1
    );
}

#[test]
fn render_ray_query_reports_portable_unsupported_capability_with_visible_fallback() {
    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: RenderRayQueryCapabilityProfile::portable_unsupported(
            "test backend has no ray-query feature",
        ),
        acceleration_resources: Vec::new(),
        max_acceleration_resource_bytes: Some(0),
    });

    assert!(inspection.is_ready());
    assert!(!inspection.ray_query_invocation_allowed);
    assert!(inspection.warning_count() >= 2);
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "ray_query_unsupported")
    );
}

#[test]
fn render_ray_query_fails_closed_when_unsupported_capability_hides_fallback() {
    let mut profile = RenderRayQueryCapabilityProfile::portable_unsupported("");
    profile.native_fallback_visible = false;

    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: profile,
        acceleration_resources: Vec::new(),
        max_acceleration_resource_bytes: None,
    });

    assert!(!inspection.is_ready());
    assert!(inspection.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == RenderRayQueryDiagnosticSeverity::Error
            && diagnostic.code == "unsupported_capability_missing_reason"
    }));
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "native_fallback_not_visible")
    );
}

#[test]
fn render_ray_query_fails_when_ready_resource_lacks_source_lineage() {
    let mut bottom = resource(RenderRayQueryAccelerationResourceKind::BottomLevel, 1);
    bottom.source_lineage.clear();

    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: supported_profile(),
        acceleration_resources: vec![
            bottom,
            resource(RenderRayQueryAccelerationResourceKind::TopLevel, 2),
        ],
        max_acceleration_resource_bytes: None,
    });

    assert!(!inspection.is_ready());
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing_source_lineage")
    );
}

#[test]
fn render_ray_query_rejects_public_backend_handle_exposure() {
    let mut bottom = resource(RenderRayQueryAccelerationResourceKind::BottomLevel, 1);
    bottom.exposes_backend_handle = true;

    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: supported_profile(),
        acceleration_resources: vec![
            bottom,
            resource(RenderRayQueryAccelerationResourceKind::TopLevel, 2),
        ],
        max_acceleration_resource_bytes: None,
    });

    assert!(!inspection.is_ready());
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "backend_handle_exposed")
    );
}

#[test]
fn render_ray_query_reports_stale_resource_invalidation_without_allowing_invocation() {
    let mut bottom = resource(RenderRayQueryAccelerationResourceKind::BottomLevel, 1);
    bottom.status = RenderRayQueryAccelerationResourceStatus::StaleSource;
    bottom.invalidation_reason = Some("source_generation_changed".to_string());

    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: supported_profile(),
        acceleration_resources: vec![
            bottom,
            resource(RenderRayQueryAccelerationResourceKind::TopLevel, 2),
        ],
        max_acceleration_resource_bytes: None,
    });

    assert!(!inspection.is_ready());
    assert!(!inspection.ray_query_invocation_allowed);
    assert_eq!(
        inspection.acceleration_resource_counts.stale_resource_count,
        1
    );
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "stale_acceleration_resource")
    );
    assert!(inspection.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "missing_ready_bottom_level_acceleration_resource"
    }));
}

#[test]
fn render_ray_query_fails_when_acceleration_resources_exceed_budget() {
    let inspection = inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: supported_profile(),
        acceleration_resources: vec![
            resource(RenderRayQueryAccelerationResourceKind::BottomLevel, 1),
            resource(RenderRayQueryAccelerationResourceKind::TopLevel, 2),
        ],
        max_acceleration_resource_bytes: Some(1024),
    });

    assert!(!inspection.is_ready());
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "acceleration_resource_budget_exceeded")
    );
}
