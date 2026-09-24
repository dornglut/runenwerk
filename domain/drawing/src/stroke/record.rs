//! File: domain/drawing/src/stroke/record.rs
//! Purpose: Committed stroke record contracts.

use crate::{
    BrushId, CanvasRect, DrawingDocumentRevision, LayerStackEntryId, PaintSourceId, StrokeId,
    StrokeSample,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn is_valid(self) -> bool {
        [self.r, self.g, self.b, self.a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaintTarget {
    StackEntry(LayerStackEntryId),
    PaintSource(PaintSourceId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeRecord {
    pub stroke_id: StrokeId,
    pub target: PaintTarget,
    pub brush_id: BrushId,
    pub color: ColorRgba,
    pub samples: Vec<StrokeSample>,
    pub bounds: CanvasRect,
    pub source_revision: DrawingDocumentRevision,
}

impl StrokeRecord {
    pub fn new(
        stroke_id: StrokeId,
        target: PaintTarget,
        brush_id: BrushId,
        color: ColorRgba,
        samples: impl IntoIterator<Item = StrokeSample>,
        source_revision: DrawingDocumentRevision,
    ) -> Option<Self> {
        let samples = samples.into_iter().collect::<Vec<_>>();
        let bounds = CanvasRect::from_points(samples.iter().map(|sample| sample.position))?;
        Some(Self {
            stroke_id,
            target,
            brush_id,
            color,
            samples,
            bounds,
            source_revision,
        })
    }
}
