//! File: apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs
//! Purpose: Explicit runtime viewport instance ownership.

use std::collections::{BTreeMap, BTreeSet};

use editor_shell::{
    PanelInstanceId, ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, WorkspaceState,
};
use editor_viewport::ViewportId;

use crate::runtime::viewport::MAIN_VIEWPORT_ID;

const FIRST_ALLOCATED_VIEWPORT_ID: u64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportInstanceRecord {
    pub viewport_id: ViewportId,
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub panel_instance_id: PanelInstanceId,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct ViewportInstanceRegistryResource {
    next_viewport_id: u64,
    records_by_tool_surface: BTreeMap<ToolSurfaceInstanceId, ViewportInstanceRecord>,
    tool_surface_by_viewport: BTreeMap<ViewportId, ToolSurfaceInstanceId>,
}

impl Default for ViewportInstanceRegistryResource {
    fn default() -> Self {
        Self {
            next_viewport_id: FIRST_ALLOCATED_VIEWPORT_ID,
            records_by_tool_surface: BTreeMap::new(),
            tool_surface_by_viewport: BTreeMap::new(),
        }
    }
}

impl ViewportInstanceRegistryResource {
    pub fn sync_from_workspace_state(&mut self, workspace_state: &WorkspaceState) {
        let mut active_tool_surfaces = BTreeSet::new();
        for tool_surface in workspace_state.tool_surfaces() {
            if tool_surface.tool_surface_kind != ToolSurfaceKind::Viewport {
                continue;
            }
            let ToolSurfaceMount::Mounted { panel_id } = tool_surface.mount else {
                continue;
            };
            active_tool_surfaces.insert(tool_surface.id);
            if let Some(viewport_id) = tool_surface.viewport_instance_id {
                self.restore_instance(tool_surface.id, panel_id, viewport_id);
            } else {
                self.ensure_for_tool_surface(tool_surface.id, panel_id);
            }
        }

        self.records_by_tool_surface
            .retain(|tool_surface_id, record| {
                let keep = active_tool_surfaces.contains(tool_surface_id);
                if !keep {
                    self.tool_surface_by_viewport.remove(&record.viewport_id);
                }
                keep
            });
    }

    pub fn ensure_for_tool_surface(
        &mut self,
        tool_surface_id: ToolSurfaceInstanceId,
        panel_instance_id: PanelInstanceId,
    ) -> ViewportInstanceRecord {
        if let Some(record) = self.records_by_tool_surface.get_mut(&tool_surface_id) {
            record.panel_instance_id = panel_instance_id;
            return *record;
        }

        let viewport_id = self.allocate_viewport_id();
        let record = ViewportInstanceRecord {
            viewport_id,
            tool_surface_id,
            panel_instance_id,
        };
        self.records_by_tool_surface.insert(tool_surface_id, record);
        self.tool_surface_by_viewport
            .insert(viewport_id, tool_surface_id);
        record
    }

    pub fn restore_instance(
        &mut self,
        tool_surface_id: ToolSurfaceInstanceId,
        panel_instance_id: PanelInstanceId,
        viewport_id: ViewportId,
    ) -> ViewportInstanceRecord {
        if let Some(record) = self.records_by_tool_surface.get_mut(&tool_surface_id)
            && record.viewport_id == viewport_id
        {
            record.panel_instance_id = panel_instance_id;
            self.next_viewport_id = self.next_viewport_id.max(viewport_id.0.saturating_add(1));
            return *record;
        }
        if let Some(previous_tool_surface) = self.tool_surface_by_viewport.get(&viewport_id)
            && *previous_tool_surface != tool_surface_id
        {
            self.records_by_tool_surface.remove(previous_tool_surface);
        }
        if let Some(previous_record) = self.records_by_tool_surface.get(&tool_surface_id) {
            self.tool_surface_by_viewport
                .remove(&previous_record.viewport_id);
        }

        self.next_viewport_id = self.next_viewport_id.max(viewport_id.0.saturating_add(1));
        let record = ViewportInstanceRecord {
            viewport_id,
            tool_surface_id,
            panel_instance_id,
        };
        self.records_by_tool_surface.insert(tool_surface_id, record);
        self.tool_surface_by_viewport
            .insert(viewport_id, tool_surface_id);
        record
    }

    pub fn duplicate_instance(
        &mut self,
        source_tool_surface_id: ToolSurfaceInstanceId,
        target_tool_surface_id: ToolSurfaceInstanceId,
        target_panel_instance_id: PanelInstanceId,
    ) -> Option<ViewportInstanceRecord> {
        self.records_by_tool_surface.get(&source_tool_surface_id)?;
        Some(self.ensure_for_tool_surface(target_tool_surface_id, target_panel_instance_id))
    }

    pub fn close_instance(
        &mut self,
        tool_surface_id: ToolSurfaceInstanceId,
    ) -> Option<ViewportInstanceRecord> {
        let record = self.records_by_tool_surface.remove(&tool_surface_id)?;
        self.tool_surface_by_viewport.remove(&record.viewport_id);
        Some(record)
    }

    pub fn viewport_for_tool_surface(
        &self,
        tool_surface_id: ToolSurfaceInstanceId,
    ) -> Option<ViewportId> {
        self.records_by_tool_surface
            .get(&tool_surface_id)
            .map(|record| record.viewport_id)
    }

    pub fn record_for_viewport(&self, viewport_id: ViewportId) -> Option<ViewportInstanceRecord> {
        let tool_surface_id = self.tool_surface_by_viewport.get(&viewport_id)?;
        self.records_by_tool_surface.get(tool_surface_id).copied()
    }

    pub fn records(&self) -> impl Iterator<Item = ViewportInstanceRecord> + '_ {
        self.records_by_tool_surface.values().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.records_by_tool_surface.is_empty()
    }

    fn allocate_viewport_id(&mut self) -> ViewportId {
        let mut viewport_id = ViewportId(self.next_viewport_id.max(FIRST_ALLOCATED_VIEWPORT_ID));
        while viewport_id == MAIN_VIEWPORT_ID
            || self.tool_surface_by_viewport.contains_key(&viewport_id)
        {
            viewport_id = ViewportId(viewport_id.0.saturating_add(1));
        }
        self.next_viewport_id = viewport_id.0.saturating_add(1);
        viewport_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{
        WorkspaceId, WorkspaceIdentityAllocator, WorkspaceMutation, reduce_workspace,
    };

    fn ids() -> (
        ToolSurfaceInstanceId,
        ToolSurfaceInstanceId,
        PanelInstanceId,
        PanelInstanceId,
    ) {
        (
            ToolSurfaceInstanceId::try_from_raw(11).unwrap(),
            ToolSurfaceInstanceId::try_from_raw(12).unwrap(),
            PanelInstanceId::try_from_raw(21).unwrap(),
            PanelInstanceId::try_from_raw(22).unwrap(),
        )
    }

    #[test]
    fn allocates_stable_viewport_identity_per_tool_surface() {
        let (surface_a, surface_b, panel_a, panel_b) = ids();
        let mut registry = ViewportInstanceRegistryResource::default();

        let first = registry.ensure_for_tool_surface(surface_a, panel_a);
        let again = registry.ensure_for_tool_surface(surface_a, panel_b);
        let second = registry.ensure_for_tool_surface(surface_b, panel_b);

        assert_eq!(first.viewport_id, again.viewport_id);
        assert_eq!(again.panel_instance_id, panel_b);
        assert_ne!(first.viewport_id, second.viewport_id);
        assert_ne!(first.viewport_id, MAIN_VIEWPORT_ID);
    }

    #[test]
    fn restore_reuses_supplied_viewport_identity() {
        let (surface_a, _, panel_a, _) = ids();
        let mut registry = ViewportInstanceRegistryResource::default();
        let restored = registry.restore_instance(surface_a, panel_a, ViewportId(77));

        assert_eq!(restored.viewport_id, ViewportId(77));
        assert_eq!(
            registry.viewport_for_tool_surface(surface_a),
            Some(ViewportId(77))
        );
    }

    #[test]
    fn sync_retains_only_mounted_viewport_surfaces() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let mut registry = ViewportInstanceRegistryResource::default();

        registry.sync_from_workspace_state(&workspace);

        assert!(registry.records().any(|record| {
            workspace
                .tool_surface(record.tool_surface_id)
                .is_some_and(|surface| surface.tool_surface_kind == ToolSurfaceKind::Viewport)
        }));
    }

    #[test]
    fn sync_restores_persisted_viewport_identity() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let viewport_surface = workspace
            .tool_surfaces()
            .find(|surface| surface.tool_surface_kind == ToolSurfaceKind::Viewport)
            .map(|surface| surface.id)
            .expect("bootstrap workspace should contain a viewport surface");
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetToolSurfaceViewportInstanceId {
                tool_surface_id: viewport_surface,
                viewport_instance_id: Some(ViewportId(77)),
            },
        )
        .expect("restore id mutation should be valid");
        let mut registry = ViewportInstanceRegistryResource::default();

        registry.sync_from_workspace_state(&workspace);

        assert_eq!(
            registry.viewport_for_tool_surface(viewport_surface),
            Some(ViewportId(77))
        );
    }
}
