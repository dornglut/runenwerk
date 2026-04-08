//! File: domain/editor/editor_shell/src/view_models/viewport.rs
//! Purpose: Viewport shell view model.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportViewModel {
    pub selected_entity: Option<EntityId>,
    pub hovered_entity: Option<EntityId>,
    pub drag_in_progress: bool,
    pub preview_active: bool,
}
