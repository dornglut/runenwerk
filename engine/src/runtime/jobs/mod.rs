//! Runtime-owned product job execution.
//!
//! Domain crates describe product jobs. The engine runtime owns execution,
//! worker resources, completions, and diagnostics.

mod diagnostics;
mod executor;
mod product;
mod task;
mod types;

pub use diagnostics::*;
pub use executor::*;
pub use product::*;
pub use task::*;
pub use types::*;
