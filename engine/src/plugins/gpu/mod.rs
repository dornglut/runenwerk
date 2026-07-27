//! Future-transferable RunenGPU contract boundaries.

pub mod api;

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
    fn gpu_g2_boundary_has_no_forbidden_dependencies_or_vocabulary() {
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
            ["Render", "Pass"].concat(),
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

        for path in paths {
            let source = fs::read_to_string(&path).expect("GPU boundary source should be readable");
            for line in source.lines().filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//") && !trimmed.starts_with('*')
            }) {
                assert!(
                    forbidden.iter().all(|token| !line.contains(token)),
                    "forbidden GPU boundary import in {}: {}",
                    path.display(),
                    line
                );
            }
        }
    }

    #[test]
    fn gpu_g2_retired_authority_and_forwarding_paths_are_absent() {
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
