//! File: domain/editor/editor_shell/src/view_models/toolbar.rs
//! Purpose: Toolbar view model for first shell slice.

use editor_core::ToolId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarButtonViewModel {
    pub id: ToolId,
    pub stable_name: &'static str,
    pub label: String,
    pub is_active: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolbarViewModel {
    pub buttons: Vec<ToolbarButtonViewModel>,
}
