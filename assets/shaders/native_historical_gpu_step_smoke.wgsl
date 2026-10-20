// Native RenderFlow #1291: actual per-step uniforms and retained mutation.
struct HistoricalParams {
    values: vec4<u32>,
};

struct HistoricalIteration {
    substep_index: u32,
    submitted_substeps: u32,
    max_substeps: u32,
    saturated_frames_low: u32,
    fixed_dt_seconds: f32,
    accumulator_seconds: f32,
    reserved0: f32,
    reserved1: f32,
};

@group(0) @binding(0) var<uniform> params: HistoricalParams;
@group(0) @binding(1) var<storage, read_write> counter: array<u32>;
@group(0) @binding(2) var<uniform> iteration: HistoricalIteration;

@compute @workgroup_size(1)
fn cs_main() {
    // A specific captured input and source-time fixed-step control affect
    // the actual GPU result; the current ECS uniform must never substitute.
    if (iteration.substep_index < iteration.submitted_substeps) {
        counter[0] = counter[0] + params.values.x;
    }
}
