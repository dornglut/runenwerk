use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl SdfSphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl SdfField3 for SdfSphere {
    fn sample(&self, point: Vec3) -> SdfSample {
        let radius = self.radius.max(0.0);
        SdfSample::new((point - self.center).length() - radius)
    }

    fn bounds(&self) -> FieldBounds {
        let extent = Vec3::splat(self.radius.max(0.0));
        FieldBounds::bounded(Aabb3::from_center_extents(self.center, extent))
    }
}
