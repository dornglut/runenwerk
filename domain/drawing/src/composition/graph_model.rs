//! File: domain/drawing/src/composition/graph_model.rs
//! Purpose: Drawing semantic wrapper around domain/graph structure.

use std::collections::BTreeMap;

use graph::{CyclePolicy, GraphDefinition, GraphId, NodeId};

use crate::{CompositeOutputId, DrawingCompositeNode};

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingCompositeGraph {
    pub schema_version: u32,
    pub graph: GraphDefinition,
    pub nodes: BTreeMap<NodeId, DrawingCompositeNode>,
    pub root_stack_node: NodeId,
    pub active_output: Option<CompositeOutputId>,
}

impl DrawingCompositeGraph {
    pub fn new(
        graph: GraphDefinition,
        root_stack_node: NodeId,
        nodes: impl IntoIterator<Item = (NodeId, DrawingCompositeNode)>,
        active_output: Option<CompositeOutputId>,
    ) -> Self {
        Self {
            schema_version: 1,
            graph,
            nodes: nodes.into_iter().collect(),
            root_stack_node,
            active_output,
        }
    }

    pub fn empty(name: impl Into<String>, root_stack_node: NodeId) -> Self {
        Self {
            schema_version: 1,
            graph: GraphDefinition::new(
                GraphId::new(1),
                name,
                CyclePolicy::RejectDirectedCycles,
                [],
                [],
            ),
            nodes: BTreeMap::new(),
            root_stack_node,
            active_output: None,
        }
    }

    pub fn root_stack(&self) -> Option<&crate::LayerStackNode> {
        match self.nodes.get(&self.root_stack_node) {
            Some(DrawingCompositeNode::LayerStack(stack)) => Some(stack),
            _ => None,
        }
    }

    pub fn root_stack_mut(&mut self) -> Option<&mut crate::LayerStackNode> {
        match self.nodes.get_mut(&self.root_stack_node) {
            Some(DrawingCompositeNode::LayerStack(stack)) => Some(stack),
            _ => None,
        }
    }
}
