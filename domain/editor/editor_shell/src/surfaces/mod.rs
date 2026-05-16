//! File: domain/editor/editor_shell/src/surfaces/mod.rs
//! Purpose: Surface-owned workflow contracts for editor tool surfaces.

pub mod asset;
pub mod editor_definition;
pub mod entity_table;
pub mod inspector;
pub mod outliner;
pub mod sdf_operation;
pub mod viewport;

pub use asset::*;
pub use editor_definition::*;
pub use entity_table::*;
pub use inspector::*;
pub use outliner::*;
pub use sdf_operation::*;
pub use viewport::*;
