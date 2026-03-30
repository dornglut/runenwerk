use geometry::Aabb3;
use glam::{Affine3A, Quat, Vec3};

use sdf::primitives::{SdfBox3, SdfSphere};
use sdf::transform::{Affine, Rotate, Scale, Translate};
use sdf::{FieldBounds, SdfField3};

const EPS: f32 = 1e-4;

#[test]
fn translate_remaps_samples_and_bounds() {
    let translated = Translate::new(SdfSphere::new(Vec3::ZERO, 1.0), Vec3::new(2.0, 0.0, 0.0));

    assert!((translated.sample(Vec3::new(2.0, 0.0, 0.0)).distance + 1.0).abs() < EPS);

    let FieldBounds::Bounded(bounds) = translated.bounds() else {
        panic!("translated sphere should stay bounded");
    };
    assert_eq!(
        bounds,
        Aabb3::from_corners(Vec3::new(1.0, -1.0, -1.0), Vec3::new(3.0, 1.0, 1.0))
    );
}

#[test]
fn rotate_remaps_sample_space() {
    let box3 = SdfBox3::new(Vec3::new(2.0, 0.0, 0.0), Vec3::splat(0.5));
    let rotated = Rotate::new(box3, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

    assert!(rotated.sample(Vec3::new(0.0, 2.0, 0.0)).distance < 0.0);

    let FieldBounds::Bounded(bounds) = rotated.bounds() else {
        panic!("rotated box should remain bounded");
    };
    assert!(bounds.contains_point(Vec3::new(0.0, 2.0, 0.0)));
}

#[test]
fn uniform_scale_updates_sample_and_bounds() {
    let scaled = Scale::new(SdfSphere::new(Vec3::ZERO, 1.0), 2.0);

    assert!((scaled.sample(Vec3::ZERO).distance + 2.0).abs() < EPS);
    assert!(scaled.sample(Vec3::new(2.0, 0.0, 0.0)).distance.abs() < EPS);

    let FieldBounds::Bounded(bounds) = scaled.bounds() else {
        panic!("scaled sphere should be bounded");
    };
    assert_eq!(bounds.min, Vec3::splat(-2.0));
    assert_eq!(bounds.max, Vec3::splat(2.0));
}

#[test]
fn affine_transform_supports_translation() {
    let transform = Affine3A::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let affine = Affine::new(SdfSphere::new(Vec3::ZERO, 1.0), transform);

    assert!((affine.sample(Vec3::new(1.0, 2.0, 3.0)).distance + 1.0).abs() < EPS);

    let FieldBounds::Bounded(bounds) = affine.bounds() else {
        panic!("affine transformed sphere should stay bounded");
    };
    assert_eq!(bounds.min, Vec3::new(0.0, 1.0, 2.0));
    assert_eq!(bounds.max, Vec3::new(2.0, 3.0, 4.0));
}
