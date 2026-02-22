struct Camera {
    position: vec3<f32>,
    _pad0: f32,

    forward: vec3<f32>,
    _pad1: f32,

    up: vec3<f32>,
    _pad2: f32,

    right: vec3<f32>,
    _pad3: f32,

    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
};

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VSOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );

    let pos = positions[vertex_index];
    let uv = pos.xy * 0.5 + vec2<f32>(0.5, 0.5);
    return VSOut(vec4<f32>(pos, 0.0, 1.0), uv);
}

@group(0) @binding(0)
var<uniform> camera: Camera;

// Signed distance functions
fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_plane(p: vec3<f32>, height: f32) -> f32 {
    return p.y - height;
}

// Raymarching function that handles sphere and plane
fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> vec2<f32> {
    var t = 0.0;
    for (var i = 0u; i < 128u; i++) {
        let p = ro + rd * t;

        let d_sphere = sd_sphere(p, 1.0);
        let d_plane = sd_plane(p, -1.0);

        let d = min(d_sphere, d_plane);

        if (d < 0.001) {
            // 0.0 = sphere, 1.0 = plane
            let hit_type = select(0.0, 1.0, d_plane < d_sphere);
            return vec2<f32>(t, hit_type);
        }

        t += d;
        if (t > 100.0) {
            break;
        }
    }
    return vec2<f32>(-1.0, -1.0);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Convert UV to NDC (-1 to 1)
    let p = uv * 2.0 - vec2<f32>(1.0);
    let fov_scale = tan(camera.fov * 0.5);

    // Generate ray
    let ray_origin = camera.position;
    let ray_dir = normalize(
        camera.forward +
        p.x * camera.right * fov_scale * camera.aspect_ratio +
        p.y * camera.up * fov_scale
    );

    // Raymarch
    let hit = raymarch(ray_origin, ray_dir);

    if (hit.x > 0.0) {
        let hit_pos = ray_origin + ray_dir * hit.x;

        // Compute normal
        let normal = select(
            normalize(hit_pos),          // sphere normal
            vec3<f32>(0.0, 1.0, 0.0),   // plane normal
            hit.y > 0.5
        );

        // Simple directional light
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let diffuse = max(dot(normal, light_dir), 0.0);

        // Color
        let color = select(
            vec3<f32>(0.8, 0.2, 0.2),   // sphere color
            vec3<f32>(0.8, 0.8, 0.8),   // plane color
            hit.y > 0.5
        );

        return vec4<f32>(color * diffuse, 1.0);
    }

    // Background
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
