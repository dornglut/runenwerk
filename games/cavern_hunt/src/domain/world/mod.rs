// World domain modules retained for generation/runtime adapters.
// Authoritative runtime ownership for simulation queries is in `engine::plugins::world`.
pub mod geometry_graph;
pub mod worldgen;

pub use geometry_graph::*;
pub use worldgen::*;
