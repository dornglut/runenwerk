use glam::{Quat, Vec3};

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

use super::map_bounded_aabb;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rotate<F> {
    pub field: F,
    pub rotation: Quat,
    inverse: Quat,
}

impl<F> Rotate<F> {
    pub fn new(field: F, rotation: Quat) -> Self {
        let rotation = rotation.normalize();
        let inverse = rotation.inverse();
        Self {
            field,
            rotation,
            inverse,
        }
    }
}

impl<F> SdfField3 for Rotate<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let local = self.inverse * point;
        self.field.sample(local)
    }

    fn bounds(&self) -> FieldBounds {
        map_bounded_aabb(self.field.bounds(), |point| self.rotation * point)
    }
}
