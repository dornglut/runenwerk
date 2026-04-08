//! File: domain/editor/editor_inspector/src/adapter.rs
//! Purpose: Adapter contracts for generating inspector sections from domain targets.

use crate::{InspectTarget, InspectorSection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorAdapterError {
    UnsupportedTarget,
    TargetNotFound,
    TypeNotRegistered,
    ValueNotAvailable,
}

pub trait InspectorAdapter {
    type Error;

    fn supports(&self, target: &InspectTarget) -> bool;

    fn build_sections(&self, target: &InspectTarget) -> Result<Vec<InspectorSection>, Self::Error>;
}
