use glam::{Vec2, Vec3};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LineSegment2 {
    pub a: Vec2,
    pub b: Vec2,
}

impl LineSegment2 {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }

    pub fn direction(&self) -> Vec2 {
        self.b - self.a
    }

    pub fn length(&self) -> f32 {
        self.direction().length()
    }

    pub fn point_at(&self, t: f32) -> Vec2 {
        self.a + self.direction() * t
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LineSegment3 {
    pub a: Vec3,
    pub b: Vec3,
}

impl LineSegment3 {
    pub fn new(a: Vec3, b: Vec3) -> Self {
        Self { a, b }
    }

    pub fn direction(&self) -> Vec3 {
        self.b - self.a
    }

    pub fn length(&self) -> f32 {
        self.direction().length()
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.a + self.direction() * t
    }
}
