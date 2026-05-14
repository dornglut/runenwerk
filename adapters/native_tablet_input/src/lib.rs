//! File: adapters/native_tablet_input/src/lib.rs
//! Purpose: Native tablet packet capture and normalization into platform-neutral UI input events.

pub mod backend;
pub mod mapping;
pub mod model;
pub mod runtime;

pub use backend::*;
pub use mapping::*;
pub use model::*;
pub use runtime::*;
