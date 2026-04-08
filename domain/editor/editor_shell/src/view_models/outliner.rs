//! File: domain/editor/editor_shell/src/view_models/outliner.rs
//! Purpose: Outliner shell view model.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerRowViewModel {
    pub entity: EntityId,
    pub display_name: String,
    pub depth: usize,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutlinerViewModel {
    pub rows: Vec<OutlinerRowViewModel>,
}
