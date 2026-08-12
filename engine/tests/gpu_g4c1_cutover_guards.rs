//! Structural G4C1/G4C2 cutover guards.
//!
//! These intentionally guard topology only. Behavioural G4C1/G4C2 tests exercise the public
//! typed API; this file prevents superseded raw ownership from quietly returning.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_sources(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
    {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

fn compact(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//") && !line.starts_with("* ") && !line.starts_with("*/")
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn read(manifest: &Path, relative: &str) -> String {
    fs::read_to_string(manifest.join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

fn inventory(manifest: &Path, tokens: &[&str]) -> BTreeMap<(String, String), usize> {
    let mut paths = Vec::new();
    collect_rust_sources(&manifest.join("src"), &mut paths);
    paths.sort();

    let mut result = BTreeMap::new();
    for path in paths {
        let source = compact(&fs::read_to_string(&path).expect("Rust source should be readable"));
        let relative = path
            .strip_prefix(manifest)
            .expect("source stays in engine")
            .to_string_lossy()
            .into_owned();
        for token in tokens {
            let count = source.matches(token).count();
            if count != 0 {
                result.insert((relative.clone(), (*token).to_string()), count);
            }
        }
    }
    result
}

fn token_paths(manifest: &Path, token: &str) -> BTreeSet<String> {
    inventory(manifest, &[token])
        .into_keys()
        .map(|(path, _)| path)
        .collect()
}

fn terminal_impl_blocks(source: &str) -> Vec<String> {
    let source = compact(source);
    let mut blocks = Vec::new();
    let mut offset = 0;
    while offset < source.len() {
        let tail = &source[offset..];
        let current_render = tail.find("implCurrentRender");
        let current_surface = tail.find("implCurrentSurface");
        let relative = match (current_render, current_surface) {
            (Some(render), Some(surface)) => render.min(surface),
            (Some(render), None) => render,
            (None, Some(surface)) => surface,
            (None, None) => break,
        };
        let start = offset + relative;
        let open = source[start..]
            .find('{')
            .map(|index| start + index)
            .expect("current bridge terminal implementation must have a body");
        let mut depth = 0_u32;
        let end = source[open..]
            .char_indices()
            .find_map(|(index, character)| match character {
                '{' => {
                    depth = depth.saturating_add(1);
                    None
                }
                '}' => {
                    depth = depth.saturating_sub(1);
                    (depth == 0).then_some(open + index)
                }
                _ => None,
            })
            .expect("current bridge terminal implementation body must close");
        blocks.push(source[start..=end].to_owned());
        offset = end.saturating_add(1);
    }
    blocks
}

#[test]
fn g4c1_generic_resource_creation_has_one_private_owner_and_one_surface_view_exception() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let owner = "src/plugins/gpu/backend/wgpu/resource_realization/mod.rs";
    for token in [
        ".create_buffer(",
        ".create_texture(",
        ".create_sampler(",
        ".create_query_set(",
    ] {
        assert_eq!(
            token_paths(&manifest, token),
            BTreeSet::from([owner.to_owned()]),
            "G4C1 {token} escaped its private realization owner"
        );
    }
    assert_eq!(
        token_paths(&manifest, ".create_view("),
        BTreeSet::from([
            owner.to_owned(),
            "src/plugins/render/renderer/mod.rs".to_owned(),
        ]),
        "G4C1 texture-view creation escaped its owner or the one presentation exception"
    );
    assert!(
        token_paths(&manifest, ".create_buffer_init(").is_empty(),
        "G4C1 must not retain a second buffer-creation path"
    );
}

#[test]
fn g4c2_is_the_only_shader_layout_and_bind_group_creation_owner() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let owner = "src/plugins/gpu/backend/wgpu/program_binding_realization/mod.rs";
    for token in [
        ".create_shader_module(",
        ".create_bind_group_layout(",
        ".create_pipeline_layout(",
        ".create_bind_group(",
    ] {
        assert_eq!(
            inventory(&manifest, &[token]),
            BTreeMap::from([((owner.to_owned(), token.to_owned()), 1)]),
            "G4C2 {token} must have exactly one source-wide private realization owner"
        );
    }
}

#[test]
fn g4c2_replaces_the_resource_bridge_and_surface_shader_binding_exception() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let forbidden = inventory(
        &manifest,
        &[
            "CurrentRenderResourceBridge",
            "CurrentRenderBindGroupTerminal",
            "CurrentRenderMaterialBindingTerminal",
            "CurrentRenderSampledTextureBindingTerminal",
            "SurfaceTextureView",
            "SamplerPlaceholder",
            "request_with_resource_realization_policy",
        ],
    );
    assert!(
        forbidden.is_empty(),
        "superseded G4C1/G4C2 compatibility authority remains: {forbidden:#?}"
    );

    let bridge_path = "src/plugins/gpu/backend/wgpu/program_binding_realization/current_render_pipeline_bridge.rs";
    let bridge = compact(&read(&manifest, bridge_path));
    assert_eq!(
        inventory(&manifest, &["structCurrentRenderPipelineBridge"]),
        BTreeMap::from([(
            (
                bridge_path.to_owned(),
                "structCurrentRenderPipelineBridge".to_owned()
            ),
            1,
        )]),
        "exactly one successor bridge may remain until G4C3"
    );
    for forbidden in [
        "pubstructCurrentRenderPipelineBridge",
        "Deref",
        "AsRef<",
        "FnOnce",
    ] {
        assert!(
            !bridge.contains(forbidden),
            "successor bridge exposed forbidden raw authority: {forbidden}"
        );
    }
    assert!(bridge.contains("for_pipeline_creation("));
    assert!(bridge.contains("for_pipeline_bind_groups("));

    let bindings = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/bindings.rs",
    ));
    assert!(bindings.contains("SurfaceColorisnotaG4C2sampledorstorageshaderresourcebeforeG7"));
    assert!(!bindings.contains("RuntimeTextureRef::Surface(texture)=>"));
}

