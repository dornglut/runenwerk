use glam::Vec3;

use crate::field::SdfField3;

use super::project::{ProjectSettings, project_point_to_surface};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClosestPointHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub signed_distance: f32,
    pub iterations: u32,
}

pub fn closest_point_on_surface(
    field: &impl SdfField3,
    start: Vec3,
    settings: ProjectSettings,
) -> Option<ClosestPointHit> {
    let hit = project_point_to_surface(field, start, settings)?;
    Some(ClosestPointHit {
        point: hit.position,
        normal: hit.normal,
        signed_distance: hit.distance,
        iterations: hit.iterations,
    })
}
