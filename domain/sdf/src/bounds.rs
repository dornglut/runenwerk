use geometry::Aabb3;
use glam::Vec3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FieldBounds {
    Unbounded,
    Bounded(Aabb3),
}

impl FieldBounds {
    pub fn bounded(aabb: Aabb3) -> Self {
        Self::Bounded(aabb)
    }

    pub fn as_aabb(&self) -> Option<Aabb3> {
        match self {
            Self::Unbounded => None,
            Self::Bounded(aabb) => Some(*aabb),
        }
    }

    pub fn translated(&self, offset: Vec3) -> Self {
        match self {
            Self::Unbounded => Self::Unbounded,
            Self::Bounded(aabb) => Self::Bounded(Aabb3::new(aabb.min + offset, aabb.max + offset)),
        }
    }

    pub fn expanded_scalar(&self, padding: f32) -> Self {
        match self {
            Self::Unbounded => Self::Unbounded,
            Self::Bounded(aabb) => {
                let extent = Vec3::splat(padding.max(0.0));
                Self::Bounded(Aabb3::new(aabb.min - extent, aabb.max + extent))
            }
        }
    }

    pub fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::Bounded(a), Self::Bounded(b)) => Self::Bounded(a.union(&b)),
            _ => Self::Unbounded,
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        match (self, other) {
            (Self::Bounded(a), Self::Bounded(b)) => {
                if a.intersects(&b) {
                    let min = a.min.max(b.min);
                    let max = a.max.min(b.max);
                    Self::Bounded(Aabb3::new(min, max))
                } else {
                    // Conservative fallback for disjoint boxes.
                    Self::Bounded(a.union(&b))
                }
            }
            (Self::Unbounded, bounded @ Self::Bounded(_))
            | (bounded @ Self::Bounded(_), Self::Unbounded) => bounded,
            (Self::Unbounded, Self::Unbounded) => Self::Unbounded,
        }
    }
}
