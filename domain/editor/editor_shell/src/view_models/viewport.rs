//! File: domain/editor/editor_shell/src/view_models/viewport.rs
//! Purpose: Viewport shell view model.

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportProductChoiceViewModel {
    pub viewport_id: ViewportId,
    pub product_id: ExpressionProductId,
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportViewModel {
    pub viewport_id: Option<ViewportId>,
    pub selected_primary_product_id: Option<ExpressionProductId>,
    pub product_choices: Vec<ViewportProductChoiceViewModel>,
    pub details_visible: bool,
    pub statistics_visible: bool,
    pub options_menu_open: bool,
    pub selected_entity: Option<EntityId>,
    pub hovered_entity: Option<EntityId>,
    pub drag_in_progress: bool,
    pub preview_active: bool,
}
