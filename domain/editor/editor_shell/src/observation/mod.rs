//! File: domain/editor/editor_shell/src/observation/mod.rs
//! Purpose: Observation-frame contracts for shell consumers.

mod frame;
mod inspector;
mod outliner;
mod toolbar;
mod viewport;

pub use frame::*;
pub use inspector::*;
pub use outliner::*;
pub use toolbar::*;
pub use viewport::*;
