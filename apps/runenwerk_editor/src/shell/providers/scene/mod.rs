//! File: apps/runenwerk_editor/src/shell/providers/scene/mod.rs
//! Purpose: Scene document surface providers.

pub mod entity_table;
pub mod inspector;
pub mod outliner;
pub mod viewport;

pub use entity_table::*;
pub use inspector::*;
pub use outliner::*;
pub use viewport::*;
