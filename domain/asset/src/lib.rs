//! Engine-agnostic asset contracts.
//!
//! This crate owns asset identity, taxonomy, source/artifact descriptors,
//! deterministic import planning, dependency graph contracts, catalog
//! validation, diagnostics, and ratification. It intentionally does not do host
//! IO, execute import tools, allocate runtime resources, define SDF sampling, or
//! own editor UI behavior.

pub mod artifact;
pub mod catalog;
pub mod dependency_graph;
pub mod diagnostics;
pub mod id;
pub mod import_plan;
pub mod import_settings;
pub mod kind;
pub mod ratification;
pub mod source;

pub use artifact::*;
pub use catalog::*;
pub use dependency_graph::*;
pub use diagnostics::*;
pub use id::*;
pub use import_plan::*;
pub use import_settings::*;
pub use kind::*;
pub use ratification::*;
pub use source::*;
