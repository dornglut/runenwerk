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

fn compact_executable_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//") && !line.starts_with('*')
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn source_inventory(manifest: &Path, tokens: &[&str]) -> BTreeMap<(String, String), usize> {
    let mut paths = Vec::new();
    collect_rust_sources(&manifest.join("src"), &mut paths);
    paths.sort();

    let mut inventory = BTreeMap::new();
    for path in paths {
        let source = compact_executable_source(
            &fs::read_to_string(&path).expect("Rust source should be readable"),
        );
        let relative = path
            .strip_prefix(manifest)
            .expect("source should remain in the engine crate")
            .to_string_lossy()
            .into_owned();
        for token in tokens {
            let count = source.matches(token).count();
            if count != 0 {
                inventory.insert((relative.clone(), (*token).to_string()), count);
            }
        }
    }
    inventory
}

fn terminal_impl_blocks(source: &str) -> Vec<String> {
    let compact = compact_executable_source(source);
    let mut blocks = Vec::new();
    let mut offset = 0;
    while offset < compact.len() {
        let tail = &compact[offset..];
        let render = tail.find("implCurrentRender");
        let surface = tail.find("implCurrentSurface");
        let relative = match (render, surface) {
            (Some(render), Some(surface)) => render.min(surface),
            (Some(render), None) => render,
            (None, Some(surface)) => surface,
            (None, None) => break,
        };
        let start = offset + relative;
        let open = compact[start..]
            .find('{')
            .map(|index| start + index)
            .expect("terminal implementation must have a body");
        let mut depth = 0_u32;
        let mut end = None;
        for (index, character) in compact[open..].char_indices() {
            match character {
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(open + index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.expect("terminal implementation body must close");
        blocks.push(compact[start..=end].to_string());
        offset = end.saturating_add(1);
    }
    blocks
}

fn expected_inventory(
    entries: &[(&str, &str, usize)],
) -> BTreeMap<(String, String), usize> {
    entries
        .iter()
        .map(|(path, token, count)| ((path.to_string(), token.to_string()), *count))
        .collect()
}

#[test]
fn g4c1_generic_resource_creation_has_one_private_owner_and_two_surface_view_exemptions() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let creation_tokens = [
        ".create_buffer(",
        ".create_buffer_init(",
        ".create_texture(",
        ".create_view(",
        ".create_sampler(",
        ".create_query_set(",
    ];
    let actual = source_inventory(&manifest, &creation_tokens);
    let realization = "src/plugins/gpu/backend/wgpu/resource_realization/mod.rs";
    let expected = BTreeMap::from([
        ((realization.to_string(), ".create_buffer(".to_string()), 1),
        ((realization.to_string(), ".create_texture(".to_string()), 1),
        ((realization.to_string(), ".create_view(".to_string()), 1),
        ((realization.to_string(), ".create_sampler(".to_string()), 1),
        (
            (realization.to_string(), ".create_query_set(".to_string()),
            1,
        ),
        (
            (
                "src/plugins/render/renderer/mod.rs".to_string(),
                ".create_view(".to_string(),
            ),
            1,
        ),
        (
            (
                "src/plugins/render/renderer/render_flow/bindings.rs".to_string(),
                ".create_view(".to_string(),
            ),
            1,
        ),
    ]);

    assert_eq!(
        actual, expected,
        "G4C1 raw generic resource-creation inventory changed"
    );

    let renderer = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/renderer/mod.rs"))
            .expect("renderer source should be readable"),
    );
    assert!(renderer.contains("get_current_texture(render_surface_id)?;"));
    assert!(renderer.contains("frame.texture.create_view(&Default::default())"));

    let bindings = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/renderer/render_flow/bindings.rs"))
            .expect("render binding source should be readable"),
    );
    assert!(bindings.contains(
        "RuntimeTextureRef::Surface(texture)=>Ok(RuntimeBindingResource::SurfaceTextureView(texture.create_view("
    ));
}

