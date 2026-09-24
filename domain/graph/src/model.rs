//! File: domain/graph/src/model.rs
//! Purpose: Domain-neutral graph definition model.

use crate::{EdgeId, GraphId, NodeId, PortId, PortTypeId};
use resource_ref::ResourceRef;

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
    pub metadata: Vec<GraphMetadataEntry>,
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
            metadata: Vec::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: impl IntoIterator<Item = GraphMetadataEntry>) -> Self {
        self.metadata = metadata.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDefinition {
    pub id: NodeId,
    pub name: String,
    pub ports: Vec<PortDefinition>,
    pub metadata: Vec<GraphMetadataEntry>,
    pub values: Vec<GraphMetadataEntry>,
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
            metadata: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: impl IntoIterator<Item = GraphMetadataEntry>) -> Self {
        self.metadata = metadata.into_iter().collect();
        self
    }

    pub fn with_values(mut self, values: impl IntoIterator<Item = GraphMetadataEntry>) -> Self {
        self.values = values.into_iter().collect();
        self
    }

    pub fn value(&self, key: &str) -> Option<&GraphValue> {
        self.values
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }

    pub fn metadata_value(&self, key: &str) -> Option<&GraphValue> {
        self.metadata
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphMetadataEntry {
    pub key: String,
    pub value: GraphValue,
}

impl GraphMetadataEntry {
    pub fn new(key: impl Into<String>, value: GraphValue) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValue {
    Bool(bool),
    Integer(i64),
    Decimal(String),
    Text(String),
    Resource(ResourceRef),
}

impl GraphValue {
    pub fn decimal(value: impl Into<String>) -> Self {
        Self::Decimal(value.into())
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn resource(value: ResourceRef) -> Self {
        Self::Resource(value)
    }

    pub fn canonical_component(&self) -> String {
        match self {
            Self::Bool(value) => format!("bool:{value}"),
            Self::Integer(value) => format!("integer:{value}"),
            Self::Decimal(value) => format!("decimal:{}:{value}", value.len()),
            Self::Text(value) => format!("text:{}:{value}", value.len()),
            Self::Resource(value) => format!("resource:{}", value.canonical_component()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_metadata_and_values_are_explicit_graph_contracts() {
        let texture_ref =
            ResourceRef::new("asset.catalog", "texture.rock").expect("valid resource ref");
        let node = NodeDefinition::new(NodeId::new(1), "texture.sample_2d", [])
            .with_metadata([GraphMetadataEntry::new(
                "ui.category",
                GraphValue::text("Textures"),
            )])
            .with_values([GraphMetadataEntry::new(
                "texture_ref",
                GraphValue::resource(texture_ref.clone()),
            )]);

        assert_eq!(
            node.metadata_value("ui.category"),
            Some(&GraphValue::text("Textures"))
        );
        assert_eq!(
            node.value("texture_ref"),
            Some(&GraphValue::resource(texture_ref))
        );
    }

    #[test]
    fn graph_value_resource_encoding_is_collision_resistant() {
        let left = GraphValue::resource(
            ResourceRef::new("asset.catalog", "a:b")
                .expect("valid resource ref")
                .with_artifact("c"),
        );
        let right = GraphValue::resource(
            ResourceRef::new("asset.catalog", "a")
                .expect("valid resource ref")
                .with_artifact("b:c"),
        );

        assert_ne!(left.canonical_component(), right.canonical_component());
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
