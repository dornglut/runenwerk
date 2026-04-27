//! File: apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs
//! Purpose: Resolve viewport presentation selection into producer-owned surfaces.

use editor_shell::viewport_embed_slot_for;
use editor_viewport::{ExpressionProductId, ViewportId, ViewportSurfacePresentationSlot};
use ui_render_data::{ViewportSurfaceBinding, ViewportSurfaceBindingRegistry};

use crate::runtime::viewport::{
    PRODUCT_ID_OVERLAY, PRODUCT_ID_PICKING_IDS, PRODUCT_ID_SCENE_COLOR,
    ViewportPresentationStateResource, ViewportSurfaceSet, ViewportSurfaceSetResource,
    ViewportSurfaceSlot,
};

pub fn resolve_product_to_surface_slot(
    product_id: ExpressionProductId,
) -> Option<ViewportSurfaceSlot> {
    if product_id == PRODUCT_ID_SCENE_COLOR {
        Some(ViewportSurfaceSlot::PrimaryColor)
    } else if product_id == PRODUCT_ID_PICKING_IDS {
        Some(ViewportSurfaceSlot::PickingIds)
    } else if product_id == PRODUCT_ID_OVERLAY {
        Some(ViewportSurfaceSlot::Overlay)
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
    registry.bind(
        viewport_id.0,
        viewport_embed_slot_for(target_slot),
        ViewportSurfaceBinding::new(surface_handle.flow_id, surface_handle.resource_id),
    );
}

pub fn build_surface_binding_registry(
    viewport_surface_sets: &ViewportSurfaceSetResource,
    viewport_presentations: &ViewportPresentationStateResource,
) -> ViewportSurfaceBindingRegistry {
    let mut registry = ViewportSurfaceBindingRegistry::default();

    for viewport_id in viewport_surface_sets.viewport_ids() {
        let Some(surface_set) = viewport_surface_sets.surface_set(viewport_id) else {
            continue;
        };
        let Some(presentation_state) = viewport_presentations.state_for(viewport_id) else {
            continue;
        };
        let Some(primary_slot) =
            resolve_product_to_surface_slot(presentation_state.selected_primary_product_id)
        else {
            continue;
        };

        bind_surface_slot(
            &mut registry,
            viewport_id,
            surface_set,
            primary_slot,
            ViewportSurfacePresentationSlot::Primary,
        );
        bind_surface_slot(
            &mut registry,
            viewport_id,
            surface_set,
            ViewportSurfaceSlot::PickingIds,
            ViewportSurfacePresentationSlot::Picking,
        );
        bind_surface_slot(
            &mut registry,
            viewport_id,
            surface_set,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfacePresentationSlot::Overlay,
        );
    }

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::viewport::{
        EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_OVERLAY, VIEWPORT_RESOURCE_PICKING_IDS,
        VIEWPORT_RESOURCE_SCENE_COLOR, ViewportSurfaceHandle, initial_presentation_state,
    };

    #[test]
    fn unknown_product_never_resolves_to_surface_slot() {
        assert!(resolve_product_to_surface_slot(ExpressionProductId(999)).is_none());
    }

    #[test]
    fn binding_registry_primary_slot_tracks_selected_product_by_viewport() {
        let viewport_id = ViewportId(10);
        let mut surface_sets = ViewportSurfaceSetResource::default();
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::PickingIds,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_PICKING_IDS),
        );
        surface_sets.set_surface(
            viewport_id,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_OVERLAY),
        );

        let mut presentations = ViewportPresentationStateResource::default();
        let mut state = initial_presentation_state(viewport_id);
        state.select_primary_product(PRODUCT_ID_PICKING_IDS);
        presentations.upsert_state(state);

        let registry = build_surface_binding_registry(&surface_sets, &presentations);
        let binding = registry
            .get(
                viewport_id.0,
                viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
            )
            .expect("primary binding should exist for selected product");

        assert_eq!(binding.flow_id.as_str(), EDITOR_MAIN_FLOW_ID);
        assert_eq!(binding.resource_id.as_str(), VIEWPORT_RESOURCE_PICKING_IDS);
    }

    #[test]
    fn registry_omits_unowned_or_unselected_viewports() {
        let owned_viewport = ViewportId(2);
        let unselected_viewport = ViewportId(3);
        let mut surface_sets = ViewportSurfaceSetResource::default();
        surface_sets.set_surface(
            owned_viewport,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR),
        );
        surface_sets.set_surface(
            unselected_viewport,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR),
        );

        let mut presentations = ViewportPresentationStateResource::default();
        presentations.upsert_state(initial_presentation_state(owned_viewport));

        let registry = build_surface_binding_registry(&surface_sets, &presentations);

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
