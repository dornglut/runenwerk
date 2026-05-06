//! File: apps/runenwerk_editor/src/editor_features/viewport/sdf_tools.rs
//! Purpose: Viewport-facing SDF authoring actions routed through scene command contracts.

use editor_core::{ChangeOrigin, EntityId, GoverningChangeError};
use editor_scene::{SceneTransform, SdfBooleanIntent, SdfPrimitiveKind, SdfPrimitiveSpec};

use crate::editor_features::create_sdf_primitive_with_history_from_origin;
use crate::editor_runtime::RunenwerkEditorRuntime;

#[derive(Debug, Clone, PartialEq)]
pub struct SdfPrimitiveToolAction {
    pub parent: Option<EntityId>,
    pub display_name: String,
    pub kind: SdfPrimitiveKind,
    pub boolean: SdfBooleanIntent,
    pub transform: SceneTransform,
}

impl SdfPrimitiveToolAction {
    pub fn add(kind: SdfPrimitiveKind, display_name: impl Into<String>) -> Self {
        Self {
            parent: None,
            display_name: display_name.into(),
            kind,
            boolean: SdfBooleanIntent::Add,
            transform: SceneTransform::identity(),
        }
    }
}

pub fn dispatch_sdf_primitive_tool_action(
    runtime: &mut RunenwerkEditorRuntime,
    action: SdfPrimitiveToolAction,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    create_sdf_primitive_with_history_from_origin(
        runtime,
        action.parent,
        action.display_name,
        SdfPrimitiveSpec::new(action.kind, action.boolean).with_transform(action.transform),
        origin,
    )
}
