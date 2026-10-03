use geometry::{Aabb2, Aabb3};
use glam::{Vec2, Vec3};

#[test]
fn aabb2_strict_constructor_preserves_ordering() {
    let aabb = Aabb2::new(Vec2::new(2.0, -1.0), Vec2::new(-2.0, 3.0));
    assert_eq!(aabb.min, Vec2::new(2.0, -1.0));
    assert_eq!(aabb.max, Vec2::new(-2.0, 3.0));
    assert!(!aabb.is_valid());
}

#[test]
fn aabb2_from_corners_normalizes_inputs() {
    let aabb = Aabb2::from_corners(Vec2::new(2.0, -1.0), Vec2::new(-2.0, 3.0));
    assert_eq!(aabb.min, Vec2::new(-2.0, -1.0));
    assert_eq!(aabb.max, Vec2::new(2.0, 3.0));
    assert!(aabb.is_valid());
}

#[test]
fn aabb2_center_size_extents_are_consistent() {
    let aabb = Aabb2::from_corners(Vec2::new(-2.0, 1.0), Vec2::new(4.0, 7.0));
    assert_eq!(aabb.center(), Vec2::new(1.0, 4.0));
    assert_eq!(aabb.size(), Vec2::new(6.0, 6.0));
    assert_eq!(aabb.extents(), Vec2::new(3.0, 3.0));
}

#[test]
fn aabb2_contains_and_intersection_are_inclusive() {
    let outer = Aabb2::from_corners(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0));
    let touching = Aabb2::from_corners(Vec2::new(2.0, -1.0), Vec2::new(3.0, 1.0));
    let inside = Aabb2::from_corners(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));

    assert!(outer.contains_point(Vec2::new(2.0, 0.0)));
    assert!(outer.contains_aabb(&inside));
    assert!(!outer.contains_aabb(&touching));
    assert!(outer.intersects(&touching));
}

#[test]
fn aabb2_union_and_expansion_work() {
    let a = Aabb2::from_corners(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
    let b = Aabb2::from_corners(Vec2::new(2.0, -2.0), Vec2::new(3.0, 4.0));
    let union = a.union(&b);

    assert_eq!(union.min, Vec2::new(-1.0, -2.0));
    assert_eq!(union.max, Vec2::new(3.0, 4.0));
    assert_eq!(
        a.expanded_by_point(Vec2::new(3.0, -4.0)).max,
        Vec2::new(3.0, 1.0)
    );
    assert_eq!(
        a.expanded_by_aabb(&b),
        Aabb2::from_corners(Vec2::new(-1.0, -2.0), Vec2::new(3.0, 4.0))
    );
}

#[test]
fn aabb2_from_points_handles_empty_and_negative_coordinates() {
    assert!(Aabb2::from_points([]).is_none());

    let points = [
        Vec2::new(-10.0, 3.0),
        Vec2::new(4.0, -8.0),
        Vec2::new(-2.0, 1.0),
    ];
    let aabb = Aabb2::from_points(points).expect("points should build an aabb");
    assert_eq!(aabb.min, Vec2::new(-10.0, -8.0));
    assert_eq!(aabb.max, Vec2::new(4.0, 3.0));
}

#[test]
fn aabb3_constructors_and_basic_queries() {
    let aabb = Aabb3::from_corners(Vec3::new(4.0, -1.0, 2.0), Vec3::new(-2.0, 3.0, -5.0));
    assert_eq!(aabb.min, Vec3::new(-2.0, -1.0, -5.0));
    assert_eq!(aabb.max, Vec3::new(4.0, 3.0, 2.0));
    assert_eq!(aabb.center(), Vec3::new(1.0, 1.0, -1.5));
    assert_eq!(aabb.size(), Vec3::new(6.0, 4.0, 7.0));
    assert_eq!(aabb.extents(), Vec3::new(3.0, 2.0, 3.5));
}

#[test]
fn aabb3_surface_area_volume_zero_size_and_disjoint_cases() {
    let flat = Aabb3::from_corners(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 3.0));
    assert_eq!(flat.volume(), 0.0);
    assert_eq!(flat.surface_area(), 12.0);

    let a = Aabb3::from_corners(Vec3::ZERO, Vec3::splat(1.0));
    let b = Aabb3::from_corners(Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0));
    let c = Aabb3::from_corners(Vec3::new(2.1, 2.1, 2.1), Vec3::new(3.0, 3.0, 3.0));

    assert!(a.intersects(&b));
    assert!(!a.intersects(&c));
    assert!(a.contains_point(Vec3::new(1.0, 0.5, 0.5)));
}
