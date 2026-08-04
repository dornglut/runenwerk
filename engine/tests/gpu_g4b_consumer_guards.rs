use std::fs;
use std::path::{Path, PathBuf};

const EXECUTE_PASSES: &str = "src/plugins/render/renderer/render_flow/execute_passes.rs";
const PIPELINE_CACHE: &str = "src/plugins/render/renderer/pipeline_cache.rs";

#[test]
fn resolved_renderer_programs_admit_before_wgpu_shader_module_creation() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = read(&manifest_dir, EXECUTE_PASSES);
    let paths = [
        (
            "compute",
            "fn encode_compute_pass(",
            "fn encode_fullscreen_pass(",
        ),
        (
            "fullscreen",
            "fn encode_fullscreen_pass(",
            "fn encode_graphics_pass(",
        ),
        (
            "graphics",
            "fn encode_graphics_pass(",
            "fn encode_texture_copy(",
        ),
    ];

    for (label, start, end) in paths {
        let section = section(&source, start, end, EXECUTE_PASSES);
        let admission = unique_position(
            section,
            "admit_resolved_program_source(",
            label,
            "resolved source admission",
        );
        let module_creation = unique_position(
            section,
            ".get_or_create_shader_module(",
            label,
            "WGPU shader-module creation",
        );
        assert!(
            admission < module_creation,
            "{label} creates or obtains a WGPU shader module before admitting its exact resolved canonical WGSL source"
        );
        assert_eq!(
            section.matches("ShaderSource::Wgsl(").count(),
            1,
            "{label} must have exactly one current WGPU WGSL realization site"
        );
    }
}

#[test]
fn renderer_program_source_admission_has_one_retaining_gateway() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let execution = read(&manifest_dir, EXECUTE_PASSES);
    let cache = read(&manifest_dir, PIPELINE_CACHE);

    let execution_gateway = section(
        &execution,
        "fn admit_resolved_program_source(",
        "fn reject_material_shader_fallback(",
        EXECUTE_PASSES,
    );
    assert_eq!(
        execution_gateway
            .matches("cache.admit_program_source(")
            .count(),
        1,
        "resolved renderer source admission must delegate to the cache gateway exactly once"
    );
    assert!(
        !execution.contains("RendererProgramSourceAuthority::new("),
        "render execution must not create a parallel source authority"
    );
    assert!(
        !execution.contains(".admit_wgsl("),
        "render execution must not bypass the retaining cache gateway"
    );

    assert_eq!(
        cache.matches("pub(crate) fn admit_program_source(").count(),
        1,
        "the renderer cache must expose exactly one source-admission gateway"
    );
    let cache_gateway = section(
        &cache,
        "pub(crate) fn admit_program_source(",
        "pub fn get_or_create_shader_module<",
        PIPELINE_CACHE,
    );
    assert_eq!(
        cache_gateway.matches(".admit_and_retain_wgsl(").count(),
        1,
        "the renderer cache gateway must admit and retain through the one renderer authority"
    );
    assert_eq!(
        cache
            .matches("RendererProgramSourceAuthority::new(")
            .count(),
        1,
        "the renderer cache must construct exactly one source authority"
    );
    assert!(
        !cache.contains(".admit_wgsl("),
        "the renderer cache must not bypass renderer-lifetime source retention"
    );
}

#[test]
fn builtin_program_sources_use_the_same_retaining_gateway() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = read(&manifest_dir, PIPELINE_CACHE);
    let default_impl = section(
        &cache,
        "impl Default for FlowPipelineArtifactCache",
        "impl FlowPipelineArtifactCache",
        PIPELINE_CACHE,
    );
    assert_eq!(
        default_impl
            .matches("admit_builtin_program_source(")
            .count(),
        3,
        "renderer cache construction must admit compute, fullscreen, and graphics builtins"
    );

    let builtin_gateway = section(
        &cache,
        "fn admit_builtin_program_source(",
        "#[cfg(test)]",
        PIPELINE_CACHE,
    );
    assert_eq!(
        builtin_gateway.matches(".admit_program_source(").count(),
        1,
        "builtin sources must use the same retaining cache gateway as resolved sources"
    );
}

fn read(manifest_dir: &Path, relative: &str) -> String {
    fs::read_to_string(manifest_dir.join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

fn section<'a>(source: &'a str, start: &str, end: &str, path: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("{path} no longer contains start marker {start:?}"));
    let tail = &source[start_index..];
    let end_index = tail
        .find(end)
        .unwrap_or_else(|| panic!("{path} no longer contains end marker {end:?} after {start:?}"));
    &tail[..end_index]
}

fn unique_position(section: &str, token: &str, path: &str, role: &str) -> usize {
    assert_eq!(
        section.matches(token).count(),
        1,
        "{path} must contain exactly one {role} token {token:?}"
    );
    section
        .find(token)
        .expect("counted token must remain findable")
}
