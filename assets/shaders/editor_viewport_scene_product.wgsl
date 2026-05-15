struct EditorViewportSceneProductUniform {
    surface : vec4<f32>,
    viewport : vec4<f32>,
    camera_position : vec4<f32>,
    camera_forward : vec4<f32>,
    camera_right : vec4<f32>,
    camera_up : vec4<f32>,
    object_transform : vec4<f32>,
    primitive_params_a : vec4<f32>,
    primitive_params_b : vec4<f32>,
    primitive_flags : vec4<u32>,
    primitive_slot_transforms : array<vec4<f32>, 64>,
    primitive_slot_params_a : array<vec4<f32>, 64>,
    primitive_slot_params_b : array<vec4<f32>, 64>,
    primitive_slot_flags : array<vec4<u32>, 64>,
};

const MAX_PRIMITIVES : u32 = 64u;

@group(0) @binding(0)
var<uniform> u : EditorViewportSceneProductUniform;

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    return out;
}

fn sdf_box(sample_pos: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>) -> f32 {
    let q = abs(sample_pos - center) - half_extents;
    let outside = length(max(q, vec3<f32>(0.0, 0.0, 0.0)));
    let inside = min(max(q.x, max(q.y, q.z)), 0.0);
    return outside + inside;
}

fn sdf_sphere(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(sample_pos - center) - radius;
}

fn sdf_capsule(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32, half_height: f32) -> f32 {
    let q = sample_pos - center;
    let clamped_y = clamp(q.y, -half_height, half_height);
    let closest = vec3<f32>(0.0, clamped_y, 0.0);
    return length(q - closest) - radius;
}

fn sdf_main_primitive(sample_pos: vec3<f32>) -> f32 {
    return sdf_primitive_slot(sample_pos, 0u);
}

fn primitive_count() -> u32 {
    return min(u.primitive_flags.y, MAX_PRIMITIVES);
}

fn sdf_primitive_slot(sample_pos: vec3<f32>, primitive_index: u32) -> f32 {
    let slot = u.primitive_slot_flags[primitive_index];
    let center = u.primitive_slot_transforms[primitive_index].xyz;
    let primitive_kind = slot.x;
    let params_a = u.primitive_slot_params_a[primitive_index];
    let params_b = u.primitive_slot_params_b[primitive_index];

    if primitive_kind == 1u {
        return sdf_sphere(sample_pos, center, max(params_a.w, 0.05));
    }

    if primitive_kind == 2u {
        return sdf_capsule(
            sample_pos,
            center,
            max(params_b.x, 0.05),
            max(params_b.y, 0.05),
        );
    }

    return sdf_box(
        sample_pos,
        center,
        max(params_a.xyz, vec3<f32>(0.05, 0.05, 0.05)),
    );
}

fn sdf_ground_box(sample_pos: vec3<f32>) -> f32 {
    return sdf_box(sample_pos, vec3<f32>(0.0, -1.0, 0.0), vec3<f32>(8.0, 0.25, 8.0));
}

fn scene_sdf(sample_pos: vec3<f32>) -> f32 {
    var distance = sdf_ground_box(sample_pos);
    let count = primitive_count();
    if count == 0u {
        return distance;
    }

    var index = 0u;
    loop {
        if index >= count {
            break;
        }
        distance = min(distance, sdf_primitive_slot(sample_pos, index));
        index = index + 1u;
    }
    return distance;
}

fn is_ground_hit(sample_pos: vec3<f32>) -> bool {
    let ground_distance = abs(sdf_ground_box(sample_pos));
    let count = primitive_count();
    if count == 0u {
        return true;
    }

    var primitive_distance = 1e9;
    var index = 0u;
    loop {
        if index >= count {
            break;
        }
        primitive_distance = min(primitive_distance, abs(sdf_primitive_slot(sample_pos, index)));
        index = index + 1u;
    }
    return ground_distance <= primitive_distance;
}

fn hit_primitive_flags(sample_pos: vec3<f32>) -> vec4<u32> {
    let count = primitive_count();
    if count == 0u {
        return vec4<u32>(0u, 0u, 0u, 0u);
    }

    var best_distance = 1e9;
    var best_flags = vec4<u32>(0u, 0u, 0u, 0u);
    var index = 0u;
    loop {
        if index >= count {
            break;
        }
        let primitive_distance = abs(sdf_primitive_slot(sample_pos, index));
        if primitive_distance < best_distance {
            best_distance = primitive_distance;
            best_flags = u.primitive_slot_flags[index];
        }
        index = index + 1u;
    }
    return best_flags;
}

fn estimate_normal(sample_pos: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let nx = scene_sdf(sample_pos + vec3<f32>(e, 0.0, 0.0)) - scene_sdf(sample_pos - vec3<f32>(e, 0.0, 0.0));
    let ny = scene_sdf(sample_pos + vec3<f32>(0.0, e, 0.0)) - scene_sdf(sample_pos - vec3<f32>(0.0, e, 0.0));
    let nz = scene_sdf(sample_pos + vec3<f32>(0.0, 0.0, e)) - scene_sdf(sample_pos - vec3<f32>(0.0, 0.0, e));
    return normalize(vec3<f32>(nx, ny, nz));
}

