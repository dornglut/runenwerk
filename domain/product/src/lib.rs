//! Domain-level field/product contract vocabulary.
//!
//! This crate owns engine-agnostic product descriptors, product jobs, query
//! snapshots, renderer selection DTOs, and ratification rules. It is not a
//! product registry and does not own runtime execution, renderer resources,
//! app workflows, or concrete asset catalogs.

pub mod descriptor;
pub mod diagnostics;
pub mod ids;
pub mod job;
pub mod policy;
pub mod query_snapshot;
pub mod ratification;
pub mod render_selection;

pub use descriptor::*;
pub use diagnostics::*;
pub use ids::*;
pub use job::*;
pub use policy::*;
pub use query_snapshot::*;
pub use ratification::*;
pub use render_selection::*;
