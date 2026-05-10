//! File: domain/drawing/src/stroke/mod.rs
//! Purpose: Stroke sample and committed stroke truth contracts.

mod record;
mod sample;

pub use record::{ColorRgba, PaintTarget, StrokeRecord};
pub use sample::{StrokeSample, StrokeToolKind, StylusTilt};
