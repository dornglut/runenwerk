use geometry::Aabb3;
use glam::{Vec2, Vec3};

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfTorus {
    pub center: Vec3,
    pub major_radius: f32,
    pub minor_radius: f32,
}

impl SdfTorus {
    pub fn new(center: Vec3, major_radius: f32, minor_radius: f32) -> Self {
        Self {
            center,
            major_radius,
            minor_radius,
        }
    }
}

impl SdfField3 for SdfTorus {
    fn sample(&self, point: Vec3) -> SdfSample {
        let major = self.major_radius.max(0.0);
        let minor = self.minor_radius.max(0.0);
        let p = point - self.center;
        let q = Vec2::new(Vec2::new(p.x, p.z).length() - major, p.y);
        SdfSample::new(q.length() - minor)
    }

    fn bounds(&self) -> FieldBounds {
        let major = self.major_radius.max(0.0);
        let minor = self.minor_radius.max(0.0);
        let ring = major + minor;
        let extents = Vec3::new(ring, minor, ring);
        FieldBounds::bounded(Aabb3::from_center_extents(self.center, extents))
    }
}
