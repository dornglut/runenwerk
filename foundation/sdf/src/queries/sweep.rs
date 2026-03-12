use glam::Vec3;

use crate::epsilon::{DEFAULT_CLASSIFY_EPSILON, DEFAULT_NORMAL_EPSILON};
use crate::field::SdfField3;
use crate::gradient::estimate_normal;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SweepSettings {
    pub steps: u32,
    pub surface_epsilon: f32,
    pub normal_epsilon: f32,
}

impl Default for SweepSettings {
    fn default() -> Self {
        Self {
            steps: 64,
            surface_epsilon: DEFAULT_CLASSIFY_EPSILON,
            normal_epsilon: DEFAULT_NORMAL_EPSILON,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SweepHit {
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub penetration: f32,
}

pub fn sweep_sphere(
    field: &impl SdfField3,
    start: Vec3,
    end: Vec3,
    radius: f32,
    settings: SweepSettings,
) -> Option<SweepHit> {
    let steps = settings.steps.max(1);
    let radius = radius.max(0.0);
    let threshold = radius + settings.surface_epsilon.max(0.0);

    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let position = start.lerp(end, t);
        let distance = field.sample(position).distance;
        if !distance.is_finite() {
            return None;
        }

        if distance <= threshold {
            let normal = estimate_normal(field, position, settings.normal_epsilon);
            return Some(SweepHit {
                t,
                position,
                normal,
                penetration: (radius - distance).max(0.0),
            });
        }
    }

    None
}
