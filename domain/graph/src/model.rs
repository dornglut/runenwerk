//! File: domain/graph/src/model.rs
//! Purpose: Domain-neutral graph definition model.

use crate::{EdgeId, GraphId, NodeId, PortId, PortTypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyclePolicy {
    AllowDirectedCycles,
    RejectDirectedCycles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortDefinition {
    pub id: PortId,
    pub name: String,
    pub direction: PortDirection,
    pub port_type: PortTypeId,
}

impl PortDefinition {
    pub fn new(
        id: PortId,
        name: impl Into<String>,
        direction: PortDirection,
        port_type: PortTypeId,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            direction,
            port_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDefinition {
    pub id: NodeId,
    pub name: String,
    pub ports: Vec<PortDefinition>,
}

impl NodeDefinition {
    pub fn new(
        id: NodeId,
        name: impl Into<String>,
        ports: impl IntoIterator<Item = PortDefinition>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            ports: ports.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeDefinition {
    pub id: EdgeId,
    pub from_port: PortId,
    pub to_port: PortId,
}

impl EdgeDefinition {
    pub const fn new(id: EdgeId, from_port: PortId, to_port: PortId) -> Self {
        Self {
            id,
            from_port,
            to_port,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDefinition {
    pub id: GraphId,
    pub name: String,
    pub cycle_policy: CyclePolicy,
    pub nodes: Vec<NodeDefinition>,
    pub edges: Vec<EdgeDefinition>,
}

impl GraphDefinition {
    pub fn new(
        id: GraphId,
        name: impl Into<String>,
        cycle_policy: CyclePolicy,
        nodes: impl IntoIterator<Item = NodeDefinition>,
        edges: impl IntoIterator<Item = EdgeDefinition>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            cycle_policy,
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
        }
    }
}
