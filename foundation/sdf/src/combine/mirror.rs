use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;
use crate::field::SdfField3;
use crate::sample::SdfSample;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MirrorAxes {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl MirrorAxes {
    pub const NONE: Self = Self {
        x: false,
        y: false,
        z: false,
    };

    pub const XYZ: Self = Self {
        x: true,
        y: true,
        z: true,
    };

    pub const fn new(x: bool, y: bool, z: bool) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mirror<F> {
    pub field: F,
    pub axes: MirrorAxes,
}

impl<F> Mirror<F> {
    pub fn new(field: F, axes: MirrorAxes) -> Self {
        Self { field, axes }
    }
}

impl<F> SdfField3 for Mirror<F>
where
    F: SdfField3,
{
    fn sample(&self, point: Vec3) -> SdfSample {
        let local = Vec3::new(
            if self.axes.x { point.x.abs() } else { point.x },
            if self.axes.y { point.y.abs() } else { point.y },
            if self.axes.z { point.z.abs() } else { point.z },
        );
        self.field.sample(local)
    }

    fn bounds(&self) -> FieldBounds {
        match self.field.bounds() {
            FieldBounds::Unbounded => FieldBounds::Unbounded,
            FieldBounds::Bounded(aabb) => {
                let mut min = aabb.min;
                let mut max = aabb.max;

                if self.axes.x {
                    let extent = min.x.abs().max(max.x.abs());
                    min.x = -extent;
                    max.x = extent;
                }
                if self.axes.y {
                    let extent = min.y.abs().max(max.y.abs());
                    min.y = -extent;
                    max.y = extent;
                }
                if self.axes.z {
                    let extent = min.z.abs().max(max.z.abs());
                    min.z = -extent;
                    max.z = extent;
                }

                FieldBounds::Bounded(Aabb3::new(min, max))
            }
        }
    }
}
