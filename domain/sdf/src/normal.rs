use glam::Vec3;

use crate::epsilon::DEFAULT_NORMAL_EPSILON;
use crate::field::SdfField3;
use crate::gradient::estimate_normal;

pub fn normal_at(field: &impl SdfField3, point: Vec3, epsilon: f32) -> Vec3 {
    estimate_normal(field, point, epsilon)
}

pub fn normal_at_default(field: &impl SdfField3, point: Vec3) -> Vec3 {
    normal_at(field, point, DEFAULT_NORMAL_EPSILON)
}
