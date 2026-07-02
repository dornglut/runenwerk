//! File: domain/ui/ui_runtime/src/output/mod.rs
//! Purpose: Build renderer-facing UI frame data from retained tree state.

pub mod build_ui_frame;
pub mod evidence;
mod emit;
mod primitives;
mod text;

pub use build_ui_frame::{InteractionVisualState, build_ui_frame};
pub use evidence::*;
