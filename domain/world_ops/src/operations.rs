use crate::{OperationId, WorldRevision};
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
    Torus {
        center_q: QuantizedVec3,
        major_radius_q: i32,
        minor_radius_q: i32,
    },
    Plane {
        center_q: QuantizedVec3,
        normal_q: QuantizedVec3,
        half_extents_q: QuantizedVec3,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsgBooleanMode {
    Add,
    Subtract,
    Intersect,
    SmoothAdd { radius_q: i32 },
    SmoothSubtract { radius_q: i32 },
    SmoothIntersect { radius_q: i32 },
}

impl CsgBooleanMode {
    pub const fn smooth_radius_q(self) -> Option<i32> {
        match self {
            Self::SmoothAdd { radius_q }
            | Self::SmoothSubtract { radius_q }
            | Self::SmoothIntersect { radius_q } => Some(radius_q),
            Self::Add | Self::Subtract | Self::Intersect => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsgBrushOperation {
    pub brush: BrushShape,
    pub mode: CsgBooleanMode,
    pub material_channel: Option<u16>,
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
    CsgBrush(CsgBrushOperation),
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
    use super::{
        BrushShape, CsgBooleanMode, CsgBrushOperation, Operation, QuantizedVec3, quantize_position,
    };

    #[test]
    fn quantization_rounds_to_fixed_scale() {
        assert_eq!(
            quantize_position([1.25, -2.25, 0.0], 4),
            QuantizedVec3 { x: 5, y: -9, z: 0 }
        );
    }

    #[test]
    fn csg_brush_operation_carries_p1_boolean_semantics() {
        let operation = Operation::CsgBrush(CsgBrushOperation {
            brush: BrushShape::Torus {
                center_q: QuantizedVec3::default(),
                major_radius_q: 8,
                minor_radius_q: 2,
            },
            mode: CsgBooleanMode::SmoothIntersect { radius_q: 4 },
            material_channel: Some(3),
        });

        let Operation::CsgBrush(brush) = operation else {
            panic!("expected normalized CSG brush operation");
        };
        assert_eq!(brush.mode.smooth_radius_q(), Some(4));
        assert_eq!(brush.material_channel, Some(3));
    }
}
