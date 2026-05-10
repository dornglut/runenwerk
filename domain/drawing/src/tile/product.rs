//! File: domain/drawing/src/tile/product.rs
//! Purpose: Drawing tile product metadata descriptors with no pixel payload.

use crate::{
    CanvasRect, CanvasTileId, CompositeOutputId, DrawingDocumentRevision, DrawingProductLineage,
    DrawingTileProductId, FormationVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductQualityClass {
    Preview,
    Final,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingTileProductSource {
    pub quality_class: ProductQualityClass,
    pub source_document_revision: DrawingDocumentRevision,
    pub source_output: CompositeOutputId,
    pub lineage: DrawingProductLineage,
    pub formation_version: FormationVersion,
    pub invalidation_bounds: CanvasRect,
}

impl DrawingTileProductSource {
    pub fn new(
        quality_class: ProductQualityClass,
        source_document_revision: DrawingDocumentRevision,
        source_output: CompositeOutputId,
        lineage: DrawingProductLineage,
        formation_version: FormationVersion,
        invalidation_bounds: CanvasRect,
    ) -> Self {
        Self {
            quality_class,
            source_document_revision,
            source_output,
            lineage,
            formation_version,
            invalidation_bounds,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingTileProduct {
    pub product_id: DrawingTileProductId,
    pub tile_id: CanvasTileId,
    pub quality_class: ProductQualityClass,
    pub source_document_revision: DrawingDocumentRevision,
    pub source_output: CompositeOutputId,
    pub lineage: DrawingProductLineage,
    pub formation_version: FormationVersion,
    pub invalidation_bounds: CanvasRect,
    pub last_good_product: Option<DrawingTileProductId>,
}

impl DrawingTileProduct {
    pub fn new(
        product_id: DrawingTileProductId,
        tile_id: CanvasTileId,
        source: DrawingTileProductSource,
    ) -> Self {
        Self {
            product_id,
            tile_id,
            quality_class: source.quality_class,
            source_document_revision: source.source_document_revision,
            source_output: source.source_output,
            lineage: source.lineage,
            formation_version: source.formation_version,
            invalidation_bounds: source.invalidation_bounds,
            last_good_product: None,
        }
    }

    pub fn with_last_good(mut self, last_good_product: DrawingTileProductId) -> Self {
        self.last_good_product = Some(last_good_product);
        self
    }
}