#[test]
fn g4c2_successor_bridge_keeps_lent_backend_objects_private_and_nonretaining() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let records = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/resource_realization/records.rs",
    ));
    assert!(
        records.contains("pub(incrate::plugins::gpu::backend::wgpu)object:"),
        "residual G4C1 objects must be visible only to the private WGPU backend subtree"
    );
    assert!(
        !records.contains("pub(crate)object:"),
        "G4C2 must not widen residual G4C1 backend objects into a crate-wide raw escape path"
    );

    let mut paths = Vec::new();
    collect_rust_sources(&manifest.join("src"), &mut paths);
    let mut terminal_count = 0;
    for path in paths {
        let relative = path
            .strip_prefix(&manifest)
            .expect("source stays in engine")
            .display()
            .to_string();
        let source = fs::read_to_string(&path).expect("terminal source should be readable");
        for terminal in terminal_impl_blocks(&source) {
            terminal_count += 1;
            for forbidden in [
                "program.clone()",
                "layout.clone()",
                "bind_group.clone()",
                "buffer.clone()",
                "texture.clone()",
                "view.clone()",
                "sampler.clone()",
                "query_set.clone()",
                "Clone::clone(",
                "ToOwned::to_owned(",
                "realize_",
                "pollster::block_on(",
                "current_render_device_queue(",
            ] {
                assert!(
                    !terminal.contains(forbidden),
                    "current pipeline bridge terminal exceeded its lexical lending role via {forbidden} in {relative}"
                );
            }
        }
    }
    assert!(
        terminal_count != 0,
        "the successor bridge must retain audited current terminals until G4C3"
    );
}

