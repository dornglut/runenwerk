use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Subtract<A, B> {
    pub left: A,
    pub right: B,
}

impl<A, B> Subtract<A, B> {
    pub fn new(left: A, right: B) -> Self {
        Self { left, right }
    }
}

impl<A, B> SdfField3 for Subtract<A, B>
where
    A: SdfField3,
    B: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let a = self.left.sample(point).distance;
        let b = self.right.sample(point).distance;
        SdfSample::new(a.max(-b))
    }

    fn bounds(&self) -> FieldBounds {
        // Conservative policy: subtraction cannot expand beyond the left operand.
        self.left.bounds()
    }
}
