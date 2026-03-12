use geometry::Ray3;
use glam::Vec3;

use sdf::combine::{DomainWarp, Mirror, MirrorAxes, Repeat};
use sdf::ops::Union;
use sdf::primitives::{SdfCapsule, SdfPlane, SdfSphere};
use sdf::queries::raymarch::raymarch_first_hit;
use sdf::queries::sweep::{SweepSettings, sweep_sphere};
use sdf::{FieldBounds, SdfField3};

#[test]
fn degenerate_capsule_behaves_like_sphere() {
    let capsule = SdfCapsule::new(Vec3::ZERO, Vec3::ZERO, 1.0);
    let sphere = SdfSphere::new(Vec3::ZERO, 1.0);

    let point = Vec3::new(1.5, -0.25, 0.5);
    let dc = capsule.sample(point).distance;
    let ds = sphere.sample(point).distance;
    assert!((dc - ds).abs() < 1e-5);
}

#[test]
fn unbounded_fields_propagate_through_union_bounds() {
    let bounded = SdfSphere::new(Vec3::ZERO, 1.0);
    let unbounded = SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let union = Union::new(bounded, unbounded);

    assert_eq!(union.bounds(), FieldBounds::Unbounded);
}

#[test]
fn raymarch_with_zero_direction_returns_none() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let ray = Ray3::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::ZERO);
    assert!(raymarch_first_hit(&field, &ray, 64, 10.0, 1e-3).is_none());
}

#[test]
fn sweep_sphere_reports_first_contact() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let hit = sweep_sphere(
        &field,
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        0.5,
        SweepSettings::default(),
    )
    .expect("sweep should hit");

    assert!(hit.t < 0.5);
    assert!(hit.normal.dot(Vec3::new(-1.0, 0.0, 0.0)) > 0.8);
}

#[test]
fn combine_wrappers_are_stable_for_basic_samples() {
    let base = SdfSphere::new(Vec3::ZERO, 0.5);
    let repeated = Repeat::new(base, Vec3::new(2.0, 0.0, 0.0));
    let mirrored = Mirror::new(repeated, MirrorAxes::new(true, false, false));
    let warped = DomainWarp::new(
        mirrored,
        Vec3::new(0.1, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::ZERO,
    );

    let sample = warped.sample(Vec3::new(1.25, 0.0, 0.0));
    assert!(sample.distance.is_finite());
}
