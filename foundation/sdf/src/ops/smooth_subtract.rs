use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SmoothSubtract<A, B> {
    pub left: A,
    pub right: B,
    pub smoothness: f32,
}

impl<A, B> SmoothSubtract<A, B> {
    pub fn new(left: A, right: B, smoothness: f32) -> Self {
        Self {
            left,
            right,
            smoothness,
        }
    }
}

impl<A, B> SdfField3 for SmoothSubtract<A, B>
where
    A: SdfField3,
    B: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let a = self.left.sample(point).distance;
        let b = self.right.sample(point).distance;
        let k = self.smoothness.max(0.0);
        if k <= f32::EPSILON {
            return SdfSample::new(a.max(-b));
        }

        let h = (0.5 - 0.5 * (b + a) / k).clamp(0.0, 1.0);
        let value = a + (-b - a) * h + k * h * (1.0 - h);
        SdfSample::new(value)
    }

    fn bounds(&self) -> FieldBounds {
        self.left.bounds()
    }
}
