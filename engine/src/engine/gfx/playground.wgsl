struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VSOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    let pos = positions[vertex_index];

    let uv = pos.xy * 0.5 + vec2<f32>(0.5, 0.5);

    return VSOut(vec4<f32>(pos, 0.0, 1.0), uv);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let p = uv * 2.0 - vec2<f32>(1.0);
    return vec4<f32>(p.x * 0.5 + 0.5, p.y * 0.5 + 0.5, 0.5, 1.0);
}
