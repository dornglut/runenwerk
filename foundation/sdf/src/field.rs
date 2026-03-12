use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::sample::SdfSample;

pub trait SdfField3 {
    fn sample(&self, point: Vec3) -> SdfSample;

    fn bounds(&self) -> FieldBounds {
        FieldBounds::Unbounded
    }
}
