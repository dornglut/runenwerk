use glam::Vec3;

use sdf::primitives::{SdfBox3, SdfCapsule, SdfCylinder, SdfPlane, SdfSphere, SdfTorus};
use sdf::{FieldBounds, SdfField3};

const EPS: f32 = 1e-4;

#[test]
fn sphere_signed_distance_and_bounds_are_correct() {
    let sphere = SdfSphere::new(Vec3::ZERO, 1.0);

    assert!((sphere.sample(Vec3::ZERO).distance + 1.0).abs() < EPS);
    assert!(sphere.sample(Vec3::new(1.0, 0.0, 0.0)).distance.abs() < EPS);
    assert!((sphere.sample(Vec3::new(3.0, 0.0, 0.0)).distance - 2.0).abs() < EPS);

    let FieldBounds::Bounded(bounds) = sphere.bounds() else {
        panic!("sphere should be bounded");
    };
    assert_eq!(bounds.min, Vec3::splat(-1.0));
    assert_eq!(bounds.max, Vec3::splat(1.0));
}

#[test]
fn box_signed_distance_matches_expected_samples() {
    let box3 = SdfBox3::new(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));

    assert!((box3.sample(Vec3::ZERO).distance + 1.0).abs() < EPS);
    assert!(box3.sample(Vec3::new(1.0, 0.0, 0.0)).distance.abs() < EPS);
    assert!((box3.sample(Vec3::new(3.0, 0.0, 0.0)).distance - 2.0).abs() < EPS);
}

#[test]
fn capsule_signed_distance_and_bounds_are_correct() {
    let capsule = SdfCapsule::new(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.5);

    assert!((capsule.sample(Vec3::ZERO).distance + 0.5).abs() < EPS);
    assert!(capsule.sample(Vec3::new(0.5, 0.0, 0.0)).distance.abs() < EPS);
    assert!((capsule.sample(Vec3::new(2.0, 0.0, 0.0)).distance - 1.5).abs() < EPS);

    let FieldBounds::Bounded(bounds) = capsule.bounds() else {
        panic!("capsule should be bounded");
    };
    assert_eq!(bounds.min, Vec3::new(-0.5, -1.5, -0.5));
    assert_eq!(bounds.max, Vec3::new(0.5, 1.5, 0.5));
}

#[test]
fn plane_signed_distance_is_metric_for_unit_normals_and_unbounded() {
    let plane = SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y);

    assert!((plane.sample(Vec3::new(0.0, 2.0, 0.0)).distance - 2.0).abs() < EPS);
    assert!((plane.sample(Vec3::new(0.0, -2.0, 0.0)).distance + 2.0).abs() < EPS);
    assert_eq!(plane.bounds(), FieldBounds::Unbounded);
}

#[test]
fn torus_and_cylinder_have_expected_distances() {
    let torus = SdfTorus::new(Vec3::ZERO, 2.0, 0.5);
    assert!(torus.sample(Vec3::new(2.5, 0.0, 0.0)).distance.abs() < EPS);
    assert!((torus.sample(Vec3::new(2.0, 0.0, 0.0)).distance + 0.5).abs() < EPS);

    let cylinder = SdfCylinder::new(Vec3::ZERO, 1.0, 2.0);
    assert!((cylinder.sample(Vec3::ZERO).distance + 1.0).abs() < EPS);
    assert!(cylinder.sample(Vec3::new(1.0, 0.0, 0.0)).distance.abs() < EPS);
    assert!(cylinder.sample(Vec3::new(0.0, 3.0, 0.0)).distance > 0.0);
}
