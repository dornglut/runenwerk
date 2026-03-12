#[derive(Debug, Clone, Default, ecs::Component)]
pub struct SdfDebugViewsState {
    pub field_view_enabled: bool,
    pub influence_view_enabled: bool,
}
