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

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportHitResult {
	pub target: ViewportHitTarget,
	pub distance: f32,
}