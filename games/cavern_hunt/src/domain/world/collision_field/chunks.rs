use super::sampling::{
    distance_analytic_from_primitives, inverse_lerp, lerp, sample_brick_trilinear, sample_t,
};
use super::{
    CavernCollisionField, CavernGeometryGraph, ChunkBrick3, CollisionChunk, CollisionChunkBounds,
    CollisionChunkKey, GeometryBounds3, GeometryPrimitive3,
};

impl CavernCollisionField {
    pub(super) fn ensure_chunk(&mut self, graph: &CavernGeometryGraph, key: CollisionChunkKey) {
        let needs_rebuild = self
            .chunks
            .get(&key)
            .map(|chunk| {
                chunk.dirty
                    || chunk.revision_seen != graph.revision
                    || self.dirty_chunks.keys.contains(&key)
            })
            .unwrap_or(true);
        if !needs_rebuild {
            return;
        }
        let bounds = self.bounds_for_chunk(key);
        let overlapping = graph
            .primitives
            .iter()
            .filter(|primitive| primitive.enabled)
            .filter(|primitive| {
                primitive
                    .bounds()
                    .expanded(0.5)
                    .intersects(&GeometryBounds3 {
                        min: bounds.min,
                        max: bounds.max,
                    })
            })
            .collect::<Vec<_>>();
        let overlapping_primitives = overlapping
            .iter()
            .map(|primitive| primitive.id.0)
            .collect::<Vec<_>>();
        let brick = self.build_brick(bounds, &overlapping);
        self.chunks.insert(
            key,
            CollisionChunk {
                key,
                bounds,
                revision_seen: graph.revision,
                dirty: false,
                overlapping_primitives,
                brick: Some(brick),
            },
        );
        self.dirty_chunks.keys.remove(&key);
    }

    fn build_brick(
        &self,
        bounds: CollisionChunkBounds,
        primitives: &[&GeometryPrimitive3],
    ) -> ChunkBrick3 {
        let resolution = self.brick_resolution;
        let mut distances = Vec::with_capacity(resolution[0] * resolution[1] * resolution[2]);
        for z in 0..resolution[2] {
            for y in 0..resolution[1] {
                for x in 0..resolution[0] {
                    let sample = [
                        lerp(bounds.min[0], bounds.max[0], sample_t(x, resolution[0])),
                        lerp(bounds.min[1], bounds.max[1], sample_t(y, resolution[1])),
                        lerp(bounds.min[2], bounds.max[2], sample_t(z, resolution[2])),
                    ];
                    distances.push(distance_analytic_from_primitives(primitives, sample));
                }
            }
        }
        ChunkBrick3 {
            resolution,
            distances,
        }
    }

    pub(super) fn sample_chunk(&self, key: CollisionChunkKey, point: [f32; 3]) -> Option<f32> {
        let chunk = self.chunks.get(&key)?;
        let brick = chunk.brick.as_ref()?;
        let local = [
            inverse_lerp(chunk.bounds.min[0], chunk.bounds.max[0], point[0]),
            inverse_lerp(chunk.bounds.min[1], chunk.bounds.max[1], point[1]),
            inverse_lerp(chunk.bounds.min[2], chunk.bounds.max[2], point[2]),
        ];
        Some(sample_brick_trilinear(brick, local))
    }

    pub(super) fn chunk_key_for(&self, point: [f32; 3]) -> CollisionChunkKey {
        let origin = self.world_bounds.min;
        CollisionChunkKey {
            x: ((point[0] - origin[0]) / self.chunk_size[0]).floor() as i32,
            y: ((point[1] - origin[1]) / self.chunk_size[1]).floor() as i32,
            z: ((point[2] - origin[2]) / self.chunk_size[2]).floor() as i32,
        }
    }

    fn bounds_for_chunk(&self, key: CollisionChunkKey) -> CollisionChunkBounds {
        let origin = self.world_bounds.min;
        let min = [
            origin[0] + key.x as f32 * self.chunk_size[0],
            origin[1] + key.y as f32 * self.chunk_size[1],
            origin[2] + key.z as f32 * self.chunk_size[2],
        ];
        let max = [
            min[0] + self.chunk_size[0],
            min[1] + self.chunk_size[1],
            min[2] + self.chunk_size[2],
        ];
        CollisionChunkBounds { min, max }
    }

    pub(super) fn chunk_bounds_for_aabb(&self, bounds: GeometryBounds3) -> Vec<CollisionChunkKey> {
        let min_key = self.chunk_key_for(bounds.min);
        let max_key = self.chunk_key_for(bounds.max);
        let mut keys = Vec::new();
        for z in min_key.z..=max_key.z {
            for y in min_key.y..=max_key.y {
                for x in min_key.x..=max_key.x {
                    keys.push(CollisionChunkKey { x, y, z });
                }
            }
        }
        keys
    }
}
