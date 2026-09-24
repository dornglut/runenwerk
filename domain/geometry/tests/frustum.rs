use geometry::{Aabb3, Frustum, Plane, Sphere};
use glam::Vec3;

fn unit_box_frustum() -> Frustum {
    Frustum::new(
        Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0), // x >= -1
        Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0),  // x <= 1
        Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0),  // y <= 1
        Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0), // y >= -1
        Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0), // z >= -1
        Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0),  // z <= 1
    )
}

#[test]
fn frustum_contains_point_inside_outside_and_boundary() {
    let frustum = unit_box_frustum();
    assert!(frustum.contains_point(Vec3::ZERO));
    assert!(frustum.contains_point(Vec3::new(1.0, 0.0, 0.0)));
    assert!(!frustum.contains_point(Vec3::new(1.01, 0.0, 0.0)));
}

#[test]
fn frustum_intersects_aabb_inside_outside_and_overlap() {
    let frustum = unit_box_frustum();

    let inside = Aabb3::from_corners(Vec3::splat(-0.5), Vec3::splat(0.5));
    let outside = Aabb3::from_corners(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    let edge_overlap = Aabb3::from_corners(Vec3::new(0.9, -0.2, -0.2), Vec3::new(1.2, 0.2, 0.2));
    let corner_overlap = Aabb3::from_corners(Vec3::new(0.8, 0.8, 0.8), Vec3::new(1.4, 1.4, 1.4));

    assert!(frustum.intersects_aabb(&inside));
    assert!(!frustum.intersects_aabb(&outside));
    assert!(frustum.intersects_aabb(&edge_overlap));
    assert!(frustum.intersects_aabb(&corner_overlap));
}

#[test]
fn frustum_intersects_sphere_respects_boundary_touching() {
    let frustum = unit_box_frustum();
    let touching = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 0.5);
    let outside = Sphere::new(Vec3::new(1.6, 0.0, 0.0), 0.5);

    assert!(frustum.intersects_sphere(&touching));
    assert!(!frustum.intersects_sphere(&outside));
}
