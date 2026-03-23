use super::super::ids::{BuildGeneration, ChunkId, ChunkRevision};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub enum WorldBuildGraphPhase {
    DirtyPlan,
    OpWindowResolve,
    SdfFieldBuild,
    SummaryBuild,
    DerivedRenderBuild,
    Publish,
}

#[derive(Debug, Clone, ecs::Resource)]
pub struct WorldBuildGraphNode {
    pub chunk_id: ChunkId,
    pub phase: WorldBuildGraphPhase,
    pub target_chunk_revision: ChunkRevision,
    pub input_generation_stamp: BuildGeneration,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldBuildGraphResource {
    pub nodes: Vec<WorldBuildGraphNode>,
}