#[test]
fn g4c1_resource_bridge_is_single_crate_private_and_exactly_purpose_typed() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bridge_path = manifest.join(
        "src/plugins/gpu/backend/wgpu/resource_realization/current_render_resource_bridge.rs",
    );
    let bridge_source =
        fs::read_to_string(&bridge_path).expect("current render bridge should be readable");
    let bridge = compact_executable_source(&bridge_source);

    let definitions = source_inventory(&manifest, &["structCurrentRenderResourceBridge"]);
    assert_eq!(
        definitions,
        BTreeMap::from([(
            (
                "src/plugins/gpu/backend/wgpu/resource_realization/current_render_resource_bridge.rs"
                    .to_string(),
                "structCurrentRenderResourceBridge".to_string(),
            ),
            1,
        )]),
        "G4C1 must retain exactly one resource-reference bridge"
    );
    assert!(bridge.contains(
        "pub(crate)structCurrentRenderResourceBridge<'a>{state:&'aResourceRealizationState,}"
    ));
    assert!(!bridge.contains("pubstructCurrentRenderResourceBridge"));
    assert!(!bridge.contains("Deref"));
    assert!(!bridge.contains("AsRef<"));
    assert!(!bridge.contains("Clone,Copy"));

    let terminal_traits = [
        "CurrentRenderBufferBindingTerminal",
        "CurrentRenderBufferUploadTerminal",
        "CurrentRenderVertexBufferTerminal",
        "CurrentRenderIndexBufferTerminal",
        "CurrentRenderIndirectBufferTerminal",
        "CurrentRenderReadbackBufferTerminal",
        "CurrentRenderTextureUploadTerminal",
        "CurrentRenderTimestampWritesTerminal",
        "CurrentRenderBufferCopyTerminal",
        "CurrentRenderTextureCopyTerminal",
        "CurrentSurfaceTextureCopyTerminal",
        "CurrentRenderTextureReadbackCopyTerminal",
        "CurrentSurfaceReadbackCopyTerminal",
        "CurrentRenderSampledTextureBindingTerminal",
        "CurrentRenderTimestampResourcesTerminal",
        "CurrentRenderMaterialBindingTerminal",
        "CurrentRenderBindGroupTerminal",
        "CurrentRenderAttachmentsTerminal",
    ];
    assert_eq!(bridge.matches("purpose_terminal!(").count(), 8);
    // Surface-copy terminals intentionally use the `CurrentSurface...` vocabulary so they do
    // not masquerade as a second renderer-resource boundary. The exact terminal list below
    // still accounts for those two purpose-specific traits.
    assert_eq!(bridge.matches("pub(crate)traitCurrentRender").count(), 8);
    for terminal in terminal_traits {
        assert!(
            bridge.contains(&format!("purpose_terminal!({terminal},"))
                || bridge.contains(&format!("pub(crate)trait{terminal}")),
            "missing exact G4C1 purpose terminal {terminal}"
        );
    }
    let trait_region = bridge
        .split("#[derive(Debug)]pub(crate)structCurrentRenderResourceBridge")
        .next()
        .expect("bridge trait region should exist");
    assert!(
        !trait_region.contains("->"),
        "resource bridge terminals must have fixed unit results"
    );

    let bridge_methods = [
        "for_buffer_binding",
        "for_buffer_upload",
        "for_vertex_buffer",
        "for_index_buffer",
        "for_indirect_buffer",
        "for_buffer_copy",
        "for_buffer_readback",
        "for_texture_upload",
        "for_texture_copy",
        "for_surface_texture_copy",
        "for_texture_readback_copy",
        "for_surface_readback_copy",
        "for_sampled_texture_binding",
        "for_material_binding",
        "for_bind_group",
        "for_pass_attachments",
        "for_timestamp_writes",
        "for_timestamp_resources",
    ];
    assert_eq!(
        bridge.matches("pub(crate)fnfor_").count(),
        bridge_methods.len()
    );
    for method in bridge_methods {
        let marker = format!("pub(crate)fn{method}(");
        assert_eq!(
            bridge.matches(&marker).count(),
            1,
            "bridge method drift: {method}"
        );
        let signature = bridge
            .split(&marker)
            .nth(1)
            .and_then(|tail| tail.split('{').next())
            .expect("bridge method signature should terminate");
        for forbidden in [
            "Fn(", "FnMut", "FnOnce", "Device", "Queue", "&Buffer", "&Texture",
        ] {
            assert!(
                !signature.contains(forbidden),
                "bridge method {method} exposes forbidden generic/raw authority: {forbidden}"
            );
        }
    }
}

