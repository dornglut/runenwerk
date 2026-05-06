//! File: apps/runenwerk_editor/src/editor_runtime/document/tabs.rs
//! Purpose: App-local runtime state for generic editor document tabs.

use std::collections::BTreeMap;

use editor_core::{DocumentId, DocumentKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentTabRuntimeRecord {
    pub document_id: DocumentId,
    pub document_kind: DocumentKind,
    pub provider_compatible: bool,
}

impl DocumentTabRuntimeRecord {
    pub fn new(
        document_id: DocumentId,
        document_kind: DocumentKind,
        provider_compatible: bool,
    ) -> Self {
        Self {
            document_id,
            document_kind,
            provider_compatible,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DocumentTabRuntimeState {
    records: BTreeMap<DocumentId, DocumentTabRuntimeRecord>,
}

impl DocumentTabRuntimeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, record: DocumentTabRuntimeRecord) {
        self.records.insert(record.document_id, record);
    }

    pub fn remove(&mut self, document_id: DocumentId) -> Option<DocumentTabRuntimeRecord> {
        self.records.remove(&document_id)
    }

    pub fn record(&self, document_id: DocumentId) -> Option<&DocumentTabRuntimeRecord> {
        self.records.get(&document_id)
    }

    pub fn records(&self) -> impl Iterator<Item = &DocumentTabRuntimeRecord> {
        self.records.values()
    }
}
