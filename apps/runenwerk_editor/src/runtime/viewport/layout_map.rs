//! File: apps/runenwerk_editor/src/runtime/viewport/layout_map.rs
//! Purpose: Runtime viewport host widget -> viewport-id layout mapping.

use std::collections::BTreeMap;

use editor_shell::{StructuralWidgetRoutingContext, WidgetId};
use editor_viewport::ViewportId;
use ui_math::UiRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportLayoutEntry {
    pub viewport_id: ViewportId,
    pub host_widget_id: WidgetId,
    pub structural_context: StructuralWidgetRoutingContext,
    pub bounds: UiRect,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource, Default)]
pub struct ViewportLayoutMapResource {
    entries_by_viewport: BTreeMap<ViewportId, ViewportLayoutEntry>,
    viewport_by_structural_context: BTreeMap<StructuralWidgetRoutingContext, ViewportId>,
}

impl ViewportLayoutMapResource {
    pub fn clear(&mut self) {
        self.entries_by_viewport.clear();
        self.viewport_by_structural_context.clear();
    }

    pub fn upsert_entry(&mut self, entry: ViewportLayoutEntry) {
        self.viewport_by_structural_context
            .insert(entry.structural_context, entry.viewport_id);
        self.entries_by_viewport.insert(entry.viewport_id, entry);
    }

    pub fn entry_for_viewport(&self, viewport_id: ViewportId) -> Option<&ViewportLayoutEntry> {
        self.entries_by_viewport.get(&viewport_id)
    }

    pub fn viewport_for_structural_context(
        &self,
        structural_context: StructuralWidgetRoutingContext,
    ) -> Option<ViewportId> {
        self.viewport_by_structural_context
            .get(&structural_context)
            .copied()
    }
}
