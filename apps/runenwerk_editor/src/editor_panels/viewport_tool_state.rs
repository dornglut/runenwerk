use editor_core::EntityId;
use editor_viewport::SnapSettings;
use scene::Vec3Value;

use crate::editor_runtime::{EditorToolRuntimeState, TransformPreviewSession, TransformToolKind};
use crate::editor_tools_state::TranslateAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportToolState {
    pub hovered_entity: Option<EntityId>,
    pub active_preview: Option<ViewportPreviewState>,
    pub active_translate_axis: Option<TranslateAxis>,
    pub snap_settings: SnapSettings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportPreviewState {
    pub entity: EntityId,
    pub tool: TransformToolKind,
    pub translation_delta: Vec3Value,
    pub rotation_delta_radians: Vec3Value,
    pub scale_delta: Vec3Value,
}

impl ViewportToolState {
    pub fn from_runtime(runtime: &EditorToolRuntimeState) -> Self {
        Self {
            hovered_entity: runtime.hovered_entity(),
            active_preview: runtime.preview().map(ViewportPreviewState::from_session),
            active_translate_axis: runtime.translate_axis(),
            snap_settings: runtime.snap_settings(),
        }
    }
}

impl ViewportPreviewState {
    pub fn from_session(session: &TransformPreviewSession) -> Self {
        Self {
            entity: session.entity,
            tool: session.tool,
            translation_delta: session.translation_delta,
            rotation_delta_radians: session.rotation_delta_radians,
            scale_delta: session.scale_delta,
        }
    }
}
