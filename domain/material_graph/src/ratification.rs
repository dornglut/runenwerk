//! File: domain/material_graph/src/ratification.rs
//! Purpose: Material graph semantic ratification.

use graph::{
    EdgeDefinition, GraphValidationError, GraphValue, NodeDefinition, NodeId, PortDefinition,
    PortDirection, PortId, validate_graph,
};
use ratification::{RatificationIssue, RatificationReport};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    MaterialGraphDocument, MaterialInputContract, MaterialNodeCatalog, MaterialNodeDescriptor,
    MaterialOutputContract, MaterialValueContract, MaterialValueType,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaterialGraphIssueCode {
    EmptyLabel,
    GraphStructural,
    UnsupportedNode,
    MissingResourceReference,
    InvalidNodeSemantics,
    MissingOutputNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialGraphIssueSubject {
    Document,
    Graph,
    Node(NodeId),
    Output,
}

pub type MaterialGraphRatificationReport =
    RatificationReport<MaterialGraphIssueCode, MaterialGraphIssueSubject>;

pub fn ratify_material_graph(
    document: &MaterialGraphDocument,
    catalog: &MaterialNodeCatalog,
) -> MaterialGraphRatificationReport {
    let mut report = MaterialGraphRatificationReport::new();

    if document.label.trim().is_empty() {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::EmptyLabel,
            MaterialGraphIssueSubject::Document,
            "material graph document label must not be empty",
        ));
    }

    if let Err(error) = validate_graph(&document.graph) {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::GraphStructural,
            MaterialGraphIssueSubject::Graph,
            graph_error_message(&error),
        ));
    }

    let mut has_output = false;
    let incoming_ports = incoming_port_set(&document.graph.edges);
    for node in &document.graph.nodes {
        if node.name == "pbr.output" {
            has_output = true;
        }
        let Some(descriptor) = catalog.descriptor(&node.name) else {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::UnsupportedNode,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material graph node '{}' is not in the active catalog",
                    node.name
                ),
            ));
            continue;
        };
        ratify_node_semantics(
            node,
            descriptor,
            document.output_target,
            &incoming_ports,
            &mut report,
        );
    }

    if !has_output {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::MissingOutputNode,
            MaterialGraphIssueSubject::Output,
            "material graph must contain a pbr.output node",
        ));
    }

    report
}

fn ratify_node_semantics(
    node: &NodeDefinition,
    descriptor: &MaterialNodeDescriptor,
    output_target: crate::MaterialOutputTarget,
    incoming_ports: &BTreeSet<PortId>,
    report: &mut MaterialGraphRatificationReport,
) {
    if !descriptor.supports_output_target(output_target) {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::InvalidNodeSemantics,
            MaterialGraphIssueSubject::Node(node.id),
            format!(
                "material node '{}' does not support output target {:?}",
                node.name, output_target
            ),
        ));
    }

    let mut known_values = BTreeMap::<&str, &MaterialValueContract>::new();
    for value in &descriptor.values {
        known_values.insert(value.key.as_str(), value);
    }
    for entry in &node.values {
        let Some(contract) = known_values.get(entry.key.as_str()) else {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' uses undeclared value '{}'",
                    node.name, entry.key
                ),
            ));
            continue;
        };
        if !graph_value_matches_type(&entry.value, contract.value_type) {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' value '{}' is not a {}",
                    node.name,
                    entry.key,
                    contract.value_type.label()
                ),
            ));
        }
    }
    for value in &descriptor.values {
        if value.default_value.is_none() && node.value(value.key.as_str()).is_none() {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' requires value '{}'",
                    node.name, value.key
                ),
            ));
        }
    }

    let input_contracts = descriptor
        .inputs
        .iter()
        .map(|input| (input.name.as_str(), input))
        .collect::<BTreeMap<_, _>>();
    let output_contracts = descriptor
        .outputs
        .iter()
        .map(|output| (output.name.as_str(), output))
        .collect::<BTreeMap<_, _>>();

    for port in &node.ports {
        match port.direction {
            PortDirection::Input => {
                let Some(contract) = input_contracts.get(port.name.as_str()) else {
                    push_unknown_port_issue(report, node, port);
                    continue;
                };
                ratify_input_port(node, port, contract, report);
            }
            PortDirection::Output => {
                let Some(contract) = output_contracts.get(port.name.as_str()) else {
                    push_unknown_port_issue(report, node, port);
                    continue;
                };
                ratify_output_port(node, port, contract, report);
            }
        }
    }

    for input in &descriptor.inputs {
        if input.default_value.is_some() || node.value(input.name.as_str()).is_some() {
            continue;
        }
        let Some(port) = node
            .ports
            .iter()
            .find(|port| port.direction == PortDirection::Input && port.name == input.name)
        else {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' requires input '{}'",
                    node.name, input.name
                ),
            ));
            continue;
        };
        if !incoming_ports.contains(&port.id) {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' input '{}' must be connected or defaulted",
                    node.name, input.name
                ),
            ));
        }
    }

    for resource in &descriptor.resources {
        let Some(value) = node.value(resource.key.as_str()) else {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::MissingResourceReference,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' requires a catalog-backed {} resource reference '{}'",
                    node.name,
                    resource.kind.label(),
                    resource.key
                ),
            ));
            continue;
        };
        if !matches!(value, GraphValue::Resource(_)) {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::InvalidNodeSemantics,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material node '{}' resource '{}' is not a resource reference",
                    node.name, resource.key
                ),
            ));
        }
    }
}

