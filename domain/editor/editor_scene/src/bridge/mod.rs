//! File: domain/editor/editor_scene/src/bridge/mod.rs
//! Purpose: Scene runtime bridge and command construction.

pub mod command_builder;
pub mod scene_runtime;

pub use command_builder::*;
pub use scene_runtime::*;