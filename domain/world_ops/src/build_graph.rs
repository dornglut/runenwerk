use crate::{BuildGeneration, ChunkRevision};
use spatial::ChunkId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuildGraphPhase {
    DirtyPlan,
    OpWindowResolve,
    SdfFieldBuild,
    SummaryBuild,
    DerivedRenderBuild,
    Publish,
}

#[derive(Debug, Clone)]
pub struct BuildGraphNode {
    pub chunk_id: ChunkId,
    pub phase: BuildGraphPhase,
    pub target_chunk_revision: ChunkRevision,
    pub input_generation_stamp: BuildGeneration,
}

#[derive(Debug, Clone, Default)]
pub struct BuildGraph {
    pub nodes: Vec<BuildGraphNode>,
}
