use editor_core::SelectionTarget;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolAction {
    SelectSingle(SelectionTarget),
    ClearSelection,
    Scene(editor_scene::SceneCommandIntent),
    HoverEntity(Option<editor_core::EntityId>),
    BeginPreview,
    BeginTransformPreview(crate::editor_runtime::TransformToolKind),
    UpdatePreview,
    CommitPreview,
    CancelPreview,
}
