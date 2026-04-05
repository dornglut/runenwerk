//! File: domain/editor/editor_inspector/src/bridge/mod.rs
//! Purpose: Inspector bridge layer for external runtime/domain integrations.

pub mod ecs_adapter;
pub mod ecs_bridge;

pub use ecs_adapter::*;
pub use ecs_bridge::*;