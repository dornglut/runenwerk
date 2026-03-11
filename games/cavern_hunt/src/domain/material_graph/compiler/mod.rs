use super::*;

mod compile;
mod compile_node;
mod errors;
mod resolve_helpers;
#[cfg(test)]
mod tests;
mod types_and_constants;

pub use compile::compile_material_graph;
pub use errors::MaterialCompileError;
pub use types_and_constants::*;

use compile_node::compile_node;
use resolve_helpers::{
    MaterialDefaults, resolve_input, resolve_numeric_input, resolve_optional_slot, resolve_slot,
};
