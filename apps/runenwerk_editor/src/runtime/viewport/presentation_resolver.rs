//! File: apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs
//! Purpose: Resolve viewport presentation selection into producer-owned surfaces.

use editor_shell::viewport_embed_slot_for;
use editor_viewport::{ExpressionProductId, ViewportId, ViewportSurfacePresentationSlot};
use ui_render_data::{ViewportSurfaceBinding, ViewportSurfaceBindingRegistry};

use crate::runtime::viewport::{
    ATLAS_PRODUCT_ID, BRICKMAP_DEBUG_PRODUCT_ID, HISTORY_COLOR_PRODUCT_ID, OVERLAY_PRODUCT_ID,
    PICKING_IDS_PRODUCT_ID, SCALAR_FIELD_PRODUCT_ID, SCENE_COLOR_PRODUCT_ID,
    VECTOR_FIELD_PRODUCT_ID, VOLUME_SLICE_PRODUCT_ID, ViewportPresentationStateResource,
    ViewportProductTargetRegistryResource, ViewportSurfaceSet, ViewportSurfaceSetResource,
    ViewportSurfaceSlot,
};

pub fn resolve_product_to_surface_slot(
    product_id: ExpressionProductId,
) -> Option<ViewportSurfaceSlot> {
    if product_id == SCENE_COLOR_PRODUCT_ID {
        Some(ViewportSurfaceSlot::PrimaryColor)
    } else if product_id == PICKING_IDS_PRODUCT_ID {
        Some(ViewportSurfaceSlot::PickingIds)
    } else if product_id == OVERLAY_PRODUCT_ID {
        Some(ViewportSurfaceSlot::Overlay)
    } else if product_id == SCALAR_FIELD_PRODUCT_ID {
        Some(ViewportSurfaceSlot::ScalarField)
    } else if product_id == VECTOR_FIELD_PRODUCT_ID {
        Some(ViewportSurfaceSlot::VectorField)
    } else if product_id == ATLAS_PRODUCT_ID {
        Some(ViewportSurfaceSlot::Atlas)
    } else if product_id == VOLUME_SLICE_PRODUCT_ID {
        Some(ViewportSurfaceSlot::VolumeSlice)
    } else if product_id == BRICKMAP_DEBUG_PRODUCT_ID {
        Some(ViewportSurfaceSlot::BrickmapDebug)
    } else if product_id == HISTORY_COLOR_PRODUCT_ID {
        Some(ViewportSurfaceSlot::HistoryColor)
    } else {
        None
    }
}

fn bind_surface_slot(
    registry: &mut ViewportSurfaceBindingRegistry,
    viewport_id: ViewportId,
    surface_set: &ViewportSurfaceSet,
    source_slot: ViewportSurfaceSlot,
    target_slot: ViewportSurfacePresentationSlot,
) {
    let Some(surface_handle) = surface_set.get(source_slot) else {
        return;
    };
    if !surface_handle.is_ui_sampleable() {
        return;
    }
    registry.bind(
        viewport_id.0,
        viewport_embed_slot_for(target_slot),
        ViewportSurfaceBinding::dynamic_texture(surface_handle.namespace, surface_handle.target_id),
    );
}