fn ratify_input_port(
    node: &NodeDefinition,
    port: &PortDefinition,
    contract: &MaterialInputContract,
    report: &mut MaterialGraphRatificationReport,
) {
    if port.port_type != contract.value_type.port_type_id() {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::InvalidNodeSemantics,
            MaterialGraphIssueSubject::Node(node.id),
            format!(
                "material node '{}' input '{}' has port type {}, expected {}",
                node.name,
                port.name,
                port.port_type.raw(),
                contract.value_type.port_type_id().raw()
            ),
        ));
    }
}

fn ratify_output_port(
    node: &NodeDefinition,
    port: &PortDefinition,
    contract: &MaterialOutputContract,
    report: &mut MaterialGraphRatificationReport,
) {
    if port.port_type != contract.value_type.port_type_id() {
        report.push(RatificationIssue::error(
            MaterialGraphIssueCode::InvalidNodeSemantics,
            MaterialGraphIssueSubject::Node(node.id),
            format!(
                "material node '{}' output '{}' has port type {}, expected {}",
                node.name,
                port.name,
                port.port_type.raw(),
                contract.value_type.port_type_id().raw()
            ),
        ));
    }
}

fn push_unknown_port_issue(
    report: &mut MaterialGraphRatificationReport,
    node: &NodeDefinition,
    port: &PortDefinition,
) {
    report.push(RatificationIssue::error(
        MaterialGraphIssueCode::InvalidNodeSemantics,
        MaterialGraphIssueSubject::Node(node.id),
        format!(
            "material node '{}' declares unknown {:?} port '{}'",
            node.name, port.direction, port.name
        ),
    ));
}

fn incoming_port_set(edges: &[EdgeDefinition]) -> BTreeSet<PortId> {
    edges.iter().map(|edge| edge.to_port).collect()
}

fn graph_value_matches_type(value: &GraphValue, value_type: MaterialValueType) -> bool {
    match value_type {
        MaterialValueType::Bool => matches!(value, GraphValue::Bool(_)),
        MaterialValueType::Float => {
            matches!(value, GraphValue::Integer(_) | GraphValue::Decimal(_))
        }
        MaterialValueType::Vec2
        | MaterialValueType::Vec3
        | MaterialValueType::Vec4
        | MaterialValueType::Color => match value {
            GraphValue::Text(text) => text_vector_matches_type(text, value_type),
            _ => false,
        },
        MaterialValueType::ResourceTexture2D | MaterialValueType::ResourceTexture3D => {
            matches!(value, GraphValue::Resource(_))
        }
    }
}

fn text_vector_matches_type(text: &str, value_type: MaterialValueType) -> bool {
    let expected = match value_type {
        MaterialValueType::Vec2 => 2,
        MaterialValueType::Vec3 => 3,
        MaterialValueType::Vec4 | MaterialValueType::Color => 4,
        _ => return false,
    };
    let values = text
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    values.len() == expected && values.iter().all(|value| value.parse::<f32>().is_ok())
}

fn graph_error_message(error: &GraphValidationError) -> String {
    match error {
        GraphValidationError::DuplicateNodeId(id) => format!("duplicate node id {}", id.raw()),
        GraphValidationError::DuplicatePortId(id) => format!("duplicate port id {}", id.raw()),
        GraphValidationError::DuplicateEdgeId(id) => format!("duplicate edge id {}", id.raw()),
        GraphValidationError::MissingNode(id) => format!("missing node {}", id.raw()),
        GraphValidationError::MissingPort { edge_id, port_id } => {
            format!(
                "edge {} references missing port {}",
                edge_id.raw(),
                port_id.raw()
            )
        }
        GraphValidationError::EdgeDirectionInvalid { edge_id, .. } => {
            format!("edge {} has invalid port directions", edge_id.raw())
        }
        GraphValidationError::PortTypeMismatch { edge_id, .. } => {
            format!("edge {} has mismatched port types", edge_id.raw())
        }
        GraphValidationError::DirectedCycleDetected => "directed cycle detected".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, NodeDefinition, NodeId,
        PortDefinition, PortDirection, PortId,
    };

    #[test]
    fn ratification_rejects_invalid_vector_text_before_lowering() {
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(1),
            "Invalid Vector",
            GraphDefinition::new(
                GraphId::new(1),
                "invalid.vector",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(1),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(1),
                            "color",
                            PortDirection::Output,
                            crate::MaterialValueType::Color.port_type_id(),
                        )],
                    )
                    .with_values([GraphMetadataEntry::new(
                        "color",
                        GraphValue::Text("not a color".to_string()),
                    )]),
                    NodeDefinition::new(
                        NodeId::new(2),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(2),
                            "base_color",
                            PortDirection::Input,
                            crate::MaterialValueType::Color.port_type_id(),
                        )],
                    ),
                ],
                [],
            ),
            crate::MaterialOutputTarget::PbrPreview,
        );

        let report = ratify_material_graph(&document, &MaterialNodeCatalog::first_slice());

        assert!(report.has_blocking_issues());
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.message().contains("is not a color"))
        );
    }
}
