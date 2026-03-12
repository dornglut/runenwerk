#[derive(Debug, Clone, Default, ecs::Component)]
pub struct SdfRenderBindingsState {
    pub params_buffer_label: String,
    pub field_buffer_label: String,
}
