//! File: domain/editor/editor_shell/src/workspace/mod.rs
//! Purpose: Workspace identity contracts for shell composition.

pub mod definition_form;
pub mod identity;
pub mod profile;
pub mod state;
pub mod surface_contract;
pub mod viewport_embed_slot;
pub mod window;

pub use definition_form::*;
pub use identity::*;
pub use profile::*;
pub use state::*;
pub use surface_contract::*;
pub use viewport_embed_slot::*;
pub use window::*;
