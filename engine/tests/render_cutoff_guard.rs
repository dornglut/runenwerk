use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_render_flow_sources() -> String {
    let mut files = Vec::new();
    collect_source_files(
        Path::new("src/plugins/render/renderer/render_flow"),
        &mut files,
    );
    files.sort();

    let mut combined = String::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        combined.push_str(&strip_cfg_test_modules(&source));
        combined.push('\n');
    }
    combined
}

fn collect_source_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn strip_cfg_test_modules(source: &str) -> String {
    let mut stripped = String::new();
    let mut cursor = 0;
    while let Some(marker_offset) = source[cursor..].find("#[cfg(test)]") {
        let marker_start = cursor + marker_offset;
        stripped.push_str(&source[cursor..marker_start]);
        let after_marker = marker_start + "#[cfg(test)]".len();
        let after_marker_source = &source[after_marker..];
        let Some(mod_offset) = after_marker_source.find("mod tests") else {
            stripped.push_str("#[cfg(test)]");
            cursor = after_marker;
            continue;
        };
        if !after_marker_source[..mod_offset].trim().is_empty() {
            stripped.push_str("#[cfg(test)]");
            cursor = after_marker;
            continue;
        }
        let mod_start = after_marker + mod_offset;
        let Some(open_offset) = source[mod_start..].find('{') else {
            cursor = mod_start;
            continue;
        };
        let open_index = mod_start + open_offset;
        let mut depth = 0usize;
        let mut module_end = None;
        for (offset, ch) in source[open_index..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        module_end = Some(open_index + offset + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        cursor = module_end.unwrap_or(source.len());
    }
    stripped.push_str(&source[cursor..]);
    stripped
}

fn read_render_production_sources() -> Vec<(String, String)> {
    let mut files = Vec::new();
    collect_source_files(Path::new("src/plugins/render"), &mut files);
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let display_path = path.display().to_string();
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {display_path}: {err}"));
            (display_path, strip_cfg_test_modules(&source))
        })
        .collect()
}

fn function_body(source: &str, signature: &str) -> String {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature '{signature}'"));
    let tail = &source[start..];
    let end = tail.find("\nfn ").unwrap_or(tail.len());
    tail[..end].to_string()
}

