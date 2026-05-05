use std::collections::{BTreeMap, BTreeSet};

use editor_shell::{ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, WorkspaceState};

use crate::editor_features::viewport::ViewportInteractionState;
use crate::editor_panels::EntityTablePanelUiState;
use crate::editor_runtime::inspector_state::EditorInspectorUiState;

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceSessionState {
    pub entity_table_ui_state: EntityTablePanelUiState,
    pub inspector_ui_state: EditorInspectorUiState,
    pub viewport_interaction_state: ViewportInteractionState,
    pub viewport_details_visible: bool,
    pub console_follow_enabled: bool,
}

impl Default for SurfaceSessionState {
    fn default() -> Self {
        Self {
            entity_table_ui_state: EntityTablePanelUiState::new(),
            inspector_ui_state: EditorInspectorUiState::new(),
            viewport_interaction_state: ViewportInteractionState::new(),
            viewport_details_visible: false,
            console_follow_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SurfaceSessionStore {
    sessions_by_surface: BTreeMap<ToolSurfaceInstanceId, SurfaceSessionState>,
}

impl SurfaceSessionStore {
    pub fn session(&self, surface_id: ToolSurfaceInstanceId) -> Option<&SurfaceSessionState> {
        self.sessions_by_surface.get(&surface_id)
    }

    pub fn session_or_default(&self, surface_id: ToolSurfaceInstanceId) -> SurfaceSessionState {
        self.session(surface_id).cloned().unwrap_or_default()
    }

    pub fn session_mut(&mut self, surface_id: ToolSurfaceInstanceId) -> &mut SurfaceSessionState {
        self.sessions_by_surface.entry(surface_id).or_default()
    }

    pub fn viewport_interaction_state(
        &self,
        surface_id: ToolSurfaceInstanceId,
    ) -> Option<&ViewportInteractionState> {
        self.session(surface_id)
            .map(|session| &session.viewport_interaction_state)
    }

    pub fn viewport_interaction_state_mut(
        &mut self,
        surface_id: ToolSurfaceInstanceId,
    ) -> &mut ViewportInteractionState {
        &mut self.session_mut(surface_id).viewport_interaction_state
    }

    pub fn take_viewport_interaction_state(
        &mut self,
        surface_id: ToolSurfaceInstanceId,
    ) -> ViewportInteractionState {
        core::mem::take(&mut self.session_mut(surface_id).viewport_interaction_state)
    }

    pub fn replace_viewport_interaction_state(
        &mut self,
        surface_id: ToolSurfaceInstanceId,
        state: ViewportInteractionState,
    ) {
        self.session_mut(surface_id).viewport_interaction_state = state;
    }

    pub fn active_viewport_drag_surface(&self) -> Option<ToolSurfaceInstanceId> {
        let mut active = self
            .sessions_by_surface
            .iter()
            .filter_map(|(surface_id, session)| {
                session
                    .viewport_interaction_state
                    .drag_in_progress()
                    .then_some(*surface_id)
            });
        let first = active.next()?;
        active.next().is_none().then_some(first)
    }

    pub fn prune_for_workspace(&mut self, workspace: &WorkspaceState) {
        let live = workspace
            .tool_surfaces()
            .filter(|surface| retains_live_session(surface.tool_surface_kind))
            .filter(|surface| matches!(surface.mount, ToolSurfaceMount::Mounted { .. }))
            .map(|surface| surface.id)
            .collect::<BTreeSet<_>>();
        self.sessions_by_surface
            .retain(|surface_id, _| live.contains(surface_id));
    }

    pub fn clear_transient(&mut self) {
        self.sessions_by_surface.clear();
    }

    pub fn len(&self) -> usize {
        self.sessions_by_surface.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions_by_surface.is_empty()
    }
}

fn retains_live_session(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::Outliner
            | ToolSurfaceKind::EntityTable
            | ToolSurfaceKind::Viewport
            | ToolSurfaceKind::Inspector
            | ToolSurfaceKind::Console
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{WorkspaceId, WorkspaceIdentityAllocator, WorkspaceMutation};

    #[test]
    fn prune_for_workspace_removes_unmounted_surface_sessions() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let surface_id = workspace
            .tool_surfaces()
            .find(|surface| surface.tool_surface_kind == ToolSurfaceKind::EntityTable)
            .expect("entity table surface should exist")
            .id;
        let mut store = SurfaceSessionStore::default();
        store
            .session_mut(surface_id)
            .entity_table_ui_state
            .append_search_text("abc");

        let panel_id = workspace
            .panels()
            .find(|panel| panel.active_tool_surface == Some(surface_id))
            .expect("surface should be mounted")
            .id;
        let workspace = editor_shell::reduce_workspace(
            &workspace,
            WorkspaceMutation::DetachToolSurfaceFromPanel { panel_id },
        )
        .expect("surface should unmount");
        store.prune_for_workspace(&workspace);

        assert!(store.session(surface_id).is_none());
    }

    #[test]
    fn clear_transient_removes_all_surface_sessions() {
        let mut store = SurfaceSessionStore::default();
        store
            .session_mut(ToolSurfaceInstanceId::try_from_raw(7).unwrap())
            .entity_table_ui_state
            .append_search_text("abc");

        store.clear_transient();

        assert_eq!(store.len(), 0);
    }

    #[test]
    fn prune_for_workspace_retains_mounted_console_session() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let surface_id = workspace
            .tool_surfaces()
            .find(|surface| surface.tool_surface_kind == ToolSurfaceKind::Console)
            .expect("console surface should exist")
            .id;
        let mut store = SurfaceSessionStore::default();
        store.session_mut(surface_id).console_follow_enabled = false;

        store.prune_for_workspace(&workspace);

        assert_eq!(
            store
                .session(surface_id)
                .map(|session| session.console_follow_enabled),
            Some(false)
        );
    }
}
