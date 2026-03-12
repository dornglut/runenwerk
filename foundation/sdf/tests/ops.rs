use glam::Vec3;

use sdf::ops::{Intersect, SmoothIntersect, SmoothSubtract, SmoothUnion, Subtract, Union};
use sdf::primitives::{SdfPlane, SdfSphere};
use sdf::{FieldBounds, SdfField3};

const EPS: f32 = 1e-4;

#[test]
fn union_uses_min_distance() {
    let left = SdfSphere::new(Vec3::ZERO, 1.0);
    let right = SdfSphere::new(Vec3::new(2.0, 0.0, 0.0), 1.0);
    let union = Union::new(left, right);

    let point = Vec3::new(0.5, 0.0, 0.0);
    let expected = left
        .sample(point)
        .distance
        .min(right.sample(point).distance);
    assert!((union.sample(point).distance - expected).abs() < EPS);
}

#[test]
fn subtract_and_intersect_have_expected_sign_behavior() {
    let left = SdfSphere::new(Vec3::ZERO, 2.0);
    let right = SdfSphere::new(Vec3::ZERO, 1.0);

    let subtract = Subtract::new(left, right);
    assert!(subtract.sample(Vec3::ZERO).distance > 0.0);
    assert!(subtract.sample(Vec3::new(1.5, 0.0, 0.0)).distance < 0.0);

    let intersect = Intersect::new(left, right);
    assert!(intersect.sample(Vec3::ZERO).distance < 0.0);
    assert!(intersect.sample(Vec3::new(1.5, 0.0, 0.0)).distance > 0.0);
}

#[test]
fn bounds_compose_conservatively() {
    let a = SdfSphere::new(Vec3::ZERO, 1.0);
    let b = SdfSphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);

    let union = Union::new(a, b);
    let subtract = Subtract::new(a, b);
    let intersect = Intersect::new(a, b);

    let FieldBounds::Bounded(union_bounds) = union.bounds() else {
        panic!("union should be bounded");
    };
    assert_eq!(union_bounds.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(union_bounds.max, Vec3::new(2.5, 1.0, 1.0));

    assert_eq!(subtract.bounds(), a.bounds());

    let FieldBounds::Bounded(intersect_bounds) = intersect.bounds() else {
        panic!("intersect should be bounded");
    };
    assert!(intersect_bounds.min.x >= -1.0 && intersect_bounds.max.x <= 2.5);

    let unbounded_union = Union::new(a, SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y));
    assert_eq!(unbounded_union.bounds(), FieldBounds::Unbounded);
}

#[test]
fn smooth_ops_fall_back_to_hard_ops_when_smoothness_is_zero() {
    let left = SdfSphere::new(Vec3::ZERO, 1.0);
    let right = SdfSphere::new(Vec3::new(1.0, 0.0, 0.0), 1.0);
    let point = Vec3::new(0.25, 0.0, 0.0);

    let hard_union = Union::new(left, right).sample(point).distance;
    let smooth_union = SmoothUnion::new(left, right, 0.0).sample(point).distance;
    assert!((hard_union - smooth_union).abs() < EPS);

    let hard_subtract = Subtract::new(left, right).sample(point).distance;
    let smooth_subtract = SmoothSubtract::new(left, right, 0.0).sample(point).distance;
    assert!((hard_subtract - smooth_subtract).abs() < EPS);

    let hard_intersect = Intersect::new(left, right).sample(point).distance;
    let smooth_intersect = SmoothIntersect::new(left, right, 0.0)
        .sample(point)
        .distance;
    assert!((hard_intersect - smooth_intersect).abs() < EPS);
}

#[test]
fn smooth_union_softens_the_join_region() {
    let left = SdfSphere::new(Vec3::new(-0.5, 0.0, 0.0), 1.0);
    let right = SdfSphere::new(Vec3::new(0.5, 0.0, 0.0), 1.0);
    let point = Vec3::ZERO;

    let hard = Union::new(left, right).sample(point).distance;
    let smooth = SmoothUnion::new(left, right, 0.5).sample(point).distance;
    assert!(smooth <= hard);
}
