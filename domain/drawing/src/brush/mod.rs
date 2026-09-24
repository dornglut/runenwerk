//! File: domain/drawing/src/brush/mod.rs
//! Purpose: Brush descriptor and dynamics contracts.

mod descriptor;
mod dynamics;

pub use descriptor::{BrushDescriptor, BrushRange, InkBrushDescriptor};
pub use dynamics::{BrushDynamics, DynamicsCurve};
