//! File: domain/drawing/src/brush/descriptor.rs
//! Purpose: Authored brush descriptor contracts.

use crate::{BrushDynamics, BrushId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushRange {
    pub min: f32,
    pub max: f32,
}

impl BrushRange {
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn is_valid_positive(self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.min >= 0.0 && self.max >= self.min
    }

    pub fn is_valid_unit(self) -> bool {
        self.is_valid_positive() && self.max <= 1.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InkBrushDescriptor {
    pub size: BrushRange,
    pub opacity: BrushRange,
    pub flow: BrushRange,
    pub edge_softness: f32,
    pub viscosity: f32,
    pub absorption_response: f32,
    pub dynamics: BrushDynamics,
}

impl InkBrushDescriptor {
    pub const fn new(size: BrushRange, opacity: BrushRange, flow: BrushRange) -> Self {
        Self {
            size,
            opacity,
            flow,
            edge_softness: 0.5,
            viscosity: 0.5,
            absorption_response: 0.5,
            dynamics: BrushDynamics::none(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrushDescriptor {
    pub brush_id: BrushId,
    pub schema_version: u32,
    pub revision: u64,
    pub name: String,
    pub ink: InkBrushDescriptor,
}

impl BrushDescriptor {
    pub fn new(brush_id: BrushId, name: impl Into<String>, ink: InkBrushDescriptor) -> Self {
        Self {
            brush_id,
            schema_version: 1,
            revision: 1,
            name: name.into(),
            ink,
        }
    }
}
