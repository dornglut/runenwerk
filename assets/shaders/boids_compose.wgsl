struct BoidAgent {
    position: vec2<f32>,
    velocity: vec2<f32>,
};

struct ComposeParams {
    frame_meta: vec4<u32>, // boid_count, current_is_a, _, _
    surface: vec4<f32>, // width, height, inv_width, inv_height
    draw: vec4<f32>, // body_radius, glow_radius, tail_strength, _
    background: vec4<f32>,
    boid_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> params: ComposeParams;

@group(0) @binding(1)
var<storage, read> boids_a: array<BoidAgent>;

@group(0) @binding(2)
var<storage, read> boids_b: array<BoidAgent>;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

fn wrap_delta(delta: vec2<f32>) -> vec2<f32> {
    return delta - round(delta);
}

fn normalize_or_zero(value: vec2<f32>) -> vec2<f32> {
    let len = length(value);
    if (len <= 0.000001) {
        return vec2<f32>(0.0, 0.0);
    }
    return value / len;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let boid_count = max(params.frame_meta.x, 1u);
    let use_a = params.frame_meta.y != 0u;
    let aspect = max(params.surface.x * params.surface.w, 0.0001);

    let body_radius = max(params.draw.x, 0.001);
    let glow_radius = max(params.draw.y, body_radius * 1.5);
    let tail_strength = max(params.draw.z, 0.0);

    var min_dist = 10.0;
    var glow = 0.0;

    for (var i: u32 = 0u; i < boid_count; i = i + 1u) {
        var boid = boids_b[i];
        if (use_a) {
            boid = boids_a[i];
        }
        let delta = wrap_delta(input.uv - boid.position);
        let delta_aspect = vec2<f32>(delta.x * aspect, delta.y);
        let dist = length(delta_aspect);
        min_dist = min(min_dist, dist);

        let glow_density = exp(-dist * (10.0 / glow_radius));
        glow = glow + glow_density * 0.014;

        let dir = normalize_or_zero(boid.velocity);
        let tail_anchor = boid.position - dir * body_radius * 2.2;
        let tail_delta = wrap_delta(input.uv - tail_anchor);
        let tail_dist = length(vec2<f32>(tail_delta.x * aspect, tail_delta.y));
        glow = glow + exp(-tail_dist * 48.0) * tail_strength * 0.004;
    }

    let body = 1.0 - smoothstep(body_radius, body_radius * 1.8, min_dist);
    let halo = clamp(glow, 0.0, 1.0);
    let blend = clamp(body + halo * 0.65, 0.0, 1.0);

    let color = mix(params.background.rgb, params.boid_color.rgb, blend);
    return vec4<f32>(color, 1.0);
}
