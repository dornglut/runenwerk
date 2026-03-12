use glam::{Vec2, Vec3};

use crate::{Aabb2, Aabb3};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle2 {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

impl Triangle2 {
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self { a, b, c }
    }

    pub fn aabb(&self) -> Aabb2 {
        Aabb2::from_points([self.a, self.b, self.c]).expect("triangle has 3 points")
    }

    pub fn area(&self) -> f32 {
        ((self.b - self.a).perp_dot(self.c - self.a)).abs() * 0.5
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle3 {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle3 {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    pub fn aabb(&self) -> Aabb3 {
        Aabb3::from_points([self.a, self.b, self.c]).expect("triangle has 3 points")
    }

    pub fn area(&self) -> f32 {
        (self.b - self.a).cross(self.c - self.a).length() * 0.5
    }
}
