use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

use super::map_bounded_aabb;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Scale<F> {
    pub field: F,
    pub scale: f32,
}

impl<F> Scale<F> {
    pub fn new(field: F, scale: f32) -> Self {
        Self { field, scale }
    }
}

impl<F> SdfField3 for Scale<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let abs_scale = self.scale.abs();
        if abs_scale <= f32::EPSILON {
            return SdfSample::new(self.field.sample(Vec3::ZERO).distance);
        }

        let local = point / self.scale;
        let local_sample = self.field.sample(local);
        SdfSample::new(local_sample.distance * abs_scale)
    }

    fn bounds(&self) -> FieldBounds {
        map_bounded_aabb(self.field.bounds(), |point| point * self.scale)
    }
}
