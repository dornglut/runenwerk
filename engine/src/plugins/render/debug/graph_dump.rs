#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderDebugGraphDumpState {
    pub revision: u64,
    pub lines: Vec<String>,
}
