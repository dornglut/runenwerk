//! File: domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs
//! Purpose: Viewport panel surface embed primitive.

use std::collections::BTreeMap;

use ui_math::UiRect;

use crate::{UiPaint, UiSortKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewportSurfaceSlot {
    Primary,
    Picking,
    Overlay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportSurfaceEmbedPrimitive {
    pub viewport_id: u64,
    pub slot: ViewportSurfaceSlot,
    pub rect: UiRect,
    pub uv_rect: UiRect,
    pub tint: UiPaint,
    pub sort_key: UiSortKey,
}

impl ViewportSurfaceEmbedPrimitive {
    pub fn new(
        viewport_id: u64,
        slot: ViewportSurfaceSlot,
        rect: UiRect,
        uv_rect: UiRect,
        tint: UiPaint,
        sort_key: UiSortKey,
    ) -> Self {
        Self {
            viewport_id,
            slot,
            rect,
            uv_rect,
            tint,
            sort_key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportSurfaceBinding {
    pub flow_id: String,
    pub resource_id: String,
}

impl ViewportSurfaceBinding {
    pub fn new(flow_id: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            flow_id: flow_id.into(),
            resource_id: resource_id.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewportSurfaceBindingRegistry {
    bindings: BTreeMap<(u64, ViewportSurfaceSlot), ViewportSurfaceBinding>,
}

impl ViewportSurfaceBindingRegistry {
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn bind(
        &mut self,
        viewport_id: u64,
        slot: ViewportSurfaceSlot,
        binding: ViewportSurfaceBinding,
    ) {
        self.bindings.insert((viewport_id, slot), binding);
    }

    pub fn get(
        &self,
        viewport_id: u64,
        slot: ViewportSurfaceSlot,
    ) -> Option<&ViewportSurfaceBinding> {
        self.bindings.get(&(viewport_id, slot))
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&(u64, ViewportSurfaceSlot), &ViewportSurfaceBinding)> {
        self.bindings.iter()
    }
}
