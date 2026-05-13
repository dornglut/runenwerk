//! Drawing app-owned ink product publication and visibility state.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{CanvasTileId, DrawingInkTileProduct, DrawingTileFormationDiagnostic};
use product::{ProductDescriptorCore, ProductIdentity};

const DRAWING_INK_JOURNAL_LIMIT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingInkJournalStage {
    Formation,
    ProductPublication,
    QuerySnapshotPublication,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingInkJournalEntry {
    pub stage: DrawingInkJournalStage,
    pub accepted: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct DrawingInkRuntimeState {
    visible_products: BTreeMap<CanvasTileId, DrawingInkTileProduct>,
    candidate_products: Vec<DrawingInkTileProduct>,
    candidate_cleared_tiles: BTreeSet<CanvasTileId>,
    candidate_dirty_tiles: BTreeSet<CanvasTileId>,
    preview_products: Vec<DrawingInkTileProduct>,
    dirty_tiles: BTreeSet<CanvasTileId>,
    published_descriptors: Vec<ProductDescriptorCore>,
    accepted_snapshot_ids: BTreeSet<ProductIdentity>,
    last_formation_key: Option<String>,
    last_publication_key: Option<String>,
    last_query_snapshot_key: Option<String>,
    document_dirty: bool,
    diagnostics: Vec<DrawingTileFormationDiagnostic>,
    journal: Vec<DrawingInkJournalEntry>,
}

impl DrawingInkRuntimeState {
    pub fn formed_products(&self) -> &[DrawingInkTileProduct] {
        &self.candidate_products
    }

    pub fn candidate_cleared_tiles(&self) -> &BTreeSet<CanvasTileId> {
        &self.candidate_cleared_tiles
    }

    pub fn preview_products(&self) -> &[DrawingInkTileProduct] {
        &self.preview_products
    }

    pub fn published_descriptors(&self) -> &[ProductDescriptorCore] {
        &self.published_descriptors
    }

    pub fn accepted_snapshot_ids(&self) -> &BTreeSet<ProductIdentity> {
        &self.accepted_snapshot_ids
    }

    pub fn diagnostics(&self) -> &[DrawingTileFormationDiagnostic] {
        &self.diagnostics
    }

    pub fn journal(&self) -> &[DrawingInkJournalEntry] {
        &self.journal
    }

    pub fn last_formation_key(&self) -> Option<&str> {
        self.last_formation_key.as_deref()
    }

    pub fn last_publication_key(&self) -> Option<&str> {
        self.last_publication_key.as_deref()
    }

    pub fn last_query_snapshot_key(&self) -> Option<&str> {
        self.last_query_snapshot_key.as_deref()
    }

    pub fn document_dirty(&self) -> bool {
        self.document_dirty
    }

    pub fn visible_products(&self) -> impl Iterator<Item = &DrawingInkTileProduct> {
        self.visible_products.values()
    }

    pub fn visible_product_count(&self) -> usize {
        self.visible_products.len()
    }

    pub fn dirty_tiles(&self) -> &BTreeSet<CanvasTileId> {
        &self.dirty_tiles
    }

    pub fn next_dirty_tile_batch(&self, limit: usize) -> Vec<CanvasTileId> {
        if !self.candidate_products.is_empty()
            || !self.candidate_cleared_tiles.is_empty()
            || !self.published_descriptors.is_empty()
        {
            return Vec::new();
        }
        self.dirty_tiles
            .iter()
            .take(limit.max(1))
            .copied()
            .collect()
    }

    pub fn mark_dirty_tiles(&mut self, tiles: impl IntoIterator<Item = CanvasTileId>) {
        self.dirty_tiles.extend(tiles);
        self.document_dirty = !self.dirty_tiles.is_empty();
    }

    pub fn invalidate_after_document_change(&mut self) {
        self.document_dirty = true;
        self.last_formation_key = None;
        self.last_publication_key = None;
        self.last_query_snapshot_key = None;
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
    }

    pub fn record_preview_products(
        &mut self,
        products: Vec<DrawingInkTileProduct>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        self.preview_products = products;
        self.diagnostics = diagnostics;
    }

    pub fn clear_preview_products(&mut self) {
        self.preview_products.clear();
    }

    pub fn record_preview_failure(&mut self, diagnostics: Vec<DrawingTileFormationDiagnostic>) {
        self.preview_products.clear();
        self.diagnostics = diagnostics;
    }

    pub fn record_candidate_products(
        &mut self,
        formation_key: String,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        self.last_formation_key = Some(formation_key);
        self.candidate_products = products;
        self.candidate_cleared_tiles = cleared_tiles.into_iter().collect();
        self.candidate_dirty_tiles = dirty_tiles.into_iter().collect();
        self.diagnostics = diagnostics;
    }

    pub fn record_accepted_clear_generation(
        &mut self,
        formation_key: String,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        let dirty_tiles = dirty_tiles.into_iter().collect::<BTreeSet<_>>();
        for tile_id in cleared_tiles {
            self.visible_products.remove(&tile_id);
        }
        for tile_id in dirty_tiles {
            self.dirty_tiles.remove(&tile_id);
        }
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.preview_products.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
        self.last_formation_key = Some(formation_key);
        self.last_publication_key = None;
        self.last_query_snapshot_key = None;
        self.diagnostics = diagnostics;
        self.document_dirty = !self.dirty_tiles.is_empty();
    }

    pub fn record_failed_generation(
        &mut self,
        formation_key: String,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
        clear_preview_products: bool,
    ) {
        self.last_formation_key = Some(formation_key.clone());
        self.last_publication_key = Some(formation_key);
        self.last_query_snapshot_key = None;
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
        if clear_preview_products {
            self.preview_products.clear();
        }
        self.diagnostics = diagnostics;
    }

    pub fn record_published_descriptors(
        &mut self,
        publication_key: String,
        descriptors: Vec<ProductDescriptorCore>,
    ) {
        self.last_publication_key = Some(publication_key);
        self.published_descriptors = descriptors;
    }

    pub fn record_accepted_snapshots(
        &mut self,
        query_snapshot_key: String,
        accepted_snapshot_ids: impl IntoIterator<Item = ProductIdentity>,
    ) -> bool {
        self.last_query_snapshot_key = Some(query_snapshot_key);
        self.accepted_snapshot_ids = accepted_snapshot_ids.into_iter().collect();
        let fully_accepted = !self.candidate_products.is_empty()
            && self.candidate_products.iter().all(|product| {
                self.accepted_snapshot_ids
                    .contains(&ProductIdentity::new(product.metadata.product_id.raw()))
            });
        if fully_accepted {
            for tile_id in &self.candidate_cleared_tiles {
                self.visible_products.remove(tile_id);
            }
            for product in &self.candidate_products {
                self.visible_products
                    .insert(product.metadata.tile_id, product.clone());
            }
            for tile_id in &self.candidate_dirty_tiles {
                self.dirty_tiles.remove(tile_id);
            }
            self.candidate_products.clear();
            self.candidate_cleared_tiles.clear();
            self.candidate_dirty_tiles.clear();
            self.published_descriptors.clear();
            self.preview_products.clear();
            self.document_dirty = !self.dirty_tiles.is_empty();
        }
        fully_accepted
    }

    pub fn record_journal(
        &mut self,
        stage: DrawingInkJournalStage,
        accepted: bool,
        summary: impl Into<String>,
    ) {
        self.journal.push(DrawingInkJournalEntry {
            stage,
            accepted,
            summary: summary.into(),
        });
        if self.journal.len() > DRAWING_INK_JOURNAL_LIMIT {
            let drain = self.journal.len() - DRAWING_INK_JOURNAL_LIMIT;
            self.journal.drain(0..drain);
        }
    }
}
