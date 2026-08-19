use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn residual_copy_format_compatibility_delegates_to_runengpu() {
    let source = read("src/plugins/render/renderer/render_flow/execute_passes.rs");
    let start = source
        .find("fn copy_formats_are_raw_compatible(")
        .expect("residual copy-format adapter should remain explicit until G5C");
    let tail = &source[start..];
    let end = tail.find("#[cfg(test)]").unwrap_or(tail.len());
    let adapter = &tail[..end];

    assert!(
        adapter.contains("gpu_texture_formats_copy_compatible("),
        "residual renderer copy compatibility must delegate to RunenGPU's canonical relation"
    );
    assert!(
        !adapter.contains("remove_srgb_suffix"),
        "residual renderer copy compatibility must not recreate a WGPU-owned format relation"
    );
}

#[test]
fn fixed_step_iteration_uniforms_are_not_generic_gpu_uploads() {
    let source = read("src/plugins/render/renderer/render_flow/execute.rs");
    let generic_start = source
        .find("fn realize_projected_uniform_uploads(")
        .expect("generic projected-uniform realization should exist");
    let occurrence_offset = source[generic_start..]
        .find("fn realize_fixed_step_iteration_upload(")
        .expect("fixed-step occurrence-local upload realization should exist");
    let occurrence_start = generic_start + occurrence_offset;
    let generic = &source[generic_start..occurrence_start];

    let filter = generic
        .find("region.iteration_uniform == *buffer_id")
        .expect("generic uniform realization must identify fixed-step iteration uniforms");
    let skip = generic[filter..]
        .find("continue;")
        .map(|offset| filter + offset)
        .expect("fixed-step iteration uniforms must be skipped by generic realization");
    let prepare = generic
        .find("prepare_uniform_upload")
        .expect("generic uniform realization should still prepare ordinary uniforms");

    assert!(
        filter < skip && skip < prepare,
        "fixed-step iteration uniforms must be rejected from the generic GPU-upload path before physical upload preparation"
    );

    let occurrence_local = &source[occurrence_start..];
    assert!(
        occurrence_local.contains("region.iteration_uniform")
            && occurrence_local.contains("project_buffer_upload("),
        "actual fixed-step occurrences must retain their occurrence-local canonical upload path"
    );
}

#[test]
fn execution_limits_consume_positional_binding_and_vertex_slots() {
    let bindings = read("src/plugins/gpu/api/program/runtime_binding/set.rs");
    assert!(
        bindings.contains("required_bind_group_slots > u64::from(device_facts.max_bind_groups())"),
        "complete runtime bindings must admit the highest positional bind-group slot before backend realization"
    );
    assert!(
        bindings.contains("u64::from(group.group()) + 1"),
        "sparse logical group indices must count the positional slots that private realization requires"
    );

    let render = read("src/plugins/gpu/api/render_execution.rs");
    assert!(
        render.contains("u64::from(binding.slot()) + 1")
            && render.contains("limits.max_vertex_buffers()")
            && render.contains("bindings.required_bind_group_slots()")
            && render.contains("limits.max_bind_groups_plus_vertex_buffers()"),
        "render limit admission must consume positional vertex-buffer and bind-group slots"
    );
    assert!(
        !render.contains("vertex_buffers.len() + bindings.groups().len()"),
        "render combined-limit admission must not regress to declared cardinality for sparse slots"
    );
}
