use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use editor_shell::ToolSurfaceStableKey;
use editor_shell::{EditorCompositionRuntime, ToolSurfaceInstanceId};
use editor_viewport::ViewportId;
use ui_composition::MountedUnitId;
use ui_math::UiPoint;

use crate::editor_features::Phase12aInteractionProofHost;
use crate::editor_features::viewport::ViewportInteractionState;
use crate::editor_panels::EntityTablePanelUiState;
use crate::editor_runtime::inspector_state::EditorInspectorUiState;
use crate::shell::tool_suites::{
    ASSET_BROWSER_SURFACE_KEY, EDITOR_CONSOLE_SURFACE_KEY, FIELD_PRODUCT_VIEWER_SURFACE_KEY,
    IMPORT_INSPECTOR_SURFACE_KEY, SCENE_ENTITY_TABLE_SURFACE_KEY, SCENE_INSPECTOR_SURFACE_KEY,
    SCENE_OUTLINER_SURFACE_KEY, SCENE_VIEWPORT_SURFACE_KEY, SDF_BRUSH_BROWSER_SURFACE_KEY,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceSessionState {
    pub content_liveness: ui_composition::ContentLiveness,
    pub entity_table_ui_state: EntityTablePanelUiState,
    pub inspector_ui_state: EditorInspectorUiState,
    pub viewport_interaction_state: ViewportInteractionState,
    pub phase12_interaction_proof_host: Phase12aInteractionProofHost,
    pub viewport_details_visible: bool,
    pub viewport_statistics_visible: bool,
    pub viewport_options_menu_open: bool,
    pub viewport_tools_menu_open: bool,
    pub viewport_tool_radial_session: Option<ViewportToolRadialSession>,
    pub console_follow_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportToolRadialSession {
    pub tool_surface_id: ToolSurfaceInstanceId,
    pub viewport_id: ViewportId,
    pub anchor_position: UiPoint,
    pub opened_by_tab_hold: bool,
}

impl Default for SurfaceSessionState {
    fn default() -> Self {
        Self {
            content_liveness: ui_composition::ContentLiveness::Resolved,
            entity_table_ui_state: EntityTablePanelUiState::new(),
            inspector_ui_state: EditorInspectorUiState::new(),
            viewport_interaction_state: ViewportInteractionState::new(),
            phase12_interaction_proof_host: Phase12aInteractionProofHost::new(),
            viewport_details_visible: false,
            viewport_statistics_visible: false,
            viewport_options_menu_open: false,
            viewport_tools_menu_open: false,
            viewport_tool_radial_session: None,
            console_follow_enabled: true,
        }
    }
}

impl SurfaceSessionState {
    pub fn phase12_interaction_proof_host(&self) -> &Phase12aInteractionProofHost {
        &self.phase12_interaction_proof_host
    }

    pub fn phase12_interaction_proof_host_mut(&mut self) -> &mut Phase12aInteractionProofHost {
        &mut self.phase12_interaction_proof_host
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SurfaceSessionStore {
    sessions_by_mounted_unit: BTreeMap<MountedUnitId, SurfaceSessionState>,
}

#[cfg(test)]
pub trait SurfaceSessionTestKey {
    fn mounted_unit_id(self) -> MountedUnitId;
}

#[cfg(test)]
impl SurfaceSessionTestKey for MountedUnitId {
    fn mounted_unit_id(self) -> MountedUnitId {
        self
    }
}

#[cfg(test)]
impl SurfaceSessionTestKey for ToolSurfaceInstanceId {
    fn mounted_unit_id(self) -> MountedUnitId {
        MountedUnitId::try_from_raw(self.raw()).expect("test surface IDs must be non-zero")
    }
}

impl SurfaceSessionStore {
    #[cfg(not(test))]
    pub fn session(&self, mounted_unit_id: MountedUnitId) -> Option<&SurfaceSessionState> {
        self.sessions_by_mounted_unit.get(&mounted_unit_id)
    }

    #[cfg(test)]
    pub fn session(&self, key: impl SurfaceSessionTestKey) -> Option<&SurfaceSessionState> {
        self.sessions_by_mounted_unit.get(&key.mounted_unit_id())
    }

    #[cfg(not(test))]
    pub fn session_or_default(&self, mounted_unit_id: MountedUnitId) -> SurfaceSessionState {
        self.session(mounted_unit_id).cloned().unwrap_or_default()
    }

    #[cfg(test)]
    pub fn session_or_default(&self, key: impl SurfaceSessionTestKey) -> SurfaceSessionState {
        self.session(key).cloned().unwrap_or_default()
    }

    #[cfg(not(test))]
    pub fn session_mut(&mut self, mounted_unit_id: MountedUnitId) -> &mut SurfaceSessionState {
        self.sessions_by_mounted_unit
            .entry(mounted_unit_id)
            .or_default()
    }

    #[cfg(test)]
    pub fn session_mut(&mut self, key: impl SurfaceSessionTestKey) -> &mut SurfaceSessionState {
        self.sessions_by_mounted_unit
            .entry(key.mounted_unit_id())
            .or_default()
    }

    #[cfg(not(test))]
    pub fn viewport_interaction_state(
        &self,
        mounted_unit_id: MountedUnitId,
    ) -> Option<&ViewportInteractionState> {
        self.session(mounted_unit_id)
            .map(|session| &session.viewport_interaction_state)
    }

    #[cfg(test)]
    pub fn viewport_interaction_state(
        &self,
        key: impl SurfaceSessionTestKey,
    ) -> Option<&ViewportInteractionState> {
        self.session(key)
            .map(|session| &session.viewport_interaction_state)
    }

    #[cfg(not(test))]
    pub fn viewport_interaction_state_mut(
        &mut self,
        mounted_unit_id: MountedUnitId,
    ) -> &mut ViewportInteractionState {
        &mut self.session_mut(mounted_unit_id).viewport_interaction_state
    }

    #[cfg(test)]
    pub fn viewport_interaction_state_mut(
        &mut self,
        key: impl SurfaceSessionTestKey,
    ) -> &mut ViewportInteractionState {
        &mut self.session_mut(key).viewport_interaction_state
    }

    #[cfg(not(test))]
    pub fn take_viewport_interaction_state(
        &mut self,
        mounted_unit_id: MountedUnitId,
    ) -> ViewportInteractionState {
        core::mem::take(&mut self.session_mut(mounted_unit_id).viewport_interaction_state)
    }

    #[cfg(test)]
    pub fn take_viewport_interaction_state(
        &mut self,
        key: impl SurfaceSessionTestKey,
    ) -> ViewportInteractionState {
        core::mem::take(&mut self.session_mut(key).viewport_interaction_state)
    }

    #[cfg(not(test))]
    pub fn replace_viewport_interaction_state(
        &mut self,
        mounted_unit_id: MountedUnitId,
        state: ViewportInteractionState,
    ) {
        self.session_mut(mounted_unit_id).viewport_interaction_state = state;
    }

    #[cfg(test)]
    pub fn replace_viewport_interaction_state(
        &mut self,
        key: impl SurfaceSessionTestKey,
        state: ViewportInteractionState,
    ) {
        self.session_mut(key).viewport_interaction_state = state;
    }

    pub fn active_viewport_drag_mounted_unit(&self) -> Option<MountedUnitId> {
        let mut active =
            self.sessions_by_mounted_unit
                .iter()
                .filter_map(|(mounted_unit_id, session)| {
                    session
                        .viewport_interaction_state
                        .drag_in_progress()
                        .then_some(*mounted_unit_id)
                });
        let first = active.next()?;
        active.next().is_none().then_some(first)
    }

    pub fn prune_for_composition(&mut self, runtime: &EditorCompositionRuntime) {
        let live = runtime
            .extension()
            .mounted_units()
            .iter()
            .filter(|record| retains_live_session_key_str(&record.stable_content_key))
            .map(|record| record.mounted_unit_id)
            .collect::<BTreeSet<_>>();
        self.sessions_by_mounted_unit
            .retain(|mounted_unit_id, _| live.contains(mounted_unit_id));
    }

    #[cfg(test)]
    pub fn prune_for_workspace(&mut self, workspace: &editor_shell::WorkspaceState) {
        let live = workspace
            .tool_surfaces()
            .filter(|surface| retains_live_session_key(surface.stable_surface_key()))
            .filter(|surface| {
                matches!(
                    surface.mount,
                    editor_shell::ToolSurfaceMount::Mounted { .. }
                )
            })
            .filter_map(|surface| MountedUnitId::try_from_raw(surface.id.raw()).ok())
            .collect::<BTreeSet<_>>();
        self.sessions_by_mounted_unit
            .retain(|mounted_unit_id, _| live.contains(mounted_unit_id));
    }

    #[cfg(test)]
    pub fn active_viewport_drag_surface(&self) -> Option<ToolSurfaceInstanceId> {
        self.active_viewport_drag_mounted_unit()
            .and_then(|id| ToolSurfaceInstanceId::try_from_raw(id.raw()).ok())
    }

    pub fn clear_transient(&mut self) {
        self.sessions_by_mounted_unit.clear();
    }

    pub fn close_all_viewport_options_menus(&mut self) -> bool {
        let mut changed = false;
        for session in self.sessions_by_mounted_unit.values_mut() {
            if session.viewport_options_menu_open {
                session.viewport_options_menu_open = false;
                changed = true;
            }
        }
        changed
    }

    pub fn close_all_viewport_tool_radial_menus(&mut self) -> bool {
        let mut changed = false;
        for session in self.sessions_by_mounted_unit.values_mut() {
            if session.viewport_tool_radial_session.take().is_some() {
                changed = true;
            }
        }
        changed
    }

    pub fn close_all_viewport_tools_menus(&mut self) -> bool {
        let mut changed = false;
        for session in self.sessions_by_mounted_unit.values_mut() {
            if session.viewport_tools_menu_open {
                session.viewport_tools_menu_open = false;
                changed = true;
            }
        }
        changed
    }

    pub fn close_tab_hold_viewport_radial_menus(&mut self) -> bool {
        let mut changed = false;
        for session in self.sessions_by_mounted_unit.values_mut() {
            if session
                .viewport_tool_radial_session
                .is_some_and(|radial| radial.opened_by_tab_hold)
            {
                session.viewport_tool_radial_session = None;
                changed = true;
            }
        }
        changed
    }

    pub fn len(&self) -> usize {
        self.sessions_by_mounted_unit.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions_by_mounted_unit.is_empty()
    }
}

#[cfg(test)]
fn retains_live_session_key(key: &ToolSurfaceStableKey) -> bool {
    retains_live_session_key_str(key.as_str())
}

fn retains_live_session_key_str(key: &str) -> bool {
    matches!(
        key,
        SCENE_OUTLINER_SURFACE_KEY
            | SCENE_ENTITY_TABLE_SURFACE_KEY
            | SCENE_VIEWPORT_SURFACE_KEY
            | SCENE_INSPECTOR_SURFACE_KEY
            | EDITOR_CONSOLE_SURFACE_KEY
            | ASSET_BROWSER_SURFACE_KEY
            | IMPORT_INSPECTOR_SURFACE_KEY
            | FIELD_PRODUCT_VIEWER_SURFACE_KEY
            | SDF_BRUSH_BROWSER_SURFACE_KEY
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{
        WorkspaceId, WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceState,
    };

    #[test]
    fn prune_for_workspace_removes_unmounted_surface_sessions() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let surface_id = workspace
            .tool_surfaces()
            .find(|surface| surface.stable_surface_key().as_str() == SCENE_ENTITY_TABLE_SURFACE_KEY)
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
    fn prune_for_workspace_retains_mounted_console_surface_session() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = WorkspaceId::try_from_raw(1).unwrap();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let surface_id = workspace
            .tool_surfaces()
            .find(|surface| surface.stable_surface_key().as_str() == EDITOR_CONSOLE_SURFACE_KEY)
            .expect("editor console surface should exist")
            .id;
        let mut store = SurfaceSessionStore::default();
        store.session_mut(surface_id).console_follow_enabled = false;

        store.prune_for_workspace(&workspace);

        assert_eq!(
            store
                .session(surface_id)
                .expect("console surface session should be retained")
                .console_follow_enabled,
            false
        );
    }

    #[test]
    fn clear_transient_removes_all_surface_sessions() {
        let mut store = SurfaceSessionStore::default();
        store
            .session_mut(ToolSurfaceInstanceId::try_from_raw(7).unwrap())
            .entity_table_ui_state
            .append_search_text("abc");

        store.clear_transient();

        assert!(store.is_empty());
    }
}
