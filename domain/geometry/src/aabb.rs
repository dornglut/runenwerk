use glam::{Vec2, Vec3};

/// Axis-aligned 2D bounds with explicit minimum and maximum corners.
///
/// Constructor policy:
/// - `new(min, max)` is strict and preserves caller ordering.
/// - `from_corners(a, b)` normalizes corner ordering.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2 {
    /// Strict constructor that preserves caller-provided ordering.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Convenience constructor that normalizes corner ordering.
    pub fn from_corners(a: Vec2, b: Vec2) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    pub fn from_center_extents(center: Vec2, extents: Vec2) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Vec2>) -> Option<Self> {
        let mut iter = points.into_iter();
        let first = iter.next()?;
        let mut min = first;
        let mut max = first;
        for point in iter {
            min = min.min(point);
            max = max.max(point);
        }
        Some(Self { min, max })
    }

    pub fn is_valid(&self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.min.cmple(self.max).all()
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn extents(&self) -> Vec2 {
        self.size() * 0.5
    }

    /// Inclusive containment test.
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    /// Inclusive containment test.
    pub fn contains_aabb(&self, other: &Self) -> bool {
        other.min.cmpge(self.min).all() && other.max.cmple(self.max).all()
    }

    /// Inclusive overlap test: touching edges counts as intersection.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn expanded_by_point(&self, point: Vec2) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    pub fn expanded_by_aabb(&self, other: &Self) -> Self {
        self.union(other)
    }
}

/// Axis-aligned 3D bounds with explicit minimum and maximum corners.
///
/// Constructor policy:
/// - `new(min, max)` is strict and preserves caller ordering.
/// - `from_corners(a, b)` normalizes corner ordering.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Aabb3 {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3 {
    /// Strict constructor that preserves caller-provided ordering.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Convenience constructor that normalizes corner ordering.
    pub fn from_corners(a: Vec3, b: Vec3) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    pub fn from_center_extents(center: Vec3, extents: Vec3) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut iter = points.into_iter();
        let first = iter.next()?;
        let mut min = first;
        let mut max = first;
        for point in iter {
            min = min.min(point);
            max = max.max(point);
        }
        Some(Self { min, max })
    }

    pub fn is_valid(&self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.min.cmple(self.max).all()
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Inclusive containment test.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    /// Inclusive containment test.
    pub fn contains_aabb(&self, other: &Self) -> bool {
        other.min.cmpge(self.min).all() && other.max.cmple(self.max).all()
    }

    /// Inclusive overlap test: touching faces/edges counts as intersection.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn expanded_by_point(&self, point: Vec3) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    pub fn expanded_by_aabb(&self, other: &Self) -> Self {
        self.union(other)
    }

    pub fn surface_area(&self) -> f32 {
        let size = self.size().max(Vec3::ZERO);
        2.0 * (size.x * size.y + size.x * size.z + size.y * size.z)
    }

    pub fn volume(&self) -> f32 {
        let size = self.size().max(Vec3::ZERO);
        size.x * size.y * size.z
    }
}