pub fn build_surface_binding_registry(
    viewport_surface_sets: &ViewportSurfaceSetResource,
    viewport_presentations: &ViewportPresentationStateResource,
    viewport_product_targets: &ViewportProductTargetRegistryResource,
) -> ViewportSurfaceBindingRegistry {
    let mut registry = ViewportSurfaceBindingRegistry::default();

    for viewport_id in viewport_surface_sets.viewport_ids() {
        let Some(surface_set) = viewport_surface_sets.surface_set(viewport_id) else {
            continue;
        };
        let Some(presentation_state) = viewport_presentations.state_for(viewport_id) else {
            continue;
        };
        if let Some(primary_record) = viewport_product_targets.record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Primary,
            presentation_state.selected_primary_product_id,
        ) {
            bind_surface_slot(
                &mut registry,
                viewport_id,
                surface_set,
                primary_record.surface_slot,
                ViewportSurfacePresentationSlot::Primary,
            );
        }
        if let Some(picking_record) = viewport_product_targets.record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Picking,
            PICKING_IDS_PRODUCT_ID,
        ) {
            bind_surface_slot(
                &mut registry,
                viewport_id,
                surface_set,
                picking_record.surface_slot,
                ViewportSurfacePresentationSlot::Picking,
            );
        }
        if let Some(overlay_record) = viewport_product_targets.record_for_product(
            viewport_id,
            ViewportSurfacePresentationSlot::Overlay,
            OVERLAY_PRODUCT_ID,
        ) {
            bind_surface_slot(
                &mut registry,
                viewport_id,
                surface_set,
                overlay_record.surface_slot,
                ViewportSurfacePresentationSlot::Overlay,
            );
        }
    }

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::viewport::{
        VIEWPORT_DYNAMIC_TARGET_NAMESPACE, ViewportProductTargetRegistryResource,
        ViewportSurfaceHandle, initial_presentation_state, initial_product_descriptors,
        product_target_record_for_descriptor,
    };
    use editor_core::RealityVersion;
    use editor_viewport::ExpressionDimensions;

    fn target_registry_for(viewport_id: ViewportId) -> ViewportProductTargetRegistryResource {
        let descriptors =
            initial_product_descriptors(ExpressionDimensions::new(320, 200), RealityVersion(1));
        let mut registry = ViewportProductTargetRegistryResource::default();
        registry.replace_records(
            descriptors
                .iter()
                .filter_map(|descriptor| {
                    product_target_record_for_descriptor(viewport_id, descriptor)
                })
                .collect(),
        );
        registry
    }

    #[test]
    fn unknown_product_never_resolves_to_surface_slot() {
        assert!(resolve_product_to_surface_slot(ExpressionProductId(999)).is_none());
    }

    #[test]
    fn binding_registry_does_not_bind_non_sampleable_product_as_visual_primary() {
        let viewport_id = ViewportId(10);
        let mut surface_sets = ViewportSurfaceSetResource::default();
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::dynamic_texture(
                VIEWPORT_DYNAMIC_TARGET_NAMESPACE,
                "editor.viewport.10.1.primary",
                true,
            ),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PickingIds,
            ViewportSurfaceHandle::dynamic_texture(
                VIEWPORT_DYNAMIC_TARGET_NAMESPACE,
                "editor.viewport.10.2.picking",
                false,
            ),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceHandle::dynamic_texture(
                VIEWPORT_DYNAMIC_TARGET_NAMESPACE,
                "editor.viewport.10.3.overlay",
                true,
            ),
        );

        let mut presentations = ViewportPresentationStateResource::default();
        let mut state = initial_presentation_state(viewport_id);
        state.select_primary_product(PICKING_IDS_PRODUCT_ID);
        presentations.upsert_state(state);
        let product_targets = target_registry_for(viewport_id);

        let registry =
            build_surface_binding_registry(&surface_sets, &presentations, &product_targets);
        assert!(
            registry
                .get(
                    viewport_id.0,
                    viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
                )
                .is_none(),
            "non-sampleable picking target must not be bound as a normal visual primary surface",
        );
    }

    #[test]
    fn registry_omits_unowned_or_unselected_viewports() {
        let owned_viewport = ViewportId(2);
        let unselected_viewport = ViewportId(3);
        let mut surface_sets = ViewportSurfaceSetResource::default();
        surface_sets.set_surface(
            owned_viewport,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::dynamic_texture(
                VIEWPORT_DYNAMIC_TARGET_NAMESPACE,
                "editor.viewport.2.1.primary",
                true,
            ),
        );
        surface_sets.set_surface(
            unselected_viewport,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::dynamic_texture(
                VIEWPORT_DYNAMIC_TARGET_NAMESPACE,
                "editor.viewport.3.1.primary",
                true,
            ),
        );

        let mut presentations = ViewportPresentationStateResource::default();
        presentations.upsert_state(initial_presentation_state(owned_viewport));
        let product_targets = target_registry_for(owned_viewport);

        let registry =
            build_surface_binding_registry(&surface_sets, &presentations, &product_targets);

        assert!(
            registry
                .get(
                    owned_viewport.0,
                    viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
                )
                .is_some(),
            "owned viewport with presentation state should produce a primary binding",
        );
        assert!(
            registry
                .get(
                    unselected_viewport.0,
                    viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
                )
                .is_none(),
            "viewport without presentation state must not fallback to another viewport",
        );
    }
}
