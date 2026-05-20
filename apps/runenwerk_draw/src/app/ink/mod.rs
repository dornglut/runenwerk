//! Drawing app-owned ink product publication and visibility state.

mod cache;
mod gpu_validation;
mod journal;
mod preview;
mod publication;
mod visibility;

use std::collections::BTreeSet;

use drawing::{
    CanvasTileId, DrawingInkTileProduct, DrawingTileFormationDiagnostic,
    drawing_ink_tile_product_cache_identity,
};
use product::{ProductCacheKey, ProductDescriptorCore, ProductIdentity};

pub use cache::DrawingInkTileCacheEntryMetadata;
use cache::DrawingInkTileCacheState;
use gpu_validation::DrawingInkGpuValidationState;
pub use gpu_validation::{
    DrawingInkGpuValidationEntry, DrawingInkGpuValidationMetrics, DrawingInkGpuValidationStatus,
};
use journal::DrawingInkJournalState;
pub use journal::{DrawingInkJournalEntry, DrawingInkJournalStage};
use preview::DrawingInkPreviewState;
use publication::DrawingInkPublicationState;
use visibility::DrawingInkVisibilityState;

use crate::app::DrawingInkSurfaceKind;

#[derive(Debug, Clone, Default)]
pub struct DrawingInkRuntimeState {
    preview: DrawingInkPreviewState,
    publication: DrawingInkPublicationState,
    visibility: DrawingInkVisibilityState,
    cache: DrawingInkTileCacheState,
    gpu_validation: DrawingInkGpuValidationState,
    journal: DrawingInkJournalState,
}

impl DrawingInkRuntimeState {
    pub fn formed_products(&self) -> &[DrawingInkTileProduct] {
        self.publication.formed_products()
    }

    pub fn candidate_cleared_tiles(&self) -> &BTreeSet<CanvasTileId> {
        self.publication.candidate_cleared_tiles()
    }

    pub fn preview_products(&self) -> &[DrawingInkTileProduct] {
        self.preview.preview_products()
    }

    pub fn cached_product_key_for_source_key(&self, source_key: &str) -> Option<&ProductCacheKey> {
        self.cache.cached_product_key_for_source_key(source_key)
    }

    pub fn cached_product(&self, key: &ProductCacheKey) -> Option<&DrawingInkTileProduct> {
        self.cache.cached_product(key)
    }

    pub fn cached_product_for_source_key(
        &mut self,
        source_key: &str,
    ) -> Option<(ProductCacheKey, DrawingInkTileProduct)> {
        self.cache.cached_product_for_source_key(source_key)
    }

    pub fn published_descriptors(&self) -> &[ProductDescriptorCore] {
        self.publication.published_descriptors()
    }

    pub fn accepted_snapshot_ids(&self) -> &BTreeSet<ProductIdentity> {
        self.publication.accepted_snapshot_ids()
    }

    pub fn diagnostics(&self) -> &[DrawingTileFormationDiagnostic] {
        self.preview.diagnostics()
    }

    pub fn journal(&self) -> &[DrawingInkJournalEntry] {
        self.journal.journal()
    }

    pub fn last_formation_key(&self) -> Option<&str> {
        self.publication.last_formation_key()
    }

    pub fn last_publication_key(&self) -> Option<&str> {
        self.publication.last_publication_key()
    }

    pub fn last_query_snapshot_key(&self) -> Option<&str> {
        self.publication.last_query_snapshot_key()
    }

    pub fn pending_formation_key(&self) -> Option<&str> {
        self.publication.pending_formation_key()
    }

    pub fn document_dirty(&self) -> bool {
        self.visibility.document_dirty()
    }

    pub fn visible_products(&self) -> impl Iterator<Item = &DrawingInkTileProduct> {
        self.visibility.visible_products()
    }

    pub fn visible_product_count(&self) -> usize {
        self.visibility.visible_product_count()
    }

    pub fn dirty_tiles(&self) -> &BTreeSet<CanvasTileId> {
        self.visibility.dirty_tiles()
    }

    pub fn last_preview_dirty_tile_count(&self) -> usize {
        self.preview.last_preview_dirty_tile_count()
    }

    pub fn tile_cache_budget_bytes(&self) -> usize {
        self.cache.tile_cache_budget_bytes()
    }

    pub fn tile_cache_payload_bytes(&self) -> usize {
        self.cache.tile_cache_payload_bytes()
    }

    pub fn tile_cache_entry_count(&self) -> usize {
        self.cache.tile_cache_entry_count()
    }

    pub fn tile_cache_metadata(&self) -> Vec<DrawingInkTileCacheEntryMetadata> {
        self.cache.tile_cache_metadata()
    }

    pub fn gpu_validation_metadata(&self) -> Vec<DrawingInkGpuValidationEntry> {
        self.gpu_validation.gpu_validation_metadata()
    }

