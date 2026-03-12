use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Translate<F> {
    pub field: F,
    pub offset: Vec3,
}

impl<F> Translate<F> {
    pub fn new(field: F, offset: Vec3) -> Self {
        Self { field, offset }
    }
}

impl<F> SdfField3 for Translate<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        self.field.sample(point - self.offset)
    }

    fn bounds(&self) -> FieldBounds {
        self.field.bounds().translated(self.offset)
    }
}
