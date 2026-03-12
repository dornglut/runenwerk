use glam::Vec3;

pub fn central_difference<F>(sample_distance: F, point: Vec3, epsilon: f32) -> Vec3
where
    F: Fn(Vec3) -> f32,
{
    let h = epsilon.max(f32::EPSILON);
    let ex = Vec3::new(h, 0.0, 0.0);
    let ey = Vec3::new(0.0, h, 0.0);
    let ez = Vec3::new(0.0, 0.0, h);

    let dx = sample_distance(point + ex) - sample_distance(point - ex);
    let dy = sample_distance(point + ey) - sample_distance(point - ey);
    let dz = sample_distance(point + ez) - sample_distance(point - ez);
    Vec3::new(dx, dy, dz) / (2.0 * h)
}
