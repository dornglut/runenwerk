//! Structural G4C1/G4C2/G4C3 cutover guards.
//!
//! Behavioural tests exercise the typed API. These guards keep the accepted G4 ownership
//! topology from regressing after G5C1 removes the renderer's residual execution interval.

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
fn g4c1_logical_resource_creation_stays_private_while_g5b_staging_is_isolated() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let owner = "src/plugins/gpu/backend/wgpu/resource_realization/mod.rs";
    let execution = "src/plugins/gpu/backend/wgpu/execution.rs";
    let surface_execution = "src/plugins/gpu/backend/wgpu/surface/execution.rs";

    assert_eq!(
        token_paths(&manifest, ".create_buffer("),
        BTreeSet::from([owner.to_owned(), execution.to_owned()]),
        "buffer creation must remain either G4C1 logical realization or the one private G5B staging owner"
    );
    for token in [".create_texture(", ".create_sampler(", ".create_query_set("] {
        assert_eq!(
            token_paths(&manifest, token),
            BTreeSet::from([owner.to_owned()]),
            "G4C1 {token} escaped its private realization owner"
        );
    }
    assert_eq!(
        token_paths(&manifest, ".create_view("),
        BTreeSet::from([owner.to_owned(), surface_execution.to_owned(),]),
        "texture-view creation escaped G4C1 or G7 surface execution"
    );
    assert!(
        token_paths(&manifest, ".create_buffer_init(").is_empty(),
        "G4C1/G5B must not retain a second buffer-creation helper path"
    );

    let execution_source = compact(&read(&manifest, execution));
    assert_eq!(
        execution_source.matches(".create_buffer(").count(),
        4,
        "G5B permits exactly buffer/texture upload and readback staging buffer creation"
    );
    assert!(execution_source.contains(
        "label:Some(\"RunenGPUuploadstaging\"),size:payload.layout().byte_len(),usage:BufferUsages::COPY_SRC,mapped_at_creation:true"
    ));
    assert!(execution_source.contains(
        "label:Some(\"RunenGPUreadbackstaging\"),size:*size,usage:BufferUsages::COPY_DST|BufferUsages::MAP_READ,mapped_at_creation:false"
    ));
    assert!(execution_source.contains(
        "label:Some(\"RunenGPUtextureuploadstaging\"),size:staging_layout.staging_byte_len,usage:BufferUsages::COPY_SRC,mapped_at_creation:true"
    ));
    assert!(execution_source.contains(
        "label:Some(\"RunenGPUtexturereadbackstaging\"),size:staging_layout.staging_byte_len,usage:BufferUsages::COPY_DST|BufferUsages::MAP_READ,mapped_at_creation:false"
    ));
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
fn g4c3_is_the_only_compute_and_render_pipeline_creation_owner() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for (token, owner) in [
        (
            ".create_compute_pipeline(",
            "src/plugins/gpu/backend/wgpu/pipeline_realization/compute.rs",
        ),
        (
            ".create_render_pipeline(",
            "src/plugins/gpu/backend/wgpu/pipeline_realization/render.rs",
        ),
    ] {
        assert_eq!(
            inventory(&manifest, &[token]),
            BTreeMap::from([((owner.to_owned(), token.to_owned()), 1)]),
            "G4C3 {token} must have exactly one source-wide private realization owner"
        );
    }

    let cache = compact(&read(
        &manifest,
        "src/plugins/render/renderer/pipeline_cache.rs",
    ));
    for forbidden in [
        "GpuRealizedComputePipeline",
        "GpuRealizedRenderPipeline",
        "wgpu::ComputePipeline",
        "wgpu::RenderPipeline",
    ] {
        assert!(
            !cache.contains(forbidden),
            "renderer pipeline cache regained reusable raw pipeline authority via {forbidden}"
        );
    }

    let state = compact(&read(&manifest, "src/plugins/gpu/backend/wgpu/state.rs"));
    assert!(!state.contains("create_compute_pipeline("));
    assert!(!state.contains("create_render_pipeline("));
}

#[test]
fn accepted_g4_bridge_ladder_has_no_renderer_execution_bridge_or_predecessor_authority() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let forbidden = inventory(
        &manifest,
        &[
            "CurrentRenderResourceBridge",
            "CurrentRenderPipelineBridge",
            "CurrentRenderPipelineCreationTerminal",
            "current_render_pipeline_bridge",
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
        "superseded G4 compatibility authority remains: {forbidden:#?}"
    );

    assert!(
        !manifest
            .join("src/plugins/gpu/backend/wgpu/program_binding_realization/current_render_execution_bridge.rs")
            .exists(),
        "G5C1 must delete the residual renderer execution bridge module"
    );
    for retired in [
        "CurrentRenderExecutionBridge",
        "current_render_execution_bridge",
        "CurrentRenderDeviceQueue",
        "current_render_device_queue",
    ] {
        assert!(
            token_paths(&manifest, retired).is_empty(),
            "G5C1 must remove source-wide renderer raw authority '{retired}'"
        );
    }

    let bindings = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/bindings.rs",
    ));
    assert!(bindings.contains("hasnologicaltextureviewforG4C2shaderbindingrealization"));
    assert!(bindings.contains("resolve_logical_texture_binding("));
    assert!(!bindings.contains("RuntimeTextureRef::Surface(texture)=>"));
}