#[test]
fn g4c2_uses_one_shared_health_gate_and_the_fixed_naga_profile() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let health_path = "src/plugins/gpu/backend/wgpu/health.rs";
    let health = compact(&read(&manifest, health_path));
    for token in ["structWgpuDeviceHealth", "structWgpuErrorAttributionGate"] {
        assert_eq!(
            token_paths(&manifest, token),
            BTreeSet::from([health_path.to_owned()]),
            "G4C2 must retain exactly one shared {token} authority"
        );
    }
    for token in [".set_device_lost_callback(", ".on_uncaptured_error("] {
        assert_eq!(
            token_paths(&manifest, token),
            BTreeSet::from([health_path.to_owned()]),
            "WGPU observer installation escaped the one shared health owner"
        );
        assert_eq!(
            health.matches(token).count(),
            1,
            "the shared health owner must install each observer exactly once"
        );
    }

    let device_request = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/device_request.rs",
    ));
    assert!(device_request.contains("lethealth=Arc::new(WgpuDeviceHealth::new());"));
    assert_eq!(
        device_request
            .matches("health.install_observers(&device);")
            .count(),
        1,
        "one device request must install the one shared observer pair"
    );

    let state = compact(&read(&manifest, "src/plugins/gpu/backend/wgpu/state.rs"));
    assert!(state.contains("health:Arc<WgpuDeviceHealth>"));
    assert!(state.contains("error_attribution_gate:Arc<WgpuErrorAttributionGate>"));

    let resource = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/resource_realization/mod.rs",
    ));
    let program = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/program_binding_realization/mod.rs",
    ));
    for source in [&resource, &program] {
        assert!(source.contains("health:Arc<WgpuDeviceHealth>"));
        assert!(source.contains("error_attribution_gate:Arc<WgpuErrorAttributionGate>"));
    }
    assert!(resource.contains(
        "letgate=self.error_attribution_gate.acquire();letregistries=self.registries(resource)?;"
    ));
    assert!(program.contains("let_gate=realization.error_attribution_gate.acquire();"));
    let current_host = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/current_host.rs",
    ));
    assert!(current_host.contains(
        "_error_attribution_gate=self.state.error_attribution_gate.acquire();surface.configure(&self.state.device,config);"
    ));
    assert!(
        current_host
            .contains("_error_attribution_gate:self.backend.error_attribution_gate.acquire()")
    );

    let evidence = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/program_binding_realization/evidence.rs",
    ));
    assert!(evidence.contains("ValidationFlags::all()"));
    assert!(evidence.contains("Capabilities::default()"));
    assert!(!evidence.contains("Capabilities::all()"));
    assert!(!program.contains("ShaderSource::Naga"));
    assert!(!program.contains("pollster::block_on"));
}

