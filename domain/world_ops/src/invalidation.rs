use crate::{DirtyChunkMap, DirtyReason, Operation, OperationLog, QuantizedAabb, QuantizedVec3};
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
            dirty_reason_for_operation(&record.operation),
        );
    }
}

pub fn mark_dirty_chunks_from_quantized_bounds(
    dirty: &mut DirtyChunkMap,
    partition: &GridPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: WorldId,
    fixed_point_scale: i32,
    reason: DirtyReason,
) -> BTreeSet<ChunkId> {
    let touched_chunks =
        touched_chunks_from_quantized_bounds(partition, bounds_q, planet_id, fixed_point_scale);
    for chunk_id in touched_chunks.iter().copied() {
        dirty.mark_dirty(chunk_id, reason);
    }
    touched_chunks
}

pub fn dirty_reason_for_operation(operation: &Operation) -> DirtyReason {
    match operation {
        Operation::CsgAdd { .. }
        | Operation::CsgSubtract { .. }
        | Operation::CsgBrush(_)
        | Operation::Smooth { .. }
        | Operation::Stamp { .. }
        | Operation::DensityFieldDeform { .. } => DirtyReason::Geometry,
        Operation::MaterialFieldEdit { .. } => DirtyReason::MaterialField,
        Operation::StructurePlace { .. } | Operation::StructureRemove { .. } => {
            DirtyReason::Structure
        }
    }
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
    use crate::{
        BrushShape, CsgBooleanMode, CsgBrushOperation, Operation, dirty_reason_for_operation,
        quantize_aabb,
    };
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

    #[test]
    fn dirty_reason_follows_operation_kind() {
        let bounds = quantize_aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 1);

        assert_eq!(
            dirty_reason_for_operation(&Operation::MaterialFieldEdit {
                bounds_q: bounds,
                channel_mask: 1,
                payload: Vec::new(),
            }),
            crate::DirtyReason::MaterialField
        );
        assert_eq!(
            dirty_reason_for_operation(&Operation::StructurePlace {
                structure_kind: "tree".to_string(),
                anchor_q: bounds.min,
                orientation_q: [0, 0, 0, 1],
                payload: Vec::new(),
            }),
            crate::DirtyReason::Structure
        );
        assert_eq!(
            dirty_reason_for_operation(&Operation::DensityFieldDeform {
                bounds_q: bounds,
                payload: Vec::new(),
            }),
            crate::DirtyReason::Geometry
        );
        assert_eq!(
            dirty_reason_for_operation(&Operation::CsgBrush(CsgBrushOperation {
                brush: BrushShape::Sphere {
                    center_q: bounds.min,
                    radius_q: 1,
                },
                mode: CsgBooleanMode::SmoothSubtract { radius_q: 2 },
                material_channel: None,
            })),
            crate::DirtyReason::Geometry
        );
    }
}
