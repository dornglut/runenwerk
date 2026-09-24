//! File: domain/drawing/src/composition/source.rs
//! Purpose: Drawing composition source node descriptors.

use crate::{PaintSourceId, PaperId, ReferenceImageId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintLayerSource {
    pub paint_source_id: PaintSourceId,
    pub name: String,
}

impl PaintLayerSource {
    pub fn new(paint_source_id: PaintSourceId, name: impl Into<String>) -> Self {
        Self {
            paint_source_id,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaperSource {
    pub paper_id: PaperId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceImageSource {
    pub reference_image_id: ReferenceImageId,
    pub label: String,
}
