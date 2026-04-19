//! File: domain/editor/editor_viewport/src/expression/presentation.rs
//! Purpose: Viewport-owned presentation state contracts.

use crate::{ExpressionProductId, ViewportId};

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
    pub mode: ViewportPresentationMode,
}

impl ViewportPresentationState {
    pub fn new(viewport_id: ViewportId, selected_primary_product_id: ExpressionProductId) -> Self {
        Self {
            viewport_id,
            selected_primary_product_id,
            selected_overlay_product_ids: Vec::new(),
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
    }
}
