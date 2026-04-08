use crate::SpatialKey;
use geometry::Aabb3;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpatialEntry<K: SpatialKey> {
    pub key: K,
    pub bounds: Aabb3,
}

impl<K: SpatialKey> SpatialEntry<K> {
    pub fn new(key: K, bounds: Aabb3) -> Self {
        Self { key, bounds }
    }
}
