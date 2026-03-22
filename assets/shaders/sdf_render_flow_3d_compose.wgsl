struct ComposeParams {
    time_data: vec4<f32>, // time, pulse, wave_a, wave_b
    surface: vec4<f32>, // width, height, inv_width, inv_height
    camera: vec4<f32>, // yaw, pitch, distance, fov_radians
    fog: vec4<f32>, // density, near, far, _
    color_a: vec4<f32>,
    color_b: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> params: ComposeParams;

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct SurfaceHit {
    dist: f32,
    material: f32,
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

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
    let q = vec2<f32>(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

fn sd_plane(p: vec3<f32>, n: vec3<f32>, h: f32) -> f32 {
    return dot(p, n) + h;
}

fn rot2(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / max(k, 0.0001), 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn scene_distance(p: vec3<f32>) -> SurfaceHit {
    let t = params.time_data.x;

    var sphere_p = p - vec3<f32>(sin(t * 0.7) * 1.4, 0.9, cos(t * 0.8) * 1.0);
    let d_sphere = sd_sphere(sphere_p, 0.95);

    var box_p = p - vec3<f32>(-1.8, 0.85, 0.2);
    box_p.xz = rot2(box_p.xz, t * 0.52);
    let d_box = sd_box(box_p, vec3<f32>(0.75, 0.75, 0.75));

    var torus_p = p - vec3<f32>(1.7, 1.0, -0.6);
    torus_p.xy = rot2(torus_p.xy, t * 0.44);
    let d_torus = sd_torus(torus_p, vec2<f32>(1.05, 0.24));

    let shape = op_smooth_union(d_sphere, d_box, 0.45);
    let cluster = min(shape, d_torus);
    let ground = sd_plane(p, vec3<f32>(0.0, 1.0, 0.0), 0.0);

    let final_dist = min(cluster, ground);
    let material = select(1.0, 0.0, ground < cluster);
    return SurfaceHit(final_dist, material);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0016;
    let x = scene_distance(p + vec3<f32>(e, 0.0, 0.0)).dist - scene_distance(p - vec3<f32>(e, 0.0, 0.0)).dist;
    let y = scene_distance(p + vec3<f32>(0.0, e, 0.0)).dist - scene_distance(p - vec3<f32>(0.0, e, 0.0)).dist;
    let z = scene_distance(p + vec3<f32>(0.0, 0.0, e)).dist - scene_distance(p - vec3<f32>(0.0, 0.0, e)).dist;
    return normalize(vec3<f32>(x, y, z));
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let h = clamp(rd.y * 0.5 + 0.5, 0.0, 1.0);
    return mix(vec3<f32>(0.020, 0.028, 0.045), vec3<f32>(0.08, 0.14, 0.22), h);
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let size = max(params.surface.xy, vec2<f32>(1.0, 1.0));
    let aspect = size.x / size.y;

    let ndc = vec2<f32>(
        input.uv.x * 2.0 - 1.0,
        1.0 - input.uv.y * 2.0,
    );

    let yaw = params.camera.x;
    let pitch = params.camera.y;
    let orbit_distance = max(params.camera.z, 0.5);
    let fov = max(params.camera.w, 0.3);

    let target = vec3<f32>(0.0, 0.8, 0.0);
    let ro = target + vec3<f32>(
        sin(yaw) * orbit_distance,
        sin(pitch) * orbit_distance * 0.85 + 0.6,
        cos(yaw) * orbit_distance,
    );

    let forward = normalize(target - ro);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = normalize(cross(right, forward));

    let tan_half_fov = tan(fov * 0.5);
    let rd = normalize(
        forward + right * ndc.x * tan_half_fov * aspect + up * ndc.y * tan_half_fov
    );

    let max_steps = 148u;
    let max_distance = 80.0;
    let hit_epsilon = 0.0012;

    var t = 0.0;
    var hit = false;
    var step_count = 0u;
    var hit_material = 0.0;

    loop {
        if (step_count >= max_steps || t > max_distance) {
            break;
        }
        let sample_pos = ro + rd * t;
        let surface = scene_distance(sample_pos);
        if (surface.dist < hit_epsilon) {
            hit = true;
            hit_material = surface.material;
            break;
        }
        t = t + clamp(surface.dist, 0.01, 0.65);
        step_count = step_count + 1u;
    }

    var color = sky_color(rd);

    if (hit) {
        let p = ro + rd * t;
        let n = calc_normal(p);
        let light_dir = normalize(vec3<f32>(-0.50, 0.85, -0.40));
        let view_dir = normalize(-rd);
        let half_dir = normalize(light_dir + view_dir);

        let diff = max(dot(n, light_dir), 0.0);
        let spec = pow(max(dot(n, half_dir), 0.0), 42.0);
        let fres = pow(1.0 - max(dot(n, view_dir), 0.0), 5.0);

        let pulse = params.time_data.y;
        let wave = 0.5 + 0.5 * sin(p.x * 2.4 + p.z * 1.9 + params.time_data.x * 0.8);
        let shape_color = mix(params.color_a.rgb, params.color_b.rgb, wave);
        let ground_a = vec3<f32>(0.06, 0.08, 0.11);
        let ground_b = vec3<f32>(0.12, 0.16, 0.20);
        let checker = step(0.5, fract(p.x * 0.35) + fract(p.z * 0.35));
        let ground_color = mix(ground_a, ground_b, checker);

        let base = select(shape_color, ground_color, hit_material < 0.5);
        let ambient = select(0.21, 0.14, hit_material < 0.5);
        let lit = base * (ambient + diff * 0.86) + vec3<f32>(0.90, 0.95, 1.0) * spec * 0.32;
        let rim = vec3<f32>(0.30, 0.48, 0.80) * fres * (0.20 + pulse * 0.12);
        color = lit + rim;

        let fog_density = max(params.fog.x, 0.0);
        let fog_near = max(params.fog.y, 0.0);
        let fog_far = max(params.fog.z, fog_near + 0.001);
        let fog_t = clamp((t - fog_near) / (fog_far - fog_near), 0.0, 1.0);
        let fog_factor = 1.0 - exp(-fog_t * fog_density * 24.0);
        color = mix(color, sky_color(rd), fog_factor);
    }

    let step_visual = clamp(f32(step_count) / f32(max_steps), 0.0, 1.0);
    color = color + vec3<f32>(0.03, 0.02, 0.04) * step_visual * 0.25;
    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(0.4545));
    return vec4<f32>(color, 1.0);
}
