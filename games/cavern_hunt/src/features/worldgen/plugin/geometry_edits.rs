use super::*;
use crate::GeometryBounds3;
use engine::plugins::world::adapters::resources::PartitionConfigResource;
use engine::plugins::world::edits::{WorldEditIngressMeta, submit_world_operation};
use engine::prelude::SimulationTick;
use spatial::WorldId;
use world_ops::{BrushShape, Operation, WorldTick, quantize_aabb, quantize_position};

pub(crate) fn apply_runtime_geometry_edit(
    world: &mut World,
    edit: &GeometryEdit,
) -> bool {
    let Some(bounds) = affected_bounds_from_edit(edit) else {
        return false;
    };
    mirror_edit_into_world_op_log(world, edit, bounds)
}

fn mirror_edit_into_world_op_log(
    world: &mut World,
    edit: &GeometryEdit,
    bounds: GeometryBounds3,
) -> bool {
    let fixed_point_scale = world
        .resource::<PartitionConfigResource>()
        .map(|config| config.quantization_scale())
        .unwrap_or(1024);

    let affected_bounds_q = quantize_aabb(bounds.min, bounds.max, fixed_point_scale);
    let Some(operation) = map_geometry_edit_to_world_operation(edit, fixed_point_scale) else {
        return false;
    };
    let server_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    submit_world_operation(
        world,
        operation,
        affected_bounds_q,
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: hash_edit_for_seed(edit),
            server_tick: WorldTick(server_tick.0),
            author_connection_id: None,
        },
    )
    .is_some()
}

fn affected_bounds_from_edit(edit: &GeometryEdit) -> Option<GeometryBounds3> {
    match &edit.kind {
        GeometryEditKind::AddBlocker(shape) | GeometryEditKind::RemoveBlocker(shape) => {
            Some(shape.bounds())
        }
        GeometryEditKind::RemovePrimitive(_)
        | GeometryEditKind::EnablePrimitive(_)
        | GeometryEditKind::DisablePrimitive(_)
        | GeometryEditKind::ReplacePrimitive(_, _) => None,
    }
}

fn map_geometry_edit_to_world_operation(
    edit: &GeometryEdit,
    fixed_point_scale: i32,
) -> Option<Operation> {
    match &edit.kind {
        GeometryEditKind::AddBlocker(shape) => Some(Operation::CsgAdd {
            brush: shape_to_world_brush(shape, fixed_point_scale),
            material_channel: 1,
        }),
        GeometryEditKind::RemoveBlocker(shape) => Some(Operation::CsgSubtract {
            brush: shape_to_world_brush(shape, fixed_point_scale),
        }),
        GeometryEditKind::RemovePrimitive(_)
        | GeometryEditKind::EnablePrimitive(_)
        | GeometryEditKind::DisablePrimitive(_)
        | GeometryEditKind::ReplacePrimitive(_, _) => None,
    }
}

fn shape_to_world_brush(
    shape: &GeometryPrimitiveShape3,
    fixed_point_scale: i32,
) -> BrushShape {
    match shape {
        GeometryPrimitiveShape3::Sphere { center, radius } => BrushShape::Sphere {
            center_q: quantize_position(*center, fixed_point_scale),
            radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
        },
        GeometryPrimitiveShape3::Capsule { start, end, radius } => BrushShape::Capsule {
            start_q: quantize_position(*start, fixed_point_scale),
            end_q: quantize_position(*end, fixed_point_scale),
            radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
        },
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            let start_pos = points.first().copied().unwrap_or([0.0, 0.0, 0.0]);
            let end_pos = points.last().copied().unwrap_or([0.0, 0.0, 0.0]);
            BrushShape::Capsule {
                start_q: quantize_position(start_pos, fixed_point_scale),
                end_q: quantize_position(end_pos, fixed_point_scale),
                radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
            }
        }
        GeometryPrimitiveShape3::Box {
            center,
            half_extents,
        }
        | GeometryPrimitiveShape3::Ellipsoid {
            center,
            radii: half_extents,
        }
        | GeometryPrimitiveShape3::RoundedBox {
            center,
            half_extents,
            ..
        } => BrushShape::Box {
            center_q: quantize_position(*center, fixed_point_scale),
            half_extents_q: quantize_position(*half_extents, fixed_point_scale),
        },
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => BrushShape::Cylinder {
            center_q: quantize_position(*center, fixed_point_scale),
            radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
            half_height_q: (*half_height * fixed_point_scale.max(1) as f32).round() as i32,
        },
    }
}

fn hash_edit_for_seed(edit: &GeometryEdit) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{edit:?}").hash(&mut hasher);
    hasher.finish()
}
