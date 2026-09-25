//! File: domain/editor/editor_shell/src/workspace/mod.rs
//! Purpose: Workspace identity contracts for shell composition.

pub mod definition_form;
pub mod identity;
#[cfg(test)]
mod persisted;
pub mod profile;
#[cfg(test)]
pub mod projection;
#[cfg(test)]
pub mod projection_ratification;
#[cfg(test)]
pub mod reducer;
pub mod state;
pub mod surface_contract;
pub mod viewport_embed_slot;
pub mod window;

pub use definition_form::*;
pub use identity::*;
pub use profile::*;
#[cfg(test)]
pub(crate) use projection::*;
#[cfg(test)]
pub(crate) use reducer::*;
pub use state::*;
pub use surface_contract::*;
pub use viewport_embed_slot::*;
pub use window::*;
