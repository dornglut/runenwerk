use crate::world::geometry_graph::{
    CavernGeometryGraph, GeometryBounds3, GeometryOp, GeometryPrimitive3, GeometryRevision,
};
use std::collections::{BTreeMap, BTreeSet};

mod chunks;
mod lifecycle;
mod queries;
mod sampling;

#[cfg(test)]
mod tests;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ecs::Resource)]
pub struct CollisionFieldRevision(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ecs::Resource)]
pub struct CollisionChunkKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct CollisionChunkBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, ecs::Resource)]
pub struct ChunkBrick3 {
    pub resolution: [usize; 3],
    pub distances: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, ecs::Resource)]
pub struct CollisionChunk {
    pub key: CollisionChunkKey,
    pub bounds: CollisionChunkBounds,
    pub revision_seen: GeometryRevision,
    pub dirty: bool,
    pub overlapping_primitives: Vec<u64>,
    pub brick: Option<ChunkBrick3>,
}

#[derive(Debug, Clone, PartialEq, Default, ecs::Resource)]
pub struct DirtyChunkSet {
    pub keys: BTreeSet<CollisionChunkKey>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub enum CollisionQueryMode {
    Cached,
    Analytic,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct SweepHit3 {
    pub hit: bool,
    pub fraction: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct PushOutResult3 {
    pub collided: bool,
    pub corrected_center: [f32; 3],
    pub normal: [f32; 3],
    pub penetration: f32,
}

#[derive(Debug, Clone, PartialEq, ecs::Resource)]
pub struct CavernCollisionField {
    pub world_bounds: GeometryBounds3,
    pub chunk_size: [f32; 3],
    pub brick_resolution: [usize; 3],
    pub revision_seen: GeometryRevision,
    pub chunks: BTreeMap<CollisionChunkKey, CollisionChunk>,
    pub dirty_chunks: DirtyChunkSet,
}

impl Default for CavernCollisionField {
    fn default() -> Self {
        Self {
            world_bounds: GeometryBounds3::default(),
            chunk_size: [8.0, 8.0, 8.0],
            brick_resolution: [16, 16, 16],
            revision_seen: GeometryRevision::default(),
            chunks: BTreeMap::new(),
            dirty_chunks: DirtyChunkSet::default(),
        }
    }
}
