pub mod affine;
pub mod rotate;
pub mod scale;
pub mod translate;

pub use affine::Affine;
pub use rotate::Rotate;
pub use scale::Scale;
pub use translate::Translate;

use geometry::Aabb3;
use glam::Vec3;

use crate::bounds::FieldBounds;

pub(super) fn map_bounded_aabb(bounds: FieldBounds, mapper: impl Fn(Vec3) -> Vec3) -> FieldBounds {
    match bounds {
        FieldBounds::Unbounded => FieldBounds::Unbounded,
        FieldBounds::Bounded(aabb) => {
            let corners = [
                Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
                Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
                Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
                Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
                Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
                Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
                Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
                Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
            ];

            let mut min = mapper(corners[0]);
            let mut max = min;
            for corner in corners.into_iter().skip(1) {
                let mapped = mapper(corner);
                min = min.min(mapped);
                max = max.max(mapped);
            }
            FieldBounds::Bounded(Aabb3::new(min, max))
        }
    }
}
