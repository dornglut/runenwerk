//! Backend-neutral program, interface, and pipeline contracts.
//!
//! G4B grows this module by responsibility. Source admission is intentionally
//! independent from renderer source discovery and later backend realization.

mod contract_diagnostics;
mod descriptor;
mod diagnostics;
mod entry_point;
mod interface;
mod layout;
mod pipeline;
mod requirement_identity;
mod runtime_binding;
mod source;
mod specialization;
mod stage_io;

pub use contract_diagnostics::*;
pub use descriptor::*;
pub use diagnostics::*;
pub use entry_point::*;
pub use interface::*;
pub use layout::*;
pub use pipeline::*;
pub use runtime_binding::*;
pub use source::*;
pub use specialization::*;
pub use stage_io::*;
