// Native G5 simulation proof: the presented pixel exposes exact retained GPU state.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var result: VertexOutput;
    result.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return result;
}

@group(0) @binding(0) var<storage, read> counter: array<u32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    if (counter[0] == 6u) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    }
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
