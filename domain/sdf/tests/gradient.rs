use glam::Vec3;

use sdf::gradient::{estimate_gradient, estimate_normal};
use sdf::primitives::{SdfPlane, SdfSphere};

const EPS: f32 = 1e-2;

#[test]
fn sphere_gradient_points_outward() {
    let sphere = SdfSphere::new(Vec3::ZERO, 1.0);
    let gradient = estimate_gradient(&sphere, Vec3::new(1.0, 0.0, 0.0), 1e-3);
    assert!(gradient.dot(Vec3::X) > 0.99);
}

#[test]
fn estimated_normals_are_unit_length_for_regular_points() {
    let sphere = SdfSphere::new(Vec3::ZERO, 1.0);
    let normal = estimate_normal(&sphere, Vec3::new(1.0, 0.0, 0.0), 1e-3);
    assert!((normal.length() - 1.0).abs() < EPS);
    assert!(normal.dot(Vec3::X) > 0.99);
}

#[test]
fn finite_difference_is_stable_across_small_epsilon_changes() {
    let sphere = SdfSphere::new(Vec3::ZERO, 1.0);
    let g0 = estimate_gradient(&sphere, Vec3::new(0.6, 0.5, 0.2).normalize(), 1e-3).normalize();
    let g1 = estimate_gradient(&sphere, Vec3::new(0.6, 0.5, 0.2).normalize(), 2e-3).normalize();
    assert!(g0.dot(g1) > 0.99);
}

#[test]
fn plane_gradient_aligns_with_plane_normal() {
    let plane = SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let normal = estimate_normal(&plane, Vec3::new(3.0, 2.0, -1.0), 1e-3);
    assert!(normal.dot(Vec3::Y) > 0.99);
}
