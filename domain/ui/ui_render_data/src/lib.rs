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
        registry.bind(
            7,
            ViewportSurfaceSlot::Primary,
            ViewportSurfaceBinding::new("flow.a", "resource.primary"),
        );
        registry.bind(
            7,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceBinding::new("flow.a", "resource.overlay"),
        );

        assert_eq!(
            registry
                .get(7, ViewportSurfaceSlot::Primary)
                .map(|binding| binding.resource_id.as_str()),
            Some("resource.primary")
        );
        assert_eq!(
            registry
                .get(7, ViewportSurfaceSlot::Overlay)
                .map(|binding| binding.resource_id.as_str()),
            Some("resource.overlay")
        );
    }
}
