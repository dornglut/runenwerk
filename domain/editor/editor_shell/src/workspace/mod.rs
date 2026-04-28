//! File: domain/editor/editor_shell/src/workspace/mod.rs
//! Purpose: Workspace identity contracts for shell composition.

pub mod identity;
pub mod persisted;
pub mod projection;
pub mod projection_ratification;
pub mod reducer;
pub mod state;
pub mod surface_contract;
pub mod viewport_embed_slot;

pub use identity::*;
pub use persisted::*;
pub use projection::*;
pub use projection_ratification::*;
pub use reducer::*;
pub use state::*;
pub use surface_contract::*;
pub use viewport_embed_slot::*;
