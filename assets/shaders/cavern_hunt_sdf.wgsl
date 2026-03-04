struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    primitive_count : u32,
    agent_count : u32,
    _pad3 : vec2<u32>,
    floor_rock_height : vec4<f32>,
    camera_target_time : vec4<f32>,
    camera_orbit : vec4<f32>,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    kind : u32,
    _pad0 : vec2<u32>,
};

struct GeometryPrimitive {
    shape_kind : u32,
    op_kind : u32,
    _pad0 : vec2<u32>,
    p0 : vec4<f32>,
    p1 : vec4<f32>,
    p2 : vec4<f32>,
};

const SHAPE_SPHERE : u32 = 0u;
const SHAPE_ELLIPSOID : u32 = 1u;
const SHAPE_CAPSULE : u32 = 2u;
const SHAPE_BOX : u32 = 3u;
const SHAPE_ROUNDED_BOX : u32 = 4u;
const SHAPE_CYLINDER : u32 = 5u;

const OP_ADD_SOLID : u32 = 0u;
const OP_SUBTRACT_VOID : u32 = 1u;
const OP_MASK_WALKABLE : u32 = 2u;
const OP_BLOCKER : u32 = 3u;
const OP_HAZARD : u32 = 4u;

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> primitives : array<GeometryPrimitive>;
@group(0) @binding(3)
var<storage, read> agents : array<Agent>;

fn sd_box3(p: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>) -> f32 {
    let q = abs(p - center) - half_extents;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_ellipsoid(p: vec3<f32>, center: vec3<f32>, radii: vec3<f32>) -> f32 {
    let safe_radii = max(radii, vec3<f32>(0.001));
    let q = (p - center) / safe_radii;
    return (length(q) - 1.0) * min(safe_radii.x, min(safe_radii.y, safe_radii.z));
}

fn sd_capsule3(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 0.0001), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn sd_cylinder_y(p: vec3<f32>, center: vec3<f32>, radius: f32, half_height: f32) -> f32 {
    let q = p - center;
    let radial = length(q.xz) - radius;
    let y = abs(q.y) - half_height;
    let outside = length(max(vec2<f32>(radial, y), vec2<f32>(0.0)));
    return min(max(radial, y), 0.0) + outside;
}

fn sd_rounded_box(p: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>, radius: f32) -> f32 {
    return sd_box3(p, center, half_extents) - radius;
}

fn primitive_distance(primitive : GeometryPrimitive, p : vec3<f32>) -> f32 {
    let center = primitive.p0.xyz;
    let shape = primitive.shape_kind;
    if (shape == SHAPE_SPHERE) {
        return sd_sphere(p - center, primitive.p0.w);
    }
    if (shape == SHAPE_ELLIPSOID) {
        return sd_ellipsoid(p, center, primitive.p1.xyz);
    }
    if (shape == SHAPE_CAPSULE) {
        return sd_capsule3(p, primitive.p0.xyz, primitive.p1.xyz, primitive.p0.w);
    }
    if (shape == SHAPE_BOX) {
        return sd_box3(p, center, primitive.p1.xyz);
    }
    if (shape == SHAPE_ROUNDED_BOX) {
        return sd_rounded_box(p, center, primitive.p1.xyz, primitive.p0.w);
    }
    if (shape == SHAPE_CYLINDER) {
        return sd_cylinder_y(p, center, primitive.p0.w, primitive.p1.x);
    }
    return 1e9;
}

fn terrain_distance(p : vec3<f32>) -> f32 {
    var add_solid = 1e9;
    var subtract_void = 1e9;
    var blockers = 1e9;
    var has_add = false;
    var has_void = false;
    var has_blocker = false;

    for (var i = 0u; i < params.primitive_count; i = i + 1u) {
        let primitive = primitives[i];
        let d = primitive_distance(primitive, p);
        let op = primitive.op_kind;
        if (op == OP_ADD_SOLID) {
            add_solid = min(add_solid, d);
            has_add = true;
            continue;
        }
        if (op == OP_SUBTRACT_VOID || op == OP_MASK_WALKABLE) {
            subtract_void = min(subtract_void, d);
            has_void = true;
            continue;
        }
        if (op == OP_BLOCKER || op == OP_HAZARD) {
            blockers = min(blockers, d);
            has_blocker = true;
            continue;
        }
    }

    var solid = 1e9;
    if (has_add) {
        solid = add_solid;
    }
    if (has_void) {
        let carved = -subtract_void;
        if (has_add) {
            solid = max(solid, carved);
        } else {
            solid = carved;
        }
    }
    if (has_blocker) {
        solid = min(solid, blockers);
    }
    return solid;
}

fn blocker_surface_distance(p : vec3<f32>) -> f32 {
    var blockers = 1e9;
    for (var i = 0u; i < params.primitive_count; i = i + 1u) {
        let primitive = primitives[i];
        if (primitive.op_kind == OP_BLOCKER) {
            blockers = min(blockers, primitive_distance(primitive, p));
        }
    }
    return blockers;
}

fn scene_distance(p: vec3<f32>) -> f32 {
    var scene = terrain_distance(p);
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
    var best = terrain_distance(p);
    var color = vec3<f32>(0.12, 0.14, 0.16);
    let blocker_d = blocker_surface_distance(p);
    if (blocker_d < best + 0.02) {
        best = blocker_d;
        color = vec3<f32>(0.24, 0.21, 0.19);
    } else {
        let edge = clamp(exp(-abs(best) * 0.6), 0.0, 1.0);
        color = mix(vec3<f32>(0.10, 0.12, 0.14), vec3<f32>(0.20, 0.23, 0.26), edge);
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
