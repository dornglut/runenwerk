use std::fs;
use std::path::{Path, PathBuf};

#[path = "runengpu_g5a_execution_authority/renderer_ui_boundary.rs"]
mod renderer_ui_boundary;

fn engine_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn source(path: &str) -> String {
    fs::read_to_string(engine_path(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn transitional_g5_bridge_consumes_prepared_work_without_compiled_execution_fallback() {
    let execute = source("src/plugins/render/renderer/render_flow/execute.rs");
    let canonical_start = execute
        .find("if let Some(legacy_work) = legacy_work.as_deref() {")
        .expect(
            "transitional prepared-work execution branch must remain explicit until G5C removes it",
        );
    let residual_offset = execute[canonical_start..]
        .find("let invocation_result = (|| -> Result<()> {")
        .expect(
            "residual renderer branch must remain distinguishable from prepared generic execution",
        );
    let canonical = &execute[canonical_start..canonical_start + residual_offset];

    for required in [
        "schedule_legacy_invocation_work(legacy_work)?",
        "encode_canonical_upload_operation(",
        "encode_canonical_compute_operation(",
        "encode_canonical_copy_operation(",
        "encode_canonical_render_operation(",
        "frame.encode_resolve(context, encoder, operation)?",
        "RenderGpuWorkPayload::TimingReadback { occurrence }",
        "GpuWorkOperation::Readback(operation)",
        "frame.encode_legacy_readback(context, encoder, operation)?",
    ] {
        assert!(
            canonical.contains(required),
            "transitional G5 renderer execution must consume prepared generic work through {required}"
        );
    }
    assert!(
        !canonical.contains("encode_compiled_pass("),
        "execution-complete prepared work must not fall back to the compiled renderer execution recipe"
    );

    let schedule_start = execute
        .find("fn schedule_legacy_invocation_work(")
        .expect("transitional G3 schedule adapter must exist until the frame-level G5C cutover");
    let schedule_end = execute[schedule_start..]
        .find("fn encode_prepared_timing_tail(")
        .map(|offset| schedule_start + offset)
        .expect("transitional schedule helper must end before the residual timing-tail helper");
    assert!(
        execute[schedule_start..schedule_end].contains(".ordered_payloads()?"),
        "prepared logical work order must come from PreparedRenderWorkPlan rather than renderer pass order"
    );
}

#[test]
fn g4_compiled_pass_realization_does_not_require_physical_surface_objects() {
    let pipeline = source("src/plugins/render/renderer/render_flow/execute_passes/pipeline.rs");
    let realization_start = pipeline
        .find("fn realize_compiled_pass(")
        .expect("G4 compiled-pass realization boundary must remain explicit");
    let encoding_start = pipeline[realization_start..]
        .find("fn encode_compiled_pass(")
        .map(|offset| realization_start + offset)
        .expect(
            "legacy physical encoding boundary must remain distinguishable from G4 realization",
        );
    let realization = &pipeline[realization_start..encoding_start];

    for forbidden in [
        "frame_texture: &Texture",
        "frame_view: &TextureView",
        "resolve_color_target_from_plan(",
        "resolve_depth_target_from_plan(",
    ] {
        assert!(
            !realization.contains(forbidden),
            "G4 compiled-pass realization must not recover physical surface authority through {forbidden}"
        );
    }
    assert!(realization.contains("resolve_color_target_format_from_plan("));
    assert!(realization.contains("resolve_depth_target_format_from_plan("));

    let bindings = source("src/plugins/render/renderer/render_flow/bindings.rs");
    let binding_start = bindings
        .find("fn resolve_compiled_bind_group(")
        .expect("G4C2 bind-group realization boundary must remain explicit");
    let binding_end = bindings[binding_start..]
        .find("fn resolved_binding_texture_view(")
        .map(|offset| binding_start + offset)
        .expect("G4C2 binding helper must follow the realization boundary");
    let binding_realization = &bindings[binding_start..binding_end];
    assert!(!binding_realization.contains("frame_texture: &Texture"));
    assert!(!binding_realization.contains("resolve_texture("));
    assert!(binding_realization.contains("resolve_logical_texture_binding("));
}

#[test]
fn canonical_timing_tail_owns_one_readback_identity_without_a_logical_staging_copy() {
    let operations = source("src/plugins/render/renderer/render_flow/logical_operations.rs");
    let timing = source("src/plugins/render/renderer/render_flow/logical_timing.rs");
    let canonical = source("src/plugins/render/renderer/render_flow/canonical_work.rs");
    let execute = source("src/plugins/render/renderer/render_flow/execute.rs");
    let adapter = source("src/plugins/render/adapters/gpu_work.rs");

    assert!(operations.contains("readback: GpuReadbackOperation"));
    assert!(operations.contains("readback_id: GpuReadbackId"));
    assert!(operations.contains("GpuReadbackOperation::new("));
    assert!(!operations.contains("GpuReadbackId::allocate()?"));
    assert!(!operations.contains("readback_copy: GpuCopyOperation"));

    assert!(timing.contains("readback_id: GpuReadbackId"));
    assert!(timing.contains("let readback_id = GpuReadbackId::allocate()?;"));
    assert!(timing.contains("pub(super) const fn readback_id(&self) -> GpuReadbackId"));
    assert!(!timing.contains("readback_buffer: GpuBufferHandle"));

    assert!(canonical.contains("timing.readback_id(),"));
    assert!(execute.contains("timing.readback_id(),"));
    assert!(adapter.contains("TimingReadback"));
    assert!(adapter.contains("Some(GpuWorkNodeKind::Readback)"));
    assert!(!adapter.contains("TimingReadbackCopy"));
}

#[test]
fn canonical_physical_adapters_do_not_reconstruct_compiled_gpu_semantics() {
    for (path, operation) in [
        (
            "src/plugins/render/renderer/render_flow/canonical_compute.rs",
            "GpuComputeOperation",
        ),
        (
            "src/plugins/render/renderer/render_flow/canonical_copy.rs",
            "GpuCopyOperation",
        ),
        (
            "src/plugins/render/renderer/render_flow/canonical_render.rs",
            "GpuRenderOperation",
        ),
        (
            "src/plugins/render/renderer/render_flow/canonical_upload.rs",
            "GpuUploadOperation",
        ),
    ] {
        let adapter = source(path);
        assert!(
            adapter.contains(operation),
            "{path} must consume the canonical {operation} contract"
        );
        for forbidden in [
            "CompiledPassExecutionPlan",
            "CompiledComputeExecutionPlan",
            "CompiledCopyExecutionPlan",
            "CompiledRasterExecutionPlan",
            "resolve_texture_from_label",
            "resolve_buffer_ref",
            "context.realize_buffer(",
            "context.realize_texture(",
            "context.realize_texture_view(",
        ] {
            assert!(
                !adapter.contains(forbidden),
                "{path} must not recreate renderer-owned or lazy G4 execution authority via {forbidden}"
            );
        }
    }
}