#[test]
fn canonical_execution_keeps_backend_objects_private_without_renderer_terminals() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let records = compact(&read(
        &manifest,
        "src/plugins/gpu/backend/wgpu/resource_realization/records.rs",
    ));
    assert!(
        records.contains("pub(incrate::plugins::gpu::backend::wgpu)object:"),
        "residual G4 objects must be visible only to the private WGPU backend subtree"
    );
    assert!(
        !records.contains("pub(crate)object:"),
        "G4 must not widen private backend objects into a crate-wide raw escape path"
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
                "pipeline.clone()",
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
                    "current execution terminal exceeded lexical lending via {forbidden} in {relative}"
                );
            }
        }
    }
    assert_eq!(
        terminal_count, 0,
        "G5C1 must delete every purpose-typed renderer raw terminal implementation"
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

    assert!(
        !manifest
            .join("src/plugins/gpu/backend/wgpu/current_host.rs")
            .exists(),
        "G7A surface authority must keep the retired current-host bridge deleted"
    );
    let surface = compact(&read(&manifest, "src/plugins/gpu/backend/wgpu/surface.rs"));
    assert!(surface.contains("let_attribution_gate=error_attribution_gate.acquire();"));
    assert!(surface.contains("record.surface.configure(device,&native);"));

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
fn renderer_completes_realization_before_one_canonical_g5_acceptance() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let renderer = compact(&read(&manifest, "src/plugins/render/renderer/mod.rs"));
    assert!(
        !renderer.contains("current_render_device_queue"),
        "Gfx::render must not retain a broad raw loan around realization"
    );

    let execute = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/execute.rs",
    ));
    let realization = execute
        .find("letmutbatch=self.realize_render_batch(")
        .expect("render packet must begin with its complete G4 realization batch");
    let graph = execute
        .find("prepare_render_gpu_frame_work(")
        .expect("render packet must prepare one complete frame graph");
    let prepare = execute
        .find("context.prepare_submission(graph)")
        .expect("render packet must delegate physical preparation to RunenGPU");
    let accept = execute
        .find(".submit_prepared(prepared)")
        .expect("render packet must cross one irreversible RunenGPU acceptance boundary");
    assert!(
        realization < graph && graph < prepare && prepare < accept,
        "renderer realization and frame graph formation must complete before RunenGPU acceptance"
    );
    for forbidden in [
        "current_render_device_queue()",
        "current_render_execution_bridge()",
        "CommandEncoder",
        "queue.submit(",
        "create_compute_pipeline(",
        "create_render_pipeline(",
    ] {
        assert!(
            !execute.contains(forbidden),
            "renderer production execution must not retain raw authority via {forbidden}"
        );
    }

    let setup = compact(&read(&manifest, "src/plugins/render/renderer/setup.rs"));
    let ui_realization = setup
        .find("fnrealize_ui_program_binding_artifacts(")
        .expect("UI G4 realization must remain explicit");
    let ui_realization = &setup[ui_realization..];
    for required in [
        "context.realize_program",
        "context.realize_pipeline_layout",
        "context.realize_render_pipeline",
    ] {
        assert!(
            ui_realization.contains(required),
            "UI realization must delegate {required:?} to RunenGPU"
        );
    }
    assert!(!setup.contains("current_render_device_queue("));
    assert!(!setup.contains("create_render_pipeline("));

    let flow = compact(&read(
        &manifest,
        "src/plugins/render/renderer/render_flow/pipeline_realization.rs",
    ));
    let pass_realization = flow
        .find("fnrealize_compiled_pass(")
        .expect("flow G4 realization helper must remain present");
    let realization_phase = &flow[pass_realization..];
    assert!(realization_phase.contains("context.realize_compute_pipeline("));
    assert!(realization_phase.contains("context.realize_render_pipeline("));
    assert!(!flow.contains("current_render_device_queue("));
    assert!(!flow.contains("create_compute_pipeline("));
    assert!(!flow.contains("create_render_pipeline("));

    for path in [
        "src/plugins/render/renderer/prepare.rs",
        "src/plugins/render/renderer/dynamic_targets.rs",
        "src/plugins/render/renderer/render_flow/capture.rs",
        "src/plugins/render/renderer/render_flow/gpu_timing.rs",
    ] {
        let source = read(&manifest, path);
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("Rust source split always retains the production prefix");
        assert!(
            !compact(production).contains("current_render_device_queue("),
            "{path} production code must use typed realization/execution bridges rather than regain a raw loan"
        );
    }

    let primitive = compact(&read(
        &manifest,
        "src/plugins/render/gpu_primitives/plan.rs",
    ));
    assert!(!primitive.contains("current_render_execution_bridge("));
    assert!(!primitive.contains("current_render_device_queue("));
    assert!(!primitive.contains(".create_compute_pipeline("));
}
