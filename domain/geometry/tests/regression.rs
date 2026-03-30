use geometry::classification::{
    PlaneAabbClassification, PlanePointClassification, classify_aabb_plane, classify_point_plane,
};
use geometry::intersection::ray_aabb3_first_hit;
use geometry::{Aabb3, Plane, Ray3};
use glam::Vec3;

#[test]
fn regression_parallel_ray_on_boundary_does_not_false_negative() {
    let aabb = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(1.0));
    let ray = Ray3::new(Vec3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&ray, &aabb), Some(1.0));
}

#[test]
fn regression_point_plane_classification_boundary_is_on_plane() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    assert_eq!(
        classify_point_plane(&plane, Vec3::new(0.0, 0.0, 0.0)),
        PlanePointClassification::OnPlane
    );
}

#[test]
fn regression_aabb_plane_classification_intersecting_case() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let crossing = Aabb3::from_corners(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(
        classify_aabb_plane(&plane, &crossing),
        PlaneAabbClassification::Intersecting
    );
}
