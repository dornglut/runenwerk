use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DomainWarp<F> {
    pub field: F,
    pub amplitude: Vec3,
    pub frequency: Vec3,
    pub phase: Vec3,
}

impl<F> DomainWarp<F> {
    pub fn new(field: F, amplitude: Vec3, frequency: Vec3, phase: Vec3) -> Self {
        Self {
            field,
            amplitude,
            frequency,
            phase,
        }
    }

    fn warp_offset(&self, point: Vec3) -> Vec3 {
        let wave = Vec3::new(
            (point.x * self.frequency.x + self.phase.x).sin(),
            (point.y * self.frequency.y + self.phase.y).sin(),
            (point.z * self.frequency.z + self.phase.z).sin(),
        );
        wave * self.amplitude
    }
}

impl<F> SdfField3 for DomainWarp<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        self.field.sample(point + self.warp_offset(point))
    }

    fn bounds(&self) -> FieldBounds {
        match self.field.bounds() {
            FieldBounds::Unbounded => FieldBounds::Unbounded,
            FieldBounds::Bounded(aabb) => {
                let extent = self.amplitude.abs();
                FieldBounds::Bounded(Aabb3::new(aabb.min - extent, aabb.max + extent))
            }
        }
    }
}
