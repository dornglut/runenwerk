use geometry::intersection::{ray_aabb2_first_hit, ray_aabb3_first_hit};
use geometry::{Aabb2, Aabb3, Ray2, Ray3};
use glam::{Vec2, Vec3};

#[test]
fn ray2_at_returns_point_along_ray() {
    let ray = Ray2::new(Vec2::new(1.0, 2.0), Vec2::new(3.0, -1.0));
    assert_eq!(ray.at(2.0), Vec2::new(7.0, 0.0));
}

#[test]
fn ray3_at_returns_point_along_ray() {
    let ray = Ray3::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(-1.0, 0.5, 2.0));
    assert_eq!(ray.at(3.0), Vec3::new(-2.0, 3.5, 9.0));
}

#[test]
fn ray_aabb2_hit_miss_inside_grazing_and_reverse_direction() {
    let aabb = Aabb2::from_corners(Vec2::ZERO, Vec2::splat(2.0));

    let hit = Ray2::new(Vec2::new(-1.0, 1.0), Vec2::new(1.0, 0.0));
    assert_eq!(ray_aabb2_first_hit(&hit, &aabb), Some(1.0));

    let miss = Ray2::new(Vec2::new(-1.0, 3.0), Vec2::new(1.0, 0.0));
    assert_eq!(ray_aabb2_first_hit(&miss, &aabb), None);

    let inside = Ray2::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0));
    assert_eq!(ray_aabb2_first_hit(&inside, &aabb), Some(0.0));

    let grazing = Ray2::new(Vec2::new(-1.0, 2.0), Vec2::new(1.0, 0.0));
    assert_eq!(ray_aabb2_first_hit(&grazing, &aabb), Some(1.0));

    let reverse = Ray2::new(Vec2::new(3.0, 1.0), Vec2::new(-1.0, 0.0));
    assert_eq!(ray_aabb2_first_hit(&reverse, &aabb), Some(1.0));
}

#[test]
fn ray_aabb3_hit_miss_inside_grazing_and_reverse_direction() {
    let aabb = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(2.0));

    let hit = Ray3::new(Vec3::new(-1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&hit, &aabb), Some(1.0));

    let miss = Ray3::new(Vec3::new(-1.0, 3.0, 1.0), Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&miss, &aabb), None);

    let inside = Ray3::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.0, 1.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&inside, &aabb), Some(0.0));

    let grazing = Ray3::new(Vec3::new(-1.0, 2.0, 2.0), Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&grazing, &aabb), Some(1.0));

    let reverse = Ray3::new(Vec3::new(3.0, 1.0, 1.0), Vec3::new(-1.0, 0.0, 0.0));
    assert_eq!(ray_aabb3_first_hit(&reverse, &aabb), Some(1.0));
}

#[test]
fn ray_aabb_hit_distance_depends_on_direction_scale() {
    let aabb = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(2.0));
    let ray_unit = Ray3::new(Vec3::new(-2.0, 1.0, 1.0), Vec3::X);
    let ray_scaled = Ray3::new(Vec3::new(-2.0, 1.0, 1.0), Vec3::X * 2.0);

    let unit_t = ray_aabb3_first_hit(&ray_unit, &aabb).expect("unit ray should hit");
    let scaled_t = ray_aabb3_first_hit(&ray_scaled, &aabb).expect("scaled ray should hit");

    assert_eq!(unit_t, 2.0);
    assert_eq!(scaled_t, 1.0);
}
