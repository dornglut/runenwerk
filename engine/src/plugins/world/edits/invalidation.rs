use super::super::chunks::partition::WorldPartitionConfig;
use super::super::chunks::{dirty::ChunkDirtyReason, dirty::WorldDirtyChunkMapResource};
use super::super::ids::PlanetId;
use super::log::WorldOperationLog;
use super::operation::QuantizedAabb;

pub fn invalidate_dirty_chunks_from_op_log(
    dirty: &mut WorldDirtyChunkMapResource,
    partition: &WorldPartitionConfig,
    log: &WorldOperationLog,
    planet_id: PlanetId,
) {
    for record in &log.operations {
        let affected = dequantize_bounds(record.affected_bounds_q, 1024);
        let min = partition.chunk_coord_from_planet_local_position(affected.0);
        let max = partition.chunk_coord_from_planet_local_position(affected.1);
        for z in min.z..=max.z {
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let chunk_id = super::super::ids::ChunkId::new(
                        planet_id,
                        super::super::ids::ChunkCoord3 { x, y, z },
                    );
                    dirty.mark_dirty(chunk_id, ChunkDirtyReason::Geometry);
                }
            }
        }
    }
}

fn dequantize_bounds(bounds_q: QuantizedAabb, fixed_point_scale: i32) -> ([f32; 3], [f32; 3]) {
    let scale = fixed_point_scale.max(1) as f32;
    let min = [
        bounds_q.min.x as f32 / scale,
        bounds_q.min.y as f32 / scale,
        bounds_q.min.z as f32 / scale,
    ];
    let max = [
        bounds_q.max.x as f32 / scale,
        bounds_q.max.y as f32 / scale,
        bounds_q.max.z as f32 / scale,
    ];
    (min, max)
}
