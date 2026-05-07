use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn read_render_flow_sources() -> String {
    let mut combined = String::new();
    let mut entries = fs::read_dir("src/plugins/render/renderer/render_flow")
        .expect("failed to read render_flow source directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to collect render_flow source files");
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            combined.push_str(
                &fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display())),
            );
            combined.push('\n');
        }
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
        render_flow.contains("for pass in &flow.execution.passes"),
        "renderer must execute compiled execution plans instead of raw pass graph arrays"
    );
    assert!(
        !render_flow.contains("for pass in &flow.pass_order"),
        "renderer runtime should not iterate raw pass_order for encoding"
    );
    assert!(
        render_flow.contains("get_or_create_render_pipeline"),
        "renderer runtime should use renderer-owned artifact cache for render pipelines"
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
