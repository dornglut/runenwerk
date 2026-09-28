struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var radiance: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = vec2<f32>(
        positions[vertex_index].x * 0.5 + 0.5,
        0.5 - positions[vertex_index].y * 0.5,
    );
    return output;
}

fn pressure_signal(uv: vec2<f32>) -> vec3<f32> {
    // Reference-image coordinates intentionally remain fixed across every internal extent.
    let ref_px = uv * vec2<f32>(1920.0, 1080.0);

    let checker_cell = vec2<i32>(floor(ref_px / 3.0));
    let checker = f32((checker_cell.x + checker_cell.y) & 1);

    let diagonal_distance = abs(ref_px.x - (0.72 * ref_px.y + 180.0));
    let diagonal = select(0.0, 1.0, diagonal_distance < 0.75);

    let circle_distance = abs(distance(ref_px, vec2<f32>(1470.0, 720.0)) - 165.0);
    let circle = select(0.0, 1.0, circle_distance < 0.75);

    let vertical_band = select(
        0.0,
        1.0,
        uv.x > 0.56 && uv.x < 0.95 && uv.y > 0.06 && uv.y < 0.42,
    );
    let line_band = max(diagonal, circle);

    let pattern = max(checker * vertical_band, line_band);
    return vec3<f32>(pattern, pattern, pattern);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let extent = vec2<i32>(textureDimensions(radiance));
    let pixel = clamp(
        vec2<i32>(input.uv * vec2<f32>(extent)),
        vec2<i32>(0),
        extent - vec2<i32>(1),
    );
    let value = textureLoad(radiance, pixel, 0).x;
    let display = clamp(value * 0.25, 0.0, 1.0);
    let scene = vec3<f32>(display, display, display);
    let pressure = pressure_signal(input.uv);
    let pressure_region = select(
        0.0,
        1.0,
        (input.uv.x > 0.56 && input.uv.x < 0.95 && input.uv.y > 0.06 && input.uv.y < 0.42)
            || pressure.x > 0.0,
    );
    let color = mix(scene, pressure, 0.9 * pressure_region);
    return vec4<f32>(color, 1.0);
}
