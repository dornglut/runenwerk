struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    room_count : u32,
    tunnel_count : u32,
    agent_count : u32,
    _pad3 : u32,
    floor_rock_height : vec4<f32>,
    camera_target_time : vec4<f32>,
    camera_orbit : vec4<f32>,
};

struct Room {
    center : vec2<f32>,
    radii : vec2<f32>,
    role : u32,
    _pad0 : vec3<u32>,
};

struct Tunnel {
    start : vec2<f32>,
    end : vec2<f32>,
    radius : f32,
    _pad0 : f32,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    kind : u32,
    _pad0 : vec2<u32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> rooms : array<Room>;
@group(0) @binding(3)
var<storage, read> tunnels : array<Tunnel>;
@group(0) @binding(4)
var<storage, read> agents : array<Agent>;

fn sd_box2(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
}

fn sd_ellipse2(p: vec2<f32>, r: vec2<f32>) -> f32 {
    let q = p / max(r, vec2<f32>(0.001, 0.001));
    return (length(q) - 1.0) * min(r.x, r.y);
}

fn sd_capsule2(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 0.0001), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / max(k, 0.0001), 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn sd_extruded(d2: f32, y: f32, half_h: f32) -> f32 {
    let w = vec2<f32>(d2, abs(y) - half_h);
    return min(max(w.x, w.y), 0.0) + length(max(w, vec2<f32>(0.0)));
}

fn cave_footprint(p: vec2<f32>) -> f32 {
    var d = 1e9;
    for (var i = 0u; i < params.room_count; i = i + 1u) {
        d = min(d, sd_ellipse2(p - rooms[i].center, rooms[i].radii));
    }
    for (var i = 0u; i < params.tunnel_count; i = i + 1u) {
        d = min(d, sd_capsule2(p, tunnels[i].start, tunnels[i].end, tunnels[i].radius));
    }
    return d;
}

fn rock_distance(p: vec3<f32>) -> f32 {
    let world_center = (params.world_min + params.world_max) * 0.5;
    let world_half = (params.world_max - params.world_min) * 0.5;
    let world_sdf = sd_box2(p.xz - world_center, world_half);
    let cave_sdf = cave_footprint(p.xz);
    let rock_footprint = max(world_sdf, -cave_sdf);
    let rock_height = max(params.floor_rock_height.y, 0.5);
    return sd_extruded(rock_footprint, p.y - rock_height * 0.5, rock_height * 0.5);
}

fn floor_distance(p: vec3<f32>) -> f32 {
    let floor_half_h = 0.04;
    let cave_sdf = cave_footprint(p.xz);
    return sd_extruded(cave_sdf, p.y - params.floor_rock_height.x, floor_half_h);
}

fn scene_distance(p: vec3<f32>) -> f32 {
    var scene = min(rock_distance(p), floor_distance(p));
    for (var i = 0u; i < params.agent_count; i = i + 1u) {
        let h = select(0.45, 0.18, agents[i].kind == 5u || agents[i].kind == 6u);
        let d = sd_sphere(
            p - vec3<f32>(agents[i].pos.x, h + agents[i].radius, agents[i].pos.y),
            agents[i].radius,
        );
        scene = min(scene, d);
    }
    return scene;
}

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.002;
    let x = scene_distance(p + vec3<f32>(e, 0.0, 0.0)) - scene_distance(p - vec3<f32>(e, 0.0, 0.0));
    let y = scene_distance(p + vec3<f32>(0.0, e, 0.0)) - scene_distance(p - vec3<f32>(0.0, e, 0.0));
    let z = scene_distance(p + vec3<f32>(0.0, 0.0, e)) - scene_distance(p - vec3<f32>(0.0, 0.0, e));
    return normalize(vec3<f32>(x, y, z));
}

fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    let h = clamp(rd.y * 0.5 + 0.5, 0.0, 1.0);
    return mix(vec3<f32>(0.01, 0.015, 0.025), vec3<f32>(0.05, 0.10, 0.14), h);
}

fn material_color(p: vec3<f32>) -> vec3<f32> {
    let rock_d = rock_distance(p);
    let floor_d = floor_distance(p);
    var best = floor_d;
    var color = vec3<f32>(0.06, 0.07, 0.08);

    if (rock_d < best) {
        best = rock_d;
        let cave_edge = clamp(-cave_footprint(p.xz) * 0.18, 0.0, 1.0);
        color = mix(vec3<f32>(0.11, 0.12, 0.13), vec3<f32>(0.19, 0.22, 0.24), cave_edge);
    }

    for (var i = 0u; i < params.agent_count; i = i + 1u) {
        let h = select(0.45, 0.18, agents[i].kind == 5u || agents[i].kind == 6u);
        let d = sd_sphere(
            p - vec3<f32>(agents[i].pos.x, h + agents[i].radius, agents[i].pos.y),
            agents[i].radius,
        );
        if (d < best) {
            best = d;
            switch agents[i].kind {
                case 0u: {
                    color = vec3<f32>(0.30, 0.75, 0.98);
                }
                case 1u: {
                    color = vec3<f32>(0.79, 0.34, 0.31);
                }
                case 2u: {
                    color = vec3<f32>(0.77, 0.53, 0.22);
                }
                case 3u: {
                    color = vec3<f32>(0.73, 0.30, 0.78);
                }
                case 4u: {
                    color = vec3<f32>(0.97, 0.18, 0.27);
                }
                case 5u: {
                    color = vec3<f32>(0.93, 0.77, 0.28);
                }
                case 6u: {
                    color = vec3<f32>(0.20, 0.96, 0.58);
                }
                default: {
                    color = vec3<f32>(0.85, 0.85, 0.85);
                }
            }
        }
    }

    return color;
}

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>, max_dist: f32) -> f32 {
    var t = 0.03;
    var res = 1.0;
    for (var i = 0; i < 28; i = i + 1) {
        if (t >= max_dist) {
            break;
        }
        let h = scene_distance(ro + rd * t);
        if (h < 0.001) {
            return 0.0;
        }
        res = min(res, 10.0 * h / t);
        t = t + clamp(h, 0.03, 0.35);
    }
    return clamp(res, 0.0, 1.0);
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

    let ro = params.camera_target_time.xyz - forward * params.camera_orbit.z;
    let tan_half_fov = tan(params.camera_orbit.w * 0.5);
    let rd = normalize(forward + right * ndc.x * tan_half_fov * aspect + up * ndc.y * tan_half_fov);

    var t = 0.0;
    var hit = false;
    var step_count = 0u;
    let max_steps = 144u;
    let max_dist = 140.0;

    loop {
        if (step_count >= max_steps) {
            break;
        }
        let p = ro + rd * t;
        let d = scene_distance(p);
        if (d < 0.0012) {
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
        let ldir = normalize(vec3<f32>(-0.38, 0.86, -0.31));
        let vdir = normalize(-rd);
        let hdir = normalize(ldir + vdir);
        let diff = max(dot(n, ldir), 0.0);
        let spec = pow(max(dot(n, hdir), 0.0), 42.0);
        let shadow = soft_shadow(p + n * 0.01, ldir, 28.0);
        let base = material_color(p);
        let ambient = vec3<f32>(0.07, 0.08, 0.09);
        let bounce = vec3<f32>(0.05, 0.08, 0.10) * clamp(1.0 - n.y, 0.0, 1.0);
        color = base * (ambient + bounce + diff * shadow * 0.92)
            + vec3<f32>(0.95, 0.98, 1.0) * spec * shadow * 0.24;
    }

    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(0.4545));
    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
