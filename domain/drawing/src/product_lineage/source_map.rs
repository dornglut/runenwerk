//! File: domain/drawing/src/product_lineage/source_map.rs
//! Purpose: Product source lineage DTOs.

use crate::{BrushId, DrawingDocumentRevision, PaperId, StrokeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceRange<T> {
    pub start: T,
    pub end_inclusive: T,
}

impl<T> SourceRange<T> {
    pub const fn new(start: T, end_inclusive: T) -> Self {
        Self {
            start,
            end_inclusive,
        }
    }
}

pub type StrokeLineageRange = SourceRange<StrokeId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrushLineageRef {
    pub brush_id: BrushId,
    pub revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaperLineageRef {
    pub paper_id: PaperId,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingProductLineage {
    pub document_revision: DrawingDocumentRevision,
    pub stroke_range: Option<StrokeLineageRange>,
    pub brush_revisions: Vec<BrushLineageRef>,
    pub paper_revisions: Vec<PaperLineageRef>,
}

impl DrawingProductLineage {
    pub fn new(document_revision: DrawingDocumentRevision) -> Self {
        Self {
            document_revision,
            stroke_range: None,
            brush_revisions: Vec::new(),
            paper_revisions: Vec::new(),
        }
    }
}
