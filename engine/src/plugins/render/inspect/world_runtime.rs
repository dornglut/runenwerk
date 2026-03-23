#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldRuntimeInspectorSnapshot {
    pub chunk_dirty_count: usize,
    pub queued_interactive: usize,
    pub queued_background: usize,
    pub integrated_build_outputs: u64,
    pub dropped_stale_outputs: u64,
    pub op_log_count: u64,
}
