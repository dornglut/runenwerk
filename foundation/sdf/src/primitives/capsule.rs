use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfCapsule {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
}

impl SdfCapsule {
    pub fn new(start: Vec3, end: Vec3, radius: f32) -> Self {
        Self { start, end, radius }
    }
}

impl SdfField3 for SdfCapsule {
    fn sample(&self, point: Vec3) -> SdfSample {
        let radius = self.radius.max(0.0);
        let pa = point - self.start;
        let ba = self.end - self.start;
        let denom = ba.length_squared();
        let h = if denom <= f32::EPSILON {
            0.0
        } else {
            (pa.dot(ba) / denom).clamp(0.0, 1.0)
        };
        let closest = self.start + ba * h;
        SdfSample::new((point - closest).length() - radius)
    }

    fn bounds(&self) -> FieldBounds {
        let radius = self.radius.max(0.0);
        let extent = Vec3::splat(radius);
        let min = self.start.min(self.end) - extent;
        let max = self.start.max(self.end) + extent;
        FieldBounds::bounded(Aabb3::new(min, max))
    }
}
