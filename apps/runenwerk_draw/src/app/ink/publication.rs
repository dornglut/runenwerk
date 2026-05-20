//! Product publication and query snapshot state for ink products.

use std::collections::BTreeSet;

use drawing::{CanvasTileId, DrawingInkTileProduct};
use product::{ProductDescriptorCore, ProductIdentity};

#[derive(Debug, Clone, Default)]
pub(crate) struct DrawingInkPublicationState {
    candidate_products: Vec<DrawingInkTileProduct>,
    candidate_cleared_tiles: BTreeSet<CanvasTileId>,
    candidate_dirty_tiles: BTreeSet<CanvasTileId>,
    published_descriptors: Vec<ProductDescriptorCore>,
    accepted_snapshot_ids: BTreeSet<ProductIdentity>,
    last_formation_key: Option<String>,
    last_publication_key: Option<String>,
    last_query_snapshot_key: Option<String>,
    pending_formation_key: Option<String>,
}

impl DrawingInkPublicationState {
    pub(crate) fn formed_products(&self) -> &[DrawingInkTileProduct] {
        &self.candidate_products
    }

    pub(crate) fn candidate_cleared_tiles(&self) -> &BTreeSet<CanvasTileId> {
        &self.candidate_cleared_tiles
    }

    pub(crate) fn candidate_dirty_tiles(&self) -> &BTreeSet<CanvasTileId> {
        &self.candidate_dirty_tiles
    }

    pub(crate) fn published_descriptors(&self) -> &[ProductDescriptorCore] {
        &self.published_descriptors
    }

    pub(crate) fn accepted_snapshot_ids(&self) -> &BTreeSet<ProductIdentity> {
        &self.accepted_snapshot_ids
    }

    pub(crate) fn last_formation_key(&self) -> Option<&str> {
        self.last_formation_key.as_deref()
    }

    pub(crate) fn last_publication_key(&self) -> Option<&str> {
        self.last_publication_key.as_deref()
    }

    pub(crate) fn last_query_snapshot_key(&self) -> Option<&str> {
        self.last_query_snapshot_key.as_deref()
    }

    pub(crate) fn pending_formation_key(&self) -> Option<&str> {
        self.pending_formation_key.as_deref()
    }

    pub(crate) fn blocks_dirty_tile_batch(&self) -> bool {
        !self.candidate_products.is_empty()
            || !self.candidate_cleared_tiles.is_empty()
            || !self.published_descriptors.is_empty()
    }

    pub(crate) fn invalidate_after_document_change(&mut self) {
        self.last_formation_key = None;
        self.last_publication_key = None;
        self.last_query_snapshot_key = None;
        self.pending_formation_key = None;
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
    }

    pub(crate) fn record_candidate_products(
        &mut self,
        formation_key: String,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
    ) {
        self.pending_formation_key = None;
        self.last_formation_key = Some(formation_key);
        self.candidate_products = products;
        self.candidate_cleared_tiles = cleared_tiles.into_iter().collect();
        self.candidate_dirty_tiles = dirty_tiles.into_iter().collect();
    }

    pub(crate) fn record_accepted_clear_generation(&mut self, formation_key: String) {
        self.pending_formation_key = None;
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
        self.last_formation_key = Some(formation_key);
        self.last_publication_key = None;
        self.last_query_snapshot_key = None;
    }

    pub(crate) fn record_failed_generation(&mut self, formation_key: String) {
        self.pending_formation_key = None;
        self.last_formation_key = Some(formation_key.clone());
        self.last_publication_key = Some(formation_key);
        self.last_query_snapshot_key = None;
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
    }

    pub(crate) fn record_published_descriptors(
        &mut self,
        publication_key: String,
        descriptors: Vec<ProductDescriptorCore>,
    ) {
        self.last_publication_key = Some(publication_key);
        self.published_descriptors = descriptors;
    }

    pub(crate) fn record_pending_formation_job(&mut self, formation_key: String) {
        self.pending_formation_key = Some(formation_key);
    }

    pub(crate) fn clear_pending_formation_job(&mut self, formation_key: &str) {
        if self.pending_formation_key.as_deref() == Some(formation_key) {
            self.pending_formation_key = None;
        }
    }

    pub(crate) fn record_accepted_snapshots(
        &mut self,
        query_snapshot_key: String,
        accepted_snapshot_ids: impl IntoIterator<Item = ProductIdentity>,
    ) -> bool {
        self.last_query_snapshot_key = Some(query_snapshot_key);
        self.accepted_snapshot_ids = accepted_snapshot_ids.into_iter().collect();
        !self.candidate_products.is_empty()
            && self.candidate_products.iter().all(|product| {
                self.accepted_snapshot_ids
                    .contains(&ProductIdentity::new(product.metadata.product_id.raw()))
            })
    }

    pub(crate) fn clear_accepted_candidate(&mut self) {
        self.candidate_products.clear();
        self.candidate_cleared_tiles.clear();
        self.candidate_dirty_tiles.clear();
        self.published_descriptors.clear();
    }
}