    pub fn gpu_validation_status(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> Option<DrawingInkGpuValidationStatus> {
        self.gpu_validation
            .gpu_validation_status(surface_kind, product)
    }

    pub fn should_request_gpu_validation(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> bool {
        self.gpu_validation
            .should_request_gpu_validation(surface_kind, product)
    }

    pub fn should_request_gpu_target(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> bool {
        self.gpu_validation
            .should_request_gpu_target(surface_kind, product)
    }

    pub fn visible_surface_kind_for(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) -> DrawingInkSurfaceKind {
        self.gpu_validation
            .visible_surface_kind_for(surface_kind, product)
    }

    pub fn record_gpu_validation_pending(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
    ) {
        self.gpu_validation
            .record_gpu_validation_pending(surface_kind, product);
    }

    pub fn record_gpu_validation_pass(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
        metrics: DrawingInkGpuValidationMetrics,
    ) {
        let (accepted, summary) =
            self.gpu_validation
                .record_gpu_validation_pass(surface_kind, product, metrics);
        self.record_journal(DrawingInkJournalStage::GpuValidation, accepted, summary);
    }

    pub fn record_gpu_validation_failure(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        product: &DrawingInkTileProduct,
        reason: impl Into<String>,
    ) {
        let (accepted, summary) =
            self.gpu_validation
                .record_gpu_validation_failure(surface_kind, product, reason.into());
        self.record_journal(DrawingInkJournalStage::GpuValidation, accepted, summary);
    }

    pub fn set_tile_cache_budget_bytes(&mut self, budget_bytes: usize) {
        self.cache.set_tile_cache_budget_bytes(budget_bytes);
        self.evict_tile_cache_over_budget();
    }

    pub fn next_dirty_tile_batch(&self, limit: usize) -> Vec<CanvasTileId> {
        if self.publication.blocks_dirty_tile_batch() {
            return Vec::new();
        }
        self.visibility.next_dirty_tile_batch(limit)
    }

    pub fn mark_dirty_tiles(&mut self, tiles: impl IntoIterator<Item = CanvasTileId>) {
        self.visibility.mark_dirty_tiles(tiles);
    }

    pub fn invalidate_after_document_change(&mut self) {
        self.visibility.invalidate_after_document_change();
        self.publication.invalidate_after_document_change();
    }

    pub fn record_preview_products(
        &mut self,
        products: Vec<DrawingInkTileProduct>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        let products_to_cache = self.preview.record_preview_products(products, diagnostics);
        self.record_cached_products(products_to_cache.iter());
    }

    pub fn replace_preview_products_for_tiles(
        &mut self,
        tile_ids: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        let products_to_cache = self.preview.replace_preview_products_for_tiles(
            tile_ids,
            products,
            cleared_tiles,
            diagnostics,
        );
        self.record_cached_products(products_to_cache.iter());
    }

    pub fn clear_preview_products(&mut self) {
        self.preview.clear_preview_products();
        self.evict_tile_cache_over_budget();
    }

    pub fn record_cached_products<'a>(
        &mut self,
        products: impl IntoIterator<Item = &'a DrawingInkTileProduct>,
    ) {
        self.cache.record_cached_products(products);
        self.evict_tile_cache_over_budget();
    }

    pub fn record_preview_failure(&mut self, diagnostics: Vec<DrawingTileFormationDiagnostic>) {
        self.preview.record_preview_failure(diagnostics);
        self.evict_tile_cache_over_budget();
    }

    pub fn record_candidate_products(
        &mut self,
        formation_key: String,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        self.publication.record_candidate_products(
            formation_key,
            dirty_tiles,
            products,
            cleared_tiles,
        );
        self.preview.set_diagnostics(diagnostics);
    }

    pub fn record_accepted_clear_generation(
        &mut self,
        formation_key: String,
        dirty_tiles: impl IntoIterator<Item = CanvasTileId>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
        self.publication
            .record_accepted_clear_generation(formation_key);
        self.visibility
            .record_accepted_clear_generation(dirty_tiles, cleared_tiles);
        self.preview.clear_preview_products();
        self.preview.set_diagnostics(diagnostics);
        self.evict_tile_cache_over_budget();
    }

    pub fn record_failed_generation(
        &mut self,
        formation_key: String,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
        clear_preview_products: bool,
    ) {
        self.publication.record_failed_generation(formation_key);
        if clear_preview_products {
            self.preview.clear_preview_products_preserving_dirty_count();
        }
        self.preview.set_diagnostics(diagnostics);
        self.evict_tile_cache_over_budget();
    }

    pub fn record_published_descriptors(
        &mut self,
        publication_key: String,
        descriptors: Vec<ProductDescriptorCore>,
    ) {
        self.publication
            .record_published_descriptors(publication_key, descriptors);
    }

    pub fn record_pending_formation_job(&mut self, formation_key: String) {
        self.publication.record_pending_formation_job(formation_key);
    }

    pub fn clear_pending_formation_job(&mut self, formation_key: &str) {
        self.publication.clear_pending_formation_job(formation_key);
    }

    pub fn record_accepted_snapshots(
        &mut self,
        query_snapshot_key: String,
        accepted_snapshot_ids: impl IntoIterator<Item = ProductIdentity>,
        clear_preview_products: bool,
    ) -> bool {
        let fully_accepted = self
            .publication
            .record_accepted_snapshots(query_snapshot_key, accepted_snapshot_ids);
        if fully_accepted {
            self.visibility.promote_accepted_snapshots(
                self.publication.formed_products(),
                self.publication.candidate_cleared_tiles(),
                self.publication.candidate_dirty_tiles(),
            );
            self.publication.clear_accepted_candidate();
            if clear_preview_products {
                self.preview.clear_preview_products_preserving_dirty_count();
            }
            self.evict_tile_cache_over_budget();
        }
        fully_accepted
    }

    pub fn record_journal(
        &mut self,
        stage: DrawingInkJournalStage,
        accepted: bool,
        summary: impl Into<String>,
    ) {
        self.journal.record_journal(stage, accepted, summary);
    }

    fn evict_tile_cache_over_budget(&mut self) {
        let protected_keys = self.protected_cache_keys();
        self.cache.evict_tile_cache_over_budget(&protected_keys);
    }

    fn protected_cache_keys(&self) -> BTreeSet<ProductCacheKey> {
        self.visibility
            .visible_products()
            .chain(self.preview.preview_products().iter())
            .chain(self.publication.formed_products().iter())
            .map(|product| drawing_ink_tile_product_cache_identity(product).cache_key())
            .collect()
    }
}