#[test]
fn g4c1_resource_bridge_terminal_implementation_inventory_is_exact_and_nonretaining() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bridge =
        "src/plugins/gpu/backend/wgpu/resource_realization/current_render_resource_bridge.rs";
    let mut actual = source_inventory(&manifest, &["implCurrentRender", "implCurrentSurface"]);
    actual.remove(&(bridge.to_string(), "implCurrentRender".to_string()));
    actual.remove(&(bridge.to_string(), "implCurrentSurface".to_string()));

    let expected = expected_inventory(&[
        ("src/plugins/render/gpu_primitives/plan.rs", "implCurrentRender", 4),
        ("src/plugins/render/renderer/dynamic_targets.rs", "implCurrentRender", 1),
        ("src/plugins/render/renderer/prepare.rs", "implCurrentRender", 3),
        ("src/plugins/render/renderer/render_flow/bindings.rs", "implCurrentRender", 1),
        ("src/plugins/render/renderer/render_flow/capture.rs", "implCurrentRender", 2),
        ("src/plugins/render/renderer/render_flow/capture.rs", "implCurrentSurface", 1),
        ("src/plugins/render/renderer/render_flow/execute.rs", "implCurrentRender", 1),
        ("src/plugins/render/renderer/render_flow/execute_passes.rs", "implCurrentRender", 10),
        ("src/plugins/render/renderer/render_flow/execute_passes.rs", "implCurrentSurface", 1),
        ("src/plugins/render/renderer/render_flow/gpu_timing.rs", "implCurrentRender", 3),
        ("src/plugins/render/renderer/mod.rs", "implCurrentRender", 5),
    ]);
    assert_eq!(
        actual, expected,
        "G4C1 bridge terminal implementation inventory changed"
    );

    let audited_paths = expected
        .keys()
        .map(|(relative, _)| relative.as_str())
        .collect::<BTreeSet<_>>();
    for relative in audited_paths {
        let source = fs::read_to_string(manifest.join(relative)).unwrap_or_else(|error| {
            panic!("cannot read audited terminal source {relative}: {error}")
        });
        for block in terminal_impl_blocks(&source) {
            for forbidden in [
                "buffer.clone()",
                "texture.clone()",
                "view.clone()",
                "sampler.clone()",
                "query_set.clone()",
                "source.clone()",
                "destination.clone()",
                "buffers[0].clone()",
                "views[0].clone()",
                "samplers[0].clone()",
                "Clone::clone(",
                "ToOwned::to_owned(",
            ] {
                assert!(
                    !block.contains(forbidden),
                    "temporary bridge terminal retained raw WGPU ownership via {forbidden} in {relative}"
                );
            }
        }
    }
}

#[test]
fn g4c1_backend_fault_observation_has_no_thread_local_constructor_attribution() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = compact_executable_source(
        &fs::read_to_string(
            manifest.join("src/plugins/gpu/backend/wgpu/resource_realization/mod.rs"),
        )
        .expect("resource realization source should be readable"),
    );
    for retired in [
        "thread_local!",
        "ACTIVE_BACKEND_ERROR_SLOT",
        "BackendErrorCapture",
        "catch_unwind",
        "AssertUnwindSafe",
    ] {
        assert!(
            !source.contains(retired),
            "G4C1 must not fabricate synchronous WGPU error attribution through {retired}"
        );
    }
    assert!(source.contains("device.on_uncaptured_error"));
    assert!(source.contains("uncaptured_health.mark_uncaptured(error)"));
    assert!(source.contains(
        "self.ensure_available(resource)?;letcreated=create();self.ensure_available(resource)?;"
    ));
}

