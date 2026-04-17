//! File: domain/editor/editor_shell/src/lib.rs
//! Crate: editor_shell

pub mod commands;
pub mod composition;
pub mod expression;
pub mod ids;
pub mod observation;
pub mod runtime;
pub mod view_models;

#[cfg(test)]
mod tests;

pub use commands::*;
pub use composition::*;
pub use expression::*;
pub use ids::*;
pub use observation::*;
pub use runtime::*;
pub use view_models::*;
