use editor_scene::SceneSelectionAddress;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolAction {
    SelectSingle(SceneSelectionAddress),
    ClearSelection,
    Scene(editor_scene::SceneCommandIntent),
    HoverEntity(Option<editor_core::EntityId>),
    BeginPreview,
    BeginTransformPreview(crate::editor_runtime::TransformToolKind),
    UpdatePreview,
    CommitPreview,
    CancelPreview,
}
