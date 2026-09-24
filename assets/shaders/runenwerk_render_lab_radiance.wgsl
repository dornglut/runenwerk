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

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let extent = vec2<i32>(textureDimensions(radiance));
    let pixel = clamp(vec2<i32>(input.uv * vec2<f32>(extent)), vec2<i32>(0), extent - vec2<i32>(1));
    let value = textureLoad(radiance, pixel, 0).x;
    let display = clamp(value * 0.25, 0.0, 1.0);
    return vec4<f32>(display, display, display, 1.0);
}
