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
    model_mesh_flags : vec4<u32>,
    model_mesh_region_rects : array<vec4<f32>, 16>,
    model_mesh_region_flags : array<vec4<u32>, 16>,
};

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

fn grid_line_strength(sample_pos: vec3<f32>) -> f32 {
    let major = abs(fract(sample_pos.xz / 2.0) - vec2<f32>(0.5, 0.5));
    let minor = abs(fract(sample_pos.xz / 0.5) - vec2<f32>(0.5, 0.5));
    let major_line = max(1.0 - min(major.x, major.y) * 20.0, 0.0);
    let minor_line = max(1.0 - min(minor.x, minor.y) * 50.0, 0.0);
    return max(major_line, minor_line * 0.45);
}

fn grid_color(sample_pos: vec3<f32>) -> vec3<f32> {
    let line = grid_line_strength(sample_pos);
    let base = vec3<f32>(0.32, 0.36, 0.41);
    let axis_x = 1.0 - smoothstep(0.0, 0.025, abs(sample_pos.z));
    let axis_z = 1.0 - smoothstep(0.0, 0.025, abs(sample_pos.x));
    var color = base + vec3<f32>(0.22, 0.25, 0.28) * line;
    color = mix(color, vec3<f32>(0.42, 0.58, 0.88), axis_x * 0.35);
    color = mix(color, vec3<f32>(0.78, 0.42, 0.38), axis_z * 0.35);
    return color;
}

fn grid_overlay(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> vec4<f32> {
    if abs(ray_dir.y) < 1e-5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let t = -ray_origin.y / ray_dir.y;
    if t <= 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let hit = ray_origin + ray_dir * t;
    let line = grid_line_strength(hit);
    let distance_fade = clamp(1.0 - t * 0.035, 0.0, 1.0);
    let alpha = clamp((0.08 + line * 0.32) * distance_fade, 0.0, 0.45);
    let color = grid_color(hit);
    return vec4<f32>(color * alpha, alpha);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
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
    return grid_overlay(ray_origin, ray_dir);
}
