use geometry::Plane;
use glam::Vec3;

#[test]
fn plane_signed_distance_respects_equation_convention() {
    let plane = Plane::from_point_normal(Vec3::new(0.0, 2.0, 0.0), Vec3::Y);
    assert_eq!(plane.signed_distance(Vec3::new(0.0, 5.0, 0.0)), 3.0);
    assert_eq!(plane.signed_distance(Vec3::new(0.0, 1.0, 0.0)), -1.0);
    assert_eq!(plane.signed_distance(Vec3::new(1.0, 2.0, -7.0)), 0.0);
}

#[test]
fn plane_project_point_returns_point_on_plane() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let projected = plane.project_point(Vec3::new(1.0, 4.0, -3.0));
    assert_eq!(projected, Vec3::new(1.0, 0.0, -3.0));
    assert_eq!(plane.signed_distance(projected), 0.0);
}

#[test]
fn plane_projection_handles_non_normalized_normal() {
    let plane = Plane::from_point_normal(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
    let projected = plane.project_point(Vec3::new(0.0, 5.0, 0.0));

    assert!((projected.y - 2.0).abs() < 1e-6);
    assert!(plane.signed_distance(projected).abs() < 1e-6);
}
