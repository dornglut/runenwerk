//! File: domain/editor/editor_scene/src/sdf_authoring/graph.rs
//! Purpose: Source-backed SDF graph document semantics over the neutral graph substrate.

use std::collections::{BTreeMap, BTreeSet};

use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, GraphValidationError,
    NodeDefinition, NodeId, PortDefinition, PortDirection, PortId, PortTypeId, topological_order,
    validate_graph,
};

use crate::{
    SdfOperationDocument, SdfOperationIssue, SdfOperationIssueCode, SdfOperationIssueSeverity,
    SdfOperationIssueSubject, SdfPrimitiveSpec,
};

pub const SDF_GRAPH_VALUE_PORT_TYPE: PortTypeId = PortTypeId(1);

#[derive(Debug, Clone, PartialEq)]
pub struct SdfGraphDocument {
    pub stable_name: String,
    pub display_name: String,
    pub source_revision: u64,
    graph: GraphDefinition,
    node_semantics: BTreeMap<NodeId, SdfGraphNode>,
    next_node_id: u64,
    next_port_id: u64,
    next_edge_id: u64,
}

impl SdfGraphDocument {
    pub fn new(stable_name: impl Into<String>, display_name: impl Into<String>) -> Self {
        let stable_name = stable_name.into();
        let display_name = display_name.into();
        Self {
            stable_name: stable_name.clone(),
            display_name: display_name.clone(),
            source_revision: 1,
            graph: GraphDefinition::new(
                GraphId::new(1),
                stable_name,
                CyclePolicy::RejectDirectedCycles,
                [],
                [],
            ),
            node_semantics: BTreeMap::new(),
            next_node_id: 1,
            next_port_id: 1,
            next_edge_id: 1,
        }
    }

    pub fn graph(&self) -> &GraphDefinition {
        &self.graph
    }

    pub fn node_semantics(&self) -> &BTreeMap<NodeId, SdfGraphNode> {
        &self.node_semantics
    }

    pub fn add_primitive_node(
        &mut self,
        display_name: impl Into<String>,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
    ) -> NodeId {
        let node_id = self.allocate_node_id();
        let input = self.allocate_port_id();
        let output = self.allocate_port_id();
        let display_name = display_name.into();
        self.graph.nodes.push(NodeDefinition::new(
            node_id,
            display_name.clone(),
            [
                PortDefinition::new(input, "in", PortDirection::Input, SDF_GRAPH_VALUE_PORT_TYPE),
                PortDefinition::new(
                    output,
                    "out",
                    PortDirection::Output,
                    SDF_GRAPH_VALUE_PORT_TYPE,
                ),
            ],
        ));
        self.node_semantics.insert(
            node_id,
            SdfGraphNode::Primitive {
                display_name,
                primitive,
                material_channel,
            },
        );
        self.bump_revision();
        node_id
    }

    pub fn add_output_node(&mut self, display_name: impl Into<String>) -> NodeId {
        let node_id = self.allocate_node_id();
        let input = self.allocate_port_id();
        let display_name = display_name.into();
        self.graph.nodes.push(NodeDefinition::new(
            node_id,
            display_name.clone(),
            [PortDefinition::new(
                input,
                "in",
                PortDirection::Input,
                SDF_GRAPH_VALUE_PORT_TYPE,
            )],
        ));
        self.node_semantics
            .insert(node_id, SdfGraphNode::Output { display_name });
        self.bump_revision();
        node_id
    }

    pub fn connect_nodes(
        &mut self,
        from_node: NodeId,
        to_node: NodeId,
    ) -> Result<EdgeId, SdfGraphDocumentError> {
        let from_port = output_port_id(&self.graph, from_node)
            .ok_or(SdfGraphDocumentError::MissingNode(from_node))?;
        let to_port = input_port_id(&self.graph, to_node)
            .ok_or(SdfGraphDocumentError::MissingNode(to_node))?;
        let edge_id = self.allocate_edge_id();
        self.graph
            .edges
            .push(EdgeDefinition::new(edge_id, from_port, to_port));
        self.bump_revision();
        Ok(edge_id)
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), SdfGraphDocumentError> {
        let node = self
            .graph
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .ok_or(SdfGraphDocumentError::MissingNode(node_id))?;
        let ports = node
            .ports
            .iter()
            .map(|port| port.id)
            .collect::<BTreeSet<_>>();
        self.graph.nodes.retain(|node| node.id != node_id);
        self.graph
            .edges
            .retain(|edge| !ports.contains(&edge.from_port) && !ports.contains(&edge.to_port));
        self.node_semantics.remove(&node_id);
        self.bump_revision();
        Ok(())
    }

    pub fn update_primitive_node(
        &mut self,
        node_id: NodeId,
        primitive: SdfPrimitiveSpec,
    ) -> Result<(), SdfGraphDocumentError> {
        let Some(SdfGraphNode::Primitive {
            primitive: current, ..
        }) = self.node_semantics.get_mut(&node_id)
        else {
            return Err(SdfGraphDocumentError::MissingPrimitiveNode(node_id));
        };
        *current = primitive;
        self.bump_revision();
        Ok(())
    }

