use crate::{DirtyChunkMap, DirtyReason, OperationLog, QuantizedAabb, QuantizedVec3};
use spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use std::collections::BTreeSet;

pub fn mark_dirty_chunks_from_operation_log(
    dirty: &mut DirtyChunkMap,
    partition: &GridPartitionConfig,
    log: &OperationLog,
    fixed_point_scale: i32,
) {
    for record in &log.operations {
        mark_dirty_chunks_from_quantized_bounds(
            dirty,
            partition,
            record.affected_bounds_q,
            record.planet_id,
            fixed_point_scale,
        );
    }
}

pub fn mark_dirty_chunks_from_quantized_bounds(
    dirty: &mut DirtyChunkMap,
    partition: &GridPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: WorldId,
    fixed_point_scale: i32,
) -> BTreeSet<ChunkId> {
    let touched_chunks =
        touched_chunks_from_quantized_bounds(partition, bounds_q, planet_id, fixed_point_scale);
    for chunk_id in touched_chunks.iter().copied() {
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }
    touched_chunks
}

pub fn touched_chunks_from_quantized_bounds(
    partition: &GridPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: WorldId,
    fixed_point_scale: i32,
) -> BTreeSet<ChunkId> {
    let min = partition
        .chunk_coord_from_world_local_meters(dequantize_position(bounds_q.min, fixed_point_scale));
    let max = partition
        .chunk_coord_from_world_local_meters(dequantize_position(bounds_q.max, fixed_point_scale));
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

#[cfg(test)]
mod tests {
    use super::touched_chunks_from_quantized_bounds;
    use crate::quantize_aabb;
    use spatial::{GridPartitionConfig, WorldId};

    #[test]
    fn touched_chunks_cover_quantized_bounds() {
        let partition = GridPartitionConfig {
            chunk_edge_meters: 1.0,
            ..GridPartitionConfig::default()
        };
        let bounds = quantize_aabb([0.2, 0.2, 0.2], [2.1, 0.8, 0.8], 1);
        let chunks = touched_chunks_from_quantized_bounds(&partition, bounds, WorldId(0), 1);
        assert_eq!(chunks.len(), 12);
    }
}
