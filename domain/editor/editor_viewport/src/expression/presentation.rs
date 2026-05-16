//! File: domain/editor/editor_viewport/src/expression/presentation.rs
//! Purpose: Viewport-owned presentation state contracts.

use crate::{ExpressionProductId, ViewportFieldVisualizerSettings, ViewportId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ViewportPresentationMode {
    #[default]
    Single,
    Layered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportPresentationState {
    pub viewport_id: ViewportId,
    pub selected_primary_product_id: ExpressionProductId,
    pub selected_overlay_product_ids: Vec<ExpressionProductId>,
    pub field_visualizer_settings: ViewportFieldVisualizerSettings,
    pub mode: ViewportPresentationMode,
}

impl ViewportPresentationState {
    pub fn new(viewport_id: ViewportId, selected_primary_product_id: ExpressionProductId) -> Self {
        Self {
            viewport_id,
            selected_primary_product_id,
            selected_overlay_product_ids: Vec::new(),
            field_visualizer_settings: ViewportFieldVisualizerSettings::default(),
            mode: ViewportPresentationMode::Single,
        }
    }

    pub fn select_primary_product(&mut self, product_id: ExpressionProductId) {
        self.selected_primary_product_id = product_id;
    }

    pub fn set_overlay_products(&mut self, products: Vec<ExpressionProductId>) {
        self.selected_overlay_product_ids = products;
        if self.selected_overlay_product_ids.is_empty() {
            self.mode = ViewportPresentationMode::Single;
        } else {
            self.mode = ViewportPresentationMode::Layered;
        }
    }

    pub fn set_field_visualizer_settings(&mut self, settings: ViewportFieldVisualizerSettings) {
        self.field_visualizer_settings = settings;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_state_requires_primary_product_id() {
        let state = ViewportPresentationState::new(ViewportId(1), ExpressionProductId(11));

        assert_eq!(state.viewport_id, ViewportId(1));
        assert_eq!(state.selected_primary_product_id, ExpressionProductId(11));
        assert!(state.selected_overlay_product_ids.is_empty());
        assert_eq!(
            state.field_visualizer_settings,
            ViewportFieldVisualizerSettings::default()
        );
    }

    #[test]
    fn field_visualizer_settings_do_not_change_selected_product_identity() {
        let mut state = ViewportPresentationState::new(ViewportId(1), ExpressionProductId(11));
        let settings = ViewportFieldVisualizerSettings::default()
            .with_component(crate::ViewportFieldVisualizerComponent::Magnitude)
            .with_slice_index(7)
            .with_color_ramp(crate::ViewportFieldVisualizerColorRamp::Heat)
            .with_debug_mode(crate::ViewportFieldVisualizerDebugMode::Freshness);

        state.set_field_visualizer_settings(settings);

        assert_eq!(state.selected_primary_product_id, ExpressionProductId(11));
        assert_eq!(state.field_visualizer_settings, settings);
    }
}
