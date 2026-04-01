use super::*;
use crate::GeometryBounds3;
use engine::plugins::world::edits::operation::{
    WorldBrushShape, WorldOperation, quantize_aabb, quantize_position,
};
use engine::plugins::world::edits::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::ids::PlanetId;
use engine::prelude::SimulationTick;

pub(crate) fn apply_runtime_geometry_edit(world: &mut World, edit: &GeometryEdit) -> bool {
    let (affected, revision, world_bounds) = {
        let graph = match world.resource_mut::<CavernGeometryGraph>() {
            Ok(graph) => graph,
            Err(_) => return false,
        };
        let affected = graph.apply_edit(edit);
        (affected, graph.revision, graph.bounds)
    };
    let event = GeometryEditEvent {
        revision,
        edit: edit.clone(),
    };

    if let Some(bounds) = affected
        && let Ok(mut field) = world.resource_mut::<CavernCollisionField>()
    {
        field.invalidate_bounds(bounds);
        field.revision_seen = revision;
        field.world_bounds = world_bounds;
    }

    if let Ok(mut runtime) = world.resource_mut::<CavernGeometryRuntimeState>() {
        runtime.edit_events.push(event);
    }

    if let Some(bounds) = affected {
        mirror_edit_into_world_op_log(world, edit, bounds);
    }

    true
}

fn mirror_edit_into_world_op_log(world: &mut World, edit: &GeometryEdit, bounds: GeometryBounds3) {
    let fixed_point_scale = world
        .resource::<engine::plugins::world::WorldRuntimeConfig>()
        .map(|config| config.fixed_point_scale)
        .unwrap_or(1024);

    let affected_bounds_q = quantize_aabb(bounds.min, bounds.max, fixed_point_scale);
    let operation = map_geometry_edit_to_world_operation(edit, bounds, fixed_point_scale);
    let server_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let _ = submit_world_operation(
        world,
        operation,
        affected_bounds_q,
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: hash_edit_for_seed(edit),
            server_tick,
            author_connection_id: None,
        },
    );
}

fn map_geometry_edit_to_world_operation(
    edit: &GeometryEdit,
    bounds: GeometryBounds3,
    fixed_point_scale: i32,
) -> WorldOperation {
    match &edit.kind {
        GeometryEditKind::AddBlocker(shape) => WorldOperation::CsgAdd {
            brush: shape_to_world_brush(shape, fixed_point_scale),
            material_channel: 1,
        },
        GeometryEditKind::RemovePrimitive(id) => WorldOperation::StructureRemove {
            structure_instance_id: id.0,
        },
        GeometryEditKind::EnablePrimitive(id) | GeometryEditKind::DisablePrimitive(id) => {
            WorldOperation::MaterialFieldEdit {
                bounds_q: quantize_aabb(bounds.min, bounds.max, fixed_point_scale),
                channel_mask: 1,
                payload: id.0.to_le_bytes().to_vec(),
            }
        }
        GeometryEditKind::ReplacePrimitive(id, replacement) => WorldOperation::DensityFieldDeform {
            bounds_q: quantize_aabb(bounds.min, bounds.max, fixed_point_scale),
            payload: format!("replace:{}:{:?}", id.0, replacement.shape).into_bytes(),
        },
    }
}

fn shape_to_world_brush(
    shape: &GeometryPrimitiveShape3,
    fixed_point_scale: i32,
) -> WorldBrushShape {
    match shape {
        GeometryPrimitiveShape3::Sphere { center, radius } => WorldBrushShape::Sphere {
            center_q: quantize_position(*center, fixed_point_scale),
            radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
        },
        GeometryPrimitiveShape3::Capsule { start, end, radius } => WorldBrushShape::Capsule {
            start_q: quantize_position(*start, fixed_point_scale),
            end_q: quantize_position(*end, fixed_point_scale),
            radius_q: (*radius * fixed_point_scale.max(1) as f32).round() as i32,
        },
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            let start_pos = points.first().copied().unwrap_or([0.0, 0.0, 0.0]);
            let end_pos = points.last().copied().unwrap_or([0.0, 0.0, 0.0]);
            WorldBrushShape::Capsule {
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
        } => WorldBrushShape::Box {
            center_q: quantize_position(*center, fixed_point_scale),
            half_extents_q: quantize_position(*half_extents, fixed_point_scale),
        },
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => WorldBrushShape::Cylinder {
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
