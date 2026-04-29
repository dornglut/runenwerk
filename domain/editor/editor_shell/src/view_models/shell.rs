//! File: domain/editor/editor_shell/src/view_models/shell.rs
//! Purpose: Aggregate shell view model.

use crate::{
    ConsoleViewModel, EntityTableViewModel, InspectorViewModel, OutlinerViewModel,
    ToolbarViewModel, ViewportViewModel,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditorShellViewModel {
    pub toolbar: ToolbarViewModel,
    pub outliner: OutlinerViewModel,
    pub entity_table: EntityTableViewModel,
    pub viewport: ViewportViewModel,
    pub inspector: InspectorViewModel,
    pub console: ConsoleViewModel,
}
