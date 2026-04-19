//! File: apps/runenwerk_editor/src/runtime/expression/picking.rs
//! Purpose: Convert picking runtime resources into expression-frame contracts.

use editor_core::{ComponentTypeId, EntityId, RealityVersion};
use editor_shell::{PickingExpressionAxis, PickingExpressionFrame, PickingExpressionTarget};
use editor_viewport::{ViewportHitResult, ViewportId};
use engine::plugins::render::{EditorGizmoAxis, EditorPickingResultResource, EditorPickingTarget};

pub fn build_picking_expression_frame(
    picking: &EditorPickingResultResource,
    source_version: RealityVersion,
) -> PickingExpressionFrame {
    PickingExpressionFrame::new(
        source_version,
        map_picking_target(picking.hit.target),
        picking.hit.distance,
    )
}

pub fn viewport_hit_from_picking_expression(frame: &PickingExpressionFrame) -> ViewportHitResult {
    match frame.target {
        PickingExpressionTarget::None => ViewportHitResult::none(),
        PickingExpressionTarget::Grid => ViewportHitResult::grid(frame.distance),
        PickingExpressionTarget::Entity(entity) => {
            ViewportHitResult::entity(entity, frame.distance)
        }
        PickingExpressionTarget::ComponentHandle {
            entity,
            component_type,
        } => ViewportHitResult::component_handle(entity, component_type, frame.distance),
        PickingExpressionTarget::GizmoAxis(axis) => {
            ViewportHitResult::gizmo_axis(axis_label(axis), frame.distance)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportPickingProductFrame {
    pub viewport_id: ViewportId,
    pub expression: PickingExpressionFrame,
}

pub fn build_viewport_picking_product_frame(
    viewport_id: ViewportId,
    picking: &EditorPickingResultResource,
    source_version: RealityVersion,
) -> ViewportPickingProductFrame {
    ViewportPickingProductFrame {
        viewport_id,
        expression: build_picking_expression_frame(picking, source_version),
    }
}

pub fn viewport_hit_from_picking_product(frame: &ViewportPickingProductFrame) -> ViewportHitResult {
    viewport_hit_from_picking_expression(&frame.expression)
}

fn map_picking_target(target: EditorPickingTarget) -> PickingExpressionTarget {
    match target {
        EditorPickingTarget::None => PickingExpressionTarget::None,
        EditorPickingTarget::Grid => PickingExpressionTarget::Grid,
        EditorPickingTarget::Entity(entity) => PickingExpressionTarget::Entity(EntityId(entity)),
        EditorPickingTarget::ComponentHandle {
            entity,
            component_type,
        } => PickingExpressionTarget::ComponentHandle {
            entity: EntityId(entity),
            component_type: ComponentTypeId(component_type),
        },
        EditorPickingTarget::GizmoAxis(axis) => PickingExpressionTarget::GizmoAxis(map_axis(axis)),
    }
}

fn map_axis(axis: EditorGizmoAxis) -> PickingExpressionAxis {
    match axis {
        EditorGizmoAxis::X => PickingExpressionAxis::X,
        EditorGizmoAxis::Y => PickingExpressionAxis::Y,
        EditorGizmoAxis::Z => PickingExpressionAxis::Z,
    }
}

fn axis_label(axis: PickingExpressionAxis) -> &'static str {
    match axis {
        PickingExpressionAxis::X => "X",
        PickingExpressionAxis::Y => "Y",
        PickingExpressionAxis::Z => "Z",
    }
}
