//! File: apps/runenwerk_editor/src/runtime/viewport/product_registry.rs
//! Purpose: Session-scoped viewport product registry for editor runtime.

use std::collections::BTreeMap;

use editor_core::RealityVersion;
use editor_viewport::{
    ArtifactObservationFrame, ExpressionDimensions, ExpressionFormat, ExpressionFreshness,
    ExpressionPresentationHints, ExpressionProductDescriptor, ExpressionProductId,
    ExpressionProductKind, ExpressionSourceRealityClass, ViewportId, ViewportPresentationState,
};

pub const MAIN_VIEWPORT_ID: ViewportId = ViewportId(1);

pub const SCENE_COLOR_PRODUCT_ID: ExpressionProductId = ExpressionProductId(1);
pub const PICKING_IDS_PRODUCT_ID: ExpressionProductId = ExpressionProductId(2);
pub const OVERLAY_PRODUCT_ID: ExpressionProductId = ExpressionProductId(3);

pub fn initial_presentation_state(viewport_id: ViewportId) -> ViewportPresentationState {
    ViewportPresentationState::new(viewport_id, SCENE_COLOR_PRODUCT_ID)
}

pub fn initial_product_descriptors(
    dimensions: ExpressionDimensions,
    source_version: RealityVersion,
) -> Vec<ExpressionProductDescriptor> {
    vec![
        ExpressionProductDescriptor::new(
            SCENE_COLOR_PRODUCT_ID,
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
            PICKING_IDS_PRODUCT_ID,
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
            OVERLAY_PRODUCT_ID,
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

#[derive(Debug, Clone, ecs::Component, ecs::Resource, Default)]
pub struct ViewportProductRegistryResource {
    descriptors_by_viewport: BTreeMap<ViewportId, Vec<ExpressionProductDescriptor>>,
}

impl ViewportProductRegistryResource {
    pub fn update_viewport_descriptors(
        &mut self,
        viewport_id: ViewportId,
        descriptors: Vec<ExpressionProductDescriptor>,
    ) {
        self.descriptors_by_viewport
            .insert(viewport_id, descriptors);
    }

    pub fn descriptors_for(
        &self,
        viewport_id: ViewportId,
    ) -> Option<&[ExpressionProductDescriptor]> {
        self.descriptors_by_viewport
            .get(&viewport_id)
            .map(Vec::as_slice)
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.descriptors_by_viewport.keys().copied()
    }

    pub fn retain_viewports(&mut self, mut keep: impl FnMut(ViewportId) -> bool) {
        self.descriptors_by_viewport
            .retain(|viewport_id, _| keep(*viewport_id));
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors_by_viewport.is_empty()
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource, Default)]
pub struct ViewportPresentationStateResource {
    states_by_viewport: BTreeMap<ViewportId, ViewportPresentationState>,
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

    pub fn states(&self) -> impl Iterator<Item = &ViewportPresentationState> {
        self.states_by_viewport.values()
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.states_by_viewport.keys().copied()
    }

    pub fn retain_viewports(&mut self, mut keep: impl FnMut(ViewportId) -> bool) {
        self.states_by_viewport
            .retain(|viewport_id, _| keep(*viewport_id));
    }

    pub fn is_empty(&self) -> bool {
        self.states_by_viewport.is_empty()
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource, Default)]
pub struct ViewportArtifactObservationResource {
    frames_by_viewport: BTreeMap<ViewportId, ArtifactObservationFrame>,
}

impl ViewportArtifactObservationResource {
    pub fn frame_for(&self, viewport_id: ViewportId) -> Option<&ArtifactObservationFrame> {
        self.frames_by_viewport.get(&viewport_id)
    }

    pub fn upsert_frame(&mut self, frame: ArtifactObservationFrame) {
        self.frames_by_viewport.insert(frame.viewport_id, frame);
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.frames_by_viewport.keys().copied()
    }

    pub fn retain_viewports(&mut self, mut keep: impl FnMut(ViewportId) -> bool) {
        self.frames_by_viewport
            .retain(|viewport_id, _| keep(*viewport_id));
    }

    pub fn is_empty(&self) -> bool {
        self.frames_by_viewport.is_empty()
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
        let products =
            initial_product_descriptors(ExpressionDimensions::new(320, 200), RealityVersion(1));
        let kinds = products
            .into_iter()
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>();

        assert!(kinds.contains(&ExpressionProductKind::SceneColor2D));
        assert!(kinds.contains(&ExpressionProductKind::PickingIds2D));
        assert!(kinds.contains(&ExpressionProductKind::Overlay2D));
    }

    #[test]
    fn runtime_viewport_resources_default_to_empty_before_bootstrap() {
        assert!(ViewportProductRegistryResource::default().is_empty());
        assert!(ViewportPresentationStateResource::default().is_empty());
        assert!(ViewportArtifactObservationResource::default().is_empty());
    }

    #[test]
    fn product_registry_keeps_multiple_viewports_stable() {
        let mut registry = ViewportProductRegistryResource::default();
        let viewport_a = ViewportId(1);
        let viewport_b = ViewportId(2);
        registry.update_viewport_descriptors(
            viewport_a,
            initial_product_descriptors(ExpressionDimensions::new(320, 200), RealityVersion(1)),
        );
        registry.update_viewport_descriptors(
            viewport_b,
            initial_product_descriptors(ExpressionDimensions::new(640, 360), RealityVersion(2)),
        );

        assert_eq!(
            registry
                .descriptors_for(viewport_a)
                .expect("viewport A descriptors should exist")
                .len(),
            3
        );
        assert_eq!(
            registry
                .descriptors_for(viewport_b)
                .expect("viewport B descriptors should exist")
                .len(),
            3
        );
    }
}
