struct ComposeParams {
    time_data: vec4<f32>, // time, pulse, wave_a, wave_b
    surface: vec4<f32>, // width, height, inv_width, inv_height
    camera: vec4<f32>, // yaw, pitch, distance, fov_radians
    terrain: vec4<f32>, // base_height, height_scale, base_frequency, detail_gain
    sky: vec4<f32>, // turbidity, sun_elevation_weight, cloud_scale, cloud_speed
    view_data: vec4<u32>, // mode, _, _, _
    colors_a: vec4<f32>, // sky top
    colors_b: vec4<f32>, // sky horizon
    colors_c: vec4<f32>, // terrain lowland
    colors_d: vec4<f32>, // terrain highland
};

@group(0) @binding(0)
var<uniform> params: ComposeParams;

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

fn hash21(p: vec2<f32>) -> f32 {
    let q = vec2<f32>(
        dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3)),
    );
    return fract(sin(q.x + q.y) * 43758.5453123);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash21(i + vec2<f32>(0.0, 0.0));
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    let ab = mix(a, b, u.x);
    let cd = mix(c, d, u.x);
    return mix(ab, cd, u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var sum = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var q = p;
    for (var octave = 0; octave < 5; octave = octave + 1) {
        sum = sum + noise2(q * freq) * amp;
        freq = freq * 2.03;
        amp = amp * 0.5;
        q = q + vec2<f32>(13.7, 9.1);
    }
    return sum;
}

fn terrain_height(xz: vec2<f32>) -> f32 {
    let t = params.time_data.x;
    let freq = max(params.terrain.z, 0.001);

    let warp = fbm(xz * (freq * 0.35) + vec2<f32>(t * 0.030, -t * 0.021));
    let ridge_source = fbm(xz * freq + vec2<f32>(warp * 2.6, -warp * 1.9));
    let ridged = 1.0 - abs(ridge_source * 2.0 - 1.0);
    let detail = fbm(
        xz * freq * 2.7 + vec2<f32>(17.3, -4.6) + vec2<f32>(t * 0.080, t * 0.050),
    );
    let shape = mix(ridged, detail, 0.35);
    return params.terrain.x + shape * params.terrain.y + detail * params.terrain.w;
}

fn terrain_sdf(p: vec3<f32>) -> f32 {
    return p.y - terrain_height(p.xz);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.012;
    let x = terrain_sdf(p + vec3<f32>(e, 0.0, 0.0)) - terrain_sdf(p - vec3<f32>(e, 0.0, 0.0));
    let y = terrain_sdf(p + vec3<f32>(0.0, e, 0.0)) - terrain_sdf(p - vec3<f32>(0.0, e, 0.0));
    let z = terrain_sdf(p + vec3<f32>(0.0, 0.0, e)) - terrain_sdf(p - vec3<f32>(0.0, 0.0, e));
    return normalize(vec3<f32>(x, y, z));
}

