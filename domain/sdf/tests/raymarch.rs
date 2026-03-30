use geometry::Ray3;
use glam::Vec3;

use sdf::primitives::SdfSphere;
use sdf::queries::raymarch::raymarch_first_hit;

#[test]
fn raymarch_hits_sphere() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::X);

    let hit = raymarch_first_hit(&field, &ray, 128, 10.0, 1e-3).expect("ray should hit sphere");
    assert!((hit.t - 2.0).abs() < 1e-2);
    assert!(hit.position.distance(Vec3::new(-1.0, 0.0, 0.0)) < 1e-2);
}

#[test]
fn raymarch_reports_miss() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(-3.0, 3.0, 0.0), Vec3::X);
    assert!(raymarch_first_hit(&field, &ray, 128, 10.0, 1e-3).is_none());
}

#[test]
fn raymarch_honors_max_distance() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::X);
    assert!(raymarch_first_hit(&field, &ray, 128, 1.5, 1e-3).is_none());
}

#[test]
fn raymarch_honors_max_steps() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::X);
    assert!(raymarch_first_hit(&field, &ray, 1, 10.0, 1e-3).is_none());
}

#[test]
fn raymarch_hits_immediately_when_starting_inside() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::X);

    let hit = raymarch_first_hit(&field, &ray, 128, 10.0, 1e-3)
        .expect("inside point should register a hit");
    assert!(hit.t <= 1e-6);
}
