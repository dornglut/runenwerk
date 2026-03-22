use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
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
        "src/plugins/render/renderer/submit.rs",
        "src/plugins/render/renderer/render_flow.rs",
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

    let render_flow = read("src/plugins/render/renderer/render_flow.rs");
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
        render_flow.contains("active runtime execution is single-view only"),
        "single-view deferred contract must remain explicit and fail-fast for multi-view packets"
    );

    let submit = read("src/plugins/render/renderer/submit.rs");
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
