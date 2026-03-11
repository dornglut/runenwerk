use super::{CavernGeometryGraph, CavernTopology, GeometryOp};
use crate::{CavernLayout, CavernRunConfig, CavernSeed};

#[test]
fn topology_and_graph_are_deterministic_from_layout() {
    let layout = CavernLayout::generate(CavernSeed(7), &CavernRunConfig::default());
    let topology_a = CavernTopology::from_layout(&layout, CavernSeed(7));
    let topology_b = CavernTopology::from_layout(&layout, CavernSeed(7));
    let graph_a = CavernGeometryGraph::from_topology(&topology_a);
    let graph_b = CavernGeometryGraph::from_topology(&topology_b);
    assert_eq!(topology_a, topology_b);
    assert_eq!(graph_a, graph_b);
    assert!(
        graph_a
            .primitives
            .iter()
            .any(|primitive| primitive.op == GeometryOp::SubtractVoid)
    );
}
