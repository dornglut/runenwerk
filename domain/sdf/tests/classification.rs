use glam::Vec3;

use sdf::primitives::SdfSphere;
use sdf::queries::classify::{PointClassification, classify_point};

#[test]
fn classify_inside_outside_and_surface() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);

    assert_eq!(
        classify_point(&field, Vec3::new(0.0, 0.0, 0.0), 1e-4),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(&field, Vec3::new(1.0, 0.0, 0.0), 1e-4),
        PointClassification::OnSurface
    );
    assert_eq!(
        classify_point(&field, Vec3::new(2.0, 0.0, 0.0), 1e-4),
        PointClassification::Outside
    );
}

#[test]
fn classification_is_epsilon_aware() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let p = Vec3::new(1.0005, 0.0, 0.0);

    assert_eq!(
        classify_point(&field, p, 1e-3),
        PointClassification::OnSurface
    );
    assert_eq!(
        classify_point(&field, p, 1e-5),
        PointClassification::Outside
    );
}
