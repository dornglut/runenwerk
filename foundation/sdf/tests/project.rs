use glam::Vec3;

use sdf::primitives::{SdfPlane, SdfSphere};
use sdf::queries::project::{ProjectSettings, project_point_to_surface};

#[test]
fn project_from_outside_converges_to_surface() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let hit =
        project_point_to_surface(&field, Vec3::new(2.5, 0.0, 0.0), ProjectSettings::default())
            .expect("projection should converge");

    assert!(hit.position.distance(Vec3::new(1.0, 0.0, 0.0)) < 1e-3);
    assert!(hit.distance.abs() < 1e-3);
}

#[test]
fn project_from_inside_converges_to_surface() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let hit =
        project_point_to_surface(&field, Vec3::new(0.2, 0.0, 0.0), ProjectSettings::default())
            .expect("projection should converge");

    assert!((hit.position.length() - 1.0).abs() < 1e-3);
    assert!(hit.distance.abs() < 1e-3);
}

#[test]
fn projection_can_fail_when_iteration_budget_is_too_small() {
    let field = SdfSphere::new(Vec3::ZERO, 1.0);
    let settings = ProjectSettings {
        max_steps: 1,
        max_step: 0.01,
        ..ProjectSettings::default()
    };

    assert!(project_point_to_surface(&field, Vec3::new(10.0, 0.0, 0.0), settings).is_none());
}

#[test]
fn project_on_plane_is_accurate() {
    let field = SdfPlane::from_point_normal(Vec3::ZERO, Vec3::Y);
    let hit =
        project_point_to_surface(&field, Vec3::new(1.0, 3.0, 2.0), ProjectSettings::default())
            .expect("projection should converge on plane");

    assert!(hit.position.distance(Vec3::new(1.0, 0.0, 2.0)) < 1e-3);
    assert!(hit.distance.abs() < 1e-3);
}