#[test]
fn renderer_completes_g4c1_g4c2_realization_before_its_one_raw_device_queue_interval() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let renderer = compact(&read(&manifest, "src/plugins/render/renderer/mod.rs"));
    assert!(
        !renderer.contains("current_render_device_queue"),
        "Gfx::render must not retain a broad raw loan around renderer realization"
    );

    let execute = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/execute.rs",
    ));
    let realization = execute
        .find("letmutbatch=self.realize_render_batch(")
        .expect("render packet must begin with its G4C1/G4C2 realization batch");
    let loan_marker = "letloan=context.current_render_device_queue()";
    let loan = execute
        .find(loan_marker)
        .expect("render packet must retain its one current G4C3/G5 raw interval");
    assert_eq!(
        execute.matches(loan_marker).count(),
        1,
        "renderer production execution must retain exactly one raw device/queue interval"
    );
    assert!(
        realization < loan,
        "the G4C1/G4C2 realization batch must complete before the raw G4C3/G5 interval"
    );
    let raw_phase = &execute[loan + loan_marker.len()..];
    for forbidden in [
        "self.realize_render_batch(",
        "context.realize_",
        "pollster::block_on(context.realize",
        "current_render_device_queue()",
    ] {
        assert!(
            !raw_phase.contains(forbidden),
            "the raw G4C3/G5 phase must not re-enter G4C1/G4C2 via {forbidden}"
        );
    }
    assert!(raw_phase.contains("self.create_ui_pipelines_for_raw_phase(context,loan.device)?"));
    assert!(raw_phase.contains("self.execute_realized_batch("));

    let setup = compact(&read(&manifest, "src/plugins/render/renderer/setup.rs"));
    let program = setup
        .find("fnrealize_ui_program_binding_artifacts(")
        .expect("UI G4C2 program realization must remain explicit");
    let pipeline_creation = setup
        .find("fncreate_ui_pipelines_for_raw_phase(")
        .expect("UI G4C3 pipeline creation must remain explicit");
    assert!(
        program < pipeline_creation,
        "UI program/layout/bind-group realization must remain separate from G4C3 pipeline creation"
    );
    assert!(setup.contains("pollster::block_on(context.realize_program"));
    assert!(setup.contains("pollster::block_on(context.realize_bind_group_layout"));
    assert!(setup.contains("pollster::block_on(context.realize_pipeline_layout"));
    assert!(setup.contains("pollster::block_on(context.realize_bind_group("));
    assert!(
        !setup.contains("current_render_device_queue("),
        "UI realization and G4C3 pipeline helper must receive no raw loan themselves"
    );

    let flow = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/execute_passes.rs",
    ));
    let pass_realization = flow
        .find("fnrealize_compiled_pass(")
        .expect("flow G4C2 realization helper must remain present");
    let pass_encoding = flow
        .find("fnencode_compiled_pass(")
        .expect("flow G4C3/G5 encoding helper must remain present");
    assert!(
        pass_realization < pass_encoding,
        "flow G4C2 program/layout/bind-group realization must be structurally separate from encoding"
    );
    assert!(
        !flow.contains("current_render_device_queue("),
        "flow realization and encoding helpers must receive the one renderer raw loan instead"
    );
    assert!(
        !flow[pass_encoding..].contains(".resolve_compiled_bind_group("),
        "G4C3/G5 pass encoders must consume prepared G4C2 artifacts rather than realize bindings"
    );

    for path in [
        "src/plugins/render/renderer/prepare.rs",
        "src/plugins/render/renderer/dynamic_targets.rs",
        "src/plugins/render/renderer/render_flow/capture.rs",
    ] {
        assert!(
            !compact(&read(&manifest, path)).contains("current_render_device_queue("),
            "{path} must finish G4C1/G4C2 preparation before renderer raw execution"
        );
    }

    let primitive = compact(&read(
        &manifest,
        "src/plugins/render/gpu_primitives/plan.rs",
    ));
    let fixture = primitive
        .split("fngpu_primitives_runtime_dispatch_writes_scan_scatter_and_draw_args_when_adapter_available")
        .nth(1)
        .expect("primitive runtime proof must remain");
    let fixture_body = fixture
        .split("fnrealize_runtime_primitive_stages(")
        .next()
        .expect("primitive runtime proof body must remain");
    let realization = fixture_body
        .find("letrealized_stages=realize_runtime_primitive_stages(")
        .expect("fixture must invoke its complete G4C2 primitive realization before raw work");
    let last_resource = fixture_body
        .rfind("prepare_readback_buffer(")
        .expect("fixture must retain G4C1 readback resource realization");
    let loan = fixture_body
        .find("context.current_render_device_queue()")
        .expect("fixture must retain its current G4C3/G5 interval");
    let encoding = fixture_body
        .find("encode_runtime_primitive_stage(&context,device,&mutencoder,stage)")
        .expect("fixture must preserve its G5 primitive encoding after the raw interval begins");
    assert!(
        last_resource < realization && realization < loan && loan < encoding,
        "fixture must complete G4C1 resources, then G4C2 program/layout/bindings, before G4C3/G5"
    );
    let primitive_realization = primitive
        .split("fnrealize_runtime_primitive_stages(")
        .nth(1)
        .expect("primitive fixture realization helper must remain");
    for required in [
        "context.realize_program",
        "context.realize_bind_group_layout",
        "context.realize_pipeline_layout",
        "context.realize_bind_group",
    ] {
        assert!(
            primitive_realization.contains(required),
            "primitive fixture realization helper must delegate {required:?} to G4C2"
        );
    }
    assert!(
        primitive.contains(".for_pipeline_creation("),
        "fixture must lend G4C2 program/layout to temporary G4C3 pipeline creation"
    );
    assert!(
        primitive.contains("self.device.create_compute_pipeline("),
        "temporary G4C3 compute-pipeline creation must remain isolated in its terminal"
    );
}
