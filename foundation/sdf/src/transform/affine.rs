use glam::{Affine3A, Vec3};

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

use super::map_bounded_aabb;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Affine<F> {
    pub field: F,
    pub transform: Affine3A,
    inverse: Affine3A,
    distance_scale: f32,
}

impl<F> Affine<F> {
    pub fn new(field: F, transform: Affine3A) -> Self {
        let inverse = transform.inverse();
        let sx = transform.matrix3.x_axis.length();
        let sy = transform.matrix3.y_axis.length();
        let sz = transform.matrix3.z_axis.length();
        let min_scale = sx.min(sy).min(sz);
        let distance_scale = if min_scale.is_finite() {
            min_scale.max(0.0)
        } else {
            1.0
        };

        Self {
            field,
            transform,
            inverse,
            distance_scale,
        }
    }
}

impl<F> SdfField3 for Affine<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let local = self.inverse.transform_point3(point);
        let local_distance = self.field.sample(local).distance;
        SdfSample::new(local_distance * self.distance_scale)
    }

    fn bounds(&self) -> FieldBounds {
        map_bounded_aabb(self.field.bounds(), |point| {
            self.transform.transform_point3(point)
        })
    }
}
