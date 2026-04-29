//! File: domain/graph/src/validation.rs
//! Purpose: Domain-neutral graph invariant checks.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, NodeId, PortDirection, PortId, PortTypeId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValidationError {
    DuplicateNodeId(NodeId),
    DuplicatePortId(PortId),
    DuplicateEdgeId(EdgeId),
    MissingNode(NodeId),
    MissingPort {
        edge_id: EdgeId,
        port_id: PortId,
    },
    EdgeDirectionInvalid {
        edge_id: EdgeId,
        from_direction: PortDirection,
        to_direction: PortDirection,
    },
    PortTypeMismatch {
        edge_id: EdgeId,
        from_type: PortTypeId,
        to_type: PortTypeId,
    },
    DirectedCycleDetected,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedPort {
    pub node_id: NodeId,
    pub direction: PortDirection,
    pub port_type: PortTypeId,
}

pub(crate) fn resolved_ports(
    graph: &GraphDefinition,
) -> Result<BTreeMap<PortId, ResolvedPort>, GraphValidationError> {
    let mut node_ids = BTreeSet::new();
    let mut ports = BTreeMap::new();

    for node in &graph.nodes {
        if !node_ids.insert(node.id) {
            return Err(GraphValidationError::DuplicateNodeId(node.id));
        }

        for port in &node.ports {
            if ports
                .insert(
                    port.id,
                    ResolvedPort {
                        node_id: node.id,
                        direction: port.direction,
                        port_type: port.port_type,
                    },
                )
                .is_some()
            {
                return Err(GraphValidationError::DuplicatePortId(port.id));
            }
        }
    }

    Ok(ports)
}

pub(crate) fn validate_edges(
    graph: &GraphDefinition,
    ports: &BTreeMap<PortId, ResolvedPort>,
) -> Result<(), GraphValidationError> {
    let mut edge_ids = BTreeSet::new();

    for edge in &graph.edges {
        if !edge_ids.insert(edge.id) {
            return Err(GraphValidationError::DuplicateEdgeId(edge.id));
        }

        validate_edge(edge, ports)?;
    }

    Ok(())
}

pub fn validate_graph(graph: &GraphDefinition) -> Result<(), GraphValidationError> {
    let ports = resolved_ports(graph)?;
    validate_edges(graph, &ports)?;

    if graph.cycle_policy == CyclePolicy::RejectDirectedCycles {
        crate::traversal::topological_order(graph).map(|_| ())?;
    }

    Ok(())
}

fn validate_edge(
    edge: &EdgeDefinition,
    ports: &BTreeMap<PortId, ResolvedPort>,
) -> Result<(), GraphValidationError> {
    let from = ports
        .get(&edge.from_port)
        .copied()
        .ok_or(GraphValidationError::MissingPort {
            edge_id: edge.id,
            port_id: edge.from_port,
        })?;
    let to = ports
        .get(&edge.to_port)
        .copied()
        .ok_or(GraphValidationError::MissingPort {
            edge_id: edge.id,
            port_id: edge.to_port,
        })?;

    if from.direction != PortDirection::Output || to.direction != PortDirection::Input {
        return Err(GraphValidationError::EdgeDirectionInvalid {
            edge_id: edge.id,
            from_direction: from.direction,
            to_direction: to.direction,
        });
    }

    if from.port_type != to.port_type {
        return Err(GraphValidationError::PortTypeMismatch {
            edge_id: edge.id,
            from_type: from.port_type,
            to_type: to.port_type,
        });
    }

    Ok(())
}

pub(crate) fn node_edges(
    graph: &GraphDefinition,
    ports: &BTreeMap<PortId, ResolvedPort>,
) -> Result<BTreeMap<NodeId, BTreeSet<NodeId>>, GraphValidationError> {
    let mut edges_by_node = graph
        .nodes
        .iter()
        .map(|node| (node.id, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for edge in &graph.edges {
        let from = ports
            .get(&edge.from_port)
            .ok_or(GraphValidationError::MissingPort {
                edge_id: edge.id,
                port_id: edge.from_port,
            })?;
        let to = ports
            .get(&edge.to_port)
            .ok_or(GraphValidationError::MissingPort {
                edge_id: edge.id,
                port_id: edge.to_port,
            })?;
        edges_by_node
            .entry(from.node_id)
            .or_default()
            .insert(to.node_id);
    }

    Ok(edges_by_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, PortDefinition, PortId,
        PortTypeId,
    };

    fn valid_graph() -> GraphDefinition {
        let number = PortTypeId::new(1);
        GraphDefinition::new(
            GraphId::new(1),
            "numbers",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "source",
                    [PortDefinition::new(
                        PortId::new(1),
                        "out",
                        PortDirection::Output,
                        number,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "sink",
                    [PortDefinition::new(
                        PortId::new(2),
                        "in",
                        PortDirection::Input,
                        number,
                    )],
                ),
            ],
            [EdgeDefinition::new(
                EdgeId::new(1),
                PortId::new(1),
                PortId::new(2),
            )],
        )
    }

    #[test]
    fn valid_graph_accepts_typed_output_to_input_edges() {
        validate_graph(&valid_graph()).expect("valid graph should pass");
    }

    #[test]
    fn duplicate_node_ids_are_rejected() {
        let mut graph = valid_graph();
        graph.nodes[1].id = graph.nodes[0].id;

        let error = validate_graph(&graph).expect_err("duplicate node id should fail");

        assert_eq!(error, GraphValidationError::DuplicateNodeId(NodeId::new(1)));
    }

    #[test]
    fn missing_edge_port_is_rejected() {
        let mut graph = valid_graph();
        graph.edges[0].to_port = PortId::new(99);

        let error = validate_graph(&graph).expect_err("missing port should fail");

        assert_eq!(
            error,
            GraphValidationError::MissingPort {
                edge_id: EdgeId::new(1),
                port_id: PortId::new(99),
            }
        );
    }

    #[test]
    fn mismatched_port_types_are_rejected() {
        let mut graph = valid_graph();
        graph.nodes[1].ports[0].port_type = PortTypeId::new(2);

        let error = validate_graph(&graph).expect_err("mismatched port type should fail");

        assert_eq!(
            error,
            GraphValidationError::PortTypeMismatch {
                edge_id: EdgeId::new(1),
                from_type: PortTypeId::new(1),
                to_type: PortTypeId::new(2),
            }
        );
    }
}
