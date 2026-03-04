struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    primitive_count : u32,
    agent_count : u32,
    material_program_count : u32,
    material_op_count : u32,
    material_constant_count : u32,
    render_mode : u32,
    gi_mode : u32,
    gi_quality : u32,
    gi_sample_budget : u32,
    _pad3_0 : u32,
    _pad3_1 : u32,
    _pad3_2 : u32,
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
    material_class : u32,
    material_instance : u32,
    p0 : vec4<f32>,
    p1 : vec4<f32>,
    p2 : vec4<f32>,
};

struct MaterialProgramHeader {
    class_id : u32,
    op_offset : u32,
    op_count : u32,
    const_offset : u32,
    const_count : u32,
    base_color_slot : u32,
    roughness_slot : u32,
    metallic_slot : u32,
    normal_perturb_slot : u32,
    ao_slot : u32,
    emissive_slot : u32,
    _pad0_0 : u32,
    _pad0_1 : u32,
    _pad0_2 : u32,
};

struct MaterialOpCode {
    op : u32,
    dst : u32,
    src_a : u32,
    src_b : u32,
    src_c : u32,
    const_idx : u32,
    flags : u32,
    _pad0 : u32,
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
const MATERIAL_OP_CONST_SCALAR : u32 = 0u;
const MATERIAL_OP_CONST_VEC3 : u32 = 1u;
const MATERIAL_OP_WORLD_POS : u32 = 2u;
const MATERIAL_OP_WORLD_NORMAL : u32 = 3u;
const MATERIAL_OP_TRIPLANAR_NOISE : u32 = 4u;
const MATERIAL_OP_FBM_NOISE : u32 = 5u;
const MATERIAL_OP_SLOPE_MASK : u32 = 6u;
const MATERIAL_OP_HEIGHT_MASK : u32 = 7u;
const MATERIAL_OP_ADD : u32 = 8u;
const MATERIAL_OP_MULTIPLY : u32 = 9u;
const MATERIAL_OP_BLEND : u32 = 10u;
const MATERIAL_OP_CLAMP01 : u32 = 11u;
const MATERIAL_OP_TO_VEC3 : u32 = 12u;
const RENDER_MODE_LEGACY : u32 = 0u;
const RENDER_MODE_MATERIAL_GRAPH : u32 = 1u;
const GI_MODE_OFF : u32 = 0u;
const GI_MODE_AO_BENT : u32 = 1u;
const GI_MODE_PROBE : u32 = 2u;

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> primitives : array<GeometryPrimitive>;
@group(0) @binding(3)
var<storage, read> agents : array<Agent>;
@group(0) @binding(4)
var<storage, read> material_program_headers : array<MaterialProgramHeader>;
@group(0) @binding(5)
var<storage, read> material_ops : array<MaterialOpCode>;
@group(0) @binding(6)
var<storage, read> material_constants : array<vec4<f32>>;

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

