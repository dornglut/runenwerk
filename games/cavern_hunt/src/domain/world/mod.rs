// Legacy world domain modules remain for migration compatibility.
// Authoritative runtime ownership is moving into `engine::plugins::world`.
pub mod collision_field;
pub mod geometry_graph;
pub mod worldgen;

pub use collision_field::*;
pub use geometry_graph::*;
pub use worldgen::*;
