#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldDebugMetricsResource {
    pub interactive_queue_depth: usize,
    pub background_queue_depth: usize,
    pub enqueued_build_jobs: u64,
    pub completed_build_jobs: u64,
    pub integrated_build_outputs: u64,
    pub dropped_stale_build_outputs: u64,
    pub op_log_count: u64,
    pub replication_resyncs: u64,
    pub residency_misses: u64,
}
