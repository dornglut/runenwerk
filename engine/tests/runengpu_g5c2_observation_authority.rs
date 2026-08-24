use std::fs;
use std::path::{Path, PathBuf};

fn rust_sources_below(root: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            rust_sources_below(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
}

fn joined_sources(root: &Path) -> String {
    let mut paths = Vec::new();
    rust_sources_below(root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("Rust source should be readable"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn gfx_owns_one_nonblocking_renderer_progress_authority() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let renderer = manifest.join("src/plugins/render/renderer");
    let source = joined_sources(&renderer);

    assert_eq!(
        source.matches("context.progress();").count(),
        1,
        "Gfx must retain exactly one context progress point for renderer timing/capture"
    );
    for forbidden in [
        "device.poll(",
        "PollType::Wait",
        "pollster::block_on(context.progress",
        "std::thread::spawn",
        "tokio::",
    ] {
        assert!(
            !source.contains(forbidden),
            "renderer progress authority contains forbidden backend/blocking path: {forbidden}"
        );
    }
}

#[test]
fn renderer_observation_retains_semantics_not_execution_authority() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let observation =
        fs::read_to_string(manifest.join("src/plugins/render/renderer/render_flow/observation.rs"))
            .expect("renderer observation source should be readable");
    let timing =
        fs::read_to_string(manifest.join("src/plugins/render/renderer/render_flow/gpu_timing.rs"))
            .expect("renderer timing source should be readable");

    assert!(observation.contains("submission: GpuSubmission"));
    assert!(observation.contains("readback_id: GpuReadbackId"));
    assert!(timing.contains("timestamp_period_ns: f32"));
    for forbidden in [
        "GpuWorkOperation",
        "GpuPreparedWorkGraph",
        "GpuReadbackOperation",
        "GpuResourceAccess",
        "wgpu::",
        "CommandEncoder",
        "CommandBuffer",
        "mapped_range",
    ] {
        assert!(
            !observation.contains(forbidden),
            "renderer observation retained generic execution authority: {forbidden}"
        );
    }
}

#[test]
fn final_g5c_renderer_source_has_no_raw_execution_or_prepared_materialization_path() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_root = manifest.join("src/plugins/render");
    let source = joined_sources(&render_root);

    for retired in [
        "CurrentRenderDeviceQueue",
        "current_render_device_queue(",
        "CurrentRenderExecutionBridge",
        "current_render_execution_bridge(",
        "CurrentHostSurfaceBridge",
        "current_host_surface_bridge(",
        "request_for_current_host(",
    ] {
        assert!(
            !source.contains(retired),
            "retired G5C bridge definition/call remains in renderer production source: {retired}"
        );
    }

    for raw_authority in [
        "wgpu::Device",
        "wgpu::Queue",
        "wgpu::CommandEncoder",
        "wgpu::CommandBuffer",
        "wgpu::Surface<",
        "wgpu::SurfaceTexture",
        "create_command_encoder(",
        "queue.submit(",
        "get_mapped_range(",
        "get_timestamp_period(",
    ] {
        assert!(
            !source.contains(raw_authority),
            "renderer production source regained raw WGPU execution authority: {raw_authority}"
        );
    }

    assert!(
        !source.contains("GpuBufferInitialization::Prepared")
            && !source.contains("GpuTextureInitialization::Prepared"),
        "live renderer path must not treat Prepared descriptor metadata as physical bytes"
    );
    assert!(
        source.contains("GpuBufferInitialization::Uninitialized")
            && source.contains("GpuUploadOperation"),
        "renderer physical buffer contents must remain explicit canonical Upload work"
    );
}
