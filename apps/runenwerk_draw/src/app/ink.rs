//! Drawing app-owned ink product publication and visibility state.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{
    CanvasTileId, DrawingDocumentRevision, DrawingInkTileProduct, DrawingTileFormationDiagnostic,
    FormationVersion, ProductQualityClass, drawing_ink_tile_product_cache_identity,
};
use product::{ProductCacheKey, ProductDescriptorCore, ProductIdentity};

const DRAWING_INK_JOURNAL_LIMIT: usize = 64;
const DEFAULT_DRAWING_INK_TILE_CACHE_BUDGET_BYTES: usize = 512 * 1024 * 1024;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingInkTileCacheEntryMetadata {
    pub tile_id: CanvasTileId,
    pub quality_class: ProductQualityClass,
    pub descriptor_generation: u64,
    pub source_revision: DrawingDocumentRevision,
    pub formation_version: FormationVersion,
    pub payload_size_bytes: usize,
    pub last_access_frame: u64,
}

#[derive(Debug, Clone)]
struct DrawingInkTileCacheEntry {
    product: DrawingInkTileProduct,
    metadata: DrawingInkTileCacheEntryMetadata,
}

#[derive(Debug, Clone)]
pub struct DrawingInkRuntimeState {
    visible_products: BTreeMap<CanvasTileId, DrawingInkTileProduct>,
    candidate_products: Vec<DrawingInkTileProduct>,
    candidate_cleared_tiles: BTreeSet<CanvasTileId>,
    candidate_dirty_tiles: BTreeSet<CanvasTileId>,
    preview_products: Vec<DrawingInkTileProduct>,
    cached_products: BTreeMap<ProductCacheKey, DrawingInkTileCacheEntry>,
    cached_source_keys: BTreeMap<String, ProductCacheKey>,
    cache_budget_bytes: usize,
    cache_payload_bytes: usize,
    cache_access_frame: u64,
    dirty_tiles: BTreeSet<CanvasTileId>,
    published_descriptors: Vec<ProductDescriptorCore>,
    accepted_snapshot_ids: BTreeSet<ProductIdentity>,
    last_formation_key: Option<String>,
    last_publication_key: Option<String>,
    last_query_snapshot_key: Option<String>,
    pending_formation_key: Option<String>,
    document_dirty: bool,
    last_preview_dirty_tile_count: usize,
    diagnostics: Vec<DrawingTileFormationDiagnostic>,
    journal: Vec<DrawingInkJournalEntry>,
}

impl Default for DrawingInkRuntimeState {
    fn default() -> Self {
        Self {
            visible_products: BTreeMap::new(),
            candidate_products: Vec::new(),
            candidate_cleared_tiles: BTreeSet::new(),
            candidate_dirty_tiles: BTreeSet::new(),
            preview_products: Vec::new(),
            cached_products: BTreeMap::new(),
            cached_source_keys: BTreeMap::new(),
            cache_budget_bytes: DEFAULT_DRAWING_INK_TILE_CACHE_BUDGET_BYTES,
            cache_payload_bytes: 0,
            cache_access_frame: 0,
            dirty_tiles: BTreeSet::new(),
            published_descriptors: Vec::new(),
            accepted_snapshot_ids: BTreeSet::new(),
            last_formation_key: None,
            last_publication_key: None,
            last_query_snapshot_key: None,
            pending_formation_key: None,
            document_dirty: false,
            last_preview_dirty_tile_count: 0,
            diagnostics: Vec::new(),
            journal: Vec::new(),
        }
    }
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

    pub fn cached_product_key_for_source_key(&self, source_key: &str) -> Option<&ProductCacheKey> {
        self.cached_source_keys.get(source_key)
    }

    pub fn cached_product(&self, key: &ProductCacheKey) -> Option<&DrawingInkTileProduct> {
        self.cached_products.get(key).map(|entry| &entry.product)
    }

