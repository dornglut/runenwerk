use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Intersect<A, B> {
    pub left: A,
    pub right: B,
}

impl<A, B> Intersect<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }
}

impl<A, B> SdfField3 for Intersect<A, B>
where
    A: SdfField3,
    B: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let a = self.left.sample(point).distance;
        let b = self.right.sample(point).distance;
        SdfSample::new(a.max(b))
    }

    fn bounds(&self) -> FieldBounds {
        self.left.bounds().intersection(self.right.bounds())
    }
}
