#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderDebugOverlayState {
    pub enabled: bool,
    pub lines: Vec<String>,
}
