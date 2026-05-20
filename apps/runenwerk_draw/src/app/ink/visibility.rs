//! Visible ink product and dirty tile state.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{CanvasTileId, DrawingInkTileProduct};

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawingInkVisibilityState {
    visible_products: BTreeMap<CanvasTileId, DrawingInkTileProduct>,
    dirty_tiles: BTreeSet<CanvasTileId>,
    document_dirty: bool,
}

impl DrawingInkVisibilityState {
    pub(crate) fn document_dirty(&self) -> bool {
        self.document_dirty
    }

    pub(crate) fn visible_products(&self) -> impl Iterator<Item = &DrawingInkTileProduct> {
        self.visible_products.values()
    }

    pub(crate) fn visible_product_count(&self) -> usize {
        self.visible_products.len()
    }

    pub(crate) fn dirty_tiles(&self) -> &BTreeSet<CanvasTileId> {
        &self.dirty_tiles
    }

    pub(crate) fn next_dirty_tile_batch(&self, limit: usize) -> Vec<CanvasTileId> {
        self.dirty_tiles
            .iter()
            .take(limit.max(1))
            .copied()
            .collect()
    }

    pub(crate) fn mark_dirty_tiles(&mut self, tiles: impl IntoIterator<Item = CanvasTileId>) {
        self.dirty_tiles.extend(tiles);
        self.document_dirty = !self.dirty_tiles.is_empty();
    }

    pub(crate) fn invalidate_after_document_change(&mut self) {
        self.document_dirty = true;
    }

    pub(crate) fn record_accepted_clear_generation(
        &mut self,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
    ) {
        let dirty_tiles = dirty_tiles.into_iter().collect::<BTreeSet<_>>();
        for tile_id in cleared_tiles {
            self.visible_products.remove(&tile_id);
        }
        for tile_id in dirty_tiles {
            self.dirty_tiles.remove(&tile_id);
        }
        self.document_dirty = !self.dirty_tiles.is_empty();
    }

    pub(crate) fn promote_accepted_snapshots<'a>(
        &mut self,
        products: impl IntoIterator<Item = &'a DrawingInkTileProduct>,
        cleared_tiles: &BTreeSet<CanvasTileId>,
        dirty_tiles: &BTreeSet<CanvasTileId>,
    ) {
        for tile_id in cleared_tiles {
            self.visible_products.remove(tile_id);
        }
        for product in products {
            self.visible_products
                .insert(product.metadata.tile_id, product.clone());
        }
        for tile_id in dirty_tiles {
            self.dirty_tiles.remove(tile_id);
        }
        self.document_dirty = !self.dirty_tiles.is_empty();
    }
}
