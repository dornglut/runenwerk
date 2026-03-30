use glam::Vec3;

use crate::epsilon::DEFAULT_CLASSIFY_EPSILON;
use crate::field::SdfField3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PointClassification {
    Inside,
    OnSurface,
    Outside,
}

pub fn classify_point(field: &impl SdfField3, point: Vec3, epsilon: f32) -> PointClassification {
    let distance = field.sample(point).distance;
    let epsilon = if epsilon > 0.0 {
        epsilon
    } else {
        DEFAULT_CLASSIFY_EPSILON
    };
    if distance < -epsilon {
        PointClassification::Inside
    } else if distance > epsilon {
        PointClassification::Outside
    } else {
        PointClassification::OnSurface
    }
}

pub fn is_inside(field: &impl SdfField3, point: Vec3, epsilon: f32) -> bool {
    classify_point(field, point, epsilon) == PointClassification::Inside
}

pub fn is_outside(field: &impl SdfField3, point: Vec3, epsilon: f32) -> bool {
    classify_point(field, point, epsilon) == PointClassification::Outside
}
