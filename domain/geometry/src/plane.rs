use glam::Vec3;

/// Plane represented by `dot(normal, point) + distance = 0`.
///
/// The normal does not need to be unit length. `signed_distance` returns
/// true metric distance only when the normal is normalized.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    pub fn from_point_normal(point: Vec3, normal: Vec3) -> Self {
        let distance = -normal.dot(point);
        Self { normal, distance }
    }

    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    pub fn project_point(&self, point: Vec3) -> Vec3 {
        let normal_len_sq = self.normal.length_squared();
        if normal_len_sq <= f32::EPSILON {
            return point;
        }

        let factor = self.signed_distance(point) / normal_len_sq;
        point - self.normal * factor
    }
}
