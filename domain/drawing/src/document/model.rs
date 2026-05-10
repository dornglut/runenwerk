//! File: domain/drawing/src/document/model.rs
//! Purpose: Authored drawing document DTO root.

use crate::{
    BrushDescriptor, DrawingCompositeGraph, DrawingDocumentId, DrawingDocumentRevision,
    DrawingTileProduct, PaperDescriptor, PendingStrokeRecord, StrokeRecord,
};

pub const DRAWING_DOCUMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingDocument {
    pub document_id: DrawingDocumentId,
    pub schema_version: u32,
    pub revision: DrawingDocumentRevision,
    pub display_name: String,
    pub canvas_bounds: crate::CanvasRect,
    pub strokes: Vec<StrokeRecord>,
    pub pending_strokes: Vec<PendingStrokeRecord>,
    pub brushes: Vec<BrushDescriptor>,
    pub papers: Vec<PaperDescriptor>,
    pub composition: DrawingCompositeGraph,
    pub tile_products: Vec<DrawingTileProduct>,
}

impl DrawingDocument {
    pub fn new(
        document_id: DrawingDocumentId,
        display_name: impl Into<String>,
        canvas_bounds: crate::CanvasRect,
        composition: DrawingCompositeGraph,
    ) -> Self {
        Self {
            document_id,
            schema_version: DRAWING_DOCUMENT_SCHEMA_VERSION,
            revision: DrawingDocumentRevision::default(),
            display_name: display_name.into(),
            canvas_bounds,
            strokes: Vec::new(),
            pending_strokes: Vec::new(),
            brushes: Vec::new(),
            papers: Vec::new(),
            composition,
            tile_products: Vec::new(),
        }
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.next();
    }
}
