use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfBox3 {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl SdfBox3 {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            center,
            half_extents,
        }
    }
}

impl SdfField3 for SdfBox3 {
    fn sample(&self, point: Vec3) -> SdfSample {
        let half_extents = self.half_extents.max(Vec3::ZERO);
        let q = (point - self.center).abs() - half_extents;
        let outside = q.max(Vec3::ZERO).length();
        let inside = q.x.max(q.y).max(q.z).min(0.0);
        SdfSample::new(outside + inside)
    }

    fn bounds(&self) -> FieldBounds {
        FieldBounds::bounded(Aabb3::from_center_extents(
            self.center,
            self.half_extents.max(Vec3::ZERO),
        ))
    }
}
