use super::CavernCollisionField;
use crate::{CavernGeometryGraph, CavernLayout, CavernRunConfig, CavernSeed, CavernTopology};

#[test]
fn cached_distance_tracks_analytic_distance() {
    let layout = CavernLayout::generate(CavernSeed(17), &CavernRunConfig::default());
    let topology = CavernTopology::from_layout(&layout, CavernSeed(17));
    let graph = CavernGeometryGraph::from_topology(&topology);
    let mut field = CavernCollisionField::from_graph(&graph);
    let point = [layout.rooms[0].center[0], 0.0, layout.rooms[0].center[1]];
    let analytic = field.distance_analytic(&graph, point);
    let cached = field.distance(&graph, point);
    assert!((analytic - cached).abs() < 0.35);
}

#[test]
fn push_out_sphere_resolves_solid_penetration() {
    let layout = CavernLayout::generate(CavernSeed(3), &CavernRunConfig::default());
    let topology = CavernTopology::from_layout(&layout, CavernSeed(3));
    let graph = CavernGeometryGraph::from_topology(&topology);
    let mut field = CavernCollisionField::from_graph(&graph);
    let outside = [
        layout.world_bounds[0] - 1.0,
        0.0,
        layout.world_bounds[1] - 1.0,
    ];
    let pushed = field.push_out_sphere(&graph, outside, 0.45);
    assert!(pushed.collided);
}
