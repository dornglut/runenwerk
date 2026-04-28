//! File: domain/editor/editor_core/src/session.rs
//! Purpose: Root editor session state.

use std::collections::BTreeMap;

use crate::{
    DocumentDescriptor, DocumentId, EditorMutationError, HistoryStack, SelectionSet, ToolId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorMode {
    #[default]
    Edit,
    Play,
    Simulate,
}

#[derive(Debug, Default)]
pub struct EditorSession {
    documents: BTreeMap<DocumentId, DocumentDescriptor>,
    active_document: Option<DocumentId>,
    active_tool: Option<ToolId>,
    mode: EditorMode,
    selection: SelectionSet,
    history: HistoryStack,
}

impl EditorSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn active_document(&self) -> Option<DocumentId> {
        self.active_document
    }

    pub fn set_active_document(&mut self, document_id: Option<DocumentId>) {
        self.active_document = document_id;
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.active_tool
    }

    pub fn set_active_tool(&mut self, tool_id: Option<ToolId>) {
        self.active_tool = tool_id;
    }

    pub fn documents(&self) -> impl Iterator<Item = &DocumentDescriptor> {
        self.documents.values()
    }

    pub fn document(&self, document_id: DocumentId) -> Option<&DocumentDescriptor> {
        self.documents.get(&document_id)
    }

    pub fn document_mut(&mut self, document_id: DocumentId) -> Option<&mut DocumentDescriptor> {
        self.documents.get_mut(&document_id)
    }

    pub fn upsert_document(
        &mut self,
        descriptor: DocumentDescriptor,
    ) -> Option<DocumentDescriptor> {
        self.documents.insert(descriptor.id, descriptor)
    }

    pub fn remove_document(&mut self, document_id: DocumentId) -> Option<DocumentDescriptor> {
        if self.active_document == Some(document_id) {
            self.active_document = None;
        }

        self.documents.remove(&document_id)
    }

    pub fn selection(&self) -> &SelectionSet {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut SelectionSet {
        &mut self.selection
    }

    pub fn history(&self) -> &HistoryStack {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut HistoryStack {
        &mut self.history
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select_single(&mut self, target: crate::SelectionTarget) {
        self.selection.set_single(target);
    }

    pub fn add_selection(&mut self, target: crate::SelectionTarget) {
        self.selection.add(target);
    }

    pub fn set_document_dirty(
        &mut self,
        document_id: crate::DocumentId,
        is_dirty: bool,
    ) -> Result<(), EditorMutationError> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        document.is_dirty = is_dirty;
        Ok(())
    }
}

impl crate::CommandContext for EditorSession {
    type Error = EditorMutationError;

    fn mark_document_dirty(
        &mut self,
        document_id: crate::DocumentId,
        is_dirty: bool,
    ) -> Result<(), Self::Error> {
        self.set_document_dirty(document_id, is_dirty)
    }
}
