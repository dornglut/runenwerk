//! File: adapters/native_tablet_input/src/lib.rs
//! Purpose: Native tablet acquisition and translation into engine-owned neutral observations.

pub mod backend;
pub mod mapping;
pub mod model;
pub mod runtime;

pub use backend::*;
pub use mapping::*;
pub use model::*;
pub use runtime::*;
