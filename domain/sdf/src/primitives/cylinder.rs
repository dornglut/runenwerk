use geometry::Aabb3;
use glam::Vec2;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfCylinder {
    pub center: Vec3,
    pub radius: f32,
    pub half_height: f32,
}

impl SdfCylinder {
    pub fn new(center: Vec3, radius: f32, half_height: f32) -> Self {
        Self {
            center,
            radius,
            half_height,
        }
    }
}

impl SdfField3 for SdfCylinder {
    fn sample(&self, point: Vec3) -> SdfSample {
        let radius = self.radius.max(0.0);
        let half_height = self.half_height.max(0.0);
        let p = point - self.center;
        let d = Vec2::new(
            Vec2::new(p.x, p.z).length() - radius,
            p.y.abs() - half_height,
        );
        let outside = d.max(Vec2::ZERO).length();
        let inside = d.x.max(d.y).min(0.0);
        SdfSample::new(outside + inside)
    }

    fn bounds(&self) -> FieldBounds {
        let extents = Vec3::new(
            self.radius.max(0.0),
            self.half_height.max(0.0),
            self.radius.max(0.0),
        );
        FieldBounds::bounded(Aabb3::from_center_extents(self.center, extents))
    }
}
