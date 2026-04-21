use std::path::PathBuf;

use editor_core::RealityVersion;
use editor_viewport::{
    ArtifactObservationFrame, ExpressionDimensions, ProducerHealth, ProductAvailabilityState,
};
use engine::plugins::render::ShaderRegistryResource;
use engine::runtime::ResMut;

use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    EDITOR_MAIN_FLOW_ID, MAIN_VIEWPORT_ID, VIEWPORT_RESOURCE_OVERLAY,
    VIEWPORT_RESOURCE_PICKING_IDS, VIEWPORT_RESOURCE_SCENE_COLOR,
    ViewportArtifactObservationResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource, ViewportSurfaceHandle,
    ViewportSurfaceSetResource, ViewportSurfaceSlot, initial_presentation_state,
    initial_product_descriptors,
};

pub fn bootstrap_editor_demo_system(
    mut host: ResMut<EditorHostResource>,
    mut shader_registry: ResMut<ShaderRegistryResource>,
) {
    initialize_editor_shader_root(&mut shader_registry);
    register_mvp_component_types(host.app.runtime_mut());
    if let Err(error) = bootstrap_mvp_scene_if_empty(host.app.runtime_mut()) {
        eprintln!("editor mvp bootstrap failed: {error}");
    }
}

pub fn seed_viewport_runtime_contracts_system(
    mut viewport_surface_sets: ResMut<ViewportSurfaceSetResource>,
    mut viewport_products: ResMut<ViewportProductRegistryResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    mut viewport_observations: ResMut<ViewportArtifactObservationResource>,
    mut viewport_picking_results: ResMut<ViewportPickingResultsResource>,
) {
    if viewport_surface_sets
        .surface_set(MAIN_VIEWPORT_ID)
        .is_none()
    {
        viewport_surface_sets.set_surface(
            MAIN_VIEWPORT_ID,
            ViewportSurfaceSlot::PrimaryColor,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_SCENE_COLOR),
        );
        viewport_surface_sets.set_surface(
            MAIN_VIEWPORT_ID,
            ViewportSurfaceSlot::PickingIds,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_PICKING_IDS),
        );
        viewport_surface_sets.set_surface(
            MAIN_VIEWPORT_ID,
            ViewportSurfaceSlot::Overlay,
            ViewportSurfaceHandle::new(EDITOR_MAIN_FLOW_ID, VIEWPORT_RESOURCE_OVERLAY),
        );
    }

    let presentation_state = viewport_presentations
        .state_for(MAIN_VIEWPORT_ID)
        .cloned()
        .unwrap_or_else(|| {
            let state = initial_presentation_state(MAIN_VIEWPORT_ID);
            viewport_presentations.upsert_state(state.clone());
            state
        });

    let descriptors = viewport_products
        .descriptors_for(MAIN_VIEWPORT_ID)
        .map(|value| value.to_vec())
        .unwrap_or_else(|| {
            let descriptors =
                initial_product_descriptors(ExpressionDimensions::new(1, 1), RealityVersion(0));
            viewport_products.update_viewport_descriptors(MAIN_VIEWPORT_ID, descriptors.clone());
            descriptors
        });

    if viewport_observations.frame_for(MAIN_VIEWPORT_ID).is_none() {
        let mut frame = ArtifactObservationFrame::new(MAIN_VIEWPORT_ID, RealityVersion(0));
        frame.available_products = descriptors.clone();
        frame.selected_primary_product_id = Some(presentation_state.selected_primary_product_id);
        frame.selected_overlay_product_ids =
            presentation_state.selected_overlay_product_ids.clone();

        for descriptor in &descriptors {
            frame
                .availability_by_product
                .insert(descriptor.id, ProductAvailabilityState::Available);
            frame
                .producer_health_by_product
                .insert(descriptor.id, ProducerHealth::Healthy);
        }

        viewport_observations.upsert_frame(frame);
    }

    viewport_picking_results.retain_viewports(|viewport_id| viewport_id == MAIN_VIEWPORT_ID);
}

fn initialize_editor_shader_root(shader_registry: &mut ShaderRegistryResource) {
    let workspace_shader_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders");
    let shader_root = workspace_shader_root
        .canonicalize()
        .unwrap_or(workspace_shader_root);
    shader_registry.add_root(shader_root.to_string_lossy().to_string());
}
