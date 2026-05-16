//! App-owned Material Lab workflow.
//!
//! Domain crates own material semantics. This module owns project paths,
//! source document IO, preview-product orchestration, catalog publication, and
//! renderer prepared-data handoff.

pub mod document_io;
pub mod identity;
pub mod publication;
pub mod recipes;
pub mod renderer_handoff;
pub mod state;
pub mod workflow;

pub use document_io::*;
pub use identity::*;
pub use publication::*;
pub use recipes::*;
pub use renderer_handoff::*;
pub use state::*;
pub use workflow::*;
