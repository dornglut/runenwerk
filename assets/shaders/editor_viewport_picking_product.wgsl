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

@group(0) @binding(0)
var<uniform> u : EditorViewportSceneProductUniform;

const MAX_PRIMITIVES : u32 = 64u;

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
};

struct PickingHit {
    hit : bool,
    pick_slot : u32,
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

fn sdf_cylinder(sample_pos: vec3<f32>, center: vec3<f32>, radius: f32, half_height: f32) -> f32 {
    let local = sample_pos - center;
    let d = vec2<f32>(
        length(local.xz) - radius,
        abs(local.y) - half_height,
    );
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0, 0.0)));
}

fn sdf_torus(sample_pos: vec3<f32>, center: vec3<f32>, major_radius: f32, minor_radius: f32) -> f32 {
    let local = sample_pos - center;
    let q = vec2<f32>(length(local.xz) - major_radius, local.y);
    return length(q) - minor_radius;
}

fn sdf_plane_slab(sample_pos: vec3<f32>, center: vec3<f32>, half_extents: vec3<f32>) -> f32 {
    let slab_extents = vec3<f32>(
        max(half_extents.x, 0.05),
        max(min(half_extents.y, 0.05), 0.01),
        max(half_extents.z, 0.05),
    );
    return sdf_box(sample_pos, center, slab_extents);
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

    if primitive_kind == 3u {
        return sdf_cylinder(
            sample_pos,
            center,
            max(params_b.x, 0.05),
            max(params_b.y, 0.05),
        );
    }

    if primitive_kind == 4u {
        let major_radius = max(params_a.w * 1.5, 0.05);
        let minor_radius = max(params_a.w * 0.5, 0.05);
        return sdf_torus(sample_pos, center, major_radius, minor_radius);
    }

    if primitive_kind == 5u {
        return sdf_plane_slab(
            sample_pos,
            center,
            max(params_a.xyz, vec3<f32>(0.05, 0.05, 0.05)),
        );
    }

    return sdf_box(
        sample_pos,
        center,
        max(params_a.xyz, vec3<f32>(0.05, 0.05, 0.05)),
    );
}

fn scene_sdf(sample_pos: vec3<f32>) -> f32 {
    var distance = 1e9;
    let count = primitive_count();
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

fn nearest_pick_slot(sample_pos: vec3<f32>) -> u32 {
    let count = primitive_count();
    var best_distance = 1e9;
    var best_pick_slot = 0u;
    var index = 0u;
    loop {
        if index >= count {
            break;
        }
        let primitive_distance = abs(sdf_primitive_slot(sample_pos, index));
        if primitive_distance < best_distance {
            best_distance = primitive_distance;
            best_pick_slot = u.primitive_slot_flags[index].y;
        }
        index = index + 1u;
    }
    return best_pick_slot;
}

fn march_scene(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> PickingHit {
    var t = 0.0;
    var steps: u32 = 0u;

    loop {
        if steps >= 96u {
            break;
        }

        let sample_pos = ray_origin + ray_dir * t;
        let distance = scene_sdf(sample_pos);
        if distance < 0.001 {
            return PickingHit(true, nearest_pick_slot(sample_pos));
        }

        t = t + distance;
        if t > 64.0 {
            break;
        }

        steps = steps + 1u;
    }

    return PickingHit(false, 0u);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) u32 {
    if primitive_count() == 0u {
        return 0u;
    }

    let pixel = position.xy;
    let target_size = max(u.surface.xy, vec2<f32>(1.0, 1.0));
    let viewport_local = pixel / target_size;
    let ndc = vec2<f32>(viewport_local.x * 2.0 - 1.0, 1.0 - viewport_local.y * 2.0);
    let aspect = target_size.x / target_size.y;
    let tan_half_fov = tan(u.camera_position.w * 0.5);

    let ray_origin = u.camera_position.xyz;
    let ray_dir = normalize(
        u.camera_forward.xyz
        + u.camera_right.xyz * ndc.x * aspect * tan_half_fov
        + u.camera_up.xyz * ndc.y * tan_half_fov
    );
    let hit = march_scene(ray_origin, ray_dir);
    if hit.hit {
        return hit.pick_slot;
    }
    return 0u;
}
