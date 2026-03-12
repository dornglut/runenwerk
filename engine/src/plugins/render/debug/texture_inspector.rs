#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderTextureInspectorState {
    pub selected_texture: Option<String>,
    pub hovered_texture: Option<String>,
}
