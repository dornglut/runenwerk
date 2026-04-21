//! File: domain/editor/editor_shell/src/workspace/mod.rs
//! Purpose: Workspace identity contracts for shell composition.

pub mod identity;
pub mod persisted;
pub mod projection;
pub mod reducer;
pub mod state;

pub use identity::*;
pub use persisted::*;
pub use projection::*;
pub use reducer::*;
pub use state::*;
