//! Runtime resources for the drawing app shell.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use drawing::{CanvasTileId, DrawingInkTileProduct, ProductQualityClass};
use runen_input::{ContactId, InputContext, ToolId};

use crate::app::{DrawingInkSurfaceKind, RunenwerkDrawApp};

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct DrawingHostResource {
    pub app: RunenwerkDrawApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeInputStreamKey {
    pub context: InputContext,
    pub tool: Option<ToolId>,
    pub contact: ContactId,
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct NativeClaimStateResource {
    streams: HashMap<NativeInputStreamKey, u32>,
    suppress_fallback_this_frame: HashSet<NativeInputStreamKey>,
    native_activity_this_frame: bool,
}

impl NativeClaimStateResource {
    pub fn begin_frame(&mut self) {
        self.suppress_fallback_this_frame.clear();
        self.native_activity_this_frame = false;
        for age in self.streams.values_mut() {
            *age = age.saturating_add(1);
        }
        self.streams.retain(|_, age| {
            *age <= crate::runtime::systems::NATIVE_CONTACT_FALLBACK_SUPPRESSION_IDLE_FRAME_LIMIT
        });
    }

    pub fn observe_contact(&mut self, key: NativeInputStreamKey) {
        self.streams.insert(key, 0);
    }

    pub fn observe_terminal(&mut self, key: NativeInputStreamKey) {
        self.streams.remove(&key);
        self.suppress_fallback_this_frame.insert(key);
    }

    pub fn observe_current_frame_activity(&mut self) {
        self.native_activity_this_frame = true;
    }

    pub fn has_active_stream(&self) -> bool {
        !self.streams.is_empty()
    }

    pub fn suppresses_fallback_this_frame(&self) -> bool {
        !self.suppress_fallback_this_frame.is_empty()
    }

    pub fn has_current_frame_activity(&self) -> bool {
        self.native_activity_this_frame
    }
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct DrawingInkUploadTrackerResource {
    committed_generations: BTreeMap<DrawingInkUploadKey, u64>,
    preview_generations: BTreeMap<DrawingInkUploadKey, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DrawingInkUploadKey {
    quality_class: ProductQualityClass,
    tile_id: CanvasTileId,
}

impl DrawingInkUploadKey {
    fn from_product(product: &DrawingInkTileProduct) -> Self {
        Self {
            quality_class: product.metadata.quality_class,
            tile_id: product.metadata.tile_id,
        }
    }
}

impl DrawingInkUploadTrackerResource {
    pub fn retain_products(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
        products: &[DrawingInkTileProduct],
    ) {
        let retained_tile_ids = products
            .iter()
            .map(DrawingInkUploadKey::from_product)
            .collect::<BTreeSet<_>>();
        self.generations_mut(surface_kind)
            .retain(|key, _| retained_tile_ids.contains(key));
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
                generations
                    .get(&DrawingInkUploadKey::from_product(product))
                    .copied()
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
            generations.insert(
                DrawingInkUploadKey::from_product(product),
                product.descriptor_generation,
            );
        }
    }

    fn generations(
        &self,
        surface_kind: DrawingInkSurfaceKind,
    ) -> &BTreeMap<DrawingInkUploadKey, u64> {
        match surface_kind {
            DrawingInkSurfaceKind::Committed | DrawingInkSurfaceKind::GpuCommitted => {
                &self.committed_generations
            }
            DrawingInkSurfaceKind::Preview | DrawingInkSurfaceKind::GpuPreview => {
                &self.preview_generations
            }
        }
    }

    fn generations_mut(
        &mut self,
        surface_kind: DrawingInkSurfaceKind,
    ) -> &mut BTreeMap<DrawingInkUploadKey, u64> {
        match surface_kind {
            DrawingInkSurfaceKind::Committed | DrawingInkSurfaceKind::GpuCommitted => {
                &mut self.committed_generations
            }
            DrawingInkSurfaceKind::Preview | DrawingInkSurfaceKind::GpuPreview => {
                &mut self.preview_generations
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runen_input::{InputDeviceId, InputSourceId};

    fn stream(contact: u64) -> NativeInputStreamKey {
        NativeInputStreamKey {
            context: InputContext::new(InputSourceId::new(3), Some(InputDeviceId::new(7))),
            tool: Some(ToolId::new(1)),
            contact: ContactId::new(contact),
        }
    }

    #[test]
    fn terminal_contact_release_does_not_release_another_contact() {
        let first = stream(1);
        let second = stream(2);
        let mut claims = NativeClaimStateResource::default();
        claims.observe_contact(first);
        claims.observe_contact(second);
        claims.observe_terminal(first);

        assert!(!claims.streams.contains_key(&first));
        assert!(claims.streams.contains_key(&second));
        assert!(claims.suppresses_fallback_this_frame());
    }

    #[test]
    fn unrelated_native_traffic_does_not_refresh_another_stream() {
        let stale = stream(1);
        let unrelated = stream(2);
        let mut claims = NativeClaimStateResource::default();
        claims.observe_contact(stale);

        for _ in 0..crate::runtime::systems::NATIVE_CONTACT_FALLBACK_SUPPRESSION_IDLE_FRAME_LIMIT {
            claims.begin_frame();
            claims.observe_contact(unrelated);
        }
        assert!(claims.streams.contains_key(&stale));

        claims.begin_frame();
        assert!(!claims.streams.contains_key(&stale));
        assert!(claims.streams.contains_key(&unrelated));
    }
}