    pub fn cached_product_for_source_key(
        &mut self,
        source_key: &str,
    ) -> Option<(ProductCacheKey, DrawingInkTileProduct)> {
        let product_key = match self.cached_source_keys.get(source_key) {
            Some(product_key) => product_key.clone(),
            None => return None,
        };
        let access_frame = self.next_cache_access_frame();
        let Some(entry) = self.cached_products.get_mut(&product_key) else {
            self.cached_source_keys.remove(source_key);
            return None;
        };
        entry.metadata.last_access_frame = access_frame;
        Some((product_key, entry.product.clone()))
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

    pub fn pending_formation_key(&self) -> Option<&str> {
        self.pending_formation_key.as_deref()
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

    pub fn last_preview_dirty_tile_count(&self) -> usize {
        self.last_preview_dirty_tile_count
    }

    pub fn tile_cache_budget_bytes(&self) -> usize {
        self.cache_budget_bytes
    }

    pub fn tile_cache_payload_bytes(&self) -> usize {
        self.cache_payload_bytes
    }

    pub fn tile_cache_entry_count(&self) -> usize {
        self.cached_products.len()
    }

    pub fn tile_cache_metadata(&self) -> Vec<DrawingInkTileCacheEntryMetadata> {
        self.cached_products
            .values()
            .map(|entry| entry.metadata.clone())
            .collect()
    }

    pub fn set_tile_cache_budget_bytes(&mut self, budget_bytes: usize) {
        self.cache_budget_bytes = budget_bytes;
        self.evict_tile_cache_over_budget();
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
        self.pending_formation_key = None;
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
        self.last_preview_dirty_tile_count = products.len();
        self.preview_products = products;
        self.diagnostics = diagnostics;
        let products = self.preview_products.clone();
        self.record_cached_products(products.iter());
    }

    pub fn replace_preview_products_for_tiles(
        &mut self,
        tile_ids: impl IntoIterator<Item = CanvasTileId>,
        products: Vec<DrawingInkTileProduct>,
        cleared_tiles: impl IntoIterator<Item = CanvasTileId>,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
    ) {
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
        self.record_cached_products(products_to_cache.iter());
    }

    pub fn clear_preview_products(&mut self) {
        self.preview_products.clear();
        self.last_preview_dirty_tile_count = 0;
        self.evict_tile_cache_over_budget();
    }

    pub fn record_cached_products<'a>(
        &mut self,
        products: impl IntoIterator<Item = &'a DrawingInkTileProduct>,
    ) {
        for product in products {
            let cache_key = drawing_ink_tile_product_cache_identity(product).cache_key();
            let access_frame = self.next_cache_access_frame();
            let payload_size_bytes = product.payload.byte_len();
            if let Some(existing) = self.cached_products.get(&cache_key) {
                self.cache_payload_bytes = self
                    .cache_payload_bytes
                    .saturating_sub(existing.metadata.payload_size_bytes);
            }
            let entry = DrawingInkTileCacheEntry {
                product: product.clone(),
                metadata: DrawingInkTileCacheEntryMetadata {
                    tile_id: product.metadata.tile_id,
                    quality_class: product.metadata.quality_class,
                    descriptor_generation: product.descriptor_generation,
                    source_revision: product.metadata.source_document_revision,
                    formation_version: product.metadata.formation_version,
                    payload_size_bytes,
                    last_access_frame: access_frame,
                },
            };
            self.cached_source_keys
                .insert(product.cache_key.clone(), cache_key.clone());
            self.cached_products.insert(cache_key, entry);
            self.cache_payload_bytes = self.cache_payload_bytes.saturating_add(payload_size_bytes);
        }
        self.evict_tile_cache_over_budget();
    }

    pub fn record_preview_failure(&mut self, diagnostics: Vec<DrawingTileFormationDiagnostic>) {
        self.preview_products.clear();
        self.last_preview_dirty_tile_count = 0;
        self.diagnostics = diagnostics;
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
        self.pending_formation_key = None;
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
        self.pending_formation_key = None;
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
        self.last_preview_dirty_tile_count = 0;
        self.published_descriptors.clear();
        self.accepted_snapshot_ids.clear();
        self.last_formation_key = Some(formation_key);
        self.last_publication_key = None;
        self.last_query_snapshot_key = None;
        self.diagnostics = diagnostics;
        self.document_dirty = !self.dirty_tiles.is_empty();
        self.evict_tile_cache_over_budget();
    }

    pub fn record_failed_generation(
        &mut self,
        formation_key: String,
        diagnostics: Vec<DrawingTileFormationDiagnostic>,
        clear_preview_products: bool,
    ) {
        self.pending_formation_key = None;
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
        self.evict_tile_cache_over_budget();
    }

    pub fn record_published_descriptors(
        &mut self,
        publication_key: String,
        descriptors: Vec<ProductDescriptorCore>,
    ) {
        self.last_publication_key = Some(publication_key);
        self.published_descriptors = descriptors;
    }

    pub fn record_pending_formation_job(&mut self, formation_key: String) {
        self.pending_formation_key = Some(formation_key);
    }

    pub fn clear_pending_formation_job(&mut self, formation_key: &str) {
        if self.pending_formation_key.as_deref() == Some(formation_key) {
            self.pending_formation_key = None;
        }
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

    fn next_cache_access_frame(&mut self) -> u64 {
        self.cache_access_frame = self.cache_access_frame.saturating_add(1).max(1);
        self.cache_access_frame
    }

    fn evict_tile_cache_over_budget(&mut self) {
        while self.cache_payload_bytes > self.cache_budget_bytes {
            let protected_keys = self.protected_cache_keys();
            let Some(evicted_key) = self
                .cached_products
                .iter()
                .filter(|(key, _)| !protected_keys.contains(*key))
                .min_by_key(|(_, entry)| {
                    (
                        entry.metadata.last_access_frame,
                        entry.metadata.tile_id.level.raw(),
                        entry.metadata.tile_id.x,
                        entry.metadata.tile_id.y,
                        entry.metadata.descriptor_generation,
                    )
                })
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.remove_cached_product(&evicted_key);
        }
    }

    fn protected_cache_keys(&self) -> BTreeSet<ProductCacheKey> {
        self.visible_products
            .values()
            .chain(self.preview_products.iter())
            .chain(self.candidate_products.iter())
            .map(|product| drawing_ink_tile_product_cache_identity(product).cache_key())
            .collect()
    }

    fn remove_cached_product(&mut self, key: &ProductCacheKey) {
        let Some(entry) = self.cached_products.remove(key) else {
            return;
        };
        self.cache_payload_bytes = self
            .cache_payload_bytes
            .saturating_sub(entry.metadata.payload_size_bytes);
        if self
            .cached_source_keys
            .get(&entry.product.cache_key)
            .is_some_and(|cached_key| cached_key == key)
        {
            self.cached_source_keys.remove(&entry.product.cache_key);
        }
    }
}
