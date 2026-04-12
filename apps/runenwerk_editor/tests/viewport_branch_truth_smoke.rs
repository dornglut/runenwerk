use engine::WindowState;
use glam::{Vec3, vec3};
use runenwerk_editor::runtime::resources::{EditorViewportRenderState, EditorViewportSdfUniform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CenterBranchClass {
    InvalidViewport,
    PrimitiveMissing,
    RayMiss,
    RayHit,
}

#[test]
fn viewport_branch_truth_smoke() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(2)
        .expect("headless editor app should run");

    let window = app
        .world()
        .resource::<WindowState>()
        .expect("window state should exist");
    let viewport_state = app
        .world()
        .resource::<EditorViewportRenderState>()
        .expect("viewport render state should exist");
    let surface = (window.size_px.0.max(1), window.size_px.1.max(1));
    let snapshot = viewport_state.branch_trace_snapshot(surface);
    let uniform = viewport_state.compose_uniform(surface);

    let class = classify_center_sample(&uniform);
    assert!(
        !matches!(
            class,
            CenterBranchClass::InvalidViewport
                | CenterBranchClass::PrimitiveMissing
                | CenterBranchClass::RayMiss
        ),
        "expected center sample to avoid invalid/missing/miss branches for MVP startup; got {class:?}; snapshot: {}",
        snapshot.summary_line(),
    );

    let mut forced_missing = uniform;
    forced_missing.primitive_flags[1] = 0;
    let forced_missing_class = classify_center_sample(&forced_missing);
    assert_eq!(
        forced_missing_class,
        CenterBranchClass::PrimitiveMissing,
        "forcing primitive_flags.y=0 should classify as primitive-missing branch",
    );
}

fn classify_center_sample(uniform: &EditorViewportSdfUniform) -> CenterBranchClass {
    if uniform.viewport[2] <= 1.0 || uniform.viewport[3] <= 1.0 {
        return CenterBranchClass::InvalidViewport;
    }

    let pixel = (
        uniform.viewport[0] + uniform.viewport[2] * 0.5,
        uniform.viewport[1] + uniform.viewport[3] * 0.5,
    );
    if !viewport_contains(pixel, uniform.viewport) {
        return CenterBranchClass::InvalidViewport;
    }

    let viewport_size = (uniform.viewport[2].max(1.0), uniform.viewport[3].max(1.0));
    let viewport_local = (
        (pixel.0 - uniform.viewport[0]) / viewport_size.0,
        (pixel.1 - uniform.viewport[1]) / viewport_size.1,
    );
    let ndc = (viewport_local.0 * 2.0 - 1.0, 1.0 - viewport_local.1 * 2.0);
    let has_primitive = uniform.primitive_flags[1] != 0;
    if !has_primitive {
        return CenterBranchClass::PrimitiveMissing;
    }

    let aspect = viewport_size.0 / viewport_size.1.max(1.0);
    let tan_half_fov = (uniform.camera_position[3] * 0.5).tan();
    let ray_origin = vec3(
        uniform.camera_position[0],
        uniform.camera_position[1],
        uniform.camera_position[2],
    );
    let ray_dir = (vec3(
        uniform.camera_forward[0],
        uniform.camera_forward[1],
        uniform.camera_forward[2],
    ) + vec3(
        uniform.camera_right[0],
        uniform.camera_right[1],
        uniform.camera_right[2],
    ) * ndc.0
        * aspect
        * tan_half_fov
        + vec3(
            uniform.camera_up[0],
            uniform.camera_up[1],
            uniform.camera_up[2],
        ) * ndc.1
            * tan_half_fov)
        .normalize_or_zero();
    if ray_dir.length_squared() <= f32::EPSILON {
        return CenterBranchClass::RayMiss;
    }

    if march_scene(uniform, ray_origin, ray_dir) {
        CenterBranchClass::RayHit
    } else {
        CenterBranchClass::RayMiss
    }
}

fn viewport_contains(pixel: (f32, f32), viewport: [f32; 4]) -> bool {
    let min_corner = (viewport[0], viewport[1]);
    let max_corner = (viewport[0] + viewport[2], viewport[1] + viewport[3]);
    pixel.0 >= min_corner.0
        && pixel.1 >= min_corner.1
        && pixel.0 <= max_corner.0
        && pixel.1 <= max_corner.1
}

fn march_scene(uniform: &EditorViewportSdfUniform, ray_origin: Vec3, ray_dir: Vec3) -> bool {
    let mut t = 0.0_f32;
    for _ in 0..96_u32 {
        let point = ray_origin + ray_dir * t;
        let distance = scene_sdf(uniform, point);
        if distance < 0.001 {
            return true;
        }
        t += distance;
        if t > 64.0 {
            return false;
        }
    }
    false
}

fn scene_sdf(uniform: &EditorViewportSdfUniform, point: Vec3) -> f32 {
    let ground = sdf_ground_box(point);
    if uniform.primitive_flags[1] == 0 {
        return ground;
    }
    ground.min(sdf_main_primitive(uniform, point))
}

fn sdf_main_primitive(uniform: &EditorViewportSdfUniform, point: Vec3) -> f32 {
    let center = vec3(
        uniform.object_transform[0],
        uniform.object_transform[1],
        uniform.object_transform[2],
    );
    match uniform.primitive_flags[0] {
        1 => sdf_sphere(point, center, uniform.primitive_params_a[3].max(0.05)),
        2 => sdf_capsule(
            point,
            center,
            uniform.primitive_params_b[0].max(0.05),
            uniform.primitive_params_b[1].max(0.05),
        ),
        _ => sdf_box(
            point,
            center,
            vec3(
                uniform.primitive_params_a[0].max(0.05),
                uniform.primitive_params_a[1].max(0.05),
                uniform.primitive_params_a[2].max(0.05),
            ),
        ),
    }
}

fn sdf_ground_box(point: Vec3) -> f32 {
    sdf_box(point, vec3(0.0, -1.0, 0.0), vec3(8.0, 0.25, 8.0))
}

fn sdf_box(point: Vec3, center: Vec3, half_extents: Vec3) -> f32 {
    let q = (point - center).abs() - half_extents;
    let outside = q.max(Vec3::ZERO).length();
    let inside = q.x.max(q.y.max(q.z)).min(0.0);
    outside + inside
}

fn sdf_sphere(point: Vec3, center: Vec3, radius: f32) -> f32 {
    (point - center).length() - radius
}

fn sdf_capsule(point: Vec3, center: Vec3, radius: f32, half_height: f32) -> f32 {
    let q = point - center;
    let clamped_y = q.y.clamp(-half_height, half_height);
    let closest = vec3(0.0, clamped_y, 0.0);
    (q - closest).length() - radius
}
