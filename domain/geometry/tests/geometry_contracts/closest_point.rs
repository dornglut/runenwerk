use geometry::closest_point::{
    closest_point_on_aabb2, closest_point_on_aabb3, closest_point_on_segment2,
    closest_point_on_segment3, closest_point_on_sphere,
};
use geometry::{Aabb2, Aabb3, LineSegment2, LineSegment3, Sphere};
use glam::{Vec2, Vec3};

#[test]
fn closest_point_on_aabb_clamps_to_bounds() {
    let aabb2 = Aabb2::from_corners(Vec2::new(-1.0, -2.0), Vec2::new(3.0, 4.0));
    assert_eq!(
        closest_point_on_aabb2(&aabb2, Vec2::new(10.0, -10.0)),
        Vec2::new(3.0, -2.0)
    );

    let aabb3 = Aabb3::from_corners(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        closest_point_on_aabb3(&aabb3, Vec3::new(0.5, 4.0, -2.0)),
        Vec3::new(0.5, 2.0, -1.0)
    );
}

#[test]
fn closest_point_on_segment_handles_range_and_degenerate_segments() {
    let seg2 = LineSegment2::new(Vec2::ZERO, Vec2::new(2.0, 0.0));
    assert_eq!(
        closest_point_on_segment2(&seg2, Vec2::new(1.0, 1.0)),
        Vec2::new(1.0, 0.0)
    );
    assert_eq!(
        closest_point_on_segment2(&seg2, Vec2::new(3.0, 0.0)),
        Vec2::new(2.0, 0.0)
    );

    let degenerate2 = LineSegment2::new(Vec2::new(1.0, 2.0), Vec2::new(1.0, 2.0));
    assert_eq!(
        closest_point_on_segment2(&degenerate2, Vec2::new(3.0, 4.0)),
        Vec2::new(1.0, 2.0)
    );

    let seg3 = LineSegment3::new(Vec3::ZERO, Vec3::new(0.0, 2.0, 0.0));
    assert_eq!(
        closest_point_on_segment3(&seg3, Vec3::new(1.0, 1.0, 2.0)),
        Vec3::new(0.0, 1.0, 0.0)
    );
}

#[test]
fn closest_point_on_sphere_projects_radially() {
    let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 2.0);
    let p = closest_point_on_sphere(&sphere, Vec3::new(5.0, 2.0, 3.0));
    assert_eq!(p, Vec3::new(3.0, 2.0, 3.0));

    let at_center = closest_point_on_sphere(&sphere, sphere.center);
    assert_eq!(at_center, sphere.center);
}
