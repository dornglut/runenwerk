use geometry::classification::{
    FrustumPointClassification, PlaneAabbClassification, PlanePointClassification,
    classify_aabb_plane, classify_point_frustum, classify_point_plane,
};
use geometry::{Aabb3, Frustum, Plane};
use glam::Vec3;

fn unit_box_frustum() -> Frustum {
    Frustum::new(
        Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0),
        Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0),
        Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0),
        Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0),
    )
}

#[test]
fn classify_point_plane_reports_front_back_and_on_plane() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    assert_eq!(
        classify_point_plane(&plane, Vec3::new(0.0, 2.0, 0.0)),
        PlanePointClassification::Front
    );
    assert_eq!(
        classify_point_plane(&plane, Vec3::new(0.0, -2.0, 0.0)),
        PlanePointClassification::Back
    );
    assert_eq!(
        classify_point_plane(&plane, Vec3::new(1.0, 0.0, -1.0)),
        PlanePointClassification::OnPlane
    );
}

#[test]
fn classify_aabb_plane_reports_front_back_and_intersection() {
    let plane = Plane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let front = Aabb3::from_corners(Vec3::new(-1.0, 1.0, -1.0), Vec3::new(1.0, 2.0, 1.0));
    let back = Aabb3::from_corners(Vec3::new(-1.0, -2.0, -1.0), Vec3::new(1.0, -1.0, 1.0));
    let crossing = Aabb3::from_corners(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

    assert_eq!(
        classify_aabb_plane(&plane, &front),
        PlaneAabbClassification::Front
    );
    assert_eq!(
        classify_aabb_plane(&plane, &back),
        PlaneAabbClassification::Back
    );
    assert_eq!(
        classify_aabb_plane(&plane, &crossing),
        PlaneAabbClassification::Intersecting
    );
}

#[test]
fn classify_point_frustum_reports_inside_or_outside() {
    let frustum = unit_box_frustum();
    assert_eq!(
        classify_point_frustum(&frustum, Vec3::ZERO),
        FrustumPointClassification::Inside
    );
    assert_eq!(
        classify_point_frustum(&frustum, Vec3::new(2.0, 0.0, 0.0)),
        FrustumPointClassification::Outside
    );
}
