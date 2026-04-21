use std::collections::BTreeMap;

use editor_viewport::ViewportId;
use engine::plugins::{RenderFlowId, RenderResourceId};

/// File: apps/runenwerk_editor/src/runtime/viewport/surface_set.rs
/// Purpose: Explicit per-viewport-owned presentation surface bundles.
///
/// Governing rule:
/// Viewports consume typed expression products, resolve them through
/// viewport-scoped presentation state, and embed viewport-owned
/// presentation surfaces into the shell.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ViewportSurfaceSlot {
	PrimaryColor,
	PickingIds,
	Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportSurfaceHandle {
	pub flow_id: RenderFlowId,
	pub resource_id: RenderResourceId,
}

impl ViewportSurfaceHandle {
	pub const fn new(flow_id: RenderFlowId, resource_id: RenderResourceId) -> Self {
		Self { flow_id, resource_id }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportSurfaceSet {
	viewport_id: ViewportId,
	slots: BTreeMap<ViewportSurfaceSlot, ViewportSurfaceHandle>,
}

impl ViewportSurfaceSet {
	pub fn new(viewport_id: ViewportId) -> Self {
		Self {
			viewport_id,
			slots: BTreeMap::new(),
		}
	}

	pub fn viewport_id(&self) -> ViewportId {
		self.viewport_id
	}

	pub fn insert(
		&mut self,
		slot: ViewportSurfaceSlot,
		handle: ViewportSurfaceHandle,
	) -> Option<ViewportSurfaceHandle> {
		self.slots.insert(slot, handle)
	}

	pub fn get(&self, slot: ViewportSurfaceSlot) -> Option<ViewportSurfaceHandle> {
		self.slots.get(&slot).copied()
	}

	pub fn contains(&self, slot: ViewportSurfaceSlot) -> bool {
		self.slots.contains_key(&slot)
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ViewportSurfaceSetResource {
	sets: BTreeMap<ViewportId, ViewportSurfaceSet>,
}

impl ViewportSurfaceSetResource {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn ensure_viewport(&mut self, viewport_id: ViewportId) -> &mut ViewportSurfaceSet {
		self.sets
			.entry(viewport_id)
			.or_insert_with(|| ViewportSurfaceSet::new(viewport_id))
	}

	pub fn set_surface(
		&mut self,
		viewport_id: ViewportId,
		slot: ViewportSurfaceSlot,
		handle: ViewportSurfaceHandle,
	) -> Option<ViewportSurfaceHandle> {
		self.ensure_viewport(viewport_id).insert(slot, handle)
	}

	pub fn surface(
		&self,
		viewport_id: ViewportId,
		slot: ViewportSurfaceSlot,
	) -> Option<ViewportSurfaceHandle> {
		self.sets.get(&viewport_id).and_then(|set| set.get(slot))
	}

	pub fn surface_set(&self, viewport_id: ViewportId) -> Option<&ViewportSurfaceSet> {
		self.sets.get(&viewport_id)
	}

	pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
		self.sets.keys().copied()
	}
}