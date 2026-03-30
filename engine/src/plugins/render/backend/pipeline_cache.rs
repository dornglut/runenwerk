use crate::plugins::render::pipelines::{PipelineCacheResource, PipelineCacheStats};

#[deprecated(
    note = "use pipelines::PipelineCacheStats; backend cache stats now alias the canonical ecs stats resource"
)]
pub type BackendPipelineCacheStats = PipelineCacheStats;

#[deprecated(
    note = "use pipelines::PipelineCacheResource; ecs pipeline cache resource is stats-only and canonical"
)]
pub type BackendPipelineCacheResource = PipelineCacheResource;
