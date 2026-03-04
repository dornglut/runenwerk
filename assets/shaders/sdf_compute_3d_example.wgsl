struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    agent_count : u32,
    model_count : u32,
    paused : u32,
    _pad3 : u32,
    camera_target_time : vec4<f32>,
    camera_orbit : vec4<f32>,
    debug_view_mode : u32,
    _pad4a : u32,
    _pad4b : u32,
    _pad4c : u32,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    _pad0 : vec3<u32>,
};

struct ModelProxy {
    pos : vec2<f32>,
    radius : f32,
    _pad0 : f32,
    color : vec4<f32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> agents : array<Agent>;
@group(0) @binding(3)
var<storage, read> models : array<ModelProxy>;

fn rot2(a: f32) -> mat2x2<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat2x2<f32>(
        vec2<f32>(c, -s),
        vec2<f32>(s, c),
    );
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

fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / max(k, 0.0001), 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn op_subtract(a: f32, b: f32) -> f32 {
    return max(a, -b);
}

fn op_intersect(a: f32, b: f32) -> f32 {
    return max(a, b);
}

fn scene_distance(p: vec3<f32>) -> f32 {
    let t = params.camera_target_time.w;

    var moving_sphere = p - vec3<f32>(sin(t * 0.9) * 1.5, 0.8 + sin(t * 1.6) * 0.2, cos(t * 0.8) * 1.2);
    let d_sphere = sd_sphere(moving_sphere, 1.0);

    var spinning_box = p - vec3<f32>(-2.2, 0.85, 0.4);
    let spinning_box_xz = rot2(t * 0.65) * spinning_box.xz;
    spinning_box.x = spinning_box_xz.x;
    spinning_box.z = spinning_box_xz.y;
    let d_box = sd_box(spinning_box, vec3<f32>(0.85, 0.85, 0.85));

    var torus = p - vec3<f32>(2.0, 1.05, -0.8);
    let torus_xy = rot2(t * 0.5) * torus.xy;
    torus.x = torus_xy.x;
    torus.y = torus_xy.y;
    let d_torus = sd_torus(torus, vec2<f32>(1.15, 0.3));

    var scene = op_smooth_union(d_sphere, d_box, 0.5);
    scene = min(scene, d_torus);

    let cut_sphere = sd_sphere(p - vec3<f32>(-2.2, 0.85, 0.4), 0.75);
    scene = op_subtract(scene, cut_sphere);

    let intersect_shape = sd_box(p - vec3<f32>(0.2, 1.05, 2.0), vec3<f32>(0.75, 0.75, 0.75));
    let intersect_sphere = sd_sphere(p - vec3<f32>(0.2, 1.05, 2.0), 1.0);
    scene = min(scene, op_intersect(intersect_shape, intersect_sphere));

    let ground = sd_plane(p, vec3<f32>(0.0, 1.0, 0.0), 0.0);
    return min(scene, ground);
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0015;
    let x = scene_distance(p + vec3<f32>(e, 0.0, 0.0)) - scene_distance(p - vec3<f32>(e, 0.0, 0.0));
    let y = scene_distance(p + vec3<f32>(0.0, e, 0.0)) - scene_distance(p - vec3<f32>(0.0, e, 0.0));
    let z = scene_distance(p + vec3<f32>(0.0, 0.0, e)) - scene_distance(p - vec3<f32>(0.0, 0.0, e));
    return normalize(vec3<f32>(x, y, z));
}

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>, max_dist: f32) -> f32 {
    var t = 0.02;
    var res = 1.0;
    for (var i = 0; i < 32; i = i + 1) {
        if (t >= max_dist) {
            break;
        }
        let h = scene_distance(ro + rd * t);
        if (h < 0.001) {
            return 0.0;
        }
        res = min(res, 12.0 * h / t);
        t = t + clamp(h, 0.02, 0.25);
    }
    return clamp(res, 0.0, 1.0);
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let h = clamp(rd.y * 0.5 + 0.5, 0.0, 1.0);
    return mix(vec3<f32>(0.03, 0.04, 0.06), vec3<f32>(0.16, 0.22, 0.30), h);
}

