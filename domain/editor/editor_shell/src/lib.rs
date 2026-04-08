//! File: domain/editor/editor_shell/src/lib.rs
//! Crate: editor_shell

pub mod commands;
pub mod composition;
pub mod ids;
pub mod runtime;
pub mod view_models;

#[cfg(test)]
mod tests;

pub use commands::*;
pub use composition::*;
pub use ids::*;
pub use runtime::*;
pub use view_models::*;
