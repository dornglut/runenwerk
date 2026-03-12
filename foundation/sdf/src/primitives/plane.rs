use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfPlane {
    pub normal: Vec3,
    pub distance: f32,
}

impl SdfPlane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        Self {
            normal,
            distance: -normal.dot(point),
        }
    }
}

impl SdfField3 for SdfPlane {
    fn sample(&self, point: Vec3) -> SdfSample {
        let normal_len_sq = self.normal.length_squared();
        if normal_len_sq <= f32::EPSILON {
            return SdfSample::new(self.distance);
        }

        // Return metric distance even when normal is not unit length.
        let signed = (self.normal.dot(point) + self.distance) / normal_len_sq.sqrt();
        SdfSample::new(signed)
    }

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }
}
