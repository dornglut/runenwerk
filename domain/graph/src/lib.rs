//! Crate: graph
//! Purpose: Domain-neutral graph definitions, validation, and traversal.

pub mod ids;
pub mod model;
pub mod traversal;
pub mod validation;

pub use ids::{EdgeId, GraphId, NodeId, PortId, PortTypeId};
pub use model::{
    CyclePolicy, EdgeDefinition, GraphDefinition, NodeDefinition, PortDefinition, PortDirection,
};
pub use traversal::{reachable_nodes, topological_order};
pub use validation::{GraphValidationError, validate_graph};