fn grid_color(sample_pos: vec3<f32>) -> vec3<f32> {
    let major = abs(fract(sample_pos.xz / 2.0) - vec2<f32>(0.5, 0.5));
    let minor = abs(fract(sample_pos.xz / 0.5) - vec2<f32>(0.5, 0.5));
    let major_line = max(1.0 - min(major.x, major.y) * 20.0, 0.0);
    let minor_line = max(1.0 - min(minor.x, minor.y) * 50.0, 0.0);
    let base = vec3<f32>(0.10, 0.11, 0.13);
    return base + vec3<f32>(0.10, 0.11, 0.13) * major_line + vec3<f32>(0.04, 0.04, 0.05) * minor_line;
}

fn grid_shade(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec4<f32> {
    if abs(ray_dir.y) < 1e-5 {
        return vec4<f32>(0.09, 0.10, 0.12, 1.0);
    }

    let t = -ray_origin.y / ray_dir.y;
    if t <= 0.0 {
        return vec4<f32>(0.09, 0.10, 0.12, 1.0);
    }

    let hit = ray_origin + ray_dir * t;
    let color = grid_color(hit);
    let fog = clamp(1.0 - t * 0.03, 0.0, 1.0);
    return vec4<f32>(mix(vec3<f32>(0.09, 0.10, 0.12), color, fog), 1.0);
}

struct RaymarchResult {
    hit : bool,
    distance : f32,
};

fn march_scene(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> RaymarchResult {
    var t = 0.0;
    var hit = false;
    var steps: u32 = 0u;

    loop {
        if steps >= 96u {
            break;
        }

        let sample_pos = ray_origin + ray_dir * t;
        let distance = scene_sdf(sample_pos);
        if distance < 0.001 {
            hit = true;
            break;
        }

        t = t + distance;
        if t > 64.0 {
            break;
        }

        steps = steps + 1u;
    }

    return RaymarchResult(hit, t);
}

fn color_magenta() -> vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = position.xy;
    let debug_stage = u.primitive_flags.z;
    let target_size = max(u.surface.xy, vec2<f32>(1.0, 1.0));
    if debug_stage == 1u {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    }

    let viewport_size = target_size;
    let viewport_local = pixel / viewport_size;
    let ndc = vec2<f32>(viewport_local.x * 2.0 - 1.0, 1.0 - viewport_local.y * 2.0);
    let has_primitive = primitive_count() != 0u;

    if debug_stage == 2u {
        let gradient = vec3<f32>(
            clamp(viewport_local.x, 0.0, 1.0),
            clamp(viewport_local.y, 0.0, 1.0),
            clamp((ndc.x + 1.0) * 0.5, 0.0, 1.0)
        );
        return vec4<f32>(gradient, 1.0);
    }
    if debug_stage == 3u {
        if has_primitive {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let aspect = viewport_size.x / viewport_size.y;
    let tan_half_fov = tan(u.camera_position.w * 0.5);

    let ray_origin = u.camera_position.xyz;
    let ray_dir = normalize(
        u.camera_forward.xyz
        + u.camera_right.xyz * ndc.x * aspect * tan_half_fov
        + u.camera_up.xyz * ndc.y * tan_half_fov
    );

    var ray_hit = false;
    var ray_hit_distance = 0.0;
    if has_primitive {
        let march = march_scene(ray_origin, ray_dir);
        ray_hit = march.hit;
        ray_hit_distance = march.distance;
    }

    if debug_stage == 4u {
        if !has_primitive {
            return vec4<f32>(1.0, 0.5, 0.0, 1.0);
        }
        if ray_hit {
            return vec4<f32>(1.0, 1.0, 0.0, 1.0);
        }
        return vec4<f32>(0.0, 0.35, 1.0, 1.0);
    }

    if !has_primitive {
        return grid_shade(ray_origin, ray_dir);
    }
    if !ray_hit {
        return grid_shade(ray_origin, ray_dir);
    }

    let sample_pos = ray_origin + ray_dir * ray_hit_distance;
    let normal = estimate_normal(sample_pos);
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.35));
    let diff = max(dot(normal, light_dir), 0.1);
    let rim = pow(max(1.0 - max(dot(normal, -ray_dir), 0.0), 0.0), 2.0);
    var base = vec3<f32>(0.72, 0.74, 0.77);
    if is_ground_hit(sample_pos) {
        base = vec3<f32>(0.34, 0.37, 0.41);
    } else {
        let flags = hit_primitive_flags(sample_pos);
        if flags.z != 0u {
            base = vec3<f32>(0.86, 0.78, 0.46);
        }
        if flags.w != 0u {
            base = mix(base, vec3<f32>(0.40, 0.72, 0.96), 0.45);
        }
    }
    let lit = base * diff + vec3<f32>(0.15, 0.20, 0.28) * rim;
    return vec4<f32>(lit, 1.0);
}
