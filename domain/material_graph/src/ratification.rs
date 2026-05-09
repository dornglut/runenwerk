//! File: domain/material_graph/src/ratification.rs
//! Purpose: Material graph semantic ratification.

use graph::{GraphValidationError, NodeId, validate_graph};
use ratification::{RatificationIssue, RatificationReport};

use crate::{MaterialGraphDocument, MaterialNodeCatalog};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaterialGraphIssueCode {
    EmptyLabel,
    GraphStructural,
    UnsupportedNode,
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
    for node in &document.graph.nodes {
        if node.name == "pbr.output" {
            has_output = true;
        }
        if !catalog.contains(&node.name) {
            report.push(RatificationIssue::error(
                MaterialGraphIssueCode::UnsupportedNode,
                MaterialGraphIssueSubject::Node(node.id),
                format!(
                    "material graph node '{}' is not in the active catalog",
                    node.name
                ),
            ));
        }
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
