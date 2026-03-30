use geometry::Ray3;
use glam::Vec3;

use crate::epsilon::DEFAULT_RAY_HIT_EPSILON;
use crate::field::SdfField3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RayHit {
    pub t: f32,
    pub position: Vec3,
    pub distance: f32,
    pub steps: u32,
}

pub fn raymarch_first_hit(
    field: &impl SdfField3,
    ray: &Ray3,
    max_steps: u32,
    max_distance: f32,
    epsilon: f32,
) -> Option<RayHit> {
    let dir_len = ray.direction.length();
    if dir_len <= f32::EPSILON {
        return None;
    }

    let direction = ray.direction / dir_len;
    let mut t = 0.0;
    let max_distance = max_distance.max(0.0);
    let epsilon = if epsilon > 0.0 {
        epsilon
    } else {
        DEFAULT_RAY_HIT_EPSILON
    };

    for step in 0..max_steps {
        if t > max_distance {
            return None;
        }

        let position = ray.origin + direction * t;
        let distance = field.sample(position).distance;
        if !distance.is_finite() {
            return None;
        }
        if distance <= epsilon {
            return Some(RayHit {
                t,
                position,
                distance,
                steps: step + 1,
            });
        }
        t += distance.max(epsilon);
    }

    None
}
