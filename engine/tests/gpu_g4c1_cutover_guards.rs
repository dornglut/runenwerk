use std::collections::BTreeMap;
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
