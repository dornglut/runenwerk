//! Runtime resources for the drawing app shell.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{CanvasTileId, DrawingInkTileProduct};

use crate::app::{DrawingInkSurfaceKind, RunenwerkDrawApp};

#[derive(Debug, Default, ecs::Resource)]
pub struct DrawingHostResource {
    pub app: RunenwerkDrawApp,
}

#[derive(Debug, Default, ecs::Resource)]
pub struct DrawingInkUploadTrackerResource {
    committed_generations: BTreeMap<CanvasTileId, u64>,
    preview_generations: BTreeMap<CanvasTileId, u64>,
}

impl DrawingInkUploadTrackerResource {
    pub fn retain_products(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        products: &[DrawingInkTileProduct],
    ) {
        let retained_tile_ids = products
            .iter()
            .map(|product| product.metadata.tile_id)
            .collect::<BTreeSet<_>>();
        self.generations_mut(surface_kind)
            .retain(|tile_id, _| retained_tile_ids.contains(tile_id));
    }

    pub fn products_requiring_upload<'a>(
        &self,
        surface_kind: DrawingInkSurfaceKind,
        products: &'a [DrawingInkTileProduct],
    ) -> Vec<&'a DrawingInkTileProduct> {
        let generations = self.generations(surface_kind);
        products
            .iter()
            .filter(|product| {
                generations.get(&product.metadata.tile_id).copied()
                    != Some(product.descriptor_generation)
            })
            .collect()
    }

    pub fn record_submitted_uploads<'a>(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        products: impl IntoIterator<Item = &'a DrawingInkTileProduct>,
    ) {
        let generations = self.generations_mut(surface_kind);
        for product in products {
            generations.insert(product.metadata.tile_id, product.descriptor_generation);
        }
    }

    fn generations(&self, surface_kind: DrawingInkSurfaceKind) -> &BTreeMap<CanvasTileId, u64> {
        match surface_kind {
            DrawingInkSurfaceKind::Committed => &self.committed_generations,
            DrawingInkSurfaceKind::Preview => &self.preview_generations,
        }
    }

    fn generations_mut(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
    ) -> &mut BTreeMap<CanvasTileId, u64> {
        match surface_kind {
            DrawingInkSurfaceKind::Committed => &mut self.committed_generations,
            DrawingInkSurfaceKind::Preview => &mut self.preview_generations,
        }
    }
}