#[test]
fn g4c1_current_render_storage_texture_contract_is_write_only() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let passes = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/api/passes.rs"))
            .expect("render pass source should be readable"),
    );
    assert!(passes.contains("access:GpuStorageTextureAccess::WriteOnly"));

    let resources = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/adapters/gpu_resources.rs"))
            .expect("render GPU resource adapter should be readable"),
    );
    let normalized = resources
        .split("fnnormalized_texture_usages")
        .nth(1)
        .and_then(|tail| tail.split("pubfndetect_duplicate_resource_ids").next())
        .expect("normalized texture usage function should remain inspectable");
    assert!(normalized.contains(
        "iftexture.usage.storage{usages.insert(GpuTextureUsage::StorageWrite);}"
    ));
    assert!(!normalized.contains("GpuTextureUsage::StorageRead"));

    let dynamic = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/renderer/dynamic_targets.rs"))
            .expect("dynamic target source should be readable"),
    );
    let dynamic_usage = dynamic
        .split("fndynamic_usage_to_gpu")
        .nth(1)
        .expect("dynamic target normalized usage mapping should remain inspectable");
    assert!(dynamic_usage.contains("out.push(GpuTextureUsage::StorageWrite)"));
    assert!(!dynamic_usage.contains("GpuTextureUsage::StorageRead"));

    let capabilities = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/adapters/gpu_capabilities.rs"))
            .expect("current renderer capability adapter should be readable"),
    );
    assert!(!capabilities.contains("storage_read:true"));

    let host = compact_executable_source(
        &fs::read_to_string(manifest.join("src/plugins/render/backend/wgpu_ctx.rs"))
            .expect("current host context source should be readable"),
    );
    assert!(!host.contains("GpuFormatRole::StorageRead"));
}

#[test]
fn g4c1_r8_color_target_contract_uses_one_float_component() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = compact_executable_source(
        &fs::read_to_string(
            manifest.join("src/plugins/gpu/api/program/pipeline/render_state/target.rs"),
        )
        .expect("render target contract should be readable"),
    );
    assert!(target.contains(
        "GpuTextureFormat::R8Unorm=>(GpuShaderIoScalarClass::Float,1)"
    ));
}

#[test]
fn g4c1_synthetic_handle_and_device_queue_inventories_are_exact() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let synthetic = source_inventory(&manifest, &["from_descriptor("]);
    assert_eq!(
        synthetic,
        BTreeMap::from([
            (
                (
                    "src/plugins/gpu/api/handles.rs".to_string(),
                    "from_descriptor(".to_string(),
                ),
                6,
            ),
            (
                (
                    "src/plugins/gpu/api/graph/tests/authoring.rs".to_string(),
                    "from_descriptor(".to_string(),
                ),
                1,
            ),
        ]),
        "synthetic typed-handle construction escaped accepted RunenGPU ownership"
    );

    let loan_calls = source_inventory(&manifest, &[".current_render_device_queue("]);
    assert_eq!(
        loan_calls,
        BTreeMap::from([
            (
                (
                    "src/plugins/render/gpu_primitives/plan.rs".to_string(),
                    ".current_render_device_queue(".to_string(),
                ),
                1,
            ),
            (
                (
                    "src/plugins/render/renderer/mod.rs".to_string(),
                    ".current_render_device_queue(".to_string(),
                ),
                1,
            ),
            (
                (
                    "src/plugins/render/renderer/render_flow/gpu_timing.rs".to_string(),
                    ".current_render_device_queue(".to_string(),
                ),
                1,
            ),
        ]),
        "CurrentRenderDeviceQueue call-site inventory changed"
    );

    for test_source in [
        "src/plugins/render/gpu_primitives/plan.rs",
        "src/plugins/render/renderer/render_flow/gpu_timing.rs",
    ] {
        let source = fs::read_to_string(manifest.join(test_source))
            .expect("device/queue evidence source should be readable");
        let call = source
            .find(".current_render_device_queue()")
            .expect("expected device/queue test call should remain");
        let test_boundary = source[..call]
            .rfind("#[cfg(test)]")
            .expect("device/queue evidence loan must remain test-only");
        assert!(test_boundary < call);
    }
}
