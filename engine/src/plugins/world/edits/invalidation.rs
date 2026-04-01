use super::super::chunks::partition::WorldPartitionConfig;
use super::super::chunks::{dirty::ChunkDirtyReason, dirty::WorldDirtyChunkMapResource};
use super::super::ids::{ChunkCoord3, ChunkId, PlanetId};
use super::log::WorldOperationLog;
use super::operation::{QuantizedAabb, QuantizedVec3};
use std::collections::BTreeSet;

pub fn invalidate_dirty_chunks_from_op_log(
    dirty: &mut WorldDirtyChunkMapResource,
    partition: &WorldPartitionConfig,
    log: &WorldOperationLog,
    planet_id: PlanetId,
    fixed_point_scale: i32,
) {
    for record in &log.operations {
        invalidate_dirty_chunks_from_quantized_bounds(
            dirty,
            partition,
            record.affected_bounds_q,
            planet_id,
            fixed_point_scale,
        );
    }
}

pub fn invalidate_dirty_chunks_from_quantized_bounds(
    dirty: &mut WorldDirtyChunkMapResource,
    partition: &WorldPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: PlanetId,
    fixed_point_scale: i32,
) -> BTreeSet<ChunkId> {
    let touched_chunks =
        touched_chunks_from_quantized_bounds(partition, bounds_q, planet_id, fixed_point_scale);
    for chunk_id in touched_chunks.iter().copied() {
        dirty.mark_dirty(chunk_id, ChunkDirtyReason::Geometry);
    }
    touched_chunks
}

pub fn touched_chunks_from_quantized_bounds(
    partition: &WorldPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: PlanetId,
    fixed_point_scale: i32,
) -> BTreeSet<ChunkId> {
    let min = partition.chunk_coord_from_planet_local_position(dequantize_position(
        bounds_q.min,
        fixed_point_scale,
    ));
    let max = partition.chunk_coord_from_planet_local_position(dequantize_position(
        bounds_q.max,
        fixed_point_scale,
    ));
    let mut touched = BTreeSet::new();
    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                touched.insert(ChunkId::new(planet_id, ChunkCoord3 { x, y, z }));
            }
        }
    }
    touched
}

fn dequantize_position(position_q: QuantizedVec3, fixed_point_scale: i32) -> [f32; 3] {
    let scale = fixed_point_scale.max(1) as f32;
    [
        position_q.x as f32 / scale,
        position_q.y as f32 / scale,
        position_q.z as f32 / scale,
    ]
}
