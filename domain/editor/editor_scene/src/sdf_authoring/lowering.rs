//! File: domain/editor/editor_scene/src/sdf_authoring/lowering.rs
//! Purpose: Deterministic lowering from authored SDF operations into world_ops records.

use std::collections::BTreeSet;

use spatial::{ChunkId, GridPartitionConfig, WorldId};
use world_ops::{
    BrushShape, CsgBooleanMode, CsgBrushOperation, Operation, OperationId, OperationRecord,
    QuantizedAabb, QuantizedVec3, ReplayWindow, WorldRevision, quantize_aabb,
    touched_chunks_from_quantized_bounds,
};

use crate::{
    SceneTransform, SdfBooleanIntent, SdfOperationDocument, SdfOperationEntry, SdfOperationEntryId,
    SdfOperationIssueSeverity, SdfOperationLayerId, SdfOperationRatificationReport,
    SdfPrimitiveKind, ratify_sdf_operation_document,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SdfOperationLoweringContext {
    pub base_world_revision: WorldRevision,
    pub planet_id: WorldId,
    pub partition: GridPartitionConfig,
}

impl Default for SdfOperationLoweringContext {
    fn default() -> Self {
        Self {
            base_world_revision: WorldRevision(0),
            planet_id: WorldId(1),
            partition: GridPartitionConfig::default(),
        }
    }
}

impl SdfOperationLoweringContext {
    pub fn fixed_point_scale(&self) -> i32 {
        self.partition.quantization_scale()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfOperationSourceRef {
    pub layer_id: SdfOperationLayerId,
    pub operation_id: SdfOperationEntryId,
    pub source_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfLoweredOperationRecord {
    pub source: SdfOperationSourceRef,
    pub record: OperationRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SdfOperationWindowCandidate {
    pub source_revision: u64,
    pub replay_window: ReplayWindow,
    pub records: Vec<SdfLoweredOperationRecord>,
    pub touched_chunks: BTreeSet<ChunkId>,
    pub ratification: SdfOperationRatificationReport,
}

impl SdfOperationWindowCandidate {
    pub fn can_commit(&self) -> bool {
        !self.ratification.has_blocking_issues()
    }

    pub fn operation_count(&self) -> usize {
        self.records.len()
    }
}

pub fn lower_sdf_operation_document(
    document: &SdfOperationDocument,
    context: &SdfOperationLoweringContext,
) -> SdfOperationWindowCandidate {
    let mut ratification = ratify_sdf_operation_document(document);
    let mut records = Vec::new();
    let mut touched_chunks = BTreeSet::new();
    let fixed_point_scale = context.fixed_point_scale();

    for layer in document
        .layers()
        .iter()
        .filter(|layer| layer.metadata.enabled)
    {
        for operation in layer
            .operations
            .iter()
            .filter(|operation| operation.enabled)
        {
            let Some(record) = lower_operation(
                layer.id,
                operation,
                context,
                fixed_point_scale,
                &mut ratification,
            ) else {
                continue;
            };
            touched_chunks.extend(touched_chunks_from_quantized_bounds(
                &context.partition,
                record.record.affected_bounds_q,
                context.planet_id,
                fixed_point_scale,
            ));
            records.push(record);
        }
    }

    let target_op_inclusive = OperationId(records.len() as u64);
    SdfOperationWindowCandidate {
        source_revision: document.source_revision(),
        replay_window: ReplayWindow {
            applied_op_exclusive: OperationId(0),
            target_op_inclusive,
        },
        records,
        touched_chunks,
        ratification,
    }
}

fn lower_operation(
    layer_id: SdfOperationLayerId,
    operation: &SdfOperationEntry,
    context: &SdfOperationLoweringContext,
    fixed_point_scale: i32,
    ratification: &mut SdfOperationRatificationReport,
) -> Option<SdfLoweredOperationRecord> {
    let brush = brush_shape_for_primitive(operation, fixed_point_scale)?;
    let affected_bounds_q = affected_bounds_for_primitive(operation, fixed_point_scale);
    let mode = boolean_mode_for_operation(operation, fixed_point_scale, ratification)?;
    let material_channel = match mode {
        CsgBooleanMode::Add
        | CsgBooleanMode::Intersect
        | CsgBooleanMode::SmoothAdd { .. }
        | CsgBooleanMode::SmoothIntersect { .. } => Some(operation.material_channel),
        CsgBooleanMode::Subtract | CsgBooleanMode::SmoothSubtract { .. } => None,
    };
    let world_op = Operation::CsgBrush(CsgBrushOperation {
        brush,
        mode,
        material_channel,
    });
    let record = OperationRecord {
        op_id: OperationId(0),
        base_world_revision: context.base_world_revision,
        planet_id: context.planet_id,
        operation: world_op,
        affected_bounds_q,
        deterministic_seed: operation.deterministic_seed,
    };
    Some(SdfLoweredOperationRecord {
        source: SdfOperationSourceRef {
            layer_id,
            operation_id: operation.id,
            source_revision: operation.source_revision,
        },
        record,
    })
}

fn boolean_mode_for_operation(
    operation: &SdfOperationEntry,
    fixed_point_scale: i32,
    _ratification: &mut SdfOperationRatificationReport,
) -> Option<CsgBooleanMode> {
    let smooth_radius_q = || {
        operation
            .primitive
            .smooth_radius_meters
            .filter(|radius| radius.is_finite() && *radius > 0.0)
            .map(|radius| quantize_extent(radius, fixed_point_scale))
    };
    match operation.primitive.boolean {
        SdfBooleanIntent::Add => Some(CsgBooleanMode::Add),
        SdfBooleanIntent::Subtract => Some(CsgBooleanMode::Subtract),
        SdfBooleanIntent::Intersect => Some(CsgBooleanMode::Intersect),
        SdfBooleanIntent::SmoothAdd => Some(CsgBooleanMode::SmoothAdd {
            radius_q: smooth_radius_q()?,
        }),
        SdfBooleanIntent::SmoothSubtract => Some(CsgBooleanMode::SmoothSubtract {
            radius_q: smooth_radius_q()?,
        }),
        SdfBooleanIntent::SmoothIntersect => Some(CsgBooleanMode::SmoothIntersect {
            radius_q: smooth_radius_q()?,
        }),
    }
}

fn brush_shape_for_primitive(
    operation: &SdfOperationEntry,
    fixed_point_scale: i32,
) -> Option<BrushShape> {
    let transform = operation.primitive.transform;
    let center_q = quantized_translation(transform, fixed_point_scale);
    match operation.primitive.kind {
        SdfPrimitiveKind::Sphere => Some(BrushShape::Sphere {
            center_q,
            radius_q: quantize_extent(max_abs_scale(transform), fixed_point_scale),
        }),
        SdfPrimitiveKind::Box => Some(BrushShape::Box {
            center_q,
            half_extents_q: QuantizedVec3 {
                x: quantize_extent(transform.scale.x.abs().max(0.001), fixed_point_scale),
                y: quantize_extent(transform.scale.y.abs().max(0.001), fixed_point_scale),
                z: quantize_extent(transform.scale.z.abs().max(0.001), fixed_point_scale),
            },
        }),
        SdfPrimitiveKind::Capsule => {
            let half_height = transform.scale.y.abs().max(0.001);
            let radius = transform
                .scale
                .x
                .abs()
                .max(transform.scale.z.abs())
                .max(0.001);
            Some(BrushShape::Capsule {
                start_q: quantize_vec3(
                    [
                        transform.translation.x,
                        transform.translation.y - half_height,
                        transform.translation.z,
                    ],
                    fixed_point_scale,
                ),
                end_q: quantize_vec3(
                    [
                        transform.translation.x,
                        transform.translation.y + half_height,
                        transform.translation.z,
                    ],
                    fixed_point_scale,
                ),
                radius_q: quantize_extent(radius, fixed_point_scale),
            })
        }
        SdfPrimitiveKind::Cylinder => Some(BrushShape::Cylinder {
            center_q,
            radius_q: quantize_extent(
                transform
                    .scale
                    .x
                    .abs()
                    .max(transform.scale.z.abs())
                    .max(0.001),
                fixed_point_scale,
            ),
            half_height_q: quantize_extent(transform.scale.y.abs().max(0.001), fixed_point_scale),
        }),
        SdfPrimitiveKind::Torus => Some(BrushShape::Torus {
            center_q,
            major_radius_q: quantize_extent(
                transform
                    .scale
                    .x
                    .abs()
                    .max(transform.scale.z.abs())
                    .max(0.001),
                fixed_point_scale,
            ),
            minor_radius_q: quantize_extent(transform.scale.y.abs().max(0.001), fixed_point_scale),
        }),
        SdfPrimitiveKind::Plane => Some(BrushShape::Plane {
            center_q,
            normal_q: quantize_unit_vec3(plane_normal(transform), fixed_point_scale),
            half_extents_q: QuantizedVec3 {
                x: quantize_extent(transform.scale.x.abs().max(0.001), fixed_point_scale),
                y: quantize_extent(transform.scale.y.abs().max(0.001), fixed_point_scale),
                z: quantize_extent(transform.scale.z.abs().max(0.001), fixed_point_scale),
            },
        }),
    }
}

fn affected_bounds_for_primitive(
    operation: &SdfOperationEntry,
    fixed_point_scale: i32,
) -> QuantizedAabb {
    let transform = operation.primitive.transform;
    let radius = match operation.primitive.kind {
        SdfPrimitiveKind::Sphere | SdfPrimitiveKind::Box | SdfPrimitiveKind::Plane => {
            max_abs_scale(transform)
        }
        SdfPrimitiveKind::Capsule | SdfPrimitiveKind::Cylinder => transform
            .scale
            .y
            .abs()
            .max(transform.scale.x.abs().max(transform.scale.z.abs())),
        SdfPrimitiveKind::Torus => {
            transform.scale.x.abs().max(transform.scale.z.abs()) + transform.scale.y.abs()
        }
    }
    .max(0.001)
        + operation
            .primitive
            .smooth_radius_meters
            .filter(|radius| radius.is_finite() && *radius > 0.0)
            .unwrap_or(0.0);
    quantize_aabb(
        [
            transform.translation.x - radius,
            transform.translation.y - radius,
            transform.translation.z - radius,
        ],
        [
            transform.translation.x + radius,
            transform.translation.y + radius,
            transform.translation.z + radius,
        ],
        fixed_point_scale,
    )
}

fn quantized_translation(transform: SceneTransform, fixed_point_scale: i32) -> QuantizedVec3 {
    quantize_vec3(
        [
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        ],
        fixed_point_scale,
    )
}

fn quantize_vec3(value: [f32; 3], fixed_point_scale: i32) -> QuantizedVec3 {
    world_ops::quantize_position(value, fixed_point_scale)
}

fn quantize_extent(value: f32, fixed_point_scale: i32) -> i32 {
    ((value.max(0.001)) * fixed_point_scale.max(1) as f32)
        .round()
        .max(1.0) as i32
}

fn quantize_unit_vec3(value: [f32; 3], fixed_point_scale: i32) -> QuantizedVec3 {
    quantize_vec3(value, fixed_point_scale)
}

fn plane_normal(transform: SceneTransform) -> [f32; 3] {
    let q = transform.rotation;
    let qv = glam::Vec3::new(q.x, q.y, q.z);
    let v = glam::Vec3::Y;
    let t = 2.0 * qv.cross(v);
    let normal = v + q.w * t + qv.cross(t);
    let normal = if normal.length_squared() <= f32::EPSILON {
        glam::Vec3::Y
    } else {
        normal.normalize()
    };
    [normal.x, normal.y, normal.z]
}

fn max_abs_scale(transform: SceneTransform) -> f32 {
    transform
        .scale
        .x
        .abs()
        .max(transform.scale.y.abs())
        .max(transform.scale.z.abs())
        .max(0.001)
}

pub fn commit_blocking_issue_count(candidate: &SdfOperationWindowCandidate) -> usize {
    candidate
        .ratification
        .issues
        .iter()
        .filter(|issue| issue.severity == SdfOperationIssueSeverity::Error)
        .count()
}
