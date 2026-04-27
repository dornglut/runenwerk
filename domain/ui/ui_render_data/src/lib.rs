//! File: domain/ui/ui_render_data/src/lib.rs
//! Crate: ui_render_data

pub mod batching;
pub mod colors;
pub mod frame;
pub mod primitives;

pub use batching::*;
pub use colors::*;
pub use frame::*;
pub use primitives::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_surface_binding_registry_is_slot_scoped() {
        let mut registry = ViewportSurfaceBindingRegistry::default();
        let primary_slot = ViewportSurfaceEmbedSlotId::new(1);
        let overlay_slot = ViewportSurfaceEmbedSlotId::new(3);
        registry.bind(
            7,
            primary_slot,
            ViewportSurfaceBinding::new("flow.a", "resource.primary"),
        );
        registry.bind(
            7,
            overlay_slot,
            ViewportSurfaceBinding::new("flow.a", "resource.overlay"),
        );

        assert_eq!(
            registry
                .get(7, primary_slot)
                .map(|binding| binding.resource_id.as_str()),
            Some("resource.primary")
        );
        assert_eq!(
            registry
                .get(7, overlay_slot)
                .map(|binding| binding.resource_id.as_str()),
            Some("resource.overlay")
        );
    }
}
