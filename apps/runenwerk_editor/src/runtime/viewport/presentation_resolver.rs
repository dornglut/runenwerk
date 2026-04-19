//! File: apps/runenwerk_editor/src/runtime/viewport/presentation_resolver.rs
//! Purpose: Resolve viewport presentation selection into producer-owned surfaces.

use editor_viewport::{ExpressionProductId, ViewportPresentationState};
use ui_render_data::{ViewportSurfaceBinding, ViewportSurfaceBindingRegistry, ViewportSurfaceSlot};

use crate::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, PRODUCT_ID_OVERLAY, PRODUCT_ID_PICKING_IDS, PRODUCT_ID_SCENE_COLOR,
    VIEWPORT_RESOURCE_OVERLAY, VIEWPORT_RESOURCE_PICKING_IDS, VIEWPORT_RESOURCE_SCENE_COLOR,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedViewportSurfaceBinding {
    pub flow_id: &'static str,
    pub resource_id: &'static str,
}

pub fn default_presentation_state() -> ViewportPresentationState {
    ViewportPresentationState::new(
        crate::runtime::viewport::MAIN_VIEWPORT_ID,
        PRODUCT_ID_SCENE_COLOR,
    )
}

pub fn resolve_primary_binding(
    state: &ViewportPresentationState,
) -> Option<ResolvedViewportSurfaceBinding> {
    resolve_product_to_surface_binding(state.selected_primary_product_id)
}

pub fn resolve_product_to_surface_binding(
    product_id: ExpressionProductId,
) -> Option<ResolvedViewportSurfaceBinding> {
    let resource_id = if product_id == PRODUCT_ID_SCENE_COLOR {
        VIEWPORT_RESOURCE_SCENE_COLOR
    } else if product_id == PRODUCT_ID_PICKING_IDS {
        VIEWPORT_RESOURCE_PICKING_IDS
    } else if product_id == PRODUCT_ID_OVERLAY {
        VIEWPORT_RESOURCE_OVERLAY
    } else {
        return None;
    };

    Some(ResolvedViewportSurfaceBinding {
        flow_id: EDITOR_MAIN_FLOW_ID,
        resource_id,
    })
}

pub fn build_surface_binding_registry(
    state: &ViewportPresentationState,
) -> ViewportSurfaceBindingRegistry {
    let mut registry = ViewportSurfaceBindingRegistry::default();

    if let Some(primary_binding) = resolve_primary_binding(state) {
        registry.bind(
            state.viewport_id.0,
            ViewportSurfaceSlot::Primary,
            ViewportSurfaceBinding::new(primary_binding.flow_id, primary_binding.resource_id),
        );
    }

    if let Some(picking_binding) = resolve_product_to_surface_binding(PRODUCT_ID_PICKING_IDS) {
        registry.bind(
            state.viewport_id.0,
            ViewportSurfaceSlot::Picking,
            ViewportSurfaceBinding::new(picking_binding.flow_id, picking_binding.resource_id),
        );
    }

    if let Some(overlay_binding) = resolve_product_to_surface_binding(PRODUCT_ID_OVERLAY) {
        registry.bind(
            state.viewport_id.0,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceBinding::new(overlay_binding.flow_id, overlay_binding.resource_id),
        );
    }

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_selects_scene_color_product() {
        let state = default_presentation_state();

        assert_eq!(state.selected_primary_product_id, PRODUCT_ID_SCENE_COLOR);
    }

    #[test]
    fn unknown_product_never_resolves_to_surface_binding() {
        assert!(resolve_product_to_surface_binding(ExpressionProductId(999)).is_none());
    }

    #[test]
    fn binding_registry_primary_slot_tracks_selected_product() {
        let mut state = default_presentation_state();
        state.select_primary_product(PRODUCT_ID_PICKING_IDS);

        let registry = build_surface_binding_registry(&state);
        let binding = registry
            .get(state.viewport_id.0, ViewportSurfaceSlot::Primary)
            .expect("primary binding should exist for selected product");

        assert_eq!(binding.flow_id.as_str(), EDITOR_MAIN_FLOW_ID);
        assert_eq!(binding.resource_id.as_str(), VIEWPORT_RESOURCE_PICKING_IDS);
    }
}
