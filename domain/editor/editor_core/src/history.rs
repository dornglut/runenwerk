//! File: domain/editor/editor_core/src/history.rs
//! Purpose: Undo/redo history state and history entry metadata.

use crate::{CommandMetadata, TransactionMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub transaction: TransactionMetadata,
    pub commands: Vec<CommandMetadata>,
}

impl HistoryEntry {
    pub fn new(transaction: TransactionMetadata, commands: Vec<CommandMetadata>) -> Self {
        Self {
            transaction,
            commands,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HistoryStack {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
}

impl HistoryStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn push_applied(&mut self, entry: HistoryEntry) {
        self.undo.push(entry);
        self.redo.clear();
    }

    pub fn pop_undo(&mut self) -> Option<HistoryEntry> {
        self.undo.pop()
    }

    pub fn push_redo(&mut self, entry: HistoryEntry) {
        self.redo.push(entry);
    }

    pub fn pop_redo(&mut self) -> Option<HistoryEntry> {
        self.redo.pop()
    }

    pub fn peek_undo(&self) -> Option<&HistoryEntry> {
        self.undo.last()
    }

    pub fn peek_redo(&self) -> Option<&HistoryEntry> {
        self.redo.last()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}
