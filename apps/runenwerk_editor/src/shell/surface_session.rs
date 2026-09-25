use std::collections::{BTreeMap, BTreeSet};

use editor_shell::{EditorCompositionRuntime, ToolSurfaceInstanceId, ViewportToolKind};
use editor_viewport::ViewportId;
use ui_composition::MountedUnitId;
use ui_math::UiPoint;

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
    pub active_viewport_tool: ViewportToolKind,
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
            active_viewport_tool: ViewportToolKind::Select,
            viewport_details_visible: false,
            viewport_statistics_visible: false,
            viewport_options_menu_open: false,
            viewport_tools_menu_open: false,
            viewport_tool_radial_session: None,
            console_follow_enabled: true,
        }
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
    pub fn viewport_tool(&self, mounted_unit_id: MountedUnitId) -> ViewportToolKind {
        self.session(mounted_unit_id)
            .map(|session| session.active_viewport_tool)
            .unwrap_or_default()
    }

    pub fn close_viewport_tool_menus(&mut self, mounted_unit_id: MountedUnitId) {
        let session = self.session_mut(mounted_unit_id);
        session.viewport_tools_menu_open = false;
        session.viewport_tool_radial_session = None;
    }

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
        MATERIAL_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID,
        form_editor_profile_layout_source,
    };

    fn scene_to_material_compositions() -> (EditorCompositionRuntime, EditorCompositionRuntime) {
        let app = crate::editor_app::RunenwerkEditorApp::new();
        let host = app.workbench_host();
        let mut shell_state =
            crate::shell::RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("scene profile should form as the live composition");
        let scene = shell_state.composition_runtime().clone();
        let material = host
            .workspace_profile_registry()
            .profile(MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should be installed");
        shell_state
            .activate_workspace_profile_ref_with_registry(
                &material.profile_ref,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("material profile should activate through the live profile path");
        (scene, shell_state.composition_runtime().clone())
    }

    fn composition(profile_id: editor_shell::WorkspaceProfileId) -> EditorCompositionRuntime {
        let app = crate::editor_app::RunenwerkEditorApp::new();
        let host = app.workbench_host();
        let profile = host
            .workspace_profile_registry()
            .profile(profile_id)
            .expect("test profile should be installed");
        form_editor_profile_layout_source(
            profile.id,
            &profile.layout_source,
            host.tool_surface_registry(),
        )
        .expect("test profile should form as composition")
    }

    fn mounted_surface(
        runtime: &EditorCompositionRuntime,
        stable_key: &str,
    ) -> (MountedUnitId, ToolSurfaceInstanceId) {
        let unit = runtime
            .extension()
            .mounted_units()
            .iter()
            .find(|unit| unit.stable_content_key == stable_key)
            .expect("test composition should contain requested surface");
        (
            unit.mounted_unit_id,
            ToolSurfaceInstanceId::try_from_raw(unit.compatibility_surface_raw)
                .expect("compatibility surface identity should be valid"),
        )
    }

    #[test]
    fn prune_for_composition_removes_surface_missing_from_candidate() {
        let (scene, material) = scene_to_material_compositions();
        let (_, surface_id) = mounted_surface(&scene, SCENE_ENTITY_TABLE_SURFACE_KEY);
        let mut store = SurfaceSessionStore::default();
        store
            .session_mut(surface_id)
            .entity_table_ui_state
            .append_search_text("abc");

        store.prune_for_composition(&material);

        assert!(store.session(surface_id).is_none());
    }

    #[test]
    fn prune_for_composition_retains_mounted_console_surface_session() {
        let scene = composition(SCENE_WORKSPACE_PROFILE_ID);
        let (_, surface_id) = mounted_surface(&scene, EDITOR_CONSOLE_SURFACE_KEY);
        let mut store = SurfaceSessionStore::default();
        store.session_mut(surface_id).console_follow_enabled = false;

        store.prune_for_composition(&scene);

        assert!(
            !store
                .session(surface_id)
                .expect("console surface session should be retained")
                .console_follow_enabled
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

    #[test]
    fn viewport_tool_defaults_to_select_and_prunes_with_mounted_unit() {
        let (scene, material) = scene_to_material_compositions();
        let (mounted_unit_id, surface_id) = mounted_surface(&scene, SCENE_VIEWPORT_SURFACE_KEY);
        let mut store = SurfaceSessionStore::default();

        assert_eq!(
            store.viewport_tool(mounted_unit_id),
            ViewportToolKind::Select
        );
        store.session_mut(surface_id).active_viewport_tool = ViewportToolKind::Rotate;
        assert_eq!(
            store.viewport_tool(mounted_unit_id),
            ViewportToolKind::Rotate
        );

        store.prune_for_composition(&material);

        assert!(store.session(surface_id).is_none());
        assert_eq!(
            store.viewport_tool(mounted_unit_id),
            ViewportToolKind::Select
        );
    }
}
