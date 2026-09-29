struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct ComparisonParams {
    control: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> comparison: ComparisonParams;

@group(0) @binding(1)
var image_a: texture_2d<f32>;

@group(0) @binding(2)
var image_b: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let extent_a = textureDimensions(image_a);
    let extent_b = textureDimensions(image_b);
    let extent = min(extent_a, extent_b);
    let pixel = clamp(
        vec2<i32>(input.position.xy),
        vec2<i32>(0),
        vec2<i32>(extent) - vec2<i32>(1),
    );

    let a = textureLoad(image_a, pixel, 0);
    let b = textureLoad(image_b, pixel, 0);
    let mode = u32(round(comparison.control.x));

    if (mode == 1u) {
        return a;
    }
    if (mode == 2u) {
        return b;
    }

    let divider_x = clamp(comparison.control.y, 0.0, 1.0) * f32(extent.x);
    return select(b, a, input.position.x < divider_x);
}
