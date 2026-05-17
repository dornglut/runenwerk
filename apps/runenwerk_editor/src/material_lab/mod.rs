//! App-owned Material Lab workflow.
//!
//! Domain crates own material semantics. This module owns project paths,
//! source document IO, preview-product orchestration, catalog publication, and
//! renderer prepared-data handoff.

pub mod default_material;
pub mod document_io;
pub mod generated_artifacts;
pub mod identity;
pub mod publication;
pub mod recipes;
pub mod renderer_handoff;
pub mod resource_resolution;
pub mod state;
pub mod workflow;

pub use default_material::*;
pub use document_io::*;
pub use generated_artifacts::*;
pub use identity::*;
pub use publication::*;
pub use recipes::*;
pub use renderer_handoff::*;
pub use resource_resolution::*;
pub use state::*;
pub use workflow::*;
