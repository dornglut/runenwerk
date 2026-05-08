//! File: domain/editor/editor_shell/src/view_models/viewport.rs
//! Purpose: Viewport shell view model.

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportDebugStage, ViewportId};
use ui_math::UiPoint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportProductChoiceViewModel {
    pub viewport_id: ViewportId,
    pub product_id: ExpressionProductId,
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportViewModel {
    pub viewport_id: Option<ViewportId>,
    pub selected_primary_product_id: Option<ExpressionProductId>,
    pub product_choices: Vec<ViewportProductChoiceViewModel>,
    pub details_visible: bool,
    pub statistics_visible: bool,
    pub options_menu_open: bool,
    pub tools_menu_open: bool,
    pub tool_radial_anchor_position: Option<UiPoint>,
    pub debug_stage: ViewportDebugStage,
    pub root_background_opaque: bool,
    pub selected_entity: Option<EntityId>,
    pub hovered_entity: Option<EntityId>,
    pub drag_in_progress: bool,
    pub preview_active: bool,
}

impl Default for ViewportViewModel {
    fn default() -> Self {
        Self {
            viewport_id: None,
            selected_primary_product_id: None,
            product_choices: Vec::new(),
            details_visible: false,
            statistics_visible: false,
            options_menu_open: false,
            tools_menu_open: false,
            tool_radial_anchor_position: None,
            debug_stage: ViewportDebugStage::Scene,
            root_background_opaque: false,
            selected_entity: None,
            hovered_entity: None,
            drag_in_progress: false,
            preview_active: false,
        }
    }
}
