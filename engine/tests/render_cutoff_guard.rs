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
        render_flow.contains("ordered_payloads()?"),
        "renderer must schedule execution payloads through prepared G3 node order"
    );
    assert!(
        !render_flow.contains("for pass in &flow.pass_order"),
        "renderer runtime should not iterate raw pass_order for encoding"
    );
    assert!(
        render_flow.contains("context.realize_compute_pipeline(")
            && render_flow.contains("context.realize_render_pipeline(")
            && render_flow.contains(".for_compute_pipeline(")
            && render_flow.contains(".for_render_pipeline("),
        "renderer runtime must realize pipelines through G4C3 and consume opaque realized handles through the execution bridge"
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
fn g3_render_cutover_has_one_prepared_graph_authority_and_payload_only_sidecar() {
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
    let payload_start = adapter
        .find("pub(crate) enum RenderGpuWorkPayload")
        .expect("render G3 adapter should define its execution-only payload");
    let payload_tail = &adapter[payload_start..];
    let payload_end = payload_tail
        .find("\n}\n\nimpl RenderGpuWorkPayload")
        .expect("render execution payload declaration should precede its implementation");
    let payload = &payload_tail[..payload_end];
    assert!(payload.contains("occurrence: RenderGpuWorkOccurrenceId"));
    assert!(
        !payload.contains("pass_id:"),
        "render execution payload must not duplicate compiled pass identity after occurrence cutover"
    );
    for forbidden_truth in [
        "CompiledPassExecutionPlan",
        "GpuWorkOperation",
        "GpuRuntimeBindingSet",
        "GpuResourceAccess",
    ] {
        assert!(
            !payload.contains(forbidden_truth),
            "render execution payload must retain renderer identity only, not generic GPU truth '{forbidden_truth}'"
        );
    }

    let sidecar_start = adapter
        .find("struct RenderGpuWorkSidecar")
        .expect("render G3 adapter should define its private sidecar");
    let sidecar_tail = &adapter[sidecar_start..];
    let sidecar_end = sidecar_tail
        .find("\n}\n\nimpl RenderGpuWorkSidecar")
        .expect("sidecar declaration should precede its implementation");
    let sidecar = &sidecar_tail[..sidecar_end];
    assert!(sidecar.contains("BTreeMap<GpuPreparedWorkNodeId, RenderGpuWorkPayload>"));
    for forbidden_truth in [
        "GpuResourceAccess",
        "GpuCapabilityRequirements",
        "GpuInitialCoverage",
        "GpuWorkDependency",
        "topological_order",
    ] {
        assert!(
            !sidecar.contains(forbidden_truth),
            "render sidecar must not contain generic graph truth '{forbidden_truth}'"
        );
    }

    let execute = read("src/plugins/render/renderer/render_flow/execute.rs");
    let schedule = function_body(&execute, "fn schedule_legacy_invocation_work(");
    assert!(schedule.contains(".ordered_payloads()?"));
    for alternate_order in ["flow.execution.passes", "topological_sort", "sort_by"] {
        assert!(
            !schedule.contains(alternate_order),
            "runtime scheduling must not restore alternate order path '{alternate_order}'"
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
        .expect("render packet should drain owned invocation resolutions before raw execution");
    let legacy_prepare = render_packet
        .find("let legacy_invocation_work = canonical_resolutions")
        .expect("render packet should derive temporary legacy G3 plans from drained resolutions");
    let raw_loan = render_packet
        .find("let loan = context.current_render_device_queue();")
        .expect("transitional renderer should still expose the bounded raw loan before final G5C1 cutover");
    assert!(
        drain < legacy_prepare && legacy_prepare < raw_loan,
        "owned invocation resolutions must be drained before temporary legacy plans are prepared, and all G3 preparation must remain outside the raw device/queue loan"
    );
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
