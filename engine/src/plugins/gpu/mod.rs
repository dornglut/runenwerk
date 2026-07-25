//! Future-transferable RunenGPU contract boundaries.

pub mod api;

pub use api::{GpuWorkResourceId, GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator};

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn gpu_identity_boundary_has_no_forbidden_dependencies() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plugins/gpu");
        let forbidden = [
            "crate::plugins::render",
            "crate::plugins::ui",
            "ecs::",
            "wgpu::",
            "winit::",
            "runen_sdf",
            "runenui",
            "editor_",
            "world_sdf",
            "product::",
        ];

        for path in [
            root.join("mod.rs"),
            root.join("api/mod.rs"),
            root.join("api/work_resource_id.rs"),
        ] {
            let source = fs::read_to_string(&path).expect("GPU boundary source should be readable");
            for line in source
                .lines()
                .filter(|line| line.trim_start().starts_with("use "))
            {
                assert!(
                    forbidden.iter().all(|token| !line.contains(token)),
                    "forbidden GPU boundary import in {}: {}",
                    path.display(),
                    line
                );
            }
        }
    }
}
