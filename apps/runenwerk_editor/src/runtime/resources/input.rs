use std::collections::BTreeMap;

use editor_shell::ToolSurfaceInstanceId;
use editor_viewport::ViewportId;
use ui_composition::PresentationTargetId;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct EditorTargetViewportInteractionState {
    pub last_pointer_position: (f32, f32),
    pub last_target_viewport: Option<ViewportId>,
    pub active_camera_viewport: Option<ViewportId>,
    pub pointer_owner: EditorPointerOwner,
}

#[derive(Debug, Default, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct EditorInputBridgeState {
    pub last_logged_picking_revision: u64,
    interaction_by_target: BTreeMap<PresentationTargetId, EditorTargetViewportInteractionState>,
    pub active_shortcut_catalog_active: bool,
    pub active_shortcut_signature: Vec<(String, String, String)>,
    pub active_shortcut_action_ids: Vec<String>,
    pub active_shortcut_commands: BTreeMap<String, crate::shell::KnownEditorCommand>,
}

impl EditorInputBridgeState {
    pub(crate) fn interaction_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> EditorTargetViewportInteractionState {
        self.interaction_by_target
            .get(&target_id)
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn interaction_for_target_mut(
        &mut self,
        target_id: PresentationTargetId,
    ) -> &mut EditorTargetViewportInteractionState {
        self.interaction_by_target.entry(target_id).or_default()
    }

    pub(crate) fn clear_interaction_for_target(&mut self, target_id: PresentationTargetId) {
        self.interaction_by_target.remove(&target_id);
    }

    pub(crate) fn interaction_targets(&self) -> impl Iterator<Item = PresentationTargetId> + '_ {
        self.interaction_by_target.keys().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorPointerOwner {
    #[default]
    None,
    UiMiddleScroll,
    ViewportCamera {
        viewport_id: ViewportId,
        button: EditorCameraPointerButton,
    },
    ViewportTool {
        tool_surface_id: ToolSurfaceInstanceId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorCameraPointerButton {
    Middle,
    Secondary,
}
