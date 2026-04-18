//! File: domain/editor/editor_shell/src/observation/viewport.rs
//! Purpose: Viewport observation frame contracts.

use editor_core::EntityId;

use crate::ObservationFrameMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportObservationFrame {
    pub metadata: ObservationFrameMetadata,
    pub selected_entity: Option<EntityId>,
    pub hovered_entity: Option<EntityId>,
    pub drag_in_progress: bool,
    pub preview_active: bool,
}
