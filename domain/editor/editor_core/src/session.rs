//! File: domain/editor/editor_core/src/session.rs
//! Purpose: Root editor session state.

use std::collections::BTreeMap;

use crate::{DocumentDescriptor, DocumentId, DocumentKind, EditorMutationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModeId(pub u64);

pub const EDIT_MODE_ID: ModeId = ModeId(1);
pub const PLAY_MODE_ID: ModeId = ModeId(2);
pub const SIMULATE_MODE_ID: ModeId = ModeId(3);
pub const PREVIEW_MODE_ID: ModeId = ModeId(4);

#[derive(Debug, Default)]
pub struct EditorSession {
    documents: BTreeMap<DocumentId, DocumentDescriptor>,
    document_tabs: Vec<DocumentId>,
    active_document: Option<DocumentId>,
}

impl Default for ModeId {
    fn default() -> Self {
        EDIT_MODE_ID
    }
}

impl EditorSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_document(&self) -> Option<DocumentId> {
        self.active_document
    }

    pub fn set_active_document(
        &mut self,
        document_id: Option<DocumentId>,
    ) -> Result<(), EditorMutationError> {
        if let Some(document_id) = document_id
            && !self.documents.contains_key(&document_id)
        {
            return Err(EditorMutationError::session_rejected("document not found"));
        }

        self.active_document = document_id;
        Ok(())
    }

    pub fn activate_document(
        &mut self,
        document_id: DocumentId,
    ) -> Result<(), EditorMutationError> {
        self.set_active_document(Some(document_id))
    }

    pub fn active_document_descriptor(&self) -> Option<&DocumentDescriptor> {
        self.active_document
            .and_then(|document_id| self.document(document_id))
    }

    pub fn documents(&self) -> impl Iterator<Item = &DocumentDescriptor> {
        self.documents.values()
    }

    pub fn document_tabs(&self) -> &[DocumentId] {
        &self.document_tabs
    }

    pub fn document_tab_descriptors(&self) -> impl Iterator<Item = &DocumentDescriptor> {
        self.document_tabs
            .iter()
            .filter_map(|document_id| self.documents.get(document_id))
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
        let document_id = descriptor.id;
        let previous = self.documents.insert(document_id, descriptor);
        if previous.is_none() && !self.document_tabs.contains(&document_id) {
            self.document_tabs.push(document_id);
        }
        if self.active_document.is_none() {
            self.active_document = Some(document_id);
        }
        previous
    }

    pub fn remove_document(&mut self, document_id: DocumentId) -> Option<DocumentDescriptor> {
        self.document_tabs.retain(|tab_id| *tab_id != document_id);
        if self.active_document == Some(document_id) {
            self.active_document = None;
        }

        self.documents.remove(&document_id)
    }

    pub fn close_document(
        &mut self,
        document_id: DocumentId,
    ) -> Result<DocumentDescriptor, EditorMutationError> {
        if self.document(document_id).is_none() {
            return Err(EditorMutationError::session_rejected("document not found"));
        }

        let removed_index = self
            .document_tabs
            .iter()
            .position(|tab_id| *tab_id == document_id);
        let removed = self
            .remove_document(document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        if self.active_document.is_none() {
            self.active_document = removed_index.and_then(|index| {
                self.document_tabs
                    .get(index)
                    .or_else(|| {
                        index
                            .checked_sub(1)
                            .and_then(|prev| self.document_tabs.get(prev))
                    })
                    .copied()
            });
        }

        Ok(removed)
    }
}

impl crate::CommandContext for EditorSession {
    type Error = EditorMutationError;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(id: u64, kind: DocumentKind) -> DocumentDescriptor {
        DocumentDescriptor::new(DocumentId(id), kind, format!("Document {id}"))
    }

    #[test]
    fn upsert_document_opens_ordered_tabs_and_sets_first_active_document() {
        let mut session = EditorSession::new();

        session.upsert_document(descriptor(1, DocumentKind::Scene));
        session.upsert_document(descriptor(2, DocumentKind::MaterialGraph));

        assert_eq!(session.document_tabs(), &[DocumentId(1), DocumentId(2)]);
        assert_eq!(session.active_document(), Some(DocumentId(1)));
    }

    #[test]
    fn close_active_document_activates_neighbor_tab() {
        let mut session = EditorSession::new();
        session.upsert_document(descriptor(1, DocumentKind::Scene));
        session.upsert_document(descriptor(2, DocumentKind::Material));
        session
            .activate_document(DocumentId(2))
            .expect("document should exist");

        session
            .close_document(DocumentId(2))
            .expect("document should close");

        assert_eq!(session.document_tabs(), &[DocumentId(1)]);
        assert_eq!(session.active_document(), Some(DocumentId(1)));
    }
}
