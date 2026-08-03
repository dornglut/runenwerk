//! Backend-neutral program, interface, and pipeline contracts.
//!
//! G4B grows this module by responsibility. Source admission is intentionally
//! independent from renderer source discovery and later backend realization.

mod diagnostics;
mod source;

pub use diagnostics::*;
pub use source::*;
