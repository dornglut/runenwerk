//! File: domain/editor/editor_shell/src/observation/toolbar.rs
//! Purpose: Toolbar observation frame contracts.

use editor_core::ToolId;

use crate::ObservationFrameMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarObservedButton {
    pub id: ToolId,
    pub stable_name: &'static str,
    pub label: String,
    pub is_active: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarObservationFrame {
    pub metadata: ObservationFrameMetadata,
    pub buttons: Vec<ToolbarObservedButton>,
}
