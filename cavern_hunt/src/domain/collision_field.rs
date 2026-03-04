use crate::domain::geometry_graph::{
    CavernGeometryGraph, GeometryBounds3, GeometryOp, GeometryPrimitive3, GeometryRevision,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollisionFieldRevision(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollisionChunkKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CollisionChunkBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkBrick3 {
    pub resolution: [usize; 3],
    pub distances: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionChunk {
    pub key: CollisionChunkKey,
    pub bounds: CollisionChunkBounds,
    pub revision_seen: GeometryRevision,
    pub dirty: bool,
    pub overlapping_primitives: Vec<u64>,
    pub brick: Option<ChunkBrick3>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirtyChunkSet {
    pub keys: BTreeSet<CollisionChunkKey>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CollisionQueryMode {
    Cached,
    Analytic,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SweepHit3 {
    pub hit: bool,
    pub fraction: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PushOutResult3 {
    pub collided: bool,
    pub corrected_center: [f32; 3],
    pub normal: [f32; 3],
    pub penetration: f32,
}

#[derive(Debug, Clone, PartialEq)]
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

impl CavernCollisionField {
    pub fn from_graph(graph: &CavernGeometryGraph) -> Self {
        Self {
            world_bounds: graph.bounds,
            revision_seen: graph.revision,
            ..Default::default()
        }
    }

    pub fn sync_revision(&mut self, graph: &CavernGeometryGraph) {
        self.world_bounds = graph.bounds;
        self.revision_seen = graph.revision;
    }

    pub fn invalidate_bounds(&mut self, bounds: GeometryBounds3) {
        let chunk_bounds = self.chunk_bounds_for_aabb(bounds.expanded(1.0));
        for key in chunk_bounds {
            self.dirty_chunks.keys.insert(key);
            if let Some(chunk) = self.chunks.get_mut(&key) {
                chunk.dirty = true;
            }
        }
    }

    pub fn mark_dirty_from_graph(&mut self, graph: &CavernGeometryGraph) {
        self.sync_revision(graph);
        for key in self.chunks.keys().copied().collect::<Vec<_>>() {
            self.dirty_chunks.keys.insert(key);
            if let Some(chunk) = self.chunks.get_mut(&key) {
                chunk.dirty = true;
            }
        }
    }

    pub fn distance(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> f32 {
        let key = self.chunk_key_for(point);
        self.ensure_chunk(graph, key);
        self.sample_chunk(key, point)
            .unwrap_or_else(|| self.distance_analytic(graph, point))
    }

    pub fn distance_analytic(&self, graph: &CavernGeometryGraph, point: [f32; 3]) -> f32 {
        distance_analytic_from_primitives(
            &graph
                .primitives
                .iter()
                .filter(|primitive| primitive.enabled)
                .collect::<Vec<_>>(),
            point,
        )
    }

    pub fn normal(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> [f32; 3] {
        let e = 0.05;
        let dx = self.distance(graph, [point[0] + e, point[1], point[2]])
            - self.distance(graph, [point[0] - e, point[1], point[2]]);
        let dy = self.distance(graph, [point[0], point[1] + e, point[2]])
            - self.distance(graph, [point[0], point[1] - e, point[2]]);
        let dz = self.distance(graph, [point[0], point[1], point[2] + e])
            - self.distance(graph, [point[0], point[1], point[2] - e]);
        normalize3([dx, dy, dz])
    }

    pub fn solid_at(&mut self, graph: &CavernGeometryGraph, point: [f32; 3]) -> bool {
        self.distance(graph, point) > 0.0
    }

    pub fn push_out_sphere(
        &mut self,
        graph: &CavernGeometryGraph,
        center: [f32; 3],
        radius: f32,
    ) -> PushOutResult3 {
        let distance = self.distance(graph, center);
        let penetration = distance + radius;
        if penetration <= 0.0 {
            return PushOutResult3 {
                collided: false,
                corrected_center: center,
                normal: [0.0, 1.0, 0.0],
                penetration: 0.0,
            };
        }
        let normal = self.normal(graph, center);
        let corrected_center = [
            center[0] - normal[0] * (penetration + 0.02),
            center[1] - normal[1] * (penetration + 0.02),
            center[2] - normal[2] * (penetration + 0.02),
        ];
        PushOutResult3 {
            collided: true,
            corrected_center,
            normal,
            penetration,
        }
    }

    pub fn push_out_capsule(
        &mut self,
        graph: &CavernGeometryGraph,
        base: [f32; 3],
        height: f32,
        radius: f32,
    ) -> PushOutResult3 {
        let top = [base[0], base[1] + height, base[2]];
        let base_push = self.push_out_sphere(graph, base, radius);
        let top_push = self.push_out_sphere(graph, top, radius);
        if !base_push.collided && !top_push.collided {
            return PushOutResult3 {
                collided: false,
                corrected_center: base,
                normal: [0.0, 1.0, 0.0],
                penetration: 0.0,
            };
        }
        let chosen = if top_push.penetration > base_push.penetration {
            top_push
        } else {
            base_push
        };
        PushOutResult3 {
            collided: true,
            corrected_center: chosen.corrected_center,
            normal: chosen.normal,
            penetration: chosen.penetration,
        }
    }

    pub fn sweep_sphere(
        &mut self,
        graph: &CavernGeometryGraph,
        start: [f32; 3],
        end: [f32; 3],
        radius: f32,
    ) -> SweepHit3 {
        let delta = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let length = length3(delta);
        let steps = ((length / 0.18).ceil() as usize).max(1);
        for step in 1..=steps {
            let fraction = step as f32 / steps as f32;
            let point = [
                start[0] + delta[0] * fraction,
                start[1] + delta[1] * fraction,
                start[2] + delta[2] * fraction,
            ];
            if self.distance(graph, point) > -radius {
                return SweepHit3 {
                    hit: true,
                    fraction,
                    point,
                    normal: self.normal(graph, point),
                };
            }
        }
        SweepHit3 {
            hit: false,
            fraction: 1.0,
            point: end,
            normal: [0.0, 1.0, 0.0],
        }
    }

    pub fn sweep_capsule(
        &mut self,
        graph: &CavernGeometryGraph,
        start: [f32; 3],
        end: [f32; 3],
        half_height: f32,
        radius: f32,
    ) -> SweepHit3 {
        let lower = self.sweep_sphere(graph, start, end, radius);
        let upper = self.sweep_sphere(
            graph,
            [start[0], start[1] + half_height * 2.0, start[2]],
            [end[0], end[1] + half_height * 2.0, end[2]],
            radius,
        );
        if upper.hit && upper.fraction < lower.fraction {
            upper
        } else {
            lower
        }
    }

    pub fn find_ground_below(
        &mut self,
        graph: &CavernGeometryGraph,
        origin: [f32; 3],
        max_drop: f32,
    ) -> Option<[f32; 3]> {
        let steps = ((max_drop / 0.1).ceil() as usize).max(1);
        for step in 0..=steps {
            let y = origin[1] - step as f32 * (max_drop / steps as f32);
            let point = [origin[0], y, origin[2]];
            if self.distance(graph, point) <= 0.0 {
                return Some(point);
            }
        }
        None
    }

    pub fn active_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    fn ensure_chunk(&mut self, graph: &CavernGeometryGraph, key: CollisionChunkKey) {
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

    fn sample_chunk(&self, key: CollisionChunkKey, point: [f32; 3]) -> Option<f32> {
        let chunk = self.chunks.get(&key)?;
        let brick = chunk.brick.as_ref()?;
        let local = [
            inverse_lerp(chunk.bounds.min[0], chunk.bounds.max[0], point[0]),
            inverse_lerp(chunk.bounds.min[1], chunk.bounds.max[1], point[1]),
            inverse_lerp(chunk.bounds.min[2], chunk.bounds.max[2], point[2]),
        ];
        Some(sample_brick_trilinear(brick, local))
    }

    fn chunk_key_for(&self, point: [f32; 3]) -> CollisionChunkKey {
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

    fn chunk_bounds_for_aabb(&self, bounds: GeometryBounds3) -> Vec<CollisionChunkKey> {
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

fn sample_t(index: usize, resolution: usize) -> f32 {
    if resolution <= 1 {
        0.0
    } else {
        index as f32 / (resolution - 1) as f32
    }
}

fn inverse_lerp(min: f32, max: f32, value: f32) -> f32 {
    if (max - min).abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn sample_brick_trilinear(brick: &ChunkBrick3, local: [f32; 3]) -> f32 {
    let max_x = brick.resolution[0].saturating_sub(1);
    let max_y = brick.resolution[1].saturating_sub(1);
    let max_z = brick.resolution[2].saturating_sub(1);
    let fx = local[0] * max_x as f32;
    let fy = local[1] * max_y as f32;
    let fz = local[2] * max_z as f32;
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let z0 = fz.floor() as usize;
    let x1 = (x0 + 1).min(max_x);
    let y1 = (y0 + 1).min(max_y);
    let z1 = (z0 + 1).min(max_z);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let tz = fz - z0 as f32;
    let c000 = brick_value(brick, x0, y0, z0);
    let c100 = brick_value(brick, x1, y0, z0);
    let c010 = brick_value(brick, x0, y1, z0);
    let c110 = brick_value(brick, x1, y1, z0);
    let c001 = brick_value(brick, x0, y0, z1);
    let c101 = brick_value(brick, x1, y0, z1);
    let c011 = brick_value(brick, x0, y1, z1);
    let c111 = brick_value(brick, x1, y1, z1);

    let c00 = lerp(c000, c100, tx);
    let c10 = lerp(c010, c110, tx);
    let c01 = lerp(c001, c101, tx);
    let c11 = lerp(c011, c111, tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    lerp(c0, c1, tz)
}

fn brick_value(brick: &ChunkBrick3, x: usize, y: usize, z: usize) -> f32 {
    let width = brick.resolution[0];
    let height = brick.resolution[1];
    brick.distances[z * width * height + y * width + x]
}

fn distance_analytic_from_primitives(primitives: &[&GeometryPrimitive3], point: [f32; 3]) -> f32 {
    let mut walkable = f32::INFINITY;
    for primitive in primitives {
        let sdf = primitive.shape.signed_distance(point);
        match primitive.op {
            GeometryOp::SubtractVoid | GeometryOp::MaskWalkable => {
                walkable = walkable.min(sdf);
            }
            GeometryOp::Blocker => {
                walkable = walkable.max(-sdf);
            }
            GeometryOp::AddSolid | GeometryOp::HazardVolume => {}
        }
    }
    walkable
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let length = length3(v);
    if length <= 0.0001 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / length, v[1] / length, v[2] / length]
    }
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::CavernCollisionField;
    use crate::domain::{
        CavernGeometryGraph, CavernLayout, CavernRunConfig, CavernSeed, CavernTopology,
    };

    #[test]
    fn cached_distance_tracks_analytic_distance() {
        let layout = CavernLayout::generate(CavernSeed(17), &CavernRunConfig::default());
        let topology = CavernTopology::from_layout(&layout, CavernSeed(17));
        let graph = CavernGeometryGraph::from_topology(&topology);
        let mut field = CavernCollisionField::from_graph(&graph);
        let point = [layout.rooms[0].center[0], 0.0, layout.rooms[0].center[1]];
        let analytic = field.distance_analytic(&graph, point);
        let cached = field.distance(&graph, point);
        assert!((analytic - cached).abs() < 0.35);
    }

    #[test]
    fn push_out_sphere_resolves_solid_penetration() {
        let layout = CavernLayout::generate(CavernSeed(3), &CavernRunConfig::default());
        let topology = CavernTopology::from_layout(&layout, CavernSeed(3));
        let graph = CavernGeometryGraph::from_topology(&topology);
        let mut field = CavernCollisionField::from_graph(&graph);
        let outside = [
            layout.world_bounds[0] - 1.0,
            0.0,
            layout.world_bounds[1] - 1.0,
        ];
        let pushed = field.push_out_sphere(&graph, outside, 0.45);
        assert!(pushed.collided);
    }
}
