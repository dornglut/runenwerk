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
mod source;

pub use contract_diagnostics::*;
pub use descriptor::*;
pub use diagnostics::*;
pub use entry_point::*;
pub use interface::*;
pub use layout::*;
pub use source::*;
