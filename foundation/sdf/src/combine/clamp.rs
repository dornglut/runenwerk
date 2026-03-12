use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClampDistance<F> {
    pub field: F,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl<F> ClampDistance<F> {
    pub fn new(field: F, min_distance: f32, max_distance: f32) -> Self {
        Self {
            field,
            min_distance,
            max_distance,
        }
    }
}

impl<F> SdfField3 for ClampDistance<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let (min_distance, max_distance) = if self.min_distance <= self.max_distance {
            (self.min_distance, self.max_distance)
        } else {
            (self.max_distance, self.min_distance)
        };

        let distance = self.field.sample(point).distance;
        SdfSample::new(distance.clamp(min_distance, max_distance))
    }

    fn bounds(&self) -> FieldBounds {
        self.field.bounds()
    }
}
