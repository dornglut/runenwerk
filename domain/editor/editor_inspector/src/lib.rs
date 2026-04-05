//! File: domain/editor/editor_inspector/src/lib.rs
//! Crate: editor_inspector

pub mod adapter;
pub mod field;
pub mod resolution;
pub mod section;
pub mod target;
pub mod validation;

pub use adapter::*;
pub use field::*;
pub use resolution::*;
pub use section::*;
pub use target::*;
pub use validation::*;