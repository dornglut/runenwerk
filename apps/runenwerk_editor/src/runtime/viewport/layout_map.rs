//! File: apps/runenwerk_editor/src/runtime/viewport/layout_map.rs
//! Purpose: Runtime viewport host widget -> viewport-id layout mapping.

use std::collections::BTreeMap;

use editor_shell::{StructuralWidgetRoutingContext, ToolSurfaceInstanceId, WidgetId};
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
    viewport_by_tool_surface: BTreeMap<ToolSurfaceInstanceId, ViewportId>,
}

impl ViewportLayoutMapResource {
    pub fn clear(&mut self) {
        self.entries_by_viewport.clear();
        self.viewport_by_structural_context.clear();
        self.viewport_by_tool_surface.clear();
    }

    pub fn upsert_entry(&mut self, entry: ViewportLayoutEntry) {
        if let Some(previous) = self.entries_by_viewport.get(&entry.viewport_id)
            && let Some(previous_surface) = previous.structural_context.active_tool_surface
            && self
                .viewport_by_tool_surface
                .get(&previous_surface)
                .copied()
                == Some(entry.viewport_id)
        {
            self.viewport_by_tool_surface.remove(&previous_surface);
        }

        self.viewport_by_structural_context
            .insert(entry.structural_context, entry.viewport_id);
        if let Some(tool_surface_id) = entry.structural_context.active_tool_surface {
            self.viewport_by_tool_surface
                .insert(tool_surface_id, entry.viewport_id);
        }
        self.entries_by_viewport.insert(entry.viewport_id, entry);
    }

    pub fn entry_for_viewport(&self, viewport_id: ViewportId) -> Option<&ViewportLayoutEntry> {
        self.entries_by_viewport.get(&viewport_id)
    }

    pub fn viewport_for_structural_context(
        &self,
        structural_context: StructuralWidgetRoutingContext,
    ) -> Option<ViewportId> {
        structural_context
            .active_tool_surface
            .and_then(|tool_surface_id| self.viewport_for_tool_surface(tool_surface_id))
            .or_else(|| {
                self.viewport_by_structural_context
                    .get(&structural_context)
                    .copied()
            })
    }

    pub fn viewport_for_tool_surface(
        &self,
        tool_surface_id: ToolSurfaceInstanceId,
    ) -> Option<ViewportId> {
        self.viewport_by_tool_surface.get(&tool_surface_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{PanelInstanceId, TabStackId};
    use ui_math::UiRect;

    fn context(
        panel: u64,
        tab_stack: u64,
        tool_surface: Option<u64>,
    ) -> StructuralWidgetRoutingContext {
        StructuralWidgetRoutingContext {
            panel_instance_id: PanelInstanceId::new(panel),
            active_tool_surface: tool_surface.map(ToolSurfaceInstanceId::new),
            tab_stack_id: TabStackId::new(tab_stack),
        }
    }

    #[test]
    fn layout_map_routes_by_tool_surface_before_structural_context() {
        let mut map = ViewportLayoutMapResource::default();
        let initial = context(10, 20, Some(30));
        map.upsert_entry(ViewportLayoutEntry {
            viewport_id: ViewportId(1),
            host_widget_id: WidgetId(1000),
            structural_context: initial,
            bounds: UiRect::new(0.0, 0.0, 200.0, 120.0),
        });

        let moved_context_same_surface = context(11, 21, Some(30));
        assert_eq!(
            map.viewport_for_structural_context(moved_context_same_surface),
            Some(ViewportId(1)),
            "tool-surface binding must keep viewport route stable across tab/panel structural moves",
        );
        assert_eq!(
            map.viewport_for_tool_surface(ToolSurfaceInstanceId::new(30)),
            Some(ViewportId(1)),
        );
    }
}
