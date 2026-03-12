use glam::Vec3;

use crate::aabb::Aabb3;

/// Sphere primitive in 3D space.
///
/// `radius` is expected to be non-negative.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Inclusive containment test.
    pub fn contains_point(&self, point: Vec3) -> bool {
        let radius = self.radius.max(0.0);
        point.distance_squared(self.center) <= radius * radius
    }

    /// Inclusive overlap test: touching spheres count as intersecting.
    pub fn intersects_sphere(&self, other: &Self) -> bool {
        let r0 = self.radius.max(0.0);
        let r1 = other.radius.max(0.0);
        let radius_sum = r0 + r1;
        self.center.distance_squared(other.center) <= radius_sum * radius_sum
    }

    pub fn aabb(&self) -> Aabb3 {
        let extent = Vec3::splat(self.radius.max(0.0));
        Aabb3::from_center_extents(self.center, extent)
    }
}
