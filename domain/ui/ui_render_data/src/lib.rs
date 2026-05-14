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

    #[test]
    fn stroke_primitive_constructs_and_exports_as_ui_primitive() {
        let primitive = StrokePrimitive::new(
            [
                ui_math::UiPoint::new(1.0, 2.0),
                ui_math::UiPoint::new(5.0, 7.0),
            ],
            3.0,
            UiPaint::rgba(0.1, 0.2, 0.3, 1.0),
            UiDrawKey::new(9, None),
            UiSortKey::new(0, 2, 4),
        )
        .with_clip(ui_math::UiRect::new(0.0, 0.0, 10.0, 10.0));

        let UiPrimitive::Stroke(stroke) = UiPrimitive::from(primitive) else {
            panic!("stroke primitive should convert into UiPrimitive::Stroke");
        };
        assert_eq!(stroke.points.len(), 2);
        assert_eq!(stroke.width, 3.0);
        assert_eq!(stroke.draw_key.material_id, 9);
        assert!(stroke.clip.is_some());
    }
}
