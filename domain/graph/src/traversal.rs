//! File: domain/graph/src/traversal.rs
//! Purpose: Traversal helpers for validated graph definitions.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{GraphDefinition, GraphValidationError, NodeId, validate_graph};

pub fn reachable_nodes(
    graph: &GraphDefinition,
    start: NodeId,
) -> Result<BTreeSet<NodeId>, GraphValidationError> {
    validate_graph(graph)?;
    let ports = crate::validation::resolved_ports(graph)?;
    let edges_by_node = crate::validation::node_edges(graph, &ports)?;

    if !edges_by_node.contains_key(&start) {
        return Err(GraphValidationError::MissingNode(start));
    }

    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([start]);

    while let Some(node) = queue.pop_front() {
        if !visited.insert(node) {
            continue;
        }

        if let Some(next_nodes) = edges_by_node.get(&node) {
            for next in next_nodes {
                if !visited.contains(next) {
                    queue.push_back(*next);
                }
            }
        }
    }

    Ok(visited)
}

pub fn topological_order(graph: &GraphDefinition) -> Result<Vec<NodeId>, GraphValidationError> {
    let ports = crate::validation::resolved_ports(graph)?;
    crate::validation::validate_edges(graph, &ports)?;
    let edges_by_node = crate::validation::node_edges(graph, &ports)?;

    let mut incoming_counts = graph
        .nodes
        .iter()
        .map(|node| (node.id, 0usize))
        .collect::<BTreeMap<_, _>>();

    for to_nodes in edges_by_node.values() {
        for to_node in to_nodes {
            if let Some(count) = incoming_counts.get_mut(to_node) {
                *count += 1;
            }
        }
    }

    let mut ready = incoming_counts
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(*node))
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(graph.nodes.len());

    while let Some(node) = ready.pop_front() {
        order.push(node);

        if let Some(to_nodes) = edges_by_node.get(&node) {
            for to_node in to_nodes {
                let count = incoming_counts
                    .get_mut(to_node)
                    .expect("edge endpoints should be known after validation");
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.push_back(*to_node);
                }
            }
        }
    }

    if order.len() != graph.nodes.len() {
        return Err(GraphValidationError::DirectedCycleDetected);
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use crate::{
        CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition,
        PortDefinition, PortDirection, PortId, PortTypeId,
    };

    use super::*;

    fn chain_graph(policy: CyclePolicy) -> GraphDefinition {
        let value = PortTypeId::new(1);
        GraphDefinition::new(
            GraphId::new(1),
            "chain",
            policy,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "a",
                    [PortDefinition::new(
                        PortId::new(1),
                        "out",
                        PortDirection::Output,
                        value,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "b",
                    [
                        PortDefinition::new(PortId::new(2), "in", PortDirection::Input, value),
                        PortDefinition::new(PortId::new(3), "out", PortDirection::Output, value),
                    ],
                ),
                NodeDefinition::new(
                    NodeId::new(3),
                    "c",
                    [PortDefinition::new(
                        PortId::new(4),
                        "in",
                        PortDirection::Input,
                        value,
                    )],
                ),
            ],
            [
                EdgeDefinition::new(EdgeId::new(1), PortId::new(1), PortId::new(2)),
                EdgeDefinition::new(EdgeId::new(2), PortId::new(3), PortId::new(4)),
            ],
        )
    }

    #[test]
    fn reachable_nodes_follow_outgoing_edges() {
        let reachable = reachable_nodes(
            &chain_graph(CyclePolicy::RejectDirectedCycles),
            NodeId::new(1),
        )
        .expect("chain graph traversal should succeed");

        assert_eq!(
            reachable,
            [NodeId::new(1), NodeId::new(2), NodeId::new(3)]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn topological_order_orders_dependencies_before_consumers() {
        let order = topological_order(&chain_graph(CyclePolicy::RejectDirectedCycles))
            .expect("chain graph should be acyclic");

        assert_eq!(order, vec![NodeId::new(1), NodeId::new(2), NodeId::new(3)]);
    }

    #[test]
    fn topological_order_rejects_directed_cycles() {
        let value = PortTypeId::new(1);
        let graph = GraphDefinition::new(
            GraphId::new(2),
            "cycle",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "a",
                    [
                        PortDefinition::new(PortId::new(1), "in", PortDirection::Input, value),
                        PortDefinition::new(PortId::new(2), "out", PortDirection::Output, value),
                    ],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "b",
                    [
                        PortDefinition::new(PortId::new(3), "in", PortDirection::Input, value),
                        PortDefinition::new(PortId::new(4), "out", PortDirection::Output, value),
                    ],
                ),
            ],
            [
                EdgeDefinition::new(EdgeId::new(1), PortId::new(2), PortId::new(3)),
                EdgeDefinition::new(EdgeId::new(2), PortId::new(4), PortId::new(1)),
            ],
        );

        let error = topological_order(&graph).expect_err("cycle should fail topological order");

        assert_eq!(error, GraphValidationError::DirectedCycleDetected);
    }
}
