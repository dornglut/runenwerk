//! File: domain/ui/ui_runtime/src/layout/mod.rs
//! Purpose: Computed layout records for retained widgets.

pub mod computed_layout;
pub mod engine;

pub use computed_layout::{ComputedLayout, ComputedLayoutMap};
pub use engine::*;
