use crate::plugins::render::graph::{PassGraph, ResourceGraph};
use crate::plugins::render::{RenderFlowId, RenderPassNode, RenderResourceDescriptor};

#[derive(Debug, Clone)]
pub struct RenderFlowGraph {
    pub id: RenderFlowId,
    pub label: String,
    pub resources: ResourceGraph,
    pub passes: PassGraph,
}

impl RenderFlowGraph {
    pub fn new(id: impl Into<RenderFlowId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            resources: ResourceGraph::default(),
            passes: PassGraph::default(),
        }
    }

    pub fn add_resource(&mut self, descriptor: RenderResourceDescriptor) {
        self.resources.add_resource(descriptor);
    }

    pub fn add_pass(&mut self, pass: RenderPassNode) {
        self.passes.add_pass(pass);
    }

    pub fn merge(mut self, other: RenderFlowGraph) -> Self {
        self.resources
            .state_resources
            .extend(other.resources.state_resources);
        self.resources.resources.extend(other.resources.resources);
        self.passes.passes.extend(other.passes.passes);
        self
    }
}