#[test]
fn rb0_render_sources_do_not_encode_editor_viewport_product_workarounds() {
    let forbidden_terms = [
        "editor.viewport.v1.scene_color",
        "editor.viewport.v1.picking_ids",
        "editor.viewport.v1.overlay",
        "PRODUCT_ID_SCENE_COLOR",
        "PRODUCT_ID_PICKING_IDS",
        "PRODUCT_ID_OVERLAY",
        "VIEWPORT_RESOURCE_SCENE_COLOR",
        "VIEWPORT_RESOURCE_PICKING_IDS",
        "VIEWPORT_RESOURCE_OVERLAY",
        "RenderFlow::new(format!",
        "RenderFlow::new(&format!",
        "format!(\"runenwerk.editor.viewport",
    ];
    let offenders = read_render_production_sources()
        .into_iter()
        .flat_map(|(file, source)| {
            forbidden_terms
                .iter()
                .filter(move |term| source.contains(**term))
                .map(move |term| format!("{file}: {term}"))
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "engine render production sources must not grow editor viewport static-product or flow-per-viewport workarounds: {offenders:?}",
    );
}

#[test]
fn hard_cutoff_removes_legacy_render_symbols_and_fallbacks() {
    let forbidden_symbols = [
        "RenderFrameResourceBindings",
        "storage_state(",
        "packet.surface_size.0 + workgroup[0] - 1",
        "packet.surface_size.1 + workgroup[1] - 1",
    ];

    let files = [
        "src/plugins/render/mod.rs",
        "src/plugins/render/plugin.rs",
        "src/plugins/render/api/passes.rs",
        "src/plugins/render/composition/integration.rs",
        "src/plugins/render/runtime/frame_submit.rs",
    ];

    for file in files {
        let source = read(file);
        for symbol in forbidden_symbols {
            assert!(
                !source.contains(symbol),
                "legacy cutoff guard hit in '{file}': found forbidden symbol '{symbol}'"
            );
        }
    }

    let render_flow = read_render_flow_sources();
    assert!(
        render_flow.contains("missing prepared dispatch for pass"),
        "compute runtime path should consume prepare-projected dispatch values"
    );
    assert!(
        render_flow.contains("context.prepare_submission(graph)")
            && render_flow.contains(".submit_prepared(prepared)"),
        "normal renderer frames must prepare and irreversibly submit one canonical RunenGPU graph"
    );
    assert!(
        !render_flow.contains("for pass in &flow.pass_order"),
        "renderer runtime should not iterate raw pass_order for encoding"
    );
    assert!(
        render_flow.contains("context.realize_compute_pipeline(")
            && render_flow.contains("context.realize_render_pipeline(")
            && render_flow.contains("GpuWorkOperation::Compute")
            && render_flow.contains("GpuWorkOperation::Render"),
        "renderer runtime must realize pipelines through G4C3 and carry them only in canonical GPU operations"
    );
    assert!(
        !render_flow.contains("flow_pipeline_cache.render_pipeline")
            && !render_flow.contains("insert_render_pipeline"),
        "renderer runtime must not restore renderer-owned reusable compute/render pipeline cache authority"
    );
    assert!(
        render_flow.contains("execution_pass_feature_id(pass)"),
        "renderer runtime must resolve feature-gated dispatch generically for every execution-plan pass kind"
    );
    assert!(
        !render_flow.contains("feature_identity_for_pass_kind"),
        "runtime must not hardcode UI-only feature identity inference in active dispatch paths"
    );
    assert!(
        !render_flow.contains("active runtime execution is single-view only"),
        "renderer runtime must not preserve the old single-view fail-fast path after prepared views/invocations are active"
    );

    let submit = read("src/plugins/render/runtime/frame_submit.rs");
    let submit_fn = function_body(&submit, "pub(crate) fn frame_render_submit_system(");
    assert!(
        !submit_fn.contains("collect_flow_declared_frame_resources"),
        "submit system must not perform live ECS extraction"
    );
    assert!(
        !submit_fn.contains("project_uniform_bindings_for_pass"),
        "submit system must not perform uniform projection"
    );
    assert!(
        !submit_fn.contains("RenderFrameDataRegistry"),
        "submit system must not use RenderFrameDataRegistry on active runtime path"
    );
    assert!(
        !submit_fn.contains("ViewportSurfaceBindingRegistryResource"),
        "submit system must not extract viewport surface bindings from ECS at submit time"
    );
    assert!(
        !submit_fn.contains(".registry().clone()"),
        "submit system must not clone live viewport binding registries at submit time"
    );
    assert!(
        !submit_fn.contains("poll_updates"),
        "submit system must not poll shader hot reloads"
    );

    let pipeline_cache = read("src/plugins/render/pipelines/cache.rs");
    for symbol in [
        "PipelineKey",
        "record_hit(",
        "record_miss(",
        "revision_for(",
    ] {
        assert!(
            !pipeline_cache.contains(symbol),
            "pipeline cache ECS resource must remain canonical stats-only sink; found legacy symbol '{symbol}'"
        );
    }
}

#[test]
fn g5c1_render_cutover_has_one_frame_graph_and_no_raw_executor_sidecar() {
    let production = read_render_production_sources();
    let retired = [
        "CompiledResourceAccessKind",
        "CompiledResourceLifetimeWindow",
        "compile_resource_lifetime_windows",
        "diagnose_resource_lifetime_windows",
        "GpuPrimitiveResourceAccessKind",
        "GpuPrimitiveResourceAccess",
        "GpuPrimitiveDispatchResource",
        "PassDependencyCycleDetected",
        "UnknownPassDependency",
    ];
    let offenders = production
        .iter()
        .flat_map(|(file, source)| {
            retired
                .iter()
                .filter(move |retired| source.contains(**retired))
                .map(move |retired| format!("{file}: {retired}"))
        })
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "retired renderer correctness authority remains: {offenders:?}"
    );
    assert!(
        !Path::new("src/plugins/render/graph/resource_lifetimes.rs").exists(),
        "the retired renderer lifetime-analysis module must remain deleted"
    );

    let pass_graph = read("src/plugins/render/graph/pass_graph.rs");
    for retired_field in ["pub reads:", "pub writes:", "pub depends_on:"] {
        assert!(
            !pass_graph.contains(retired_field),
            "RenderPassNode must not restore generic correctness field '{retired_field}'"
        );
    }
    assert!(pass_graph.contains("pub non_data_order_after: Vec<RenderPassId>"));

    let adapter = read("src/plugins/render/adapters/gpu_work.rs");
    assert!(adapter.contains("pub(crate) fn prepare_render_gpu_frame_work("));
    for retired_sidecar in [
        "RenderGpuWorkPayload",
        "RenderGpuWorkSidecar",
        "PreparedRenderWorkPlan",
        "prepare_render_gpu_work(",
        "ordered_payloads(",
    ] {
        assert!(
            !adapter.contains(retired_sidecar),
            "normal-frame cutover must delete dead raw-executor sidecar authority '{retired_sidecar}'"
        );
    }
    assert!(
        adapter.contains("GpuInitialCoverage::descriptor_initialization"),
        "frame graph preparation must preserve descriptor-owned initialization evidence"
    );

    let execute = read("src/plugins/render/renderer/render_flow/execute.rs");
    for raw_executor in [
        "current_render_device_queue",
        "current_render_execution_bridge",
        "CommandEncoder",
        "SurfaceTexture",
        "queue.submit",
        "execute_realized_batch",
        "schedule_legacy_invocation_work",
    ] {
        assert!(
            !execute.contains(raw_executor),
            "normal renderer frame must not retain raw executor authority '{raw_executor}'"
        );
    }

    let invocation_start = execute
        .find("struct RealizedFlowInvocation<'a> {")
        .expect("renderer should retain one realized invocation handoff");
    let invocation_tail = &execute[invocation_start..];
    let invocation_end = invocation_tail
        .find("\n}\n\nstruct RealizedScheduledPass")
        .expect("realized invocation declaration should precede scheduled-pass state");
    let invocation = &invocation_tail[..invocation_end];
    assert!(
        invocation.contains("canonical_resolution: Option<CanonicalInvocationResolution>"),
        "realized invocation must retain owned canonical resolution as its semantic authority"
    );
    assert!(
        !invocation.contains("canonical_work") && !invocation.contains("PreparedRenderWorkPlan"),
        "realized invocation must not retain a prepared per-invocation G3 graph alongside semantic resolution"
    );

    let render_packet = function_body(&execute, "    pub(crate) fn render_packet(");
    let drain = render_packet
        .find("let canonical_resolutions = batch")
        .expect("render packet should drain owned invocation resolutions at frame scope");
    let graph_prepare = render_packet
        .find("prepare_render_gpu_frame_work(")
        .expect("render packet should prepare one complete frame graph");
    let submission_prepare = render_packet
        .find("context.prepare_submission(graph)")
        .expect("renderer should hand the frame graph to RunenGPU preparation");
    let submission_accept = render_packet
        .find(".submit_prepared(prepared)")
        .expect("renderer should irreversibly accept the prepared frame once");
    let dynamic_acceptance_bookkeeping = render_packet
        .find("record_accepted_uploads(&accepted_dynamic_uploads)")
        .expect("dynamic upload generations should be recorded after G5 acceptance");
    assert!(
        drain < graph_prepare
            && graph_prepare < submission_prepare
            && submission_prepare < submission_accept
            && submission_accept < dynamic_acceptance_bookkeeping,
        "owned invocation resolutions must aggregate before the sole RunenGPU prepare/accept boundary, and renderer generation evidence may advance only afterward"
    );
    assert_eq!(
        render_packet
            .matches("ResolvedRenderGpuWorkNode::present(")
            .count(),
        1,
        "normal frame must append exactly one terminal canonical Present"
    );
    assert!(render_packet.contains("pending_operations).into_operations()"));
    assert!(render_packet.contains("validate_prepared_uploads("));

    let backend = read("src/plugins/render/backend/wgpu_ctx.rs");
    for raw_surface_authority in [
        "wgpu::Surface",
        "SurfaceTexture",
        "get_current_texture",
        "fn present(",
        "current_host_surface_bridge",
    ] {
        assert!(
            !backend.contains(raw_surface_authority),
            "renderer backend must not retain raw surface authority '{raw_surface_authority}'"
        );
    }
    assert!(backend.contains(".acquire_surface_image(surface)"));
    let renderer = read("src/plugins/render/renderer/mod.rs");
    assert!(renderer.contains("self.ctx.acquire_surface_image(render_surface_id)?"));
    assert!(renderer.contains("let acquired_extent = acquired.texture().descriptor().extent()"));
    assert!(renderer.contains("acquired.default_view(),"));
    assert!(renderer.contains("(acquired_extent.width(), acquired_extent.height()),"));

    let ui_lowering = read("src/plugins/render/renderer/setup.rs");
    assert!(ui_lowering.contains("acquired_surface_extent.0 as f32"));
    assert!(ui_lowering.contains("acquired_surface_extent.1 as f32"));
    assert!(ui_lowering.contains("if instance_count == 0 {"));
    assert!(ui_lowering.contains("GpuDrawRange::new(0, 6)?"));
    assert!(ui_lowering.contains("GpuDrawRange::new(0, instance_count)?"));

    let present_projection = read("src/plugins/render/renderer/render_flow/logical_copy.rs");
    assert!(present_projection.contains(
        "if source_key == RuntimeResourceKey::SurfaceColor {\n        return Ok(ProjectedCopyOperation::NoWork);"
    ));
    assert!(present_projection.contains("GpuWorkOperation::Copy(operation)"));
}

