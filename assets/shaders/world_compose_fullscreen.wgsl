@group(0) @binding(0)
var world_tex : texture_2d<f32>;

@group(0) @binding(1)
var world_sampler : sampler;

struct ComposeParams {
    output_size : vec2<f32>,
    target_aspect : f32,
    fit_mode : u32,
    bar_color : vec4<f32>,
};

@group(0) @binding(2)
var<uniform> compose : ComposeParams;

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    let p = pos[vertex_index];
    var out: VsOut;
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return out;
}

fn remap_uv_contain(uv: vec2<f32>, output_aspect: f32, target_aspect: f32) -> vec3<f32> {
    var mapped = uv;
    if (output_aspect > target_aspect) {
        let width = target_aspect / output_aspect;
        mapped.x = (uv.x - 0.5) / max(width, 0.0001) + 0.5;
    } else {
        let height = output_aspect / target_aspect;
        mapped.y = (uv.y - 0.5) / max(height, 0.0001) + 0.5;
    }
    let in_bounds = select(0.0, 1.0, all(mapped >= vec2<f32>(0.0)) && all(mapped <= vec2<f32>(1.0)));
    return vec3<f32>(mapped, in_bounds);
}

fn remap_uv_cover(uv: vec2<f32>, output_aspect: f32, target_aspect: f32) -> vec2<f32> {
    var mapped = uv;
    if (output_aspect > target_aspect) {
        let height = target_aspect / output_aspect;
        mapped.y = (uv.y - 0.5) * height + 0.5;
    } else {
        let width = output_aspect / target_aspect;
        mapped.x = (uv.x - 0.5) * width + 0.5;
    }
    return mapped;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let output_aspect = compose.output_size.x / max(compose.output_size.y, 1.0);
    let target_aspect = select(output_aspect, compose.target_aspect, compose.target_aspect > 0.0001);
    let mode = compose.fit_mode;

    if (mode == 0u) {
        return textureSample(world_tex, world_sampler, input.uv);
    }

    if (mode == 1u) {
        let remap = remap_uv_contain(input.uv, output_aspect, target_aspect);
        if (remap.z < 0.5) {
            return compose.bar_color;
        }
        return textureSample(world_tex, world_sampler, remap.xy);
    }

    if (mode == 2u) {
        let uv = clamp(remap_uv_cover(input.uv, output_aspect, target_aspect), vec2<f32>(0.0), vec2<f32>(1.0));
        return textureSample(world_tex, world_sampler, uv);
    }

    if (mode == 3u) {
        let scale = output_aspect / max(target_aspect, 0.0001);
        let uv = vec2<f32>((input.uv.x - 0.5) * scale + 0.5, input.uv.y);
        if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
            return compose.bar_color;
        }
        return textureSample(world_tex, world_sampler, uv);
    }

    let scale = target_aspect / max(output_aspect, 0.0001);
    let uv = vec2<f32>(input.uv.x, (input.uv.y - 0.5) * scale + 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
        return compose.bar_color;
    }
    return textureSample(world_tex, world_sampler, uv);
}
