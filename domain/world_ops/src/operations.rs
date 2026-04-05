use crate::{OperationId, WorldRevision, WorldTick};
use serde::{Deserialize, Serialize};
use spatial::WorldId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuantizedVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuantizedAabb {
    pub min: QuantizedVec3,
    pub max: QuantizedVec3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushShape {
    Sphere {
        center_q: QuantizedVec3,
        radius_q: i32,
    },
    Capsule {
        start_q: QuantizedVec3,
        end_q: QuantizedVec3,
        radius_q: i32,
    },
    Box {
        center_q: QuantizedVec3,
        half_extents_q: QuantizedVec3,
    },
    Cylinder {
        center_q: QuantizedVec3,
        radius_q: i32,
        half_height_q: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    CsgAdd {
        brush: BrushShape,
        material_channel: u16,
    },
    CsgSubtract {
        brush: BrushShape,
    },
    Smooth {
        bounds_q: QuantizedAabb,
        kernel_radius_q: i32,
        strength_q: i32,
    },
    Stamp {
        stamp_id: String,
        anchor_q: QuantizedVec3,
        payload: Vec<u8>,
    },
    StructurePlace {
        structure_kind: String,
        anchor_q: QuantizedVec3,
        orientation_q: [i16; 4],
        payload: Vec<u8>,
    },
    StructureRemove {
        structure_instance_id: u64,
    },
    MaterialFieldEdit {
        bounds_q: QuantizedAabb,
        channel_mask: u16,
        payload: Vec<u8>,
    },
    DensityFieldDeform {
        bounds_q: QuantizedAabb,
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRecord {
    pub op_id: OperationId,
    pub base_world_revision: WorldRevision,
    #[serde(default)]
    pub planet_id: WorldId,
    pub operation: Operation,
    pub affected_bounds_q: QuantizedAabb,
    pub deterministic_seed: u64,
    pub server_tick: WorldTick,
    pub author_connection_id: Option<u64>,
}

pub fn quantize_position(position_meters: [f32; 3], fixed_point_scale: i32) -> QuantizedVec3 {
    let scale = fixed_point_scale.max(1) as f32;
    QuantizedVec3 {
        x: (position_meters[0] * scale).round() as i32,
        y: (position_meters[1] * scale).round() as i32,
        z: (position_meters[2] * scale).round() as i32,
    }
}

pub fn quantize_aabb(
    min_meters: [f32; 3],
    max_meters: [f32; 3],
    fixed_point_scale: i32,
) -> QuantizedAabb {
    QuantizedAabb {
        min: quantize_position(min_meters, fixed_point_scale),
        max: quantize_position(max_meters, fixed_point_scale),
    }
}

#[cfg(test)]
mod tests {
    use super::{QuantizedVec3, quantize_position};

    #[test]
    fn quantization_rounds_to_fixed_scale() {
        assert_eq!(
            quantize_position([1.25, -2.25, 0.0], 4),
            QuantizedVec3 {
                x: 5,
                y: -9,
                z: 0
            }
        );
    }
}
