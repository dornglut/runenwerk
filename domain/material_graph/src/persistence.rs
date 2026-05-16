//! File: domain/material_graph/src/persistence.rs
//! Purpose: Versioned source persistence contract for authored material graph documents.

use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use serde::{Deserialize, Serialize};

use crate::{MaterialGraphDocument, MaterialGraphDocumentId, MaterialOutputTarget};

pub const MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphSourceFileV1 {
    pub version: u32,
    pub document_id: u64,
    pub label: String,
    pub output_target: MaterialOutputTarget,
    pub graph: MaterialGraphDefinitionV1,
}

impl MaterialGraphSourceFileV1 {
    pub fn from_document(document: &MaterialGraphDocument) -> Self {
        Self {
            version: MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1,
            document_id: document.document_id.raw(),
            label: document.label.clone(),
            output_target: document.output_target,
            graph: MaterialGraphDefinitionV1::from_graph(&document.graph),
        }
    }

    pub fn into_document(self) -> Result<MaterialGraphDocument, MaterialGraphSourceIssue> {
        if self.version != MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1 {
            return Err(MaterialGraphSourceIssue::UnsupportedVersion(self.version));
        }
        Ok(MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(self.document_id),
            self.label,
            self.graph.into_graph(),
            self.output_target,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphDefinitionV1 {
    pub graph_id: u64,
    pub name: String,
    pub cycle_policy: MaterialGraphCyclePolicyV1,
    pub nodes: Vec<MaterialGraphNodeDefinitionV1>,
    pub edges: Vec<MaterialGraphEdgeDefinitionV1>,
}

impl MaterialGraphDefinitionV1 {
    fn from_graph(graph: &GraphDefinition) -> Self {
        Self {
            graph_id: graph.id.raw(),
            name: graph.name.clone(),
            cycle_policy: MaterialGraphCyclePolicyV1::from_graph(graph.cycle_policy),
            nodes: graph
                .nodes
                .iter()
                .map(MaterialGraphNodeDefinitionV1::from_node)
                .collect(),
            edges: graph
                .edges
                .iter()
                .map(MaterialGraphEdgeDefinitionV1::from_edge)
                .collect(),
        }
    }

    fn into_graph(self) -> GraphDefinition {
        GraphDefinition::new(
            GraphId::new(self.graph_id),
            self.name,
            self.cycle_policy.into_graph(),
            self.nodes
                .into_iter()
                .map(MaterialGraphNodeDefinitionV1::into_node),
            self.edges
                .into_iter()
                .map(MaterialGraphEdgeDefinitionV1::into_edge),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphCyclePolicyV1 {
    AllowDirectedCycles,
    RejectDirectedCycles,
}

impl MaterialGraphCyclePolicyV1 {
    fn from_graph(policy: CyclePolicy) -> Self {
        match policy {
            CyclePolicy::AllowDirectedCycles => Self::AllowDirectedCycles,
            CyclePolicy::RejectDirectedCycles => Self::RejectDirectedCycles,
        }
    }

    fn into_graph(self) -> CyclePolicy {
        match self {
            Self::AllowDirectedCycles => CyclePolicy::AllowDirectedCycles,
            Self::RejectDirectedCycles => CyclePolicy::RejectDirectedCycles,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphNodeDefinitionV1 {
    pub node_id: u64,
    pub name: String,
    pub ports: Vec<MaterialGraphPortDefinitionV1>,
}

impl MaterialGraphNodeDefinitionV1 {
    fn from_node(node: &NodeDefinition) -> Self {
        Self {
            node_id: node.id.raw(),
            name: node.name.clone(),
            ports: node
                .ports
                .iter()
                .map(MaterialGraphPortDefinitionV1::from_port)
                .collect(),
        }
    }

    fn into_node(self) -> NodeDefinition {
        NodeDefinition::new(
            NodeId::new(self.node_id),
            self.name,
            self.ports
                .into_iter()
                .map(MaterialGraphPortDefinitionV1::into_port),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphPortDefinitionV1 {
    pub port_id: u64,
    pub name: String,
    pub direction: MaterialGraphPortDirectionV1,
    pub port_type_id: u64,
}

impl MaterialGraphPortDefinitionV1 {
    fn from_port(port: &PortDefinition) -> Self {
        Self {
            port_id: port.id.raw(),
            name: port.name.clone(),
            direction: MaterialGraphPortDirectionV1::from_graph(port.direction),
            port_type_id: port.port_type.raw(),
        }
    }

    fn into_port(self) -> PortDefinition {
        PortDefinition::new(
            PortId::new(self.port_id),
            self.name,
            self.direction.into_graph(),
            PortTypeId::new(self.port_type_id),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphPortDirectionV1 {
    Input,
    Output,
}

impl MaterialGraphPortDirectionV1 {
    fn from_graph(direction: PortDirection) -> Self {
        match direction {
            PortDirection::Input => Self::Input,
            PortDirection::Output => Self::Output,
        }
    }

    fn into_graph(self) -> PortDirection {
        match self {
            Self::Input => PortDirection::Input,
            Self::Output => PortDirection::Output,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphEdgeDefinitionV1 {
    pub edge_id: u64,
    pub from_port_id: u64,
    pub to_port_id: u64,
}

impl MaterialGraphEdgeDefinitionV1 {
    fn from_edge(edge: &EdgeDefinition) -> Self {
        Self {
            edge_id: edge.id.raw(),
            from_port_id: edge.from_port.raw(),
            to_port_id: edge.to_port.raw(),
        }
    }

    fn into_edge(self) -> EdgeDefinition {
        EdgeDefinition::new(
            EdgeId::new(self.edge_id),
            PortId::new(self.from_port_id),
            PortId::new(self.to_port_id),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialGraphSourceIssue {
    UnsupportedVersion(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{PortDirection, PortTypeId};

    fn document(target: MaterialOutputTarget) -> MaterialGraphDocument {
        MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(12),
            "preview",
            GraphDefinition::new(
                GraphId::new(1),
                "pbr",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(1),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(1),
                            "color",
                            PortDirection::Output,
                            PortTypeId::new(1),
                        )],
                    ),
                    NodeDefinition::new(
                        NodeId::new(2),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(2),
                            "base_color",
                            PortDirection::Input,
                            PortTypeId::new(1),
                        )],
                    ),
                ],
                [EdgeDefinition::new(
                    EdgeId::new(1),
                    PortId::new(1),
                    PortId::new(2),
                )],
            ),
            target,
        )
    }

    #[test]
    fn material_graph_source_file_round_trips_document_identity() {
        let document = document(MaterialOutputTarget::RenderMaterial);

        let restored = MaterialGraphSourceFileV1::from_document(&document)
            .into_document()
            .expect("v1 source should decode");

        assert_eq!(restored.document_id, document.document_id);
        assert_eq!(restored.label, document.label);
        assert_eq!(restored.output_target, document.output_target);
        assert_eq!(restored.graph, document.graph);
    }

    #[test]
    fn material_graph_source_file_rejects_unknown_version() {
        let mut source =
            MaterialGraphSourceFileV1::from_document(&document(MaterialOutputTarget::PbrPreview));
        source.version = 99;

        assert_eq!(
            source.into_document(),
            Err(MaterialGraphSourceIssue::UnsupportedVersion(99))
        );
    }
}
