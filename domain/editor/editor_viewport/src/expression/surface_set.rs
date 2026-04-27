//! File: domain/editor/editor_viewport/src/expression/surface_set.rs
//! Purpose: Viewport-owned presentation surface-set contracts.

use std::collections::BTreeMap;

use crate::ViewportId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewportSurfaceSlot {
    PrimaryColor,
    PickingIds,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewportSurfacePresentationSlot {
    Primary,
    Picking,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewportSurfaceBindingId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportSurfaceDescriptor {
    pub slot: ViewportSurfaceSlot,
    pub binding_id: ViewportSurfaceBindingId,
    pub width: u32,
    pub height: u32,
}

impl ViewportSurfaceDescriptor {
    pub fn new(
        slot: ViewportSurfaceSlot,
        binding_id: ViewportSurfaceBindingId,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            slot,
            binding_id,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportSurfaceSetDescriptor {
    pub viewport_id: ViewportId,
    pub surfaces: BTreeMap<ViewportSurfaceSlot, ViewportSurfaceDescriptor>,
}

impl ViewportSurfaceSetDescriptor {
    pub fn new(viewport_id: ViewportId) -> Self {
        Self {
            viewport_id,
            surfaces: BTreeMap::new(),
        }
    }

    pub fn upsert_surface(&mut self, descriptor: ViewportSurfaceDescriptor) {
        self.surfaces.insert(descriptor.slot, descriptor);
    }

    pub fn surface(&self, slot: ViewportSurfaceSlot) -> Option<&ViewportSurfaceDescriptor> {
        self.surfaces.get(&slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_set_is_viewport_scoped_and_slot_keyed() {
        let mut set = ViewportSurfaceSetDescriptor::new(ViewportId(1));
        set.upsert_surface(ViewportSurfaceDescriptor::new(
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceBindingId(101),
            640,
            480,
        ));

        assert_eq!(set.viewport_id, ViewportId(1));
        assert_eq!(set.surfaces.len(), 1);
        assert_eq!(
            set.surface(ViewportSurfaceSlot::PrimaryColor)
                .map(|surface| surface.binding_id),
            Some(ViewportSurfaceBindingId(101))
        );
    }
}