fn albedo_from_position(p: vec3<f32>) -> vec3<f32> {
    if (p.y < 0.02) {
        let checker = step(0.5, fract(p.x * 0.4) + fract(p.z * 0.4));
        let a = vec3<f32>(0.08, 0.10, 0.12);
        let b = vec3<f32>(0.13, 0.15, 0.18);
        return mix(a, b, checker);
    }
    if (p.x < -1.0) {
        return vec3<f32>(0.90, 0.32, 0.26);
    }
    if (p.x > 1.0) {
        return vec3<f32>(0.28, 0.78, 0.93);
    }
    return vec3<f32>(0.66, 0.74, 0.92);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= u32(params.screen_size.x) || y >= u32(params.screen_size.y)) {
        return;
    }

    let size = max(params.screen_size, vec2<f32>(1.0, 1.0));
    let frag = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
    let ndc = vec2<f32>(
        (frag.x / size.x) * 2.0 - 1.0,
        1.0 - (frag.y / size.y) * 2.0,
    );
    let aspect = size.x / max(size.y, 1.0);

    let yaw = params.camera_orbit.x;
    let pitch = params.camera_orbit.y;

    let forward = normalize(vec3<f32>(
        cos(pitch) * sin(yaw),
        -sin(pitch),
        cos(pitch) * cos(yaw),
    ));
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(forward, world_up));
    let up = normalize(cross(right, forward));

    let cam_target = params.camera_target_time.xyz;
    let ro = cam_target - forward * params.camera_orbit.z;

    let tan_half_fov = tan(params.camera_orbit.w * 0.5);
    let rd = normalize(forward + right * ndc.x * tan_half_fov * aspect + up * ndc.y * tan_half_fov);

    let max_steps = 144;
    let max_dist = 96.0;
    let hit_eps = 0.0012;

    var t = 0.0;
    var hit = false;
    var step_count = 0u;

    loop {
        if (step_count >= u32(max_steps)) {
            break;
        }
        let p = ro + rd * t;
        let d = scene_distance(p);
        if (d < hit_eps) {
            hit = true;
            break;
        }
        t = t + d;
        if (t > max_dist) {
            break;
        }
        step_count = step_count + 1u;
    }

    var color = sky_color(rd);

    if (hit) {
        let p = ro + rd * t;
        let n = calc_normal(p);
        let ldir = normalize(vec3<f32>(-0.55, 0.85, -0.35));
        let vdir = normalize(-rd);
        let hdir = normalize(ldir + vdir);

        let diff = max(dot(n, ldir), 0.0);
        let spec = pow(max(dot(n, hdir), 0.0), 48.0);
        let fres = pow(1.0 - max(dot(n, vdir), 0.0), 5.0);

        let shadow = soft_shadow(p + n * 0.005, ldir, 24.0);
        let ambient = 0.22;

        let base = albedo_from_position(p);
        let lit = base * (ambient + diff * 0.88 * shadow) + vec3<f32>(0.8, 0.9, 1.0) * spec * shadow * 0.35;
        let rim = vec3<f32>(0.4, 0.6, 0.9) * fres * 0.25;
        color = lit + rim;

        if (params.debug_view_mode == 1u) {
            let depth_norm = clamp(t / max_dist, 0.0, 1.0);
            color = vec3<f32>(depth_norm, depth_norm * depth_norm, 1.0 - depth_norm);
        } else if (params.debug_view_mode == 2u) {
            color = n * 0.5 + 0.5;
        } else if (params.debug_view_mode == 3u) {
            let s = clamp(f32(step_count) / f32(max_steps), 0.0, 1.0);
            color = mix(vec3<f32>(0.08, 0.2, 0.85), vec3<f32>(1.0, 0.25, 0.08), s);
        }
    }

    if (params.paused == 1u) {
        color = mix(color, vec3<f32>(0.75, 0.78, 0.82), 0.24);
    }

    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(0.4545));
    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
