use glam::Vec3;

use crate::epsilon::{DEFAULT_MAX_PROJECT_STEPS, DEFAULT_NORMAL_EPSILON, DEFAULT_PROJECT_EPSILON};
use crate::field::SdfField3;
use crate::gradient::estimate_normal;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProjectSettings {
    pub max_steps: u32,
    pub surface_epsilon: f32,
    pub normal_epsilon: f32,
    pub max_step: f32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_MAX_PROJECT_STEPS,
            surface_epsilon: DEFAULT_PROJECT_EPSILON,
            normal_epsilon: DEFAULT_NORMAL_EPSILON,
            max_step: 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProjectHit {
    pub position: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub iterations: u32,
}

pub fn project_point_to_surface(
    field: &impl SdfField3,
    start: Vec3,
    settings: ProjectSettings,
) -> Option<ProjectHit> {
    let mut point = start;
    let surface_epsilon = settings.surface_epsilon.max(f32::EPSILON);
    let max_step = settings.max_step.abs().max(surface_epsilon);

    for iteration in 0..settings.max_steps {
        let sample = field.sample(point);
        if !sample.distance.is_finite() {
            return None;
        }

        if sample.distance.abs() <= surface_epsilon {
            let normal = estimate_normal(field, point, settings.normal_epsilon);
            return Some(ProjectHit {
                position: point,
                normal,
                distance: sample.distance,
                iterations: iteration + 1,
            });
        }

        let normal = estimate_normal(field, point, settings.normal_epsilon);
        let step = (-sample.distance).clamp(-max_step, max_step);
        point += normal * step;
    }

    None
}