    fn allocate_node_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_node_id);
        self.next_node_id = self.next_node_id.saturating_add(1).max(1);
        id
    }

    fn allocate_port_id(&mut self) -> PortId {
        let id = PortId::new(self.next_port_id);
        self.next_port_id = self.next_port_id.saturating_add(1).max(1);
        id
    }

    fn allocate_edge_id(&mut self) -> EdgeId {
        let id = EdgeId::new(self.next_edge_id);
        self.next_edge_id = self.next_edge_id.saturating_add(1).max(1);
        id
    }

    fn bump_revision(&mut self) {
        self.source_revision = self.source_revision.saturating_add(1).max(1);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdfGraphNode {
    Primitive {
        display_name: String,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
    },
    Output {
        display_name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdfGraphCommandIntent {
    AddPrimitiveNode {
        display_name: String,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
    },
    AddOutputNode {
        display_name: String,
    },
    ConnectNodes {
        from_node: NodeId,
        to_node: NodeId,
    },
    RemoveNode {
        node_id: NodeId,
    },
    UpdatePrimitiveNode {
        node_id: NodeId,
        primitive: SdfPrimitiveSpec,
    },
}

impl SdfGraphCommandIntent {
    pub fn apply_to(
        self,
        document: &mut SdfGraphDocument,
    ) -> Result<SdfGraphCommandOutcome, SdfGraphDocumentError> {
        match self {
            Self::AddPrimitiveNode {
                display_name,
                primitive,
                material_channel,
            } => Ok(SdfGraphCommandOutcome::Node(document.add_primitive_node(
                display_name,
                primitive,
                material_channel,
            ))),
            Self::AddOutputNode { display_name } => Ok(SdfGraphCommandOutcome::Node(
                document.add_output_node(display_name),
            )),
            Self::ConnectNodes { from_node, to_node } => Ok(SdfGraphCommandOutcome::Edge(
                document.connect_nodes(from_node, to_node)?,
            )),
            Self::RemoveNode { node_id } => {
                document.remove_node(node_id)?;
                Ok(SdfGraphCommandOutcome::Updated)
            }
            Self::UpdatePrimitiveNode { node_id, primitive } => {
                document.update_primitive_node(node_id, primitive)?;
                Ok(SdfGraphCommandOutcome::Updated)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfGraphCommandOutcome {
    Node(NodeId),
    Edge(EdgeId),
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdfGraphDocumentError {
    MissingNode(NodeId),
    MissingPrimitiveNode(NodeId),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SdfGraphRatificationReport {
    pub issues: Vec<SdfOperationIssue>,
}

impl SdfGraphRatificationReport {
    pub fn has_blocking_issues(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.severity == SdfOperationIssueSeverity::Error)
    }
}

pub fn ratify_sdf_graph_document(document: &SdfGraphDocument) -> SdfGraphRatificationReport {
    let mut report = SdfGraphRatificationReport::default();
    if let Err(error) = validate_graph(document.graph()) {
        report.issues.push(SdfOperationIssue::error(
            SdfOperationIssueCode::GraphStructuralError,
            SdfOperationIssueSubject::Document,
            format!(
                "SDF graph structure is invalid: {}",
                graph_error_label(&error)
            ),
        ));
    }
    if !document
        .node_semantics
        .values()
        .any(|node| matches!(node, SdfGraphNode::Output { .. }))
    {
        report.issues.push(SdfOperationIssue::error(
            SdfOperationIssueCode::MissingGraphOutput,
            SdfOperationIssueSubject::Document,
            "SDF graph must contain an output node",
        ));
    }
    for node in &document.graph.nodes {
        if !document.node_semantics.contains_key(&node.id) {
            report.issues.push(SdfOperationIssue::error(
                SdfOperationIssueCode::MissingGraphNodeSemantics,
                SdfOperationIssueSubject::Operation(node.id.raw()),
                "SDF graph node is missing SDF-owned semantics",
            ));
        }
    }
    report
}

pub fn lower_sdf_graph_document_to_operation_document(
    graph_document: &SdfGraphDocument,
) -> Result<SdfOperationDocument, SdfGraphLoweringError> {
    let ratification = ratify_sdf_graph_document(graph_document);
    if ratification.has_blocking_issues() {
        return Err(SdfGraphLoweringError::BlockingDiagnostics(ratification));
    }
    let order = topological_order(graph_document.graph())
        .map_err(SdfGraphLoweringError::GraphValidation)?;
    let mut document = SdfOperationDocument::with_default_layer(
        format!("{}_lowered_operations", graph_document.stable_name),
        format!("{} Lowered Operations", graph_document.display_name),
    );
    let layer_id = document.layers()[0].id;
    for node_id in order {
        let Some(SdfGraphNode::Primitive {
            display_name,
            primitive,
            material_channel,
        }) = graph_document.node_semantics.get(&node_id)
        else {
            continue;
        };
        document
            .add_operation(
                layer_id,
                display_name.clone(),
                primitive.clone(),
                *material_channel,
            )
            .expect("lowered operation layer exists");
    }
    Ok(document)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdfGraphLoweringError {
    BlockingDiagnostics(SdfGraphRatificationReport),
    GraphValidation(GraphValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfGraphProjection {
    pub display_name: String,
    pub source_revision: u64,
    pub node_count: usize,
    pub edge_count: usize,
    pub primitive_count: usize,
    pub output_count: usize,
    pub can_lower: bool,
    pub issues: Vec<SdfOperationIssue>,
}

pub fn project_sdf_graph_document(document: &SdfGraphDocument) -> SdfGraphProjection {
    let ratification = ratify_sdf_graph_document(document);
    SdfGraphProjection {
        display_name: document.display_name.clone(),
        source_revision: document.source_revision,
        node_count: document.graph.nodes.len(),
        edge_count: document.graph.edges.len(),
        primitive_count: document
            .node_semantics
            .values()
            .filter(|node| matches!(node, SdfGraphNode::Primitive { .. }))
            .count(),
        output_count: document
            .node_semantics
            .values()
            .filter(|node| matches!(node, SdfGraphNode::Output { .. }))
            .count(),
        can_lower: !ratification.has_blocking_issues(),
        issues: ratification.issues,
    }
}

fn input_port_id(graph: &GraphDefinition, node_id: NodeId) -> Option<PortId> {
    graph
        .nodes
        .iter()
        .find(|node| node.id == node_id)?
        .ports
        .iter()
        .find(|port| port.direction == PortDirection::Input)
        .map(|port| port.id)
}

fn output_port_id(graph: &GraphDefinition, node_id: NodeId) -> Option<PortId> {
    graph
        .nodes
        .iter()
        .find(|node| node.id == node_id)?
        .ports
        .iter()
        .find(|port| port.direction == PortDirection::Output)
        .map(|port| port.id)
}

fn graph_error_label(error: &GraphValidationError) -> &'static str {
    match error {
        GraphValidationError::DuplicateNodeId(_) => "duplicate node id",
        GraphValidationError::DuplicatePortId(_) => "duplicate port id",
        GraphValidationError::DuplicateEdgeId(_) => "duplicate edge id",
        GraphValidationError::MissingNode(_) => "missing node",
        GraphValidationError::MissingPort { .. } => "missing port",
        GraphValidationError::EdgeDirectionInvalid { .. } => "invalid edge direction",
        GraphValidationError::PortTypeMismatch { .. } => "port type mismatch",
        GraphValidationError::DirectedCycleDetected => "directed cycle",
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SdfBooleanIntent, SdfPrimitiveKind, SdfPrimitiveSpec, lower_sdf_operation_document,
    };

    use super::*;

    #[test]
    fn graph_commands_create_connect_update_and_remove_nodes() {
        let mut document = SdfGraphDocument::new("sdf_graph", "SDF Graph");
        let primitive = SdfGraphCommandIntent::AddPrimitiveNode {
            display_name: "Sphere".to_string(),
            primitive: SdfPrimitiveSpec::new(SdfPrimitiveKind::Sphere, SdfBooleanIntent::Add),
            material_channel: 1,
        }
        .apply_to(&mut document)
        .expect("add primitive");
        let output = SdfGraphCommandIntent::AddOutputNode {
            display_name: "Output".to_string(),
        }
        .apply_to(&mut document)
        .expect("add output");
        let (SdfGraphCommandOutcome::Node(primitive), SdfGraphCommandOutcome::Node(output)) =
            (primitive, output)
        else {
            panic!("expected node ids");
        };

        let edge = SdfGraphCommandIntent::ConnectNodes {
            from_node: primitive,
            to_node: output,
        }
        .apply_to(&mut document)
        .expect("connect");

        assert!(matches!(edge, SdfGraphCommandOutcome::Edge(_)));
        assert_eq!(document.graph().nodes.len(), 2);
        assert_eq!(document.graph().edges.len(), 1);

        SdfGraphCommandIntent::UpdatePrimitiveNode {
            node_id: primitive,
            primitive: SdfPrimitiveSpec::new(SdfPrimitiveKind::Box, SdfBooleanIntent::Subtract),
        }
        .apply_to(&mut document)
        .expect("update primitive");
        SdfGraphCommandIntent::RemoveNode { node_id: output }
            .apply_to(&mut document)
            .expect("remove output");
        assert!(document.graph().edges.is_empty());
    }

    #[test]
    fn graph_lowering_uses_operation_window_path() {
        let mut graph = SdfGraphDocument::new("sdf_graph", "SDF Graph");
        let primitive = graph.add_primitive_node(
            "Sphere",
            SdfPrimitiveSpec::new(SdfPrimitiveKind::Sphere, SdfBooleanIntent::Add),
            3,
        );
        let output = graph.add_output_node("Output");
        graph.connect_nodes(primitive, output).expect("connect");

        let operation_document =
            lower_sdf_graph_document_to_operation_document(&graph).expect("lower graph");
        let candidate = lower_sdf_operation_document(
            &operation_document,
            &crate::SdfOperationLoweringContext::default(),
        );

        assert!(candidate.can_commit());
        assert_eq!(candidate.operation_count(), 1);
    }
}