    // Camera-driven roof cutaway: anything above this plane is treated as void.
    // This keeps players readable under a fixed external camera.
    let roof_cut_y = params.floor_rock_height.z;
    let roof_void = roof_cut_y - p.y;
    subtract_void = min(subtract_void, roof_void);
    has_void = true;

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

fn legacy_material_color(p: vec3<f32>) -> vec3<f32> {
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

struct MaterialSurface {
    base_color : vec3<f32>,
    roughness : f32,
    metallic : f32,
    ao : f32,
    emissive : vec3<f32>,
}

fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn hash31(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

fn value_noise3(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash31(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash31(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash31(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash31(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash31(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash31(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash31(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash31(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);
    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);
    return mix(nxy0, nxy1, u.z);
}

fn fbm_noise3(p: vec3<f32>, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    var frequency = 1.0;
    var amplitude = 0.5;
    var sum = 0.0;
    var normalizer = 0.0;
    for (var i = 0u; i < 8u; i = i + 1u) {
        if (i >= octaves) {
            break;
        }
        sum = sum + value_noise3(p * frequency) * amplitude;
        normalizer = normalizer + amplitude;
        frequency = frequency * lacunarity;
        amplitude = amplitude * gain;
    }
    if (normalizer <= 0.0001) {
        return 0.0;
    }
    return sum / normalizer;
}

fn triplanar_noise(position: vec3<f32>, normal: vec3<f32>, scale: f32, sharpness: f32, seed: f32) -> f32 {
    let n = abs(normal);
    let w = pow(max(n, vec3<f32>(0.001)), vec3<f32>(max(sharpness, 0.01)));
    let weight = w / max(vec3<f32>(dot(w, vec3<f32>(1.0))), vec3<f32>(0.0001));
    let p = position * max(scale, 0.0001) + vec3<f32>(seed);
    let nx = value_noise3(vec3<f32>(p.y, p.z, p.x + seed * 0.13));
    let ny = value_noise3(vec3<f32>(p.x, p.z, p.y + seed * 0.31));
    let nz = value_noise3(vec3<f32>(p.x, p.y, p.z + seed * 0.47));
    return nx * weight.x + ny * weight.y + nz * weight.z;
}

fn reg_vec3(v: vec4<f32>) -> vec3<f32> {
    let scalar_like = abs(v.y) + abs(v.z) + abs(v.w) <= 0.0001;
    return select(v.xyz, vec3<f32>(v.x), scalar_like);
}

fn material_constant(header: MaterialProgramHeader, idx: u32) -> vec4<f32> {
    let absolute = header.const_offset + idx;
    if (absolute < params.material_constant_count) {
        return material_constants[absolute];
    }
    return vec4<f32>(0.0);
}

fn find_material_header(class_id: u32) -> MaterialProgramHeader {
    var fallback = MaterialProgramHeader(
        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u
    );
    var has_fallback = false;
    for (var i = 0u; i < params.material_program_count; i = i + 1u) {
        let header = material_program_headers[i];
        if (header.class_id == class_id) {
            return header;
        }
        if (!has_fallback && header.class_id == 0u) {
            fallback = header;
            has_fallback = true;
        }
    }
    return fallback;
}

fn evaluate_material_surface(p: vec3<f32>, n: vec3<f32>, class_id: u32) -> MaterialSurface {
    var surface = MaterialSurface(
        vec3<f32>(0.14, 0.16, 0.18),
        0.85,
        0.02,
        1.0,
        vec3<f32>(0.0),
    );
    let header = find_material_header(class_id);
    if (header.op_count == 0u || params.render_mode == RENDER_MODE_LEGACY) {
        surface.base_color = legacy_material_color(p);
        return surface;
    }

    var regs: array<vec4<f32>, 64>;
    for (var i = 0u; i < 64u; i = i + 1u) {
        regs[i] = vec4<f32>(0.0);
    }

    for (var i = 0u; i < header.op_count; i = i + 1u) {
        let op = material_ops[header.op_offset + i];
        if (op.dst >= 64u) {
            continue;
        }
        let a = regs[min(op.src_a, 63u)];
        let b = regs[min(op.src_b, 63u)];
        let c = regs[min(op.src_c, 63u)];
        let output_vec3 = (op.flags & 0x80000000u) != 0u;
        switch op.op {
            case MATERIAL_OP_CONST_SCALAR: {
                let k = material_constant(header, op.const_idx);
                regs[op.dst] = vec4<f32>(k.x, 0.0, 0.0, 0.0);
            }
            case MATERIAL_OP_CONST_VEC3: {
                let k = material_constant(header, op.const_idx);
                regs[op.dst] = vec4<f32>(k.xyz, 0.0);
            }
            case MATERIAL_OP_WORLD_POS: {
                regs[op.dst] = vec4<f32>(p, 1.0);
            }
            case MATERIAL_OP_WORLD_NORMAL: {
                regs[op.dst] = vec4<f32>(n, 1.0);
            }
            case MATERIAL_OP_TRIPLANAR_NOISE: {
                let k = material_constant(header, op.const_idx);
                let noise = triplanar_noise(reg_vec3(a), normalize(reg_vec3(b)), k.x, k.y, k.z);
                regs[op.dst] = vec4<f32>(noise, 0.0, 0.0, 0.0);
            }
            case MATERIAL_OP_FBM_NOISE: {
                let k = material_constant(header, op.const_idx);
                let octaves = max(op.flags & 0xffu, 1u);
                let noise = fbm_noise3(reg_vec3(a) * max(k.x, 0.0001) + vec3<f32>(k.w), octaves, max(k.y, 1.0), saturate(k.z));
                regs[op.dst] = vec4<f32>(noise, 0.0, 0.0, 0.0);
            }
            case MATERIAL_OP_SLOPE_MASK: {
                let k = material_constant(header, op.const_idx);
                let slope = pow(saturate(reg_vec3(a).y), max(k.x, 0.01));
                let value = select(slope, 1.0 - slope, k.y > 0.5);
                regs[op.dst] = vec4<f32>(value, 0.0, 0.0, 0.0);
            }
            case MATERIAL_OP_HEIGHT_MASK: {
                let k = material_constant(header, op.const_idx);
                let t = smoothstep(k.x - abs(k.z), k.y + abs(k.z), reg_vec3(a).y);
                regs[op.dst] = vec4<f32>(t, 0.0, 0.0, 0.0);
            }
            case MATERIAL_OP_ADD: {
                if (output_vec3) {
                    regs[op.dst] = vec4<f32>(reg_vec3(a) + reg_vec3(b), 0.0);
                } else {
                    regs[op.dst] = vec4<f32>(a.x + b.x, 0.0, 0.0, 0.0);
                }
            }
            case MATERIAL_OP_MULTIPLY: {
                if (output_vec3) {
                    regs[op.dst] = vec4<f32>(reg_vec3(a) * reg_vec3(b), 0.0);
                } else {
                    regs[op.dst] = vec4<f32>(a.x * b.x, 0.0, 0.0, 0.0);
                }
            }
            case MATERIAL_OP_BLEND: {
                let mask = saturate(c.x);
                if (output_vec3) {
                    regs[op.dst] = vec4<f32>(mix(reg_vec3(a), reg_vec3(b), vec3<f32>(mask)), 0.0);
                } else {
                    regs[op.dst] = vec4<f32>(mix(a.x, b.x, mask), 0.0, 0.0, 0.0);
                }
            }
            case MATERIAL_OP_CLAMP01: {
                if (output_vec3) {
                    regs[op.dst] = vec4<f32>(clamp(reg_vec3(a), vec3<f32>(0.0), vec3<f32>(1.0)), 0.0);
                } else {
                    regs[op.dst] = vec4<f32>(saturate(a.x), 0.0, 0.0, 0.0);
                }
            }
            case MATERIAL_OP_TO_VEC3: {
                regs[op.dst] = vec4<f32>(vec3<f32>(a.x), 0.0);
            }
            default: {}
        }
    }

    surface.base_color = clamp(reg_vec3(regs[min(header.base_color_slot, 63u)]), vec3<f32>(0.0), vec3<f32>(1.2));
    surface.roughness = clamp(regs[min(header.roughness_slot, 63u)].x, 0.04, 1.0);
    surface.metallic = saturate(regs[min(header.metallic_slot, 63u)].x);
    surface.ao = saturate(regs[min(header.ao_slot, 63u)].x);
    surface.emissive = max(reg_vec3(regs[min(header.emissive_slot, 63u)]), vec3<f32>(0.0));
    return surface;
}

fn nearest_agent_distance(p: vec3<f32>) -> f32 {
    var best = 1e9;
    for (var i = 0u; i < params.agent_count; i = i + 1u) {
        let h = select(0.45, 0.18, agents[i].kind == 5u || agents[i].kind == 6u);
        let d = sd_sphere(
            p - vec3<f32>(agents[i].pos.x, h + agents[i].radius, agents[i].pos.y),
            agents[i].radius,
        );
        best = min(best, d);
    }
    return best;
}

fn terrain_material_class(p: vec3<f32>, terrain_d: f32) -> u32 {
    var blocker_d = 1e9;
    var blocker_class = 1u;
    var void_d = 1e9;
    var void_class = 0u;
    for (var i = 0u; i < params.primitive_count; i = i + 1u) {
        let primitive = primitives[i];
        let d = primitive_distance(primitive, p);
        if ((primitive.op_kind == OP_BLOCKER || primitive.op_kind == OP_HAZARD) && d < blocker_d) {
            blocker_d = d;
            blocker_class = primitive.material_class;
        }
        if ((primitive.op_kind == OP_SUBTRACT_VOID || primitive.op_kind == OP_MASK_WALKABLE) && d < void_d) {
            void_d = d;
            void_class = primitive.material_class;
        }
    }
    if (blocker_d < terrain_d + 0.02) {
        return blocker_class;
    }
    return void_class;
}

fn ggx_distribution(ndoth: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = ndoth * ndoth * (a2 - 1.0) + 1.0;
    return a2 / max(3.14159265 * d * d, 0.0001);
}

fn smith_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return ndotv / max(ndotv * (1.0 - k) + k, 0.0001);
}

fn smith_geometry(ndotv: f32, ndotl: f32, roughness: f32) -> f32 {
    return smith_schlick_ggx(ndotv, roughness) * smith_schlick_ggx(ndotl, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn hemisphere_dir(n: vec3<f32>, i: u32, sample_count: u32, seed: f32) -> vec3<f32> {
    let fi = f32(i) + 0.5;
    let count = max(f32(sample_count), 1.0);
    let phi = 6.2831853 * fract(fi * 0.6180339 + seed * 0.37);
    let cos_theta = 1.0 - fi / count;
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
    let tangent = normalize(select(
        cross(n, vec3<f32>(0.0, 1.0, 0.0)),
        cross(n, vec3<f32>(1.0, 0.0, 0.0)),
        abs(n.y) > 0.9
    ));
    let bitangent = normalize(cross(n, tangent));
    return normalize(
        tangent * cos(phi) * sin_theta +
        bitangent * sin(phi) * sin_theta +
        n * cos_theta
    );
}

fn ambient_occlusion_and_bent_normal(p: vec3<f32>, n: vec3<f32>, sample_count: u32) -> vec4<f32> {
    var occlusion = 0.0;
    var bent = vec3<f32>(0.0);
    for (var i = 0u; i < 24u; i = i + 1u) {
        if (i >= sample_count) {
            break;
        }
        let dir = hemisphere_dir(n, i, sample_count, hash31(p + vec3<f32>(f32(i) * 0.37)));
        let dist = 0.22 + f32(i) * 0.11;
        let h = scene_distance(p + dir * dist);
        let occ = saturate((dist - h) / dist);
        occlusion = occlusion + occ;
        bent = bent + dir * (1.0 - occ);
    }
    let denom = max(f32(sample_count), 1.0);
    let ao = saturate(1.0 - occlusion / denom);
    let bent_normal = normalize(select(n, bent, length(bent) > 0.001));
    return vec4<f32>(bent_normal, ao);
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
        let shadow = soft_shadow(p + n * 0.01, ldir, 28.0);
        let terrain_d = terrain_distance(p);
        let agent_d = nearest_agent_distance(p);
        let is_agent_surface = agent_d < terrain_d + 0.01;

        var surface = MaterialSurface(
            legacy_material_color(p),
            select(0.82, 0.55, is_agent_surface),
            0.02,
            1.0,
            vec3<f32>(0.0),
        );
        if (!is_agent_surface && params.render_mode == RENDER_MODE_MATERIAL_GRAPH) {
            let class_id = terrain_material_class(p, terrain_d);
            surface = evaluate_material_surface(p, n, class_id);
        }

        var ao = 1.0;
        var bent_normal = n;
        if (params.gi_mode != GI_MODE_OFF) {
            let samples = max(min(params.gi_sample_budget, 24u), 4u);
            let ao_bent = ambient_occlusion_and_bent_normal(p, n, samples);
            bent_normal = ao_bent.xyz;
            ao = ao_bent.w;
        }

        let hdir = normalize(ldir + vdir);
        let ndotl = max(dot(n, ldir), 0.0);
        let ndotv = max(dot(n, vdir), 0.0);
        let ndoth = max(dot(n, hdir), 0.0);
        let vdh = max(dot(vdir, hdir), 0.0);
        let roughness = clamp(surface.roughness, 0.04, 1.0);
        let metallic = clamp(surface.metallic, 0.0, 1.0);
        let f0 = mix(vec3<f32>(0.04), surface.base_color, vec3<f32>(metallic));
        let f = fresnel_schlick(vdh, f0);
        let d = ggx_distribution(ndoth, roughness);
        let g = smith_geometry(ndotv, ndotl, roughness);
        let specular = (d * g * f) / max(4.0 * ndotv * ndotl, 0.0001);
        let kd = (vec3<f32>(1.0) - f) * (1.0 - metallic);
        let diffuse = kd * surface.base_color / 3.14159265;
        let direct = (diffuse + specular) * ndotl * shadow;

        let bent_sky = sky_color(bent_normal);
        var ambient_strength = vec3<f32>(0.05, 0.06, 0.07);
        if (params.gi_mode == GI_MODE_PROBE) {
            ambient_strength = ambient_strength + vec3<f32>(0.025, 0.03, 0.035);
        }
        let ambient = (ambient_strength + bent_sky * 0.35) * surface.base_color * ao * surface.ao;
        color = ambient + direct + surface.emissive;
    }

    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(0.4545));
    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
