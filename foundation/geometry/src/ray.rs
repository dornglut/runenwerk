use glam::{Vec2, Vec3};

/// 2D ray with explicit origin and direction.
///
/// Direction is not normalized by the type. Algorithms that require
/// normalization document that requirement.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray2 {
    pub origin: Vec2,
    pub direction: Vec2,
}

impl Ray2 {
    pub fn new(origin: Vec2, direction: Vec2) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f32) -> Vec2 {
        self.origin + self.direction * t
    }
}

/// 3D ray with explicit origin and direction.
///
/// Direction is not normalized by the type. Algorithms that require
/// normalization document that requirement.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray3 {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray3 {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
