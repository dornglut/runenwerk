use glam::Vec3;

use sdf::normal::normal_at_default;
use sdf::primitives::{SdfPlane, SdfSphere};

#[test]
fn sphere_normal_points_outward() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let n = normal_at_default(&field, Vec3::new(1.0, 0.0, 0.0));
    assert!(n.dot(Vec3::X) > 0.99);
}

#[test]
fn plane_normal_is_stable() {
    let field = SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let n = normal_at_default(&field, Vec3::new(2.0, 3.0, -1.0));
    assert!(n.dot(Vec3::Y) > 0.99);
}

#[test]
fn nearby_points_have_consistent_normals_on_sphere() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let p0 = Vec3::new(0.0, 1.0, 0.0);
    let p1 = Vec3::new(0.01, 1.0, 0.0).normalize();
    let n0 = normal_at_default(&field, p0);
    let n1 = normal_at_default(&field, p1);
    assert!(n0.dot(n1) > 0.99);
}
