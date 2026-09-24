//! File: domain/editor/editor_viewport/src/hit.rs

use editor_core::{ComponentTypeId, EntityId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewportHitTarget {
    Entity(EntityId),
    ComponentHandle {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
    GizmoAxis(&'static str),
    Grid,
    None,
}

impl ViewportHitTarget {
    pub fn entity(entity: EntityId) -> Self {
        Self::Entity(entity)
    }

    pub fn component_handle(entity: EntityId, component_type: ComponentTypeId) -> Self {
        Self::ComponentHandle {
            entity,
            component_type,
        }
    }

    pub fn gizmo_axis(axis: &'static str) -> Self {
        Self::GizmoAxis(axis)
    }

    pub fn grid() -> Self {
        Self::Grid
    }

    pub fn none() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportHitResult {
    pub target: ViewportHitTarget,
    pub distance: f32,
}

impl ViewportHitResult {
    pub fn new(target: ViewportHitTarget, distance: f32) -> Self {
        Self { target, distance }
    }

    pub fn entity(entity: EntityId, distance: f32) -> Self {
        Self::new(ViewportHitTarget::entity(entity), distance)
    }

    pub fn component_handle(
        entity: EntityId,
        component_type: ComponentTypeId,
        distance: f32,
    ) -> Self {
        Self::new(
            ViewportHitTarget::component_handle(entity, component_type),
            distance,
        )
    }

    pub fn gizmo_axis(axis: &'static str, distance: f32) -> Self {
        Self::new(ViewportHitTarget::gizmo_axis(axis), distance)
    }

    pub fn grid(distance: f32) -> Self {
        Self::new(ViewportHitTarget::grid(), distance)
    }

    pub fn none() -> Self {
        Self::new(ViewportHitTarget::none(), f32::INFINITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_entity_hit_result() {
        let hit = ViewportHitResult::entity(EntityId(1), 2.5);

        assert_eq!(hit.target, ViewportHitTarget::Entity(EntityId(1)));
        assert_eq!(hit.distance, 2.5);
    }

    #[test]
    fn builds_none_hit_with_infinite_distance() {
        let hit = ViewportHitResult::none();

        assert_eq!(hit.target, ViewportHitTarget::None);
        assert!(hit.distance.is_infinite());
    }
}
