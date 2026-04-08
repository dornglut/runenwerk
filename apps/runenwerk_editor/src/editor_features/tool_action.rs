use editor_core::SelectionTarget;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolAction {
    SelectSingle(SelectionTarget),
    ClearSelection,
    Scene(editor_scene::SceneCommandIntent),
    HoverEntity(Option<editor_core::EntityId>),
    BeginPreview,
    UpdatePreview,
    CommitPreview,
    CancelPreview,
}
