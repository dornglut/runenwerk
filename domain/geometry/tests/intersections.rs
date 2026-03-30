use geometry::intersection::{
    frustum_aabb3_intersects, ray_plane_first_hit, ray_sphere_first_hit, ray_triangle3_first_hit,
    segment_aabb3_intersects, sphere_aabb3_intersects,
};
use geometry::{Aabb3, Frustum, Plane, Ray3, Sphere, Triangle3};
use glam::Vec3;

fn test_frustum() -> Frustum {
    Frustum::new(
        Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0),
        Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0),
        Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0),
    )
}

#[test]
fn ray_sphere_hit_miss_and_inside_cases() {
    let sphere = Sphere::new(Vec3::ZERO, 1.0);

    let hit = Ray3::new(Vec3::new(-2.0, 0.0, 0.0), Vec3::X);
    assert_eq!(ray_sphere_first_hit(&hit, &sphere), Some(1.0));

    let miss = Ray3::new(Vec3::new(-2.0, 2.0, 0.0), Vec3::X);
    assert_eq!(ray_sphere_first_hit(&miss, &sphere), None);

    let inside = Ray3::new(Vec3::new(0.5, 0.0, 0.0), Vec3::X);
    assert_eq!(ray_sphere_first_hit(&inside, &sphere), Some(0.0));
}

#[test]
fn sphere_aabb_boundary_touching_counts_as_intersection() {
    let sphere = Sphere::new(Vec3::ZERO, 1.0);
    let touching = Aabb3::from_corners(Vec3::new(1.0, -0.5, -0.5), Vec3::new(2.0, 0.5, 0.5));
    let separate = Aabb3::from_corners(Vec3::new(1.01, -0.5, -0.5), Vec3::new(2.0, 0.5, 0.5));

    assert!(sphere_aabb3_intersects(&sphere, &touching));
    assert!(!sphere_aabb3_intersects(&sphere, &separate));
}

#[test]
fn frustum_aabb_intersection_free_function_matches_method_behavior() {
    let frustum = test_frustum();
    let inside = Aabb3::from_corners(Vec3::splat(-0.5), Vec3::splat(0.5));
    let outside = Aabb3::from_corners(Vec3::splat(2.0), Vec3::splat(3.0));

    assert!(frustum_aabb3_intersects(&frustum, &inside));
    assert!(!frustum_aabb3_intersects(&frustum, &outside));
}

#[test]
fn ray_plane_and_ray_triangle_queries_return_expected_hit_distance() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let ray = Ray3::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, -2.0, 0.0));
    assert_eq!(ray_plane_first_hit(&ray, &plane), Some(1.0));

    let tri = Triangle3::new(
        Vec3::new(-1.0, 0.0, -1.0),
        Vec3::new(1.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 1.0),
    );
    assert_eq!(ray_triangle3_first_hit(&ray, &tri), Some(1.0));
}

#[test]
fn segment_aabb_intersection_handles_boundary_touching() {
    let aabb = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(1.0));
    let touching = geometry::LineSegment3::new(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(0.0, 0.5, 0.5));
    let separate =
        geometry::LineSegment3::new(Vec3::new(-1.0, 2.0, 0.5), Vec3::new(-0.5, 2.0, 0.5));

    assert!(segment_aabb3_intersects(&touching, &aabb));
    assert!(!segment_aabb3_intersects(&separate, &aabb));
}
