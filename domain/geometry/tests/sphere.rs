use geometry::Sphere;
use glam::Vec3;

#[test]
fn sphere_contains_point_is_inclusive() {
    let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 2.0);
    assert!(sphere.contains_point(Vec3::new(3.0, 2.0, 3.0)));
    assert!(sphere.contains_point(Vec3::new(1.0, 2.0, 3.0)));
    assert!(!sphere.contains_point(Vec3::new(3.1, 2.0, 3.0)));
}

#[test]
fn sphere_sphere_overlap_is_inclusive() {
    let a = Sphere::new(Vec3::ZERO, 1.0);
    let b = Sphere::new(Vec3::new(2.0, 0.0, 0.0), 1.0);
    let c = Sphere::new(Vec3::new(2.1, 0.0, 0.0), 1.0);

    assert!(a.intersects_sphere(&b));
    assert!(!a.intersects_sphere(&c));
}

#[test]
fn sphere_aabb_conversion_is_center_plus_radius_extent() {
    let sphere = Sphere::new(Vec3::new(2.0, -1.0, 0.5), 3.0);
    let aabb = sphere.aabb();
    assert_eq!(aabb.min, Vec3::new(-1.0, -4.0, -2.5));
    assert_eq!(aabb.max, Vec3::new(5.0, 2.0, 3.5));
}
