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
            ViewportSurfaceBinding::dynamic_texture("viewport", "target.primary"),
        );
        registry.bind(
            7,
            overlay_slot,
            ViewportSurfaceBinding::dynamic_texture("viewport", "target.overlay"),
        );

        let primary_binding = registry
            .get(7, primary_slot)
            .expect("primary slot binding should exist");
        let ViewportSurfaceBindingSource::DynamicTexture {
            namespace,
            target_id,
        } = &primary_binding.source;
        assert_eq!(namespace, "viewport");
        assert_eq!(target_id, "target.primary");

        let overlay_binding = registry
            .get(7, overlay_slot)
            .expect("overlay slot binding should exist");
        let ViewportSurfaceBindingSource::DynamicTexture {
            namespace,
            target_id,
        } = &overlay_binding.source;
        assert_eq!(namespace, "viewport");
        assert_eq!(target_id, "target.overlay");
    }
}
