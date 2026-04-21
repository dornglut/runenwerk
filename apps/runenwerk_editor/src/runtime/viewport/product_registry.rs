//! File: apps/runenwerk_editor/src/runtime/viewport/product_registry.rs
//! Purpose: Session-scoped viewport product registry for editor runtime.

use std::collections::BTreeMap;

use editor_core::RealityVersion;
use editor_viewport::{
    ArtifactObservationFrame,
    ExpressionDimensions, ExpressionFormat, ExpressionFreshness, ExpressionPresentationHints,
    ExpressionProductDescriptor, ExpressionProductId, ExpressionProductKind,
    ExpressionSourceRealityClass, ViewportId, ViewportPresentationState,
};

pub const MAIN_VIEWPORT_ID: ViewportId = ViewportId(1);

pub const PRODUCT_ID_SCENE_COLOR: ExpressionProductId = ExpressionProductId(1);
pub const PRODUCT_ID_PICKING_IDS: ExpressionProductId = ExpressionProductId(2);
pub const PRODUCT_ID_OVERLAY: ExpressionProductId = ExpressionProductId(3);

pub fn initial_product_descriptors(
    dimensions: ExpressionDimensions,
    source_version: RealityVersion,
) -> Vec<ExpressionProductDescriptor> {
    vec![
        ExpressionProductDescriptor::new(
            PRODUCT_ID_SCENE_COLOR,
            ExpressionProductKind::SceneColor2D,
            dimensions,
            ExpressionFormat::Rgba8Unorm,
            "editor.viewport.scene_producer",
            ExpressionSourceRealityClass::ObservedScene,
            source_version,
            ExpressionFreshness::Current,
            ExpressionPresentationHints {
                srgb: true,
                premultiplied_alpha: false,
                y_flipped: false,
            },
            None,
        ),
        ExpressionProductDescriptor::new(
            PRODUCT_ID_PICKING_IDS,
            ExpressionProductKind::PickingIds2D,
            dimensions,
            ExpressionFormat::R32Uint,
            "editor.viewport.picking_producer",
            ExpressionSourceRealityClass::DerivedPicking,
            source_version,
            ExpressionFreshness::Current,
            ExpressionPresentationHints {
                srgb: false,
                premultiplied_alpha: false,
                y_flipped: false,
            },
            None,
        ),
        ExpressionProductDescriptor::new(
            PRODUCT_ID_OVERLAY,
            ExpressionProductKind::Overlay2D,
            dimensions,
            ExpressionFormat::Rgba8Unorm,
            "editor.viewport.overlay_producer",
            ExpressionSourceRealityClass::DerivedOverlay,
            source_version,
            ExpressionFreshness::Current,
            ExpressionPresentationHints {
                srgb: true,
                premultiplied_alpha: true,
                y_flipped: false,
            },
            None,
        ),
    ]
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct ViewportProductRegistryResource {
    descriptors_by_viewport: BTreeMap<ViewportId, Vec<ExpressionProductDescriptor>>,
}

impl Default for ViewportProductRegistryResource {
    fn default() -> Self {
        let mut descriptors_by_viewport = BTreeMap::new();
        descriptors_by_viewport.insert(
            MAIN_VIEWPORT_ID,
            initial_product_descriptors(ExpressionDimensions::new(1, 1), RealityVersion(0)),
        );
        Self {
            descriptors_by_viewport,
        }
    }
}

impl ViewportProductRegistryResource {
    pub fn update_viewport_descriptors(
        &mut self,
        viewport_id: ViewportId,
        descriptors: Vec<ExpressionProductDescriptor>,
    ) {
        self.descriptors_by_viewport.insert(viewport_id, descriptors);
    }

    pub fn descriptors_for(&self, viewport_id: ViewportId) -> Option<&[ExpressionProductDescriptor]> {
        self.descriptors_by_viewport
            .get(&viewport_id)
            .map(Vec::as_slice)
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct ViewportPresentationStateResource {
    states_by_viewport: BTreeMap<ViewportId, ViewportPresentationState>,
}

impl Default for ViewportPresentationStateResource {
    fn default() -> Self {
        let mut states_by_viewport = BTreeMap::new();
        states_by_viewport.insert(
            MAIN_VIEWPORT_ID,
            crate::runtime::viewport::default_presentation_state(),
        );
        Self { states_by_viewport }
    }
}

impl ViewportPresentationStateResource {
    pub fn state_for(&self, viewport_id: ViewportId) -> Option<&ViewportPresentationState> {
        self.states_by_viewport.get(&viewport_id)
    }

    pub fn state_for_mut(
        &mut self,
        viewport_id: ViewportId,
    ) -> Option<&mut ViewportPresentationState> {
        self.states_by_viewport.get_mut(&viewport_id)
    }

    pub fn upsert_state(&mut self, state: ViewportPresentationState) {
        self.states_by_viewport.insert(state.viewport_id, state);
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct ViewportArtifactObservationResource {
    frames_by_viewport: BTreeMap<ViewportId, ArtifactObservationFrame>,
}

impl Default for ViewportArtifactObservationResource {
    fn default() -> Self {
        let source_version = RealityVersion(0);
        let presentation_state = crate::runtime::viewport::default_presentation_state();
        let descriptors = initial_product_descriptors(ExpressionDimensions::new(1, 1), source_version);
        let mut frame = ArtifactObservationFrame::new(MAIN_VIEWPORT_ID, source_version);
        frame.available_products = descriptors.clone();
        frame.selected_primary_product_id = Some(presentation_state.selected_primary_product_id);
        frame.selected_overlay_product_ids = presentation_state.selected_overlay_product_ids.clone();

        for descriptor in &descriptors {
            frame
                .availability_by_product
                .insert(descriptor.id, editor_viewport::ProductAvailabilityState::Available);
            frame
                .producer_health_by_product
                .insert(descriptor.id, editor_viewport::ProducerHealth::Healthy);
        }

        let mut frames_by_viewport = BTreeMap::new();
        frames_by_viewport.insert(MAIN_VIEWPORT_ID, frame);
        Self { frames_by_viewport }
    }
}

impl ViewportArtifactObservationResource {
    pub fn frame_for(&self, viewport_id: ViewportId) -> Option<&ArtifactObservationFrame> {
        self.frames_by_viewport.get(&viewport_id)
    }

    pub fn upsert_frame(&mut self, frame: ArtifactObservationFrame) {
        self.frames_by_viewport.insert(frame.viewport_id, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_one_uses_stable_single_viewport_session_id() {
        assert_eq!(MAIN_VIEWPORT_ID, ViewportId(1));
    }

    #[test]
    fn initial_registry_contains_locked_product_kind_subset() {
        let products = initial_product_descriptors(ExpressionDimensions::new(320, 200), RealityVersion(1));
        let kinds = products.into_iter().map(|descriptor| descriptor.kind).collect::<Vec<_>>();

        assert!(kinds.contains(&ExpressionProductKind::SceneColor2D));
        assert!(kinds.contains(&ExpressionProductKind::PickingIds2D));
        assert!(kinds.contains(&ExpressionProductKind::Overlay2D));
    }
}
