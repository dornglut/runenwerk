//! Future-transferable RunenGPU contract boundaries.

pub mod api;
mod backend;

pub(crate) use api::GpuWorkAuthoringErrorContext;
pub use api::*;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn rust_sources_below(root: &Path, paths: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(root).expect("source directory should be readable") {
            let path = entry.expect("source entry should be readable").path();
            if path.is_dir() {
                rust_sources_below(&path, paths);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                paths.push(path);
            }
        }
    }

    #[test]
    fn gpu_g2_g3_boundary_has_no_forbidden_dependencies_or_vocabulary() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugins/gpu");
        let forbidden = [
            ["crate::plugins::", "render"].concat(),
            ["crate::plugins::", "ui"].concat(),
            ["ecs", "::"].concat(),
            ["wgpu", "::"].concat(),
            ["winit", "::"].concat(),
            ["runen", "_sdf"].concat(),
            ["runen", "ui"].concat(),
            ["editor", "_"].concat(),
            ["world", "_sdf"].concat(),
            ["product", "::"].concat(),
            ["image", "::"].concat(),
            ["png", "::"].concat(),
            ["exr", "::"].concat(),
            ["ff", "mpeg"].concat(),
            ["Render", "Flow"].concat(),
            ["Render", "Feature"].concat(),
            ["Render", "Target"].concat(),
            ["Render", "Surface"].concat(),
            ["Render", "Frame"].concat(),
            ["Render", "Shader"].concat(),
            ["Native", "Window"].concat(),
            ["Type", "Id"].concat(),
            ["std::any::", "Any"].concat(),
            ["include", "!("].concat(),
            ["shader", "_asset"].concat(),
            ["assets/", "shaders"].concat(),
        ];

        let mut paths = Vec::new();
        rust_sources_below(&root, &mut paths);
        paths.sort();
        paths.dedup();
        let raw_backend_token = ["wgpu", "::"].concat();
        let private_wgpu_backend_root = root.join("backend/wgpu");

        for path in paths {
            let source = fs::read_to_string(&path).expect("GPU boundary source should be readable");
            let private_wgpu_backend =
                path.starts_with(&private_wgpu_backend_root) || path == root.join("backend/mod.rs");
            for line in source.lines().filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//") && !trimmed.starts_with('*')
            }) {
                assert!(
                    forbidden
                        .iter()
                        .filter(|token| !private_wgpu_backend || *token != &raw_backend_token)
                        .all(|token| !line.contains(token)),
                    "forbidden GPU boundary import in {}: {}",
                    path.display(),
                    line
                );
            }
        }
    }

    #[test]
    fn g4a_keeps_wgpu_authority_private_and_retired_renderer_authority_deleted() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let gpu_root = manifest.join("src/plugins/gpu");
        let backend = gpu_root.join("backend/wgpu");
        let backend_root = backend.join("mod.rs");
        let current_host = backend.join("current_host.rs");
        let wgpu_context = manifest.join("src/plugins/render/backend/wgpu_ctx.rs");
        let renderer_root = manifest.join("src/plugins/render");

        assert!(
            backend_root.exists(),
            "G4A must retain exactly one private WGPU owner"
        );
        assert!(
            !manifest
                .join("src/plugins/render/backend/device.rs")
                .exists(),
            "renderer device-request authority must remain deleted"
        );

        let context_source =
            fs::read_to_string(wgpu_context).expect("renderer GPU context should be readable");
        let mut backend_paths = Vec::new();
        rust_sources_below(&backend, &mut backend_paths);
        backend_paths.sort();
        let backend_source = backend_paths
            .iter()
            .map(|path| fs::read_to_string(path).expect("private WGPU backend should be readable"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !context_source.contains("pub device") && !context_source.contains("pub queue"),
            "WgpuCtx must not restore public device or queue authority"
        );
        assert!(
            !current_host.exists(),
            "the pre-G7 current-host compatibility bridge must remain deleted"
        );
        for retired_surface_authority in [
            "request_for_current_host",
            "current_host_surface_bridge",
            "CurrentHostSurfaceBridge",
        ] {
            assert!(
                !backend_source.contains(retired_surface_authority)
                    && !context_source.contains(retired_surface_authority),
                "retired current-host surface authority remains: {retired_surface_authority}"
            );
        }
        assert!(
            context_source.contains("surface: GpuSurfaceHandle")
                && context_source.contains("config: GpuSurfaceConfiguration")
                && context_source.contains("GpuContext::request_for_surface(")
                && context_source.contains(".acquire_surface_image(surface)"),
            "renderer surface state must retain only G7A logical mapping and acquisition"
        );

        let source_root = manifest.join("src");
        let mut source_paths = Vec::new();
        rust_sources_below(&source_root, &mut source_paths);
        let creation_tokens = [
            ["Instance", "::new(InstanceDescriptor"].concat(),
            ["request", "_adapter(&RequestAdapterOptions"].concat(),
            ["request", "_device(&DeviceDescriptor"].concat(),
        ];
        for path in source_paths {
            let source = fs::read_to_string(&path).expect("source should be readable");
            for creation in &creation_tokens {
                if source.contains(creation) {
                    assert!(
                        path.starts_with(&backend),
                        "replaced instance/adapter/device creation escaped the private RunenGPU owner: {}",
                        path.display()
                    );
                }
            }
        }

        let mut render_sources = Vec::new();
        rust_sources_below(&renderer_root, &mut render_sources);
        for path in render_sources {
            let source = fs::read_to_string(&path).expect("source file should be readable");
            assert!(
                !source.contains("RenderBackendTimingCapabilities"),
                "retired timing authority remains in {}",
                path.display()
            );
        }
    }

    #[test]
    fn g4c1_realization_core_has_one_context_owner_and_no_transfer_or_public_raw_authority() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let gpu_root = manifest.join("src/plugins/gpu");
        let realization_root = gpu_root.join("backend/wgpu/resource_realization");
        let context_state = fs::read_to_string(gpu_root.join("backend/wgpu/state.rs"))
            .expect("private context state should be readable");
        let context_descriptor = fs::read_to_string(gpu_root.join("api/context/descriptor.rs"))
            .expect("context descriptor should be readable");
        let public_realization = fs::read_to_string(gpu_root.join("api/realization.rs"))
            .expect("public realization contract should be readable");
        let mut realization_paths = Vec::new();
        rust_sources_below(&realization_root, &mut realization_paths);
        realization_paths.sort();
        let realization_source = realization_paths
            .iter()
            .map(|path| fs::read_to_string(path).expect("realization source should be readable"))
            .collect::<Vec<_>>()
            .join("\n");
        let realization_creation_source = realization_paths
            .iter()
            .filter(|path| {
                path.file_name().and_then(|name| name.to_str())
                    != Some("current_render_resource_bridge.rs")
            })
            .map(|path| fs::read_to_string(path).expect("realization source should be readable"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(context_state.contains("resource_realization: ResourceRealizationState"));
        assert_eq!(
            context_state
                .matches("resource_realization: ResourceRealizationState")
                .count(),
            1,
            "the existing private context must own exactly one resource-realization state"
        );
        assert!(
            !context_descriptor.contains("GpuResourceRealizationPolicy"),
            "operational record policy must not enter adapter selection or retry identity"
        );
        assert!(public_realization.contains("max_records: NonZeroUsize"));
        assert!(!public_realization.contains(&["wgpu", "::"].concat()));
        assert!(!realization_source.contains("async fn"));
        assert!(!realization_source.contains("GpuLogicalLease"));
        assert!(!realization_source.contains("Weak<"));
        for forbidden_per_kind_quota in [
            "max_buffers",
            "max_textures",
            "max_texture_views",
            "max_samplers",
            "max_query_sets",
        ] {
            assert!(!public_realization.contains(forbidden_per_kind_quota));
            assert!(!realization_source.contains(forbidden_per_kind_quota));
        }

        for forbidden_transfer in [
            ["create_buffer", "_init"].concat(),
            ["queue", ".write_buffer"].concat(),
            ["queue", ".write_texture"].concat(),
            ["copy_buffer", "_to"].concat(),
            ["copy_texture", "_to"].concat(),
            ["map", "_async"].concat(),
            ["device", ".poll"].concat(),
        ] {
            assert!(
                !realization_creation_source.contains(&forbidden_transfer),
                "G4C1 object creation must not absorb G5 transfer/lifecycle authority: {forbidden_transfer}"
            );
        }
        for creation in [
            ["device", ".create_", "buffer(&BufferDescriptor"].concat(),
            ["device", ".create_", "texture(&TextureDescriptor"].concat(),
            [".create_", "view(&TextureViewDescriptor"].concat(),
            ["device", ".create_", "sampler(&SamplerDescriptor"].concat(),
            ["device", ".create_", "query_set(&QuerySetDescriptor"].concat(),
        ] {
            assert_eq!(
                realization_creation_source.matches(&creation).count(),
                1,
                "each accepted resource family must have one private creation terminal: {creation}"
            );
        }
    }

    #[test]
    fn gpu_g2_g3_retired_authority_and_forwarding_paths_are_absent() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source_root = manifest.join("src/plugins");
        let mut paths = Vec::new();
        rust_sources_below(&source_root, &mut paths);
        paths.sort();

        let retired = [
            ["RenderBackendCapability", "Profile"].concat(),
            ["RenderBackendCapability", "Inspection"].concat(),
            ["runtime", "_default"].concat(),
            ["unsupported", "_for_tests"].concat(),
            ["RenderResource", "Descriptor"].concat(),
            ["ImportedResource", "Kind"].concat(),
            ["Uniform", "Handle"].concat(),
            ["StorageArray", "Handle"].concat(),
            ["DoubleBuffer", "Handle"].concat(),
            ["CompiledResourceAccess", "Kind"].concat(),
            ["CompiledResourceLifetime", "Window"].concat(),
            ["compile_resource_lifetime", "_windows"].concat(),
            ["diagnose_resource_lifetime", "_windows"].concat(),
            ["GpuPrimitiveResource", "AccessKind"].concat(),
            ["GpuPrimitiveResource", "Access"].concat(),
            ["GpuPrimitiveDispatch", "Resource"].concat(),
            ["PassDependencyCycle", "Detected"].concat(),
            ["UnknownPass", "Dependency"].concat(),
        ];
        for path in paths {
            let source = fs::read_to_string(&path).expect("source file should be readable");
            for line in source.lines().filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//") && !trimmed.starts_with('*')
            }) {
                assert!(
                    retired.iter().all(|token| !line.contains(token)),
                    "retired G2 authority remains in {}: {}",
                    path.display(),
                    line
                );
            }
        }

        for retired_path in [
            "src/plugins/render/graph/capabilities.rs",
            "src/plugins/render/resource/descriptors.rs",
            "src/plugins/render/resource/import.rs",
            "src/plugins/render/resource/lifetime.rs",
            "src/plugins/render/api/resources.rs",
            "src/plugins/render/graph/resource_lifetimes.rs",
        ] {
            assert!(
                !manifest.join(retired_path).exists(),
                "retired forwarding or duplicate authority path remains: {retired_path}"
            );
        }
    }

    #[test]
    fn gpu_g2_render_lowering_cannot_restore_optional_descriptor_ambiguity() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let adapter = manifest.join("src/plugins/render/adapters/gpu_resources.rs");
        let source = fs::read_to_string(adapter).expect("GPU resource adapter should be readable");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let obsolete_optional_type = ["Option<", "GpuResourceDescriptor", ">"].concat();
        let obsolete_method = ["fngpu", "_descriptor("].concat();

        assert!(
            !compact.contains(&obsolete_optional_type),
            "render GPU lowering must use an explicit non-optional outcome"
        );
        assert!(
            !compact.contains(&obsolete_method),
            "the obsolete optional GPU descriptor lowering method must remain deleted"
        );
    }
}