fn sun_dir() -> vec3<f32> {
    let elevation = clamp(params.sky.y, 0.12, 0.92);
    return normalize(vec3<f32>(0.44, elevation, -0.56));
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let h = clamp(rd.y * 0.5 + 0.5, 0.0, 1.0);
    var color = mix(params.colors_b.rgb, params.colors_a.rgb, pow(h, 1.35));

    let sun = sun_dir();
    let sun_dot = max(dot(rd, sun), 0.0);
    let sun_core = pow(sun_dot, 820.0);
    let sun_halo = pow(sun_dot, 24.0);
    color = color
        + vec3<f32>(1.0, 0.92, 0.78) * sun_core * 2.5
        + vec3<f32>(1.0, 0.66, 0.44) * sun_halo * 0.24;

    let cloud_uv = rd.xz / max(rd.y + 0.16, 0.12) * params.sky.z * 0.60
        + vec2<f32>(
            params.time_data.x * params.sky.w * 0.035,
            -params.time_data.x * params.sky.w * 0.022,
        );
    let cloud_noise = fbm(cloud_uv);
    let cloud = smoothstep(0.56, 0.84, cloud_noise);
    let cloud_weight = cloud * clamp(rd.y * 1.9 + 0.55, 0.0, 1.0) * 0.48;
    let cloud_tint = mix(vec3<f32>(0.93, 0.94, 0.96), vec3<f32>(1.0, 0.68, 0.50), sun_halo);
    color = mix(color, cloud_tint, cloud_weight);

    let haze = clamp((1.0 - h) * params.sky.x * 0.45, 0.0, 0.5);
    color = color + vec3<f32>(0.22, 0.14, 0.10) * haze;
    return color;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let view_mode = params.view_data.x;
    let size = max(params.surface.xy, vec2<f32>(1.0, 1.0));
    let aspect = size.x / size.y;

    let ndc = vec2<f32>(
        input.uv.x * 2.0 - 1.0,
        1.0 - input.uv.y * 2.0,
    );

    let yaw = params.camera.x;
    let pitch = params.camera.y;
    let orbit_distance = max(params.camera.z, 2.0);
    let fov = max(params.camera.w, 0.3);

    let focus = vec3<f32>(0.0, 0.6, 0.0);
    let ro = focus + vec3<f32>(
        sin(yaw) * orbit_distance,
        2.0 + sin(pitch) * orbit_distance * 0.62,
        cos(yaw) * orbit_distance,
    );

    let forward = normalize(focus - ro);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = normalize(cross(right, forward));
    let tan_half_fov = tan(fov * 0.5);
    let rd = normalize(
        forward + right * ndc.x * tan_half_fov * aspect + up * ndc.y * tan_half_fov,
    );

    let max_steps = 168u;
    let max_distance = 160.0;
    let hit_epsilon = 0.003;

    var t = 0.0;
    var hit = false;
    var step_count = 0u;
    loop {
        if (step_count >= max_steps || t > max_distance) {
            break;
        }
        let p = ro + rd * t;
        let dist = terrain_sdf(p);
        if (dist < hit_epsilon) {
            hit = true;
            break;
        }
        t = t + clamp(max(dist, 0.0) * 0.85, 0.02, 1.6);
        step_count = step_count + 1u;
    }

    var color = sky_color(rd);
    var hit_normal = vec3<f32>(0.0, 1.0, 0.0);
    var height_vis = 0.0;
    var depth01 = 1.0;

    if (hit) {
        let p = ro + rd * t;
        let n = calc_normal(p);
        hit_normal = n;

        let sun = sun_dir();
        let diff = max(dot(n, sun), 0.0);
        let view_dir = normalize(-rd);
        let half_dir = normalize(sun + view_dir);
        let spec = pow(max(dot(n, half_dir), 0.0), 48.0);
        let fres = pow(1.0 - max(dot(n, view_dir), 0.0), 4.2);

        let up_amount = clamp(dot(n, vec3<f32>(0.0, 1.0, 0.0)), 0.0, 1.0);
        let slope = 1.0 - up_amount;
        let h = terrain_height(p.xz);
        let h_norm = clamp((h - params.terrain.x) / max(params.terrain.y * 1.6, 0.01), 0.0, 1.0);
        height_vis = h_norm;

        var base = mix(params.colors_c.rgb, params.colors_d.rgb, h_norm);
        let surface_noise = fbm(p.xz * 0.9 + vec2<f32>(params.time_data.x * 0.03, 0.0));
        base = base * (0.84 + surface_noise * 0.28);
        base = mix(base, base * 0.56, slope * 0.48);

        let ambient = 0.20 + up_amount * 0.25;
        let lit = base * (ambient + diff * 0.96)
            + vec3<f32>(1.0, 0.94, 0.86) * spec * 0.36
            + vec3<f32>(0.18, 0.32, 0.50) * fres * (0.20 + params.time_data.y * 0.12);

        let fog_start = 18.0;
        let fog_end = 145.0;
        let fog_t = clamp((t - fog_start) / (fog_end - fog_start), 0.0, 1.0);
        let fog = 1.0 - exp(-fog_t * (0.6 + params.sky.x * 0.9));
        color = mix(lit, sky_color(rd), fog);
        depth01 = clamp(t / max_distance, 0.0, 1.0);
    }

    let step_visual = clamp(f32(step_count) / f32(max_steps), 0.0, 1.0);
    if (view_mode == 1u) {
        let terrain_height_color = vec3<f32>(
            height_vis * 0.75 + 0.10,
            height_vis,
            1.0 - height_vis * 0.65,
        );
        color = select(vec3<f32>(0.0), terrain_height_color, hit);
    } else if (view_mode == 2u) {
        let normal_color = hit_normal * 0.5 + vec3<f32>(0.5);
        color = select(sky_color(rd), normal_color, hit);
    } else if (view_mode == 3u) {
        let miss_tint = vec3<f32>(0.02, 0.02, 0.05);
        let hit_tint = vec3<f32>(step_visual, step_visual * step_visual, 1.0 - step_visual);
        color = select(miss_tint, hit_tint, hit);
    } else {
        let distance_veil = (1.0 - depth01) * 0.08;
        color = color + vec3<f32>(0.02, 0.03, 0.04) * distance_veil;
    }

    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(0.4545));
    return vec4<f32>(color, 1.0);
}
