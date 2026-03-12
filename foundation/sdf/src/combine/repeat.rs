use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Repeat<F> {
    pub field: F,
    pub period: Vec3,
}

impl<F> Repeat<F> {
    pub fn new(field: F, period: Vec3) -> Self {
        Self { field, period }
    }
}

impl<F> SdfField3 for Repeat<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let x = repeat_axis(point.x, self.period.x);
        let y = repeat_axis(point.y, self.period.y);
        let z = repeat_axis(point.z, self.period.z);
        self.field.sample(Vec3::new(x, y, z))
    }

    fn bounds(&self) -> FieldBounds {
        // Repetition tiles the field infinitely in at least one axis.
        FieldBounds::Unbounded
    }
}

fn repeat_axis(value: f32, period: f32) -> f32 {
    let size = period.abs();
    if size <= f32::EPSILON {
        value
    } else {
        (value + size * 0.5).rem_euclid(size) - size * 0.5
    }
}
