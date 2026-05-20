//! In-memory app-owned ink tile payload cache.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{
    CanvasTileId, DrawingDocumentRevision, DrawingInkTileProduct, FormationVersion,
    ProductQualityClass, drawing_ink_tile_product_cache_identity,
};
use product::ProductCacheKey;

const DEFAULT_DRAWING_INK_TILE_CACHE_BUDGET_BYTES: usize = 512 * 1024 * 1024;

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
pub(crate) struct DrawingInkTileCacheState {
    cached_products: BTreeMap<ProductCacheKey, DrawingInkTileCacheEntry>,
    cached_source_keys: BTreeMap<String, ProductCacheKey>,
    cache_budget_bytes: usize,
    cache_payload_bytes: usize,
    cache_access_frame: u64,
}

impl Default for DrawingInkTileCacheState {
    fn default() -> Self {
        Self {
            cached_products: BTreeMap::new(),
            cached_source_keys: BTreeMap::new(),
            cache_budget_bytes: DEFAULT_DRAWING_INK_TILE_CACHE_BUDGET_BYTES,
            cache_payload_bytes: 0,
            cache_access_frame: 0,
        }
    }
}

impl DrawingInkTileCacheState {
    pub(crate) fn cached_product_key_for_source_key(
        &self,
        source_key: &str,
    ) -> Option<&ProductCacheKey> {
        self.cached_source_keys.get(source_key)
    }

    pub(crate) fn cached_product(&self, key: &ProductCacheKey) -> Option<&DrawingInkTileProduct> {
        self.cached_products.get(key).map(|entry| &entry.product)
    }

    pub(crate) fn cached_product_for_source_key(
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

    pub(crate) fn tile_cache_budget_bytes(&self) -> usize {
        self.cache_budget_bytes
    }

    pub(crate) fn tile_cache_payload_bytes(&self) -> usize {
        self.cache_payload_bytes
    }

    pub(crate) fn tile_cache_entry_count(&self) -> usize {
        self.cached_products.len()
    }

    pub(crate) fn tile_cache_metadata(&self) -> Vec<DrawingInkTileCacheEntryMetadata> {
        self.cached_products
            .values()
            .map(|entry| entry.metadata.clone())
            .collect()
    }

    pub(crate) fn set_tile_cache_budget_bytes(&mut self, budget_bytes: usize) {
        self.cache_budget_bytes = budget_bytes;
    }

    pub(crate) fn record_cached_products<'a>(
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
    }

    pub(crate) fn evict_tile_cache_over_budget(
        &mut self,
        protected_keys: &BTreeSet<ProductCacheKey>,
    ) {
        while self.cache_payload_bytes > self.cache_budget_bytes {
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

    fn next_cache_access_frame(&mut self) -> u64 {
        self.cache_access_frame = self.cache_access_frame.saturating_add(1).max(1);
        self.cache_access_frame
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
