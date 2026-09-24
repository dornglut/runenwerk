//! File: domain/editor/editor_shell/src/observation/outliner.rs
//! Purpose: Outliner observation frame contracts.

use editor_core::EntityId;

use crate::ObservationFrameMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerObservedRow {
    pub entity: EntityId,
    pub display_name: String,
    pub depth: usize,
    pub has_children: bool,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinerObservationFrame {
    pub metadata: ObservationFrameMetadata,
    pub rows: Vec<OutlinerObservedRow>,
}
