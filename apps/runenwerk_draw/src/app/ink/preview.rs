//! Preview ink product state.

use std::collections::BTreeSet;

use drawing::{CanvasTileId, DrawingInkTileProduct, DrawingTileFormationDiagnostic};

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawingInkPreviewState {
    preview_products: Vec<DrawingInkTileProduct>,
    diagnostics: Vec<DrawingTileFormationDiagnostic>,
    last_preview_dirty_tile_count: usize,
}

impl DrawingInkPreviewState {
    pub(crate) fn preview_products(&self) -> &[DrawingInkTileProduct] {
        &self.preview_products
    }

    pub(crate) fn diagnostics(&self) -> &[DrawingTileFormationDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn last_preview_dirty_tile_count(&self) -> usize {
        self.last_preview_dirty_tile_count
    }

    pub(crate) fn set_diagnostics(&mut self, diagnostics: Vec<DrawingTileFormationDiagnostic>) {
        self.diagnostics = diagnostics;
    }

    pub(crate) fn record_preview_products(
        &mut self,
        products: Vec<DrawingInkTileProduct>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) -> Vec<DrawingInkTileProduct> {
        self.last_preview_dirty_tile_count = products.len();
        self.preview_products = products;
        self.diagnostics = diagnostics;
        self.preview_products.clone()
    }

    pub(crate) fn replace_preview_products_for_tiles(
        &mut self,
        tile_ids: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) -> Vec<DrawingInkTileProduct> {
        let mut replacement_tile_ids = tile_ids.into_iter().collect::<BTreeSet<_>>();
        replacement_tile_ids.extend(cleared_tiles);
        let products_to_cache = products.clone();
        self.preview_products
            .retain(|product| !replacement_tile_ids.contains(&product.metadata.tile_id));
        self.preview_products.extend(products);
        self.preview_products.sort_by_key(|product| {
            (
                product.metadata.tile_id.level.raw(),
                product.metadata.tile_id.x,
                product.metadata.tile_id.y,
                product.metadata.product_id.raw(),
            )
        });
        self.last_preview_dirty_tile_count = replacement_tile_ids.len();
        self.diagnostics = diagnostics;
        products_to_cache
    }

    pub(crate) fn clear_preview_products(&mut self) {
        self.preview_products.clear();
        self.last_preview_dirty_tile_count = 0;
    }

    pub(crate) fn clear_preview_products_preserving_dirty_count(&mut self) {
        self.preview_products.clear();
    }

    pub(crate) fn record_preview_failure(
        &mut self,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        self.preview_products.clear();
        self.last_preview_dirty_tile_count = 0;
        self.diagnostics = diagnostics;
    }
}
