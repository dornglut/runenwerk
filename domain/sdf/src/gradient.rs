use glam::Vec3;

use crate::epsilon::DEFAULT_NORMAL_EPSILON;
use crate::field::SdfField3;
use crate::util::finite_difference::central_difference;

pub fn estimate_gradient(field: &impl SdfField3, point: Vec3, epsilon: f32) -> Vec3 {
    central_difference(|p| field.sample(p).distance, point, epsilon)
}

pub fn estimate_gradient_default(field: &impl SdfField3, point: Vec3) -> Vec3 {
    estimate_gradient(field, point, DEFAULT_NORMAL_EPSILON)
}

pub fn estimate_normal(field: &impl SdfField3, point: Vec3, epsilon: f32) -> Vec3 {
    let gradient = estimate_gradient(field, point, epsilon);
    if gradient.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        gradient.normalize()
    }
}

pub fn estimate_normal_default(field: &impl SdfField3, point: Vec3) -> Vec3 {
    estimate_normal(field, point, DEFAULT_NORMAL_EPSILON)
}
