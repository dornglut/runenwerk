use crate::storage::{
    SDF_PAGE_EDGE_BRICKS, SdfBrickRecord, SdfChunkPayload, SdfChunkStore, SdfPageCoord3,
};
use serde::{Deserialize, Serialize};
use spatial::WorldLocalPosition;
use spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionSample {
    pub chunk_id: ChunkId,
    pub distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionHit {
    pub chunk_id: ChunkId,
    pub hit_position: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereSweep {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionReadiness {
    MissingPayload { chunk_id: ChunkId },
    Ready,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollisionSweepOutcome {
    MissingPayload { chunk_id: ChunkId },
    Hit(CollisionHit),
    Clear,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct CollisionQueryService;

impl CollisionQueryService {
    pub fn collision_readiness_for_position(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        world_position: [f32; 3],
    ) -> CollisionReadiness {
        let chunk_id = partition.chunk_id_from_position(
            planet_id,
            WorldLocalPosition {
                meters: world_position,
            },
        );
        if store.chunks.contains_key(&chunk_id) {
            CollisionReadiness::Ready
        } else {
            CollisionReadiness::MissingPayload { chunk_id }
        }
    }

    pub fn collision_readiness_for_sweep(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        query: SphereSweep,
    ) -> CollisionReadiness {
        let required_chunks = required_chunks_for_sweep(partition, planet_id, query);
        if let Some(chunk_id) = first_missing_payload_chunk(store, &required_chunks) {
            CollisionReadiness::MissingPayload { chunk_id }
        } else {
            CollisionReadiness::Ready
        }
    }

    pub fn chunk_has_authoritative_payload(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        world_position: [f32; 3],
    ) -> bool {
        matches!(
            self.collision_readiness_for_position(partition, store, planet_id, world_position),
            CollisionReadiness::Ready
        )
    }

    pub fn sample_signed_distance(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        world_position: [f32; 3],
    ) -> Option<CollisionSample> {
        let chunk_id = partition.chunk_id_from_position(
            planet_id,
            WorldLocalPosition {
                meters: world_position,
            },
        );
        let payload = store.chunks.get(&chunk_id)?;
        let summary_distance =
            sample_payload_signed_distance(partition, payload, chunk_id, world_position);
        Some(CollisionSample {
            chunk_id,
            distance: summary_distance,
        })
    }

    pub fn sweep_sphere(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        query: SphereSweep,
    ) -> Option<CollisionHit> {
        match self.sweep_sphere_authoritative(partition, store, planet_id, query) {
            CollisionSweepOutcome::Hit(hit) => Some(hit),
            CollisionSweepOutcome::MissingPayload { .. } | CollisionSweepOutcome::Clear => None,
        }
    }

    pub fn sweep_sphere_authoritative(
        &self,
        partition: &GridPartitionConfig,
        store: &SdfChunkStore,
        planet_id: WorldId,
        query: SphereSweep,
    ) -> CollisionSweepOutcome {
        let required_chunks = required_chunks_for_sweep(partition, planet_id, query);
        if let Some(chunk_id) = first_missing_payload_chunk(store, &required_chunks) {
            return CollisionSweepOutcome::MissingPayload { chunk_id };
        }

        let sweep_delta = sub3(query.end, query.start);
        let sweep_length = length3(sweep_delta);
        let step_size_meters =
            (partition.chunk_edge_meters.max(1.0) / (SDF_PAGE_EDGE_BRICKS as f32 * 2.0)).max(0.05);
        let sweep_steps = ((sweep_length / step_size_meters).ceil() as usize).clamp(1, 512);

        let start_sample = self
            .sample_signed_distance(partition, store, planet_id, query.start)
            .unwrap_or(CollisionSample {
                chunk_id: partition.chunk_id_from_position(
                    planet_id,
                    WorldLocalPosition {
                        meters: query.start,
                    },
                ),
                distance: -1.0,
            });
        if start_sample.distance <= query.radius.max(0.0) {
            return CollisionSweepOutcome::Hit(CollisionHit {
                chunk_id: start_sample.chunk_id,
                hit_position: query.start,
                normal: estimate_hit_normal(self, partition, store, planet_id, &query, query.start),
                distance: start_sample.distance,
            });
        }

        let refinement = CollisionSweepRefinement::new(self, partition, store, planet_id, &query);
        let mut clear_t = 0.0_f32;
        for step in 1..=sweep_steps {
            let test_t = step as f32 / sweep_steps as f32;
            if !refinement.collides_at(test_t) {
                clear_t = test_t;
                continue;
            }

            let hit_t = refine_collision_hit_t(&refinement, clear_t, test_t, 8);
            let hit_position = lerp3(query.start, query.end, hit_t);
            let hit_sample = self
                .sample_signed_distance(partition, store, planet_id, hit_position)
                .unwrap_or(CollisionSample {
                    chunk_id: partition.chunk_id_from_position(
                        planet_id,
                        WorldLocalPosition {
                            meters: hit_position,
                        },
                    ),
                    distance: -1.0,
                });
            return CollisionSweepOutcome::Hit(CollisionHit {
                chunk_id: hit_sample.chunk_id,
                hit_position,
                normal: estimate_hit_normal(
                    self,
                    partition,
                    store,
                    planet_id,
                    &query,
                    hit_position,
                ),
                distance: hit_sample.distance,
            });
        }

        CollisionSweepOutcome::Clear
    }
}

fn chunk_payload_has_occupied_voxels(payload: &SdfChunkPayload) -> bool {
    payload.page_table.values().any(|page| {
        page.bricks
            .values()
            .any(|brick| brick.metadata.occupancy_mask != 0)
    })
}

fn first_missing_payload_chunk(
    store: &SdfChunkStore,
    required_chunks: &[ChunkId],
) -> Option<ChunkId> {
    required_chunks
        .iter()
        .copied()
        .find(|chunk_id| !store.chunks.contains_key(chunk_id))
}

fn required_chunks_for_sweep(
    partition: &GridPartitionConfig,
    planet_id: WorldId,
    query: SphereSweep,
) -> Vec<ChunkId> {
    let extent = query.radius.max(0.0);
    let min_world = [
        query.start[0].min(query.end[0]) - extent,
        query.start[1].min(query.end[1]) - extent,
        query.start[2].min(query.end[2]) - extent,
    ];
    let max_world = [
        query.start[0].max(query.end[0]) + extent,
        query.start[1].max(query.end[1]) + extent,
        query.start[2].max(query.end[2]) + extent,
    ];
    let min_chunk =
        partition.chunk_coord_from_world_local_position(WorldLocalPosition { meters: min_world });
    let max_chunk =
        partition.chunk_coord_from_world_local_position(WorldLocalPosition { meters: max_world });
    enumerate_chunks_inclusive(planet_id, min_chunk, max_chunk)
}

fn enumerate_chunks_inclusive(
    planet_id: WorldId,
    min: ChunkCoord3,
    max: ChunkCoord3,
) -> Vec<ChunkId> {
    let mut chunks = Vec::new();
    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                chunks.push(ChunkId::new(planet_id, ChunkCoord3 { x, y, z }));
            }
        }
    }
    chunks
}

fn sample_payload_signed_distance(
    partition: &GridPartitionConfig,
    payload: &SdfChunkPayload,
    chunk_id: ChunkId,
    world_position: [f32; 3],
) -> f32 {
    let default_distance = payload_default_signed_distance(payload);
    let Some((page_coord, brick_key, local_in_brick)) =
        payload_brick_lookup(partition, payload, chunk_id, world_position)
    else {
        return default_distance;
    };

    let Some(page) = payload.page_table.get(&page_coord) else {
        return default_distance;
    };
    let Some(brick) = page.bricks.get(&brick_key) else {
        return default_distance;
    };

    sample_brick_signed_distance(brick, local_in_brick)
}

fn payload_default_signed_distance(payload: &SdfChunkPayload) -> f32 {
    if chunk_payload_has_occupied_voxels(payload) {
        -1.0
    } else {
        1.0
    }
}

fn payload_brick_lookup(
    partition: &GridPartitionConfig,
    payload: &SdfChunkPayload,
    chunk_id: ChunkId,
    world_position: [f32; 3],
) -> Option<(SdfPageCoord3, [u8; 3], [f32; 3])> {
    let (min_page, max_page) = payload_page_bounds(payload)?;
    let edge = partition.chunk_edge_meters.max(1.0);
    let local = chunk_local_position(partition, chunk_id, world_position);
    let local_clamped = [
        local[0].clamp(0.0, edge * (1.0 - 1.0e-6)),
        local[1].clamp(0.0, edge * (1.0 - 1.0e-6)),
        local[2].clamp(0.0, edge * (1.0 - 1.0e-6)),
    ];

    let page_span = [
        (max_page.x - min_page.x + 1).max(1) as i32,
        (max_page.y - min_page.y + 1).max(1) as i32,
        (max_page.z - min_page.z + 1).max(1) as i32,
    ];

    let (page_offset_x, brick_x, local_x) =
        quantize_payload_axis(local_clamped[0], edge, page_span[0]);
    let (page_offset_y, brick_y, local_y) =
        quantize_payload_axis(local_clamped[1], edge, page_span[1]);
    let (page_offset_z, brick_z, local_z) =
        quantize_payload_axis(local_clamped[2], edge, page_span[2]);

    Some((
        SdfPageCoord3 {
            x: min_page.x + page_offset_x as i16,
            y: min_page.y + page_offset_y as i16,
            z: min_page.z + page_offset_z as i16,
        },
        [brick_x, brick_y, brick_z],
        [local_x, local_y, local_z],
    ))
}

fn quantize_payload_axis(local_axis: f32, edge: f32, page_span: i32) -> (i32, u8, f32) {
    let span = page_span.max(1);
    let page_coord_f = (local_axis / edge) * span as f32;
    let page_offset = page_coord_f.floor().clamp(0.0, (span - 1) as f32) as i32;
    let page_local = page_coord_f - page_offset as f32;
    let brick_coord_f = page_local * SDF_PAGE_EDGE_BRICKS as f32;
    let brick_index = brick_coord_f
        .floor()
        .clamp(0.0, (SDF_PAGE_EDGE_BRICKS - 1) as f32) as u8;
    let brick_local = (brick_coord_f - brick_index as f32).clamp(0.0, 1.0 - 1.0e-6);
    (page_offset, brick_index, brick_local)
}

fn payload_page_bounds(payload: &SdfChunkPayload) -> Option<(SdfPageCoord3, SdfPageCoord3)> {
    let mut pages = payload.page_table.keys().copied();
    let first = pages.next()?;
    let mut min = first;
    let mut max = first;
    for page in pages {
        min.x = min.x.min(page.x);
        min.y = min.y.min(page.y);
        min.z = min.z.min(page.z);
        max.x = max.x.max(page.x);
        max.y = max.y.max(page.y);
        max.z = max.z.max(page.z);
    }
    Some((min, max))
}

fn sample_brick_signed_distance(brick: &SdfBrickRecord, local_in_brick: [f32; 3]) -> f32 {
    let occupancy_sign = occupancy_sign_from_mask(brick.metadata.occupancy_mask, local_in_brick);
    sample_sign_from_brick_samples(&brick.samples.distances, local_in_brick)
        .unwrap_or(occupancy_sign)
}

fn occupancy_sign_from_mask(mask: u8, local_in_brick: [f32; 3]) -> f32 {
    if mask == 0 {
        return 1.0;
    }
    if mask == u8::MAX {
        return -1.0;
    }

    let octant_x = u8::from(local_in_brick[0] >= 0.5);
    let octant_y = u8::from(local_in_brick[1] >= 0.5);
    let octant_z = u8::from(local_in_brick[2] >= 0.5);
    let octant_index = octant_x | (octant_y << 1) | (octant_z << 2);
    let occupied = (mask & (1 << octant_index)) != 0;
    if occupied { -1.0 } else { 1.0 }
}

fn sample_sign_from_brick_samples(samples: &[i16], local_in_brick: [f32; 3]) -> Option<f32> {
    let sample = sample_scalar_from_cube_samples(samples, local_in_brick)?;
    if sample.abs() <= f32::EPSILON {
        None
    } else {
        Some(sample.signum())
    }
}

fn sample_scalar_from_cube_samples(samples: &[i16], local_in_brick: [f32; 3]) -> Option<f32> {
    if samples.is_empty() {
        return None;
    }
    let len = samples.len();
    let dim = ((len as f32).cbrt().round() as usize).max(1);
    if dim.saturating_mul(dim).saturating_mul(dim) != len {
        return Some(samples[0] as f32);
    }
    if dim == 1 {
        return Some(samples[0] as f32);
    }

    let sample_at =
        |x: usize, y: usize, z: usize| -> f32 { samples[cube_sample_index(dim, x, y, z)] as f32 };
    let edge = (dim - 1) as f32;
    let fx = local_in_brick[0].clamp(0.0, 1.0) * edge;
    let fy = local_in_brick[1].clamp(0.0, 1.0) * edge;
    let fz = local_in_brick[2].clamp(0.0, 1.0) * edge;

    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let z0 = fz.floor() as usize;
    let x1 = (x0 + 1).min(dim - 1);
    let y1 = (y0 + 1).min(dim - 1);
    let z1 = (z0 + 1).min(dim - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let tz = fz - z0 as f32;

    let c00 = lerp(sample_at(x0, y0, z0), sample_at(x1, y0, z0), tx);
    let c10 = lerp(sample_at(x0, y1, z0), sample_at(x1, y1, z0), tx);
    let c01 = lerp(sample_at(x0, y0, z1), sample_at(x1, y0, z1), tx);
    let c11 = lerp(sample_at(x0, y1, z1), sample_at(x1, y1, z1), tx);
    let c0 = lerp(c00, c10, ty);
    let c1 = lerp(c01, c11, ty);
    Some(lerp(c0, c1, tz))
}

fn cube_sample_index(dim: usize, x: usize, y: usize, z: usize) -> usize {
    z * dim * dim + y * dim + x
}

struct CollisionSweepRefinement<'a> {
    service: &'a CollisionQueryService,
    partition: &'a GridPartitionConfig,
    store: &'a SdfChunkStore,
    planet_id: WorldId,
    query: &'a SphereSweep,
}

impl<'a> CollisionSweepRefinement<'a> {
    fn new(
        service: &'a CollisionQueryService,
        partition: &'a GridPartitionConfig,
        store: &'a SdfChunkStore,
        planet_id: WorldId,
        query: &'a SphereSweep,
    ) -> Self {
        Self {
            service,
            partition,
            store,
            planet_id,
            query,
        }
    }

    fn collides_at(&self, t: f32) -> bool {
        collides_at_sweep_t(
            self.service,
            self.partition,
            self.store,
            self.planet_id,
            self.query,
            t,
        )
    }
}

fn collides_at_sweep_t(
    service: &CollisionQueryService,
    partition: &GridPartitionConfig,
    store: &SdfChunkStore,
    planet_id: WorldId,
    query: &SphereSweep,
    t: f32,
) -> bool {
    let position = lerp3(query.start, query.end, t.clamp(0.0, 1.0));
    let distance = service
        .sample_signed_distance(partition, store, planet_id, position)
        .map(|sample| sample.distance)
        .unwrap_or(-1.0);
    distance <= query.radius.max(0.0)
}

fn refine_collision_hit_t(
    refinement: &CollisionSweepRefinement<'_>,
    mut clear_t: f32,
    mut colliding_t: f32,
    iterations: usize,
) -> f32 {
    for _ in 0..iterations {
        let mid = (clear_t + colliding_t) * 0.5;
        if refinement.collides_at(mid) {
            colliding_t = mid;
        } else {
            clear_t = mid;
        }
    }
    colliding_t
}

fn estimate_hit_normal(
    service: &CollisionQueryService,
    partition: &GridPartitionConfig,
    store: &SdfChunkStore,
    planet_id: WorldId,
    query: &SphereSweep,
    hit_position: [f32; 3],
) -> [f32; 3] {
    let epsilon = (partition.chunk_edge_meters.max(1.0) / 64.0).clamp(0.01, 0.5);
    let sample_with_offset = |offset: [f32; 3]| -> f32 {
        let position = [
            hit_position[0] + offset[0],
            hit_position[1] + offset[1],
            hit_position[2] + offset[2],
        ];
        service
            .sample_signed_distance(partition, store, planet_id, position)
            .map(|sample| sample.distance)
            .unwrap_or(1.0)
    };

    let gradient = [
        sample_with_offset([epsilon, 0.0, 0.0]) - sample_with_offset([-epsilon, 0.0, 0.0]),
        sample_with_offset([0.0, epsilon, 0.0]) - sample_with_offset([0.0, -epsilon, 0.0]),
        sample_with_offset([0.0, 0.0, epsilon]) - sample_with_offset([0.0, 0.0, -epsilon]),
    ];
    let gradient_length = length3(gradient);
    if gradient_length > 1.0e-6 {
        return normalize3(gradient);
    }

    let fallback = normalize3(sub3(query.start, query.end));
    if length3(fallback) > 1.0e-6 {
        fallback
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn chunk_local_position(
    partition: &GridPartitionConfig,
    chunk_id: ChunkId,
    world_position: [f32; 3],
) -> [f32; 3] {
    let edge = partition.chunk_edge_meters.max(1.0);
    let chunk_min = [
        chunk_id.coord.x as f32 * edge,
        chunk_id.coord.y as f32 * edge,
        chunk_id.coord.z as f32 * edge,
    ];
    [
        world_position[0] - chunk_min[0],
        world_position[1] - chunk_min[1],
        world_position[2] - chunk_min[2],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn length3(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = length3(value);
    if length <= f32::EPSILON {
        [0.0, 0.0, 0.0]
    } else {
        [value[0] / length, value[1] / length, value[2] / length]
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp3(start: [f32; 3], end: [f32; 3], t: f32) -> [f32; 3] {
    [
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
        start[2] + (end[2] - start[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        SDF_PAGE_EDGE_BRICKS, SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfPageRecord,
    };
    use world_ops::{ChunkGeneration, ChunkRevision};

    fn clear_payload(chunk_id: ChunkId) -> SdfChunkPayload {
        SdfChunkPayload {
            chunk_id,
            chunk_revision: ChunkRevision::default(),
            chunk_generation: ChunkGeneration::default(),
            page_table: Default::default(),
            hierarchy_revision: 0,
            checksum: 0,
        }
    }

    fn occupied_payload(chunk_id: ChunkId) -> SdfChunkPayload {
        let mut payload = clear_payload(chunk_id);
        let mut page = SdfPageRecord {
            page_generation: 0,
            bricks: Default::default(),
        };
        page.bricks.insert(
            [0, 0, 0],
            SdfBrickRecord {
                metadata: SdfBrickMetadata {
                    occupancy_mask: 0xFF,
                    ..SdfBrickMetadata::default()
                },
                samples: SdfBrickSamples {
                    distances: vec![-1; 8],
                },
            },
        );
        payload
            .page_table
            .insert(SdfPageCoord3 { x: 0, y: 0, z: 0 }, page);
        payload
    }

    fn uniform_page_payload(chunk_id: ChunkId, occupancy_mask: u8) -> SdfChunkPayload {
        let mut payload = clear_payload(chunk_id);
        let mut page = SdfPageRecord {
            page_generation: 0,
            bricks: Default::default(),
        };
        for z in 0..SDF_PAGE_EDGE_BRICKS as u8 {
            for y in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                for x in 0..SDF_PAGE_EDGE_BRICKS as u8 {
                    page.bricks.insert(
                        [x, y, z],
                        SdfBrickRecord {
                            metadata: SdfBrickMetadata {
                                occupancy_mask,
                                ..SdfBrickMetadata::default()
                            },
                            samples: SdfBrickSamples::default(),
                        },
                    );
                }
            }
        }
        payload
            .page_table
            .insert(SdfPageCoord3 { x: 0, y: 0, z: 0 }, page);
        payload
    }

    #[test]
    fn sweep_readiness_reports_first_missing_chunk_in_path_bounds() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let mut store = SdfChunkStore::default();
        let planet_id = WorldId(0);
        let loaded_chunks = [
            ChunkId::new(planet_id, ChunkCoord3 { x: 0, y: 0, z: 0 }),
            ChunkId::new(planet_id, ChunkCoord3 { x: 2, y: 0, z: 0 }),
        ];
        for chunk_id in loaded_chunks {
            store.chunks.insert(chunk_id, clear_payload(chunk_id));
        }

        let readiness = service.collision_readiness_for_sweep(
            &partition,
            &store,
            planet_id,
            SphereSweep {
                start: [0.1, 0.1, 0.1],
                end: [2.1, 0.1, 0.1],
                radius: 0.0,
            },
        );
        assert_eq!(
            readiness,
            CollisionReadiness::MissingPayload {
                chunk_id: ChunkId::new(planet_id, ChunkCoord3 { x: 1, y: 0, z: 0 }),
            }
        );
    }

    #[test]
    fn sweep_readiness_is_ready_when_all_required_chunks_are_loaded() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let mut store = SdfChunkStore::default();
        let planet_id = WorldId(0);

        for x in 0..=2 {
            let chunk_id = ChunkId::new(planet_id, ChunkCoord3 { x, y: 0, z: 0 });
            store.chunks.insert(chunk_id, clear_payload(chunk_id));
        }

        let readiness = service.collision_readiness_for_sweep(
            &partition,
            &store,
            planet_id,
            SphereSweep {
                start: [0.1, 0.1, 0.1],
                end: [2.1, 0.1, 0.1],
                radius: 0.0,
            },
        );
        assert_eq!(readiness, CollisionReadiness::Ready);
    }

    #[test]
    fn sweep_hits_first_occupied_chunk_along_path_bounds() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let mut store = SdfChunkStore::default();
        let planet_id = WorldId(0);
        let chunk0 = ChunkId::new(planet_id, ChunkCoord3 { x: 0, y: 0, z: 0 });
        let chunk1 = ChunkId::new(planet_id, ChunkCoord3 { x: 1, y: 0, z: 0 });
        let chunk2 = ChunkId::new(planet_id, ChunkCoord3 { x: 2, y: 0, z: 0 });
        store.chunks.insert(chunk0, clear_payload(chunk0));
        store.chunks.insert(chunk1, occupied_payload(chunk1));
        store.chunks.insert(chunk2, occupied_payload(chunk2));

        let hit = service
            .sweep_sphere(
                &partition,
                &store,
                planet_id,
                SphereSweep {
                    start: [0.1, 0.1, 0.1],
                    end: [2.9, 0.1, 0.1],
                    radius: 0.0,
                },
            )
            .expect("first occupied chunk should be reported as sweep hit");
        assert_eq!(hit.chunk_id, chunk1);
        assert!(
            (hit.hit_position[0] - 1.0).abs() <= 0.001,
            "hit position should be the first chunk-boundary entry point"
        );
    }

    #[test]
    fn sample_signed_distance_respects_brick_octant_occupancy() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let mut store = SdfChunkStore::default();
        let planet_id = WorldId(0);
        let chunk_id = ChunkId::new(planet_id, ChunkCoord3::default());
        store
            .chunks
            .insert(chunk_id, uniform_page_payload(chunk_id, 0b1000_0000));

        let low_octant = service
            .sample_signed_distance(&partition, &store, planet_id, [0.1, 0.1, 0.1])
            .expect("sample should be available for loaded chunk");
        let high_octant = service
            .sample_signed_distance(&partition, &store, planet_id, [0.9, 0.9, 0.9])
            .expect("sample should be available for loaded chunk");

        assert!(
            low_octant.distance > 0.0,
            "clear octant should report positive signed distance"
        );
        assert!(
            high_octant.distance < 0.0,
            "occupied octant should report negative signed distance"
        );
    }

    #[test]
    fn sweep_can_stay_clear_inside_partial_occupancy_chunk() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let mut store = SdfChunkStore::default();
        let planet_id = WorldId(0);
        let chunk_id = ChunkId::new(planet_id, ChunkCoord3::default());
        store
            .chunks
            .insert(chunk_id, uniform_page_payload(chunk_id, 0b1000_0000));

        let outcome = service.sweep_sphere_authoritative(
            &partition,
            &store,
            planet_id,
            SphereSweep {
                start: [0.1, 0.1, 0.1],
                end: [0.9, 0.1, 0.1],
                radius: 0.0,
            },
        );
        assert_eq!(
            outcome,
            CollisionSweepOutcome::Clear,
            "sweep confined to clear octants should remain clear even when chunk contains occupied octants"
        );
    }

    #[test]
    fn authoritative_sweep_outcome_distinguishes_clear_and_missing_payload() {
        let service = CollisionQueryService;
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            region_chunk_dims: [8, 8, 8],
            fixed_point_scale: 1024,
        };
        let planet_id = WorldId(0);

        let clear_chunk0 = ChunkId::new(planet_id, ChunkCoord3 { x: 0, y: 0, z: 0 });
        let clear_chunk1 = ChunkId::new(planet_id, ChunkCoord3 { x: 1, y: 0, z: 0 });
        let clear_query = SphereSweep {
            start: [0.1, 0.1, 0.1],
            end: [1.8, 0.1, 0.1],
            radius: 0.0,
        };
        let mut clear_store = SdfChunkStore::default();
        clear_store
            .chunks
            .insert(clear_chunk0, clear_payload(clear_chunk0));
        clear_store
            .chunks
            .insert(clear_chunk1, clear_payload(clear_chunk1));
        let clear_outcome =
            service.sweep_sphere_authoritative(&partition, &clear_store, planet_id, clear_query);
        assert_eq!(clear_outcome, CollisionSweepOutcome::Clear);

        let missing_query = SphereSweep {
            start: [0.1, 0.1, 0.1],
            end: [2.8, 0.1, 0.1],
            radius: 0.0,
        };
        let mut missing_store = SdfChunkStore::default();
        missing_store
            .chunks
            .insert(clear_chunk0, clear_payload(clear_chunk0));
        let chunk2 = ChunkId::new(planet_id, ChunkCoord3 { x: 2, y: 0, z: 0 });
        missing_store.chunks.insert(chunk2, clear_payload(chunk2));
        let missing_outcome = service.sweep_sphere_authoritative(
            &partition,
            &missing_store,
            planet_id,
            missing_query,
        );
        assert_eq!(
            missing_outcome,
            CollisionSweepOutcome::MissingPayload {
                chunk_id: ChunkId::new(planet_id, ChunkCoord3 { x: 1, y: 0, z: 0 }),
            }
        );
    }

    #[test]
    fn payload_serialization_roundtrip_keeps_store_data() {
        let chunk_id = ChunkId::new(WorldId(1), ChunkCoord3 { x: 3, y: -2, z: 7 });
        let payload = occupied_payload(chunk_id);
        let mut store = SdfChunkStore::default();
        store.chunks.insert(chunk_id, payload.clone());

        let encoded = postcard::to_allocvec(&payload).expect("serialize payload");
        let decoded =
            postcard::from_bytes::<SdfChunkPayload>(&encoded).expect("deserialize payload");
        assert_eq!(decoded.chunk_id, payload.chunk_id);
        assert_eq!(decoded.page_table.len(), payload.page_table.len());

        let page = store
            .chunks
            .get(&chunk_id)
            .and_then(|chunk| chunk.page_table.get(&SdfPageCoord3 { x: 0, y: 0, z: 0 }))
            .expect("stored payload page should exist");
        assert_eq!(page.bricks.len(), 1);
    }
}
