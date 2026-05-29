//! Session-local operation history for UI Designer self-authoring.

use super::*;

#[derive(Debug, Clone)]
pub struct EditorLabOperationHistoryEntry {
    pub id: String,
    pub label: String,
    pub document_id: EditorDefinitionId,
    pub before: EditorDefinitionDocument,
    pub after: EditorDefinitionDocument,
    pub report: EditorLabOperationReport,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EditorLabOperationHistorySnapshot {
    pub undo_count: usize,
    pub redo_count: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Default)]
pub(super) struct EditorLabOperationHistory {
    pub(super) undo: Vec<EditorLabOperationHistoryEntry>,
    pub(super) redo: Vec<EditorLabOperationHistoryEntry>,
    pub(super) next_sequence: u64,
}
