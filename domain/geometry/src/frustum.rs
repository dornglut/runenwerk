use glam::Vec3;

use crate::{Aabb3, Plane, Sphere};

/// View frustum defined by six clipping planes.
///
/// Plane orientation convention:
/// - Plane normals face outward from the frustum volume.
/// - A point is inside the frustum when all plane signed distances are `<= 0`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frustum {
    pub left: Plane,
    pub right: Plane,
    pub top: Plane,
    pub bottom: Plane,
    pub near: Plane,
    pub far: Plane,
}

impl Frustum {
    pub fn new(
        left: Plane,
        right: Plane,
        top: Plane,
        bottom: Plane,
        near: Plane,
        far: Plane,
    ) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
            near,
            far,
        }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        self.planes()
            .iter()
            .all(|plane| plane.signed_distance(point) <= 0.0)
    }

    pub fn intersects_aabb(&self, aabb: &Aabb3) -> bool {
        // Separating-axis style test against frustum planes.
        for plane in self.planes() {
            let nearest_vertex = Vec3::new(
                if plane.normal.x >= 0.0 {
                    aabb.min.x
                } else {
                    aabb.max.x
                },
                if plane.normal.y >= 0.0 {
                    aabb.min.y
                } else {
                    aabb.max.y
                },
                if plane.normal.z >= 0.0 {
                    aabb.min.z
                } else {
                    aabb.max.z
                },
            );
            if plane.signed_distance(nearest_vertex) > 0.0 {
                return false;
            }
        }

        true
    }

    pub fn intersects_sphere(&self, sphere: &Sphere) -> bool {
        let radius = sphere.radius.max(0.0);
        self.planes()
            .iter()
            .all(|plane| plane.signed_distance(sphere.center) <= radius)
    }

    pub fn planes(&self) -> [Plane; 6] {
        [
            self.left,
            self.right,
            self.top,
            self.bottom,
            self.near,
            self.far,
        ]
    }
}