#[test]
fn rb1_rb4_submit_consumes_prepared_view_data_without_single_view_fallbacks() {
    let execute = read("src/plugins/render/renderer/render_flow/execute.rs");
    for forbidden in [
        "packet.view_count > 1",
        "single-view only",
        "multi-view execution is explicitly deferred",
    ] {
        assert!(
            !execute.contains(forbidden),
            "render execution must not reject prepared multi-view packets through legacy fail-fast marker '{forbidden}'",
        );
    }

    let submit = read("src/plugins/render/runtime/frame_submit.rs");
    let submit_fn = function_body(&submit, "pub(crate) fn frame_render_submit_system(");
    for forbidden in [
        "ViewportSurfaceBindingRegistryResource",
        "world.resource::<ViewportSurfaceBindingRegistryResource>",
        "world.get_resource::<ViewportSurfaceBindingRegistryResource>",
        "Res<ViewportSurfaceBindingRegistryResource>",
        "prepared_frame.main_view()",
        "unwrap_or(prepared_frame.surface.target_size_px)",
    ] {
        assert!(
            !submit_fn.contains(forbidden),
            "render submit must consume prepared view/product data instead of submit-time fallback/extraction marker '{forbidden}'",
        );
    }
}

#[test]
fn render_flow_submit_runs_compiler_preflight_without_live_world_extraction() {
    let execute = read("src/plugins/render/renderer/render_flow/execute.rs");
    let render_packet = function_body(&execute, "    pub(crate) fn render_packet(");
    assert!(
        render_packet.contains("preflight_prepared_frame"),
        "render_packet must run typed prepared-frame graph preflight through the renderer-owned cache before backend encoding"
    );
    assert!(
        render_packet.contains("compiled_flows"),
        "preflight must consume compiled flows, not raw flow graph extraction"
    );
    assert!(
        !render_packet.contains("preflight_prepared_render_frame("),
        "render_packet must not rerun full structural preflight directly every frame"
    );
    for forbidden in [
        "WorldMut",
        "world.resource",
        "ResMut",
        "RenderFrameDataRegistry",
        "project_uniform_bindings_for_pass",
    ] {
        assert!(
            !render_packet.contains(forbidden),
            "render_packet must not perform live ECS extraction while preflighting submit data; found '{forbidden}'"
        );
    }
}
