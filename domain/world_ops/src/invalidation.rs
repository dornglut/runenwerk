use crate::{
    DirtyChunkMap, DirtyReason, Operation, OperationLog, QuantizedAabb, QuantizedVec3,
    WorldQuantizationScale,
};
use runen_spatial::{
    ChunkCoord3, ChunkId, GridPartitionConfig, SpatialMathError, WorldId, WorldPosition,
};
use std::collections::BTreeSet;

pub fn mark_dirty_chunks_from_operation_log(
    dirty: &mut DirtyChunkMap,
    partition: &GridPartitionConfig,
    log: &OperationLog,
    scale: WorldQuantizationScale,
) -> Result<(), SpatialMathError> {
    for record in &log.operations {
        mark_dirty_chunks_from_quantized_bounds(
            dirty,
            partition,
            record.affected_bounds_q,
            record.planet_id,
            scale,
            dirty_reason_for_operation(&record.operation),
        )?;
    }
    Ok(())
}

pub fn mark_dirty_chunks_from_quantized_bounds(
    dirty: &mut DirtyChunkMap,
    partition: &GridPartitionConfig,
    bounds_q: QuantizedAabb,
    planet_id: WorldId,
    scale: WorldQuantizationScale,
    reason: DirtyReason,
) -> Result<BTreeSet<ChunkId>, SpatialMathError> {
    let touched_chunks =
        touched_chunks_from_quantized_bounds(partition, bounds_q, planet_id, scale)?;
    for chunk_id in touched_chunks.iter().copied() {
        dirty.mark_dirty(chunk_id, reason);
    }
    Ok(touched_chunks)
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
    scale: WorldQuantizationScale,
) -> Result<BTreeSet<ChunkId>, SpatialMathError> {
    let min_position = WorldPosition::try_new(planet_id, dequantize_position(bounds_q.min, scale))?;
    let max_position = WorldPosition::try_new(planet_id, dequantize_position(bounds_q.max, scale))?;
    let min = partition.chunk_coord_from_world_position(min_position)?;
    let max = partition.chunk_coord_from_world_position(max_position)?;
    let mut touched = BTreeSet::new();
    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                touched.insert(ChunkId::new(planet_id, ChunkCoord3 { x, y, z }));
            }
        }
    }
    Ok(touched)
}

fn dequantize_position(position_q: QuantizedVec3, scale: WorldQuantizationScale) -> [f64; 3] {
    let scale = f64::from(scale.get());
    [
        f64::from(position_q.x) / scale,
        f64::from(position_q.y) / scale,
        f64::from(position_q.z) / scale,
    ]
}

#[cfg(test)]
mod tests {
    use super::touched_chunks_from_quantized_bounds;
    use crate::{
        BrushShape, CsgBooleanMode, CsgBrushOperation, Operation, WorldQuantizationScale,
        dirty_reason_for_operation, quantize_aabb,
    };
    use runen_spatial::{GridPartitionConfig, WorldId};

    #[test]
    fn touched_chunks_cover_quantized_bounds() {
        let partition = GridPartitionConfig::try_new(1.0, [8, 8, 8])
            .expect("test partition configuration is valid");
        let scale = WorldQuantizationScale::try_new(1).expect("positive scale is valid");
        let bounds = quantize_aabb([0.2, 0.2, 0.2], [2.1, 0.8, 0.8], scale);
        let chunks =
            touched_chunks_from_quantized_bounds(&partition, bounds, WorldId::new(0), scale)
                .expect("test bounds map into spatial coordinates");
        assert_eq!(chunks.len(), 12);
    }

    #[test]
    fn dirty_reason_follows_operation_kind() {
        let scale = WorldQuantizationScale::try_new(1).expect("positive scale is valid");
        let bounds = quantize_aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], scale);

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
